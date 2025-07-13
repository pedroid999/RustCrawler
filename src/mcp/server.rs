use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::crawler::{Crawler, CrawlerConfig};
use crate::mcp::tools::{CrawlTool, GetRobotsTool, GetStatsTool};

#[derive(Clone)]
pub struct RustCrawlerMcpServer {
    crawler: Arc<RwLock<Option<Crawler>>>,
    crawl_results: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    stats: Arc<RwLock<HashMap<String, u64>>>,
}

impl RustCrawlerMcpServer {
    pub fn new() -> Self {
        Self {
            crawler: Arc::new(RwLock::new(None)),
            crawl_results: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize_crawler(&self, config: CrawlerConfig) -> Result<()> {
        let crawler = Crawler::new(config)?;
        let mut crawler_guard = self.crawler.write().await;
        *crawler_guard = Some(crawler);
        Ok(())
    }

    pub async fn handle_tool_call(&self, tool_name: &str, arguments: Value) -> Result<String> {
        match tool_name {
            "crawl_website" => {
                let crawl_tool = CrawlTool::new(
                    self.crawler.clone(),
                    self.crawl_results.clone(),
                    self.stats.clone(),
                );
                crawl_tool.execute(arguments).await
            }
            "get_robots_txt" => {
                let robots_tool = GetRobotsTool::new();
                robots_tool.execute(arguments).await
            }
            "get_crawl_stats" => {
                let stats_tool = GetStatsTool::new(self.stats.clone());
                stats_tool.execute(arguments).await
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    pub async fn get_resource(&self, uri: &str) -> Result<String> {
        if uri.starts_with("crawl://results/") {
            let session_id = uri.strip_prefix("crawl://results/").unwrap();
            let results = self.crawl_results.read().await;
            
            if let Some(result) = results.get(session_id) {
                Ok(serde_json::to_string_pretty(result)?)
            } else {
                Err(anyhow::anyhow!("Crawl session not found: {}", session_id))
            }
        } else if uri == "crawl://stats" {
            let stats = self.stats.read().await;
            Ok(serde_json::to_string_pretty(&*stats)?)
        } else {
            Err(anyhow::anyhow!("Unknown resource: {}", uri))
        }
    }

    pub fn get_tools(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "crawl_website",
                "description": "Crawl a website with specified parameters and return the results",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The starting URL to crawl"
                        },
                        "max_depth": {
                            "type": "number",
                            "description": "Maximum crawl depth (default: 1)"
                        },
                        "max_pages": {
                            "type": "number",
                            "description": "Maximum number of pages to crawl (default: 10)"
                        },
                        "rate_limit": {
                            "type": "number",
                            "description": "Rate limit in requests per second (default: 1)"
                        },
                        "respect_robots": {
                            "type": "boolean",
                            "description": "Whether to respect robots.txt (default: true)"
                        },
                        "follow_redirects": {
                            "type": "boolean",
                            "description": "Whether to follow HTTP redirects (default: true)"
                        }
                    },
                    "required": ["url"]
                }
            }),
            serde_json::json!({
                "name": "get_robots_txt",
                "description": "Fetch and parse robots.txt for a given domain",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "The domain to fetch robots.txt from"
                        }
                    },
                    "required": ["domain"]
                }
            }),
            serde_json::json!({
                "name": "get_crawl_stats",
                "description": "Get statistics about recent crawl operations",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            })
        ]
    }

    pub fn get_resources(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "uriTemplate": "crawl://results/{session_id}",
                "name": "Crawl Results",
                "description": "Access crawl results by session ID",
                "mimeType": "application/json"
            }),
            serde_json::json!({
                "uriTemplate": "crawl://stats",
                "name": "Crawl Statistics",
                "description": "Current crawling statistics and metrics",
                "mimeType": "application/json"
            })
        ]
    }
}

impl Default for RustCrawlerMcpServer {
    fn default() -> Self {
        Self::new()
    }
}