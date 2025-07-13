use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::crawler::{Crawler, CrawlerConfig};
use crate::robots::RobotsManager;

pub struct CrawlTool {
    crawler: Arc<RwLock<Option<Crawler>>>,
    crawl_results: Arc<RwLock<HashMap<String, Value>>>,
    stats: Arc<RwLock<HashMap<String, u64>>>,
}

impl CrawlTool {
    pub fn new(
        crawler: Arc<RwLock<Option<Crawler>>>,
        crawl_results: Arc<RwLock<HashMap<String, Value>>>,
        stats: Arc<RwLock<HashMap<String, u64>>>,
    ) -> Self {
        Self {
            crawler,
            crawl_results,
            stats,
        }
    }

    pub async fn execute(&self, arguments: Value) -> Result<String> {
        let url = arguments["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        let max_depth = arguments["max_depth"].as_u64().unwrap_or(1) as usize;
        let max_pages = arguments["max_pages"].as_u64().map(|n| n as usize);
        let rate_limit = arguments["rate_limit"].as_f64().unwrap_or(1.0);
        let respect_robots = arguments["respect_robots"].as_bool().unwrap_or(true);
        let follow_redirects = arguments["follow_redirects"].as_bool().unwrap_or(true);

        // Create crawler configuration
        let config = CrawlerConfig {
            max_concurrency: 10,
            user_agent: "RustCrawler-MCP/0.1.0".to_string(),
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            rate_limit: if rate_limit > 0.0 {
                Some(std::time::Duration::from_secs_f64(1.0 / rate_limit))
            } else {
                None
            },
            proxy: None,
            max_pages,
            max_depth: Some(max_depth),
            respect_robots,
            follow_redirects,
        };

        // Initialize crawler if not already done
        {
            let crawler_guard = self.crawler.read().await;
            if crawler_guard.is_none() {
                drop(crawler_guard);
                let crawler = Crawler::new(config)?;
                let mut crawler_guard = self.crawler.write().await;
                *crawler_guard = Some(crawler);
            }
        }

        // Perform crawl
        let crawler_guard = self.crawler.read().await;
        let crawler = crawler_guard.as_ref().unwrap();
        
        let start_time = SystemTime::now();
        let results = crawler.crawl(vec![url.to_string()]).await?;
        let crawl_duration = start_time.elapsed()?.as_secs();

        // Generate session ID and store results
        let session_id = Uuid::new_v4().to_string();
        let crawl_summary = serde_json::json!({
            "session_id": session_id,
            "start_url": url,
            "pages_crawled": results.len(),
            "crawl_duration_seconds": crawl_duration,
            "config": {
                "max_depth": max_depth,
                "max_pages": max_pages,
                "rate_limit": rate_limit,
                "respect_robots": respect_robots,
                "follow_redirects": follow_redirects
            },
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "url": r.url,
                    "status_code": r.status_code,
                    "title": r.title,
                    "links_found": r.links.len(),
                    "crawl_time_ms": r.crawl_time.as_millis(),
                    "depth": r.depth
                })
            }).collect::<Vec<_>>()
        });

        // Store results
        {
            let mut results_guard = self.crawl_results.write().await;
            results_guard.insert(session_id.clone(), crawl_summary.clone());
        }

        // Update stats
        {
            let mut stats_guard = self.stats.write().await;
            *stats_guard.entry("total_crawls".to_string()).or_insert(0) += 1;
            *stats_guard.entry("total_pages_crawled".to_string()).or_insert(0) += results.len() as u64;
            *stats_guard.entry("total_crawl_time_seconds".to_string()).or_insert(0) += crawl_duration;
        }

        Ok(format!(
            "Crawl completed successfully!\n\nSession ID: {}\nPages crawled: {}\nDuration: {}s\n\nUse resource crawl://results/{} to get detailed results.",
            session_id,
            results.len(),
            crawl_duration,
            session_id
        ))
    }
}

pub struct GetRobotsTool;

impl GetRobotsTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, arguments: Value) -> Result<String> {
        let domain = arguments["domain"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: domain"))?;

        // Create a simple HTTP client to fetch robots.txt
        let client = reqwest::Client::new();
        let robots_url = format!("https://{}/robots.txt", domain);
        
        let response = client.get(&robots_url).send().await?;
        
        if response.status().is_success() {
            let content = response.text().await?;
            Ok(format!(
                "Robots.txt for {}:\n\n{}",
                domain,
                content
            ))
        } else {
            Ok(format!(
                "No robots.txt found for {} (HTTP {})",
                domain,
                response.status()
            ))
        }
    }
}

pub struct GetStatsTool {
    stats: Arc<RwLock<HashMap<String, u64>>>,
}

impl GetStatsTool {
    pub fn new(stats: Arc<RwLock<HashMap<String, u64>>>) -> Self {
        Self { stats }
    }

    pub async fn execute(&self, _arguments: Value) -> Result<String> {
        let stats = self.stats.read().await;
        
        if stats.is_empty() {
            Ok("No crawl statistics available yet.".to_string())
        } else {
            let stats_json = serde_json::to_string_pretty(&*stats)?;
            Ok(format!("Current crawl statistics:\n\n{}", stats_json))
        }
    }
}