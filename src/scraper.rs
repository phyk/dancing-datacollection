use robotstxt::DefaultMatcher;
use std::collections::HashMap;
use url::Url;
use anyhow::Result;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::path::Path;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub sources: Sources,
}

#[derive(Debug, Deserialize)]
pub struct Sources {
    pub urls: Vec<String>,
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

        let base_url = format!("{}://{}/", url.scheme(), url.host_str().unwrap_or(""));
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
}

pub struct Scraper {
    client: reqwest::Client,
    robots_checker: RobotsChecker,
}

impl Scraper {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            robots_checker: RobotsChecker::new(),
        }
    }

    pub async fn scrape_all(&mut self, config: &Config) -> Result<()> {
        for base_url in &config.sources.urls {
            if !self.robots_checker.is_allowed(base_url).await {
                log::warn!("robots.txt disallows scraping {}, skipping", base_url);
                continue;
            }

            log::info!("Scraping base URL: {}", base_url);
            let html_content = self.client.get(base_url).send().await?.text().await?;
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
        let html_content = self.client.get(url_str).send().await?.text().await?;
        let event_name = self.extract_event_name(&html_content)?;
        let sanitized_event_name = self.sanitize_name(&event_name);

        let data_dir = Path::new("data").join(&sanitized_event_name);
        fs::create_dir_all(&data_dir)?;

        let filename = Path::new(url_str).file_name().and_then(|n| n.to_str()).unwrap_or("index.htm");
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
            match self.client.get(rel_url.as_str()).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let content = resp.text().await?;
                        self.save_file(&data_dir, rel_file, &content)?;
                    }
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
            Ok(title_elem.text().collect::<Vec<_>>().join(" ").trim().to_string())
        } else {
            Ok("unknown_event".to_string())
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        let mut sanitized: String = name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
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
