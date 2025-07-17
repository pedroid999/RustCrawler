use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSession {
    pub session_id: String,
    pub start_url: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub status: CrawlStatus,
    pub pages_crawled: usize,
    pub total_links_found: usize,
    pub config: CrawlSessionConfig,
    pub results: Vec<CrawlPageResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrawlStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSessionConfig {
    pub max_depth: usize,
    pub max_pages: Option<usize>,
    pub rate_limit: f64,
    pub respect_robots: bool,
    pub follow_redirects: bool,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlPageResult {
    pub url: String,
    pub status_code: u16,
    pub title: Option<String>,
    pub content_length: Option<usize>,
    pub links_found: Vec<String>,
    pub crawl_time_ms: u128,
    pub depth: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlStatistics {
    pub total_sessions: u64,
    pub total_pages_crawled: u64,
    pub total_crawl_time_seconds: u64,
    pub average_pages_per_session: f64,
    pub average_crawl_time_per_page_ms: f64,
    pub status_code_distribution: HashMap<u16, u64>,
    pub top_domains: HashMap<String, u64>,
    pub recent_sessions: Vec<String>,
}

impl Default for CrawlStatistics {
    fn default() -> Self {
        Self {
            total_sessions: 0,
            total_pages_crawled: 0,
            total_crawl_time_seconds: 0,
            average_pages_per_session: 0.0,
            average_crawl_time_per_page_ms: 0.0,
            status_code_distribution: HashMap::new(),
            top_domains: HashMap::new(),
            recent_sessions: Vec::new(),
        }
    }
}

impl CrawlStatistics {
    pub fn update_from_session(&mut self, session: &CrawlSession) {
        self.total_sessions += 1;
        self.total_pages_crawled += session.pages_crawled as u64;

        if let Some(end_time) = session.end_time {
            let duration = end_time.saturating_sub(session.start_time);
            self.total_crawl_time_seconds += duration;
        }

        // Update averages
        if self.total_sessions > 0 {
            self.average_pages_per_session =
                self.total_pages_crawled as f64 / self.total_sessions as f64;
        }

        if self.total_pages_crawled > 0 && self.total_crawl_time_seconds > 0 {
            self.average_crawl_time_per_page_ms =
                (self.total_crawl_time_seconds * 1000) as f64 / self.total_pages_crawled as f64;
        }

        // Update status code distribution
        for result in &session.results {
            *self
                .status_code_distribution
                .entry(result.status_code)
                .or_insert(0) += 1;
        }

        // Update top domains
        if let Ok(url) = url::Url::parse(&session.start_url) {
            if let Some(domain) = url.domain() {
                *self.top_domains.entry(domain.to_string()).or_insert(0) += 1;
            }
        }

        // Update recent sessions (keep last 10)
        self.recent_sessions.push(session.session_id.clone());
        if self.recent_sessions.len() > 10 {
            self.recent_sessions.remove(0);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsInfo {
    pub domain: String,
    pub robots_txt_url: String,
    pub content: Option<String>,
    pub last_fetched: u64,
    pub status: RobotsStatus,
    pub parsed_rules: Vec<RobotsRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RobotsStatus {
    Available,
    NotFound,
    Forbidden,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotsRule {
    pub user_agent: String,
    pub disallowed_paths: Vec<String>,
    pub allowed_paths: Vec<String>,
    pub crawl_delay: Option<f64>,
    pub sitemap_urls: Vec<String>,
}
