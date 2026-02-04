use anyhow::Result;
use robotstxt::DefaultMatcher;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration, Instant};
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub sources: Sources,
    pub levels: Option<HashMap<String, LevelConfig>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Sources {
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LevelConfig {
    pub min_dances: Option<u32>,
    pub min_dances_legacy: Option<u32>,
    pub min_dances_2026: Option<u32>,
}

impl Config {
    pub fn get_min_dances(&self, level: &crate::models::Level, date: &chrono::NaiveDate) -> u32 {
        use chrono::Datelike;
        let level_str = format!("{:?}", level);
        if let Some(levels) = &self.levels {
            if let Some(config) = levels.get(&level_str) {
                if let Some(min) = config.min_dances {
                    return min;
                }
                let is_2026_or_later = date.year() >= 2026;
                if is_2026_or_later {
                    return config
                        .min_dances_2026
                        .or(config.min_dances_legacy)
                        .unwrap_or(0);
                } else {
                    return config.min_dances_legacy.unwrap_or(0);
                }
            }
        }
        0
    }
}

pub struct RobotsChecker {
    matchers: HashMap<String, String>, // base_url -> robots_txt content
}

impl RobotsChecker {
    pub fn new() -> Self {
        Self {
            matchers: HashMap::new(),
        }
    }

    pub async fn is_allowed(&mut self, url_str: &str) -> bool {
        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return true,
        };

        let base_url = self.get_base_url(&url);
        let robots_url = format!("{}robots.txt", base_url);

        if !self.matchers.contains_key(&base_url) {
            match reqwest::get(&robots_url).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let content = resp.text().await.unwrap_or_default();
                        self.matchers.insert(base_url.clone(), content);
                    } else {
                        self.matchers.insert(base_url.clone(), String::new());
                    }
                }
                Err(_) => {
                    self.matchers.insert(base_url.clone(), String::new());
                }
            }
        }

        let content = self.matchers.get(&base_url).unwrap();
        if content.is_empty() {
            return true;
        }

        let mut matcher = DefaultMatcher::default();
        matcher.allowed_by_robots(content, vec!["*"], url_str)
    }

    pub fn get_crawl_delay(&self, url_str: &str) -> Option<f32> {
        let url = Url::parse(url_str).ok()?;
        let base_url = self.get_base_url(&url);
        let content = self.matchers.get(&base_url)?;
        if content.is_empty() {
            return None;
        }

        let mut in_relevant_block = false;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let line_lower = line.to_lowercase();
            if line_lower.starts_with("user-agent:") {
                let ua = line.split(':').nth(1).unwrap_or("").trim();
                in_relevant_block = ua == "*";
            } else if in_relevant_block && line_lower.starts_with("crawl-delay:") {
                if let Some(val_str) = line.split(':').nth(1) {
                    if let Ok(val) = val_str.trim().parse::<f32>() {
                        return Some(val);
                    }
                }
            }
        }
        None
    }

    fn get_base_url(&self, url: &Url) -> String {
        format!("{}://{}/", url.scheme(), url.host_str().unwrap_or(""))
    }
}

pub struct Scraper {
    client: reqwest::Client,
    robots_checker: RobotsChecker,
    last_request_times: HashMap<String, Instant>,
}

