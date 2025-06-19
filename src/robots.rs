use anyhow::{Context, Result};
use dashmap::DashMap;
use log::{debug, warn};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use url::Url;

#[derive(Debug, Clone)]
pub struct RobotsInfo {
    pub content: Option<String>,
    pub crawl_delay: Option<Duration>,
    pub last_accessed: Option<Instant>,
}

impl Default for RobotsInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl RobotsInfo {
    pub fn new() -> Self {
        Self {
            content: None,
            crawl_delay: None,
            last_accessed: None,
        }
    }

    pub fn with_content(content: String) -> Self {
        // Parse crawl delay from robots.txt content
        let crawl_delay = Self::parse_crawl_delay(&content);

        Self {
            content: Some(content),
            crawl_delay,
            last_accessed: None,
        }
    }

    fn parse_crawl_delay(content: &str) -> Option<Duration> {
        for line in content.lines() {
            let line = line.trim().to_lowercase();
            if line.starts_with("crawl-delay:") {
                if let Some(delay_str) = line.split(':').nth(1) {
                    if let Ok(delay) = delay_str.trim().parse::<u64>() {
                        return Some(Duration::from_secs(delay));
                    }
                }
            }
        }
        None
    }

    pub fn can_fetch(&self, user_agent: &str, url: &str) -> bool {
        match &self.content {
            Some(content) => {
                // Simple robots.txt parsing
                self.parse_robots_txt(content, user_agent, url)
            }
            None => true, // If we don't have robots.txt, allow crawling
        }
    }

    fn parse_robots_txt(&self, content: &str, user_agent: &str, url: &str) -> bool {
        let mut in_relevant_section = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Check for User-agent directive
            if line.to_lowercase().starts_with("user-agent:") {
                if let Some(agent) = line.split(':').nth(1) {
                    let current_user_agent = agent.trim().to_lowercase();
                    in_relevant_section = current_user_agent == "*" || 
                                        current_user_agent == user_agent.to_lowercase() ||
                                        user_agent.to_lowercase().contains(&current_user_agent);
                }
                continue;
            }
            
            // Only process rules if we're in a relevant section
            if !in_relevant_section {
                continue;
            }
            
            // Check for Disallow directive
            if line.to_lowercase().starts_with("disallow:") {
                if let Some(path) = line.split(':').nth(1) {
                    let path = path.trim();
                    
                    // Empty disallow means allow everything
                    if path.is_empty() {
                        continue;
                    }
                    
                    // Check if URL matches the disallowed path
                    if let Ok(parsed_url) = Url::parse(url) {
                        let url_path = parsed_url.path();
                        if path == "/" || url_path.starts_with(path) {
                            return false; // URL is disallowed
                        }
                    }
                }
            }
            
            // Check for Allow directive (takes precedence over Disallow)
            if line.to_lowercase().starts_with("allow:") {
                if let Some(path) = line.split(':').nth(1) {
                    let path = path.trim();
                    
                    // Check if URL matches the allowed path
                    if let Ok(parsed_url) = Url::parse(url) {
                        let url_path = parsed_url.path();
                        if path == "/" || url_path.starts_with(path) {
                            return true; // URL is explicitly allowed
                        }
                    }
                }
            }
        }
        
        true // Allow by default
    }

    pub fn should_wait(&self) -> Option<Duration> {
        if let (Some(crawl_delay), Some(last_accessed)) = (self.crawl_delay, self.last_accessed) {
            let elapsed = last_accessed.elapsed();
            if elapsed < crawl_delay {
                Some(crawl_delay - elapsed)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update_last_accessed(&mut self) {
        self.last_accessed = Some(Instant::now());
    }
}

#[derive(Debug)]
pub struct RobotsManager {
    client: Client,
    robots_cache: Arc<DashMap<String, RobotsInfo>>,
    user_agent: String,
}

impl RobotsManager {
    pub fn new(client: Client, user_agent: String) -> Self {
        Self {
            client,
            robots_cache: Arc::new(DashMap::new()),
            user_agent,
        }
    }

    pub async fn check_robots_compliance(&self, url: &Url) -> Result<bool> {
        let host = url.host_str().context("URL has no host")?;
        let domain = format!("{}://{}", url.scheme(), host);

        // Get or fetch robots.txt for this domain
        let robots_info = self.get_or_fetch_robots(&domain).await?;

        // Check if URL is allowed
        let allowed = robots_info.can_fetch(&self.user_agent, url.as_str());
        
        if !allowed {
            debug!("URL blocked by robots.txt: {}", url);
        }

        Ok(allowed)
    }

    pub async fn should_delay(&self, url: &Url) -> Result<Option<Duration>> {
        let host = url.host_str().context("URL has no host")?;
        let domain = format!("{}://{}", url.scheme(), host);

        if let Some(robots_info) = self.robots_cache.get_mut(&domain) {
            Ok(robots_info.should_wait())
        } else {
            Ok(None)
        }
    }

    pub async fn update_last_access(&self, url: &Url) -> Result<()> {
        let host = url.host_str().context("URL has no host")?;
        let domain = format!("{}://{}", url.scheme(), host);

        if let Some(mut robots_info) = self.robots_cache.get_mut(&domain) {
            robots_info.update_last_accessed();
        }

        Ok(())
    }

    async fn get_or_fetch_robots(&self, domain: &str) -> Result<RobotsInfo> {
        // Check cache first
        if let Some(robots_info) = self.robots_cache.get(domain) {
            return Ok(robots_info.clone());
        }

        // Fetch robots.txt
        let robots_info = self.fetch_robots(domain).await?;
        self.robots_cache.insert(domain.to_string(), robots_info.clone());
        
        Ok(robots_info)
    }

    async fn fetch_robots(&self, domain: &str) -> Result<RobotsInfo> {
        let robots_url = format!("{}/robots.txt", domain);
        debug!("Fetching robots.txt from: {}", robots_url);

        match self.client.get(&robots_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(content) => {
                            debug!("Successfully fetched robots.txt for {}", domain);
                            Ok(RobotsInfo::with_content(content))
                        }
                        Err(e) => {
                            warn!("Error reading robots.txt content for {}: {}", domain, e);
                            Ok(RobotsInfo::new())
                        }
                    }
                } else {
                    debug!("robots.txt not found for {} (status: {})", domain, response.status());
                    Ok(RobotsInfo::new()) // No robots.txt means crawling is allowed
                }
            }
            Err(e) => {
                warn!("Error fetching robots.txt for {}: {}", domain, e);
                Ok(RobotsInfo::new()) // Allow crawling if fetch fails
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_robots_info_creation() {
        let info = RobotsInfo::new();
        assert!(info.content.is_none());
        assert!(info.crawl_delay.is_none());
        assert!(info.last_accessed.is_none());
    }

    #[tokio::test]
    async fn test_robots_info_can_fetch_without_content() {
        let info = RobotsInfo::new();
        assert!(info.can_fetch("*", "https://example.com/test"));
    }

    #[tokio::test]
    async fn test_robots_manager_creation() {
        let client = Client::new();
        let manager = RobotsManager::new(client, "test-agent".to_string());
        assert_eq!(manager.user_agent, "test-agent");
        assert_eq!(manager.robots_cache.len(), 0);
    }
}