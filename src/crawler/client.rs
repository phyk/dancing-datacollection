use anyhow::{anyhow, Context, Result};
use robotstxt::DefaultMatcher;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration, Instant};
use url::Url;

const USER_AGENT: &str = concat!("dancing-datacollection/", env!("CARGO_PKG_VERSION"));
/// Pause between requests to the same host when robots.txt sets no Crawl-delay.
const DEFAULT_CRAWL_DELAY_SECS: f32 = 0.5;

fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("failed to build HTTP client")
}

pub struct RobotsChecker {
    client: reqwest::Client,
    matchers: HashMap<String, String>, // base_url -> robots_txt content
}

impl Default for RobotsChecker {
    fn default() -> Self {
        Self::with_client(build_client())
    }
}

impl RobotsChecker {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
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
            match self.client.get(&robots_url).send().await {
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
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim().to_lowercase();
            let value = value.trim();
            if key == "user-agent" {
                in_relevant_block = value == "*";
            } else if in_relevant_block && key == "crawl-delay" {
                if let Ok(val) = value.parse::<f32>() {
                    return Some(val);
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
        Self::default()
    }

    /// Fetches `url_str`, honouring robots.txt (allow rules and Crawl-delay).
    /// Fails if robots.txt disallows the URL or the server responds with a
    /// non-success status.
    async fn fetch_with_rate_limit(&mut self, url_str: &str) -> Result<String> {
        let url = Url::parse(url_str)?;
        let domain = url.host_str().unwrap_or("").to_string();

        // Also loads robots.txt for this host, so the crawl delay below is known.
        if !self.robots_checker.is_allowed(url_str).await {
            return Err(anyhow!("robots.txt disallows {}", url_str));
        }

        // Fall back to a short pause so hosts without a Crawl-delay aren't hit back to back.
        let delay_secs = self
            .robots_checker
            .get_crawl_delay(url_str)
            .unwrap_or(DEFAULT_CRAWL_DELAY_SECS);
        if let Some(&last_time) = self.last_request_times.get(&domain) {
            let elapsed = last_time.elapsed();
            let wait_duration = Duration::from_secs_f32(delay_secs);
            if elapsed < wait_duration {
                let sleep_time = wait_duration - elapsed;
                log::debug!("Respecting crawl delay: sleeping for {:?}", sleep_time);
                sleep(sleep_time).await;
            }
        }

        let resp = self.client.get(url_str).send().await;
        self.last_request_times.insert(domain, Instant::now());
        let text = resp?.error_for_status()?.text().await?;
        Ok(text)
    }

    pub async fn scrape_all(&mut self, urls: &[String]) -> Result<()> {
        for base_url in urls {
            if !self.robots_checker.is_allowed(base_url).await {
                log::warn!("robots.txt disallows scraping {}, skipping", base_url);
                continue;
            }

            log::info!("Scraping base URL: {}", base_url);
            let competition_links = self.get_competition_links(base_url).await?;

            log::info!("Found {} competition links", competition_links.len());

            for link in competition_links {
                if let Err(e) = self.scrape_competition(&link).await {
                    log::error!("Error scraping competition {}: {}", link, e);
                }
            }
        }
        Ok(())
    }

    pub async fn get_competition_links(&mut self, url_str: &str) -> Result<Vec<String>> {
        let html_content = self.fetch_with_rate_limit(url_str).await?;
        // Use specific selectors for competition links to distinguish from navigation.
        let selector_str = "center a, .pro_p_zeile a, .t_zeile a";
        self.extract_competition_links(&html_content, url_str, Some(selector_str))
    }

    pub fn extract_competition_links(
        &self,
        html: &str,
        base_url: &str,
        selector_str: Option<&str>,
    ) -> Result<Vec<String>> {
        let fragment = Html::parse_document(html);
        let selector = Selector::parse(selector_str.unwrap_or("a[href]")).unwrap();
        let base = Url::parse(base_url)?;

        let mut links = Vec::new();
        for element in fragment.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                // Filter for .htm or .html files, ignoring common non-competition files.
                if (href.ends_with(".htm") || href.ends_with(".html"))
                    && !href.contains("robots.txt")
                    && !href.contains("mailto:")
                {
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

        log::info!("Downloading competition files from: {}", url_str);
        let html_content = self.fetch_with_rate_limit(url_str).await?;
        let event_name = self.extract_event_name(&html_content)?;
        let sanitized_event_name = crate::models::sanitize_name(&event_name);

        let data_dir = Path::new("data").join(&sanitized_event_name);
        fs::create_dir_all(&data_dir)?;

        self.save_competition_files(url_str, &data_dir, &html_content)
            .await
    }

    /// Fails unless the page and all related files were downloaded, so callers never parse a partial set.
    pub async fn download_competition_files(
        &mut self,
        url_str: &str,
        target_dir: &Path,
    ) -> Result<()> {
        log::info!("Downloading competition files from: {}", url_str);
        let html_content = self.fetch_with_rate_limit(url_str).await?;
        self.save_competition_files(url_str, target_dir, &html_content)
            .await
    }

    /// Saves the already-fetched main page into `target_dir` and downloads
    /// the related files (deck.htm, ergwert.htm) next to it.
    async fn save_competition_files(
        &mut self,
        url_str: &str,
        target_dir: &Path,
        html_content: &str,
    ) -> Result<()> {
        let filename = Path::new(url_str)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("index.htm");

        if !filename.ends_with(".htm") && !filename.ends_with(".html") {
            // If the URL doesn't end in a htm file, assume index.htm
            self.save_file(target_dir, "index.htm", html_content)?;
        } else {
            self.save_file(target_dir, filename, html_content)?;
        }

        // Now download related files: deck.htm, ergwert.htm
        let base_url = Url::parse(url_str)?;
        let related_files = vec!["deck.htm", "ergwert.htm"];

        for rel_file in related_files {
            let rel_url = base_url.join(rel_file)?;
            if target_dir.join(rel_file).exists() {
                log::debug!(
                    "File {:?} already exists, skipping",
                    target_dir.join(rel_file)
                );
                continue;
            }

            log::info!("Downloading related file: {}", rel_url);
            // A missing related file leaves the competition incomplete; abort rather than parse partial data.
            let content = self
                .fetch_with_rate_limit(rel_url.as_str())
                .await
                .with_context(|| format!("failed to download {}", rel_url))?;
            self.save_file(target_dir, rel_file, &content)?;
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

    fn save_file(&self, dir: &Path, filename: &str, content: &str) -> Result<()> {
        let path = dir.join(filename);
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for Scraper {
    fn default() -> Self {
        let client = build_client();
        Self {
            robots_checker: RobotsChecker::with_client(client.clone()),
            client,
            last_request_times: HashMap::new(),
        }
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
            .extract_competition_links("not html at all", "http://example.com/", None)
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
    fn test_crawl_delay_parsing_uppercase_user_agent() {
        let mut checker = RobotsChecker::new();
        let base_url = "http://example.com/";
        let robots_txt = "USER-AGENT:  * \nCRAWL-DELAY: 2";
        checker
            .matchers
            .insert(base_url.to_string(), robots_txt.to_string());

        assert_eq!(checker.get_crawl_delay("http://example.com/x"), Some(2.0));
    }

    #[test]
    fn test_crawl_delay_other_agent_ignored() {
        let mut checker = RobotsChecker::new();
        let base_url = "http://example.com/";
        let robots_txt = "User-agent: somebot\nCrawl-delay: 9";
        checker
            .matchers
            .insert(base_url.to_string(), robots_txt.to_string());

        assert_eq!(checker.get_crawl_delay("http://example.com/"), None);
    }

    #[tokio::test]
    async fn test_fetch_disallowed_by_robots_errors_without_request() {
        let mut scraper = Scraper::new();
        let base_url = "http://robots-blocked.invalid/";
        scraper.robots_checker.matchers.insert(
            base_url.to_string(),
            "User-agent: *\nDisallow: /".to_string(),
        );

        let err = scraper
            .fetch_with_rate_limit("http://robots-blocked.invalid/page.htm")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("robots.txt disallows"));
        // No request was recorded for the domain.
        assert!(scraper.last_request_times.is_empty());
    }

    #[tokio::test]
    async fn test_download_disallowed_competition_errors() {
        let mut scraper = Scraper::new();
        scraper.robots_checker.matchers.insert(
            "http://robots-blocked.invalid/".to_string(),
            "User-agent: *\nDisallow: /".to_string(),
        );
        let dir = std::env::temp_dir().join(format!("ddc_download_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        // Callers must not get Ok for a competition whose files weren't downloaded.
        assert!(scraper
            .download_competition_files("http://robots-blocked.invalid/comp/index.htm", &dir)
            .await
            .is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        let _ = fs::remove_dir_all(&dir);
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
        let links = scraper
            .extract_competition_links(html, base_url, None)
            .unwrap();

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
        assert_eq!(crate::models::sanitize_name("Event Name!"), "Event_Name_");
        assert_eq!(
            crate::models::sanitize_name("A".repeat(100).as_str()).len(),
            64
        );
    }
}