impl Scraper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            robots_checker: RobotsChecker::new(),
            last_request_times: HashMap::new(),
        }
    }

    async fn fetch_with_rate_limit(&mut self, url_str: &str) -> Result<String> {
        let url = Url::parse(url_str)?;
        let domain = url.host_str().unwrap_or("").to_string();

        if let Some(delay_secs) = self.robots_checker.get_crawl_delay(url_str) {
            if let Some(&last_time) = self.last_request_times.get(&domain) {
                let elapsed = last_time.elapsed();
                let wait_duration = Duration::from_secs_f32(delay_secs);
                if elapsed < wait_duration {
                    let sleep_time = wait_duration - elapsed;
                    log::debug!("Respecting Crawl-delay: sleeping for {:?}", sleep_time);
                    sleep(sleep_time).await;
                }
            }
        }

        let resp = self.client.get(url_str).send().await?;
        let text = resp.text().await?;
        self.last_request_times.insert(domain, Instant::now());
        Ok(text)
    }

    pub async fn scrape_all(&mut self, config: &Config) -> Result<()> {
        for base_url in &config.sources.urls {
            if !self.robots_checker.is_allowed(base_url).await {
                log::warn!("robots.txt disallows scraping {}, skipping", base_url);
                continue;
            }

            log::info!("Scraping base URL: {}", base_url);
            let html_content = self.fetch_with_rate_limit(base_url).await?;
            let competition_links = self.extract_competition_links(&html_content, base_url)?;

            log::info!("Found {} competition links", competition_links.len());

            for link in competition_links {
                if let Err(e) = self.scrape_competition(&link).await {
                    log::error!("Error scraping competition {}: {}", link, e);
                }
            }
        }
        Ok(())
    }

    fn extract_competition_links(&self, html: &str, base_url: &str) -> Result<Vec<String>> {
        let fragment = Html::parse_document(html);
        let selector = Selector::parse("a[href]").unwrap();
        let base = Url::parse(base_url)?;

        let mut links = Vec::new();
        for element in fragment.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if href.ends_with(".htm") || href.ends_with(".html") {
                    if let Ok(full_url) = base.join(href) {
                        links.push(full_url.to_string());
                    }
                }
            }
        }
        Ok(links)
    }

    async fn scrape_competition(&mut self, url_str: &str) -> Result<()> {
        if !self.robots_checker.is_allowed(url_str).await {
            log::warn!("robots.txt disallows scraping {}, skipping", url_str);
            return Ok(());
        }

        log::info!("Scraping competition: {}", url_str);
        let html_content = self.fetch_with_rate_limit(url_str).await?;
        let event_name = self.extract_event_name(&html_content)?;
        let sanitized_event_name = self.sanitize_name(&event_name);

        let data_dir = Path::new("data").join(&sanitized_event_name);
        fs::create_dir_all(&data_dir)?;

        let filename = Path::new(url_str)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("index.htm");
        self.save_file(&data_dir, filename, &html_content)?;

        // Now download related files: erg.htm, deck.htm, tabges.htm, ergwert.htm
        let base_url = Url::parse(url_str)?;
        let related_files = vec!["erg.htm", "deck.htm", "tabges.htm", "ergwert.htm"];

        for rel_file in related_files {
            let rel_url = base_url.join(rel_file)?;
            if !self.robots_checker.is_allowed(rel_url.as_str()).await {
                continue;
            }

            let file_path = data_dir.join(rel_file);
            if file_path.exists() {
                log::debug!("File {:?} already exists, skipping (Smart Skip)", file_path);
                continue;
            }

            log::info!("Downloading related file: {}", rel_url);
            match self.fetch_with_rate_limit(rel_url.as_str()).await {
                Ok(content) => {
                    self.save_file(&data_dir, rel_file, &content)?;
                }
                Err(e) => log::error!("Failed to download {}: {}", rel_url, e),
            }
        }

        Ok(())
    }

    fn extract_event_name(&self, html: &str) -> Result<String> {
        let fragment = Html::parse_document(html);
        let selector = Selector::parse("title").unwrap();
        if let Some(title_elem) = fragment.select(&selector).next() {
            Ok(title_elem
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string())
        } else {
            Ok("unknown_event".to_string())
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        let mut sanitized: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        sanitized.truncate(64);
        sanitized
    }

    fn save_file(&self, dir: &Path, filename: &str, content: &str) -> Result<()> {
        let path = dir.join(filename);
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_robots_full_block() {
        let mut checker = RobotsChecker::new();
        let base_url = "http://example.com/";
        let robots_txt = "User-agent: *\nDisallow: /";
        checker
            .matchers
            .insert(base_url.to_string(), robots_txt.to_string());

        assert!(!checker.is_allowed("http://example.com/any").await);
        assert!(!checker.is_allowed("http://example.com/").await);
    }

    #[tokio::test]
    async fn test_robots_specific_path() {
        let mut checker = RobotsChecker::new();
        let base_url = "http://example.com/";
        let robots_txt = "User-agent: *\nDisallow: /2025/";
        checker
            .matchers
            .insert(base_url.to_string(), robots_txt.to_string());

        assert!(
            checker
                .is_allowed("http://example.com/2024/index.htm")
                .await
        );
        assert!(
            !checker
                .is_allowed("http://example.com/2025/index.htm")
                .await
        );
    }

    #[test]
    fn test_extract_competition_links_malformed() {
        let scraper = Scraper::new();
        let links = scraper
            .extract_competition_links("not html at all", "http://example.com/")
            .unwrap();
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_crawl_delay_parsing() {
        let mut checker = RobotsChecker::new();
        let base_url = "http://example.com/";
        let robots_txt = "User-agent: *\nCrawl-delay: 5.5";
        checker
            .matchers
            .insert(base_url.to_string(), robots_txt.to_string());

        assert_eq!(checker.get_crawl_delay("http://example.com/"), Some(5.5));
    }

    #[test]
    fn test_extract_competition_links() {
        let scraper = Scraper::new();
        let html = r#"
            <html>
            <body>
                <a href="comp1/index.htm">Comp 1</a>
                <a href="comp2/index.html">Comp 2</a>
                <a href="not-a-comp.png">Not a comp</a>
            </body>
            </html>
        "#;
        let base_url = "http://example.com/";
        let links = scraper.extract_competition_links(html, base_url).unwrap();

        assert_eq!(links.len(), 2);
        assert!(links.contains(&"http://example.com/comp1/index.htm".to_string()));
        assert!(links.contains(&"http://example.com/comp2/index.html".to_string()));
    }

    #[test]
    fn test_extract_event_name() {
        let scraper = Scraper::new();
        let html = "<html><head><title>  WDSF World Open Latin  </title></head></html>";
        let name = scraper.extract_event_name(html).unwrap();
        assert_eq!(name, "WDSF World Open Latin");
    }

    #[test]
    fn test_extract_event_name_malformed() {
        let scraper = Scraper::new();
        assert_eq!(scraper.extract_event_name("").unwrap(), "unknown_event");
        assert_eq!(
            scraper.extract_event_name("<html></html>").unwrap(),
            "unknown_event"
        );
    }

    #[test]
    fn test_sanitize_name() {
        let scraper = Scraper::new();
        assert_eq!(scraper.sanitize_name("Event Name!"), "Event_Name_");
        assert_eq!(scraper.sanitize_name("A".repeat(100).as_str()).len(), 64);
    }
}
