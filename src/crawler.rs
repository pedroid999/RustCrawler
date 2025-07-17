use crate::robots::RobotsManager;
use anyhow::{Context, Result};
use dashmap::DashSet;
use futures::future::join_all;
use log::{debug, error, info, warn};
use reqwest::{Client, Proxy, Response, StatusCode};
use scraper::{Html, Selector};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use url::Url;

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub url: String,
    pub status_code: u16,
    pub title: Option<String>,
    pub links: Vec<String>,
    pub depth: usize,
    pub crawl_time: Duration,
}

impl CrawlResult {
    pub fn format_output(&self) -> String {
        let title = self.title.as_deref().unwrap_or("No title");
        format!(
            "{} - {} - {} ({}ms)",
            self.url,
            self.status_code,
            title,
            self.crawl_time.as_millis()
        )
    }
}

#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub max_concurrency: usize,
    pub rate_limit: Option<Duration>,
    pub max_retries: usize,
    pub timeout: Duration,
    pub user_agent: String,
    pub max_pages: Option<usize>,
    pub max_depth: Option<usize>,
    pub respect_robots: bool,
    pub follow_redirects: bool,
    pub proxy: Option<String>,
}

pub struct Crawler {
    client: Client,
    config: CrawlerConfig,
    visited_urls: Arc<DashSet<String>>,
    pages_crawled: Arc<AtomicUsize>,
    semaphore: Arc<Semaphore>,
    robots_manager: RobotsManager,
}

impl Crawler {
    pub fn new(config: CrawlerConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        if config.follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(10));
        } else {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::none());
        }

        if let Some(proxy_url) = &config.proxy {
            let proxy = Proxy::all(proxy_url).context("Failed to create proxy")?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder
            .build()
            .context("Failed to build HTTP client")?;

        let robots_manager = RobotsManager::new(client.clone(), config.user_agent.clone());

        Ok(Self {
            client,
            visited_urls: Arc::new(DashSet::new()),
            pages_crawled: Arc::new(AtomicUsize::new(0)),
            semaphore: Arc::new(Semaphore::new(config.max_concurrency)),
            robots_manager,
            config,
        })
    }

    pub async fn crawl(&self, start_urls: Vec<String>) -> Result<Vec<CrawlResult>> {
        let mut results = Vec::new();
        let mut current_urls: Vec<(String, usize)> =
            start_urls.into_iter().map(|url| (url, 0)).collect();

        while !current_urls.is_empty() {
            // Check if we've reached max pages limit
            if let Some(max_pages) = self.config.max_pages {
                if self.pages_crawled.load(Ordering::Relaxed) >= max_pages {
                    info!("Reached maximum pages limit: {}", max_pages);
                    break;
                }
            }

            // Filter out already visited URLs and apply depth limit
            let urls_to_crawl: Vec<_> = current_urls
                .into_iter()
                .filter(|(url, depth)| {
                    if self.visited_urls.contains(url) {
                        return false;
                    }
                    if let Some(max_depth) = self.config.max_depth {
                        if *depth > max_depth {
                            return false;
                        }
                    }
                    true
                })
                .collect();

            if urls_to_crawl.is_empty() {
                break;
            }

            // Crawl URLs concurrently
            let futures: Vec<_> = urls_to_crawl
                .into_iter()
                .map(|(url, depth)| self.crawl_single_url(url, depth))
                .collect();

            let batch_results = join_all(futures).await;
            let mut next_urls = Vec::new();

            for result in batch_results {
                match result {
                    Ok(crawl_result) => {
                        // Collect links for next depth level
                        if let Some(max_depth) = self.config.max_depth {
                            if crawl_result.depth < max_depth {
                                for link in &crawl_result.links {
                                    next_urls.push((link.clone(), crawl_result.depth + 1));
                                }
                            }
                        } else {
                            for link in &crawl_result.links {
                                next_urls.push((link.clone(), crawl_result.depth + 1));
                            }
                        }

                        debug!("Crawled: {}", crawl_result.url);
                        results.push(crawl_result);
                    }
                    Err(e) => {
                        error!("Crawl error: {}", e);
                    }
                }
            }

            current_urls = next_urls;
        }

        Ok(results)
    }

    async fn crawl_single_url(&self, url: String, depth: usize) -> Result<CrawlResult> {
        // Acquire semaphore permit for concurrency control
        let _permit = self
            .semaphore
            .acquire()
            .await
            .context("Failed to acquire semaphore permit")?;

        // Rate limiting
        if let Some(rate_interval) = self.config.rate_limit {
            sleep(rate_interval).await;
        }

        // Mark URL as visited
        self.visited_urls.insert(url.clone());

        let start_time = Instant::now();
        let parsed_url = Url::parse(&url).context("Failed to parse URL")?;

        // Check robots.txt compliance
        if self.config.respect_robots {
            if !self
                .robots_manager
                .check_robots_compliance(&parsed_url)
                .await?
            {
                return Err(anyhow::anyhow!("URL blocked by robots.txt: {}", url));
            }

            // Check if we need to delay due to crawl-delay
            if let Some(delay) = self.robots_manager.should_delay(&parsed_url).await? {
                debug!("Applying crawl delay of {:?} for {}", delay, url);
                sleep(delay).await;
            }
        }

        // Perform HTTP request with retries
        let response = self
            .fetch_with_retries(&url, self.config.max_retries)
            .await?;
        let status_code = response.status().as_u16();

        // Update last access time for robots.txt compliance
        if self.config.respect_robots {
            self.robots_manager.update_last_access(&parsed_url).await?;
        }

        // Parse HTML content
        let html_content = response
            .text()
            .await
            .context("Failed to read response body")?;

        let (title, links) = self.parse_html(&html_content, &parsed_url)?;

        // Increment pages crawled counter
        self.pages_crawled.fetch_add(1, Ordering::Relaxed);

        let crawl_time = start_time.elapsed();

        Ok(CrawlResult {
            url,
            status_code,
            title,
            links,
            depth,
            crawl_time,
        })
    }

    async fn fetch_with_retries(&self, url: &str, max_retries: usize) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status = response.status();

                    // Check if we should retry based on status code
                    if (status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS)
                        && attempt < max_retries
                    {
                        let delay = Duration::from_secs(2_u64.pow(attempt as u32));
                        warn!(
                            "HTTP {} for {}, retrying in {:?} (attempt {}/{})",
                            status,
                            url,
                            delay,
                            attempt + 1,
                            max_retries + 1
                        );
                        sleep(delay).await;
                        continue;
                    }

                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        let delay = Duration::from_secs(2_u64.pow(attempt as u32));
                        warn!(
                            "Request failed for {}, retrying in {:?} (attempt {}/{}): {}",
                            url,
                            delay,
                            attempt + 1,
                            max_retries + 1,
                            last_error.as_ref().unwrap()
                        );
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to fetch {} after {} attempts: {}",
            url,
            max_retries + 1,
            last_error.unwrap()
        ))
    }

    fn parse_html(&self, html: &str, base_url: &Url) -> Result<(Option<String>, Vec<String>)> {
        let document = Html::parse_document(html);

        // Extract title
        let title_selector = Selector::parse("title")
            .map_err(|e| anyhow::anyhow!("Failed to parse title selector: {}", e))?;

        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty());

        // Extract links
        let link_selector = Selector::parse("a[href]")
            .map_err(|e| anyhow::anyhow!("Failed to parse link selector: {}", e))?;

        let mut links = Vec::new();
        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Resolve relative URLs to absolute URLs
                match base_url.join(href) {
                    Ok(absolute_url) => {
                        let url_str = absolute_url.to_string();
                        // Only include HTTP/HTTPS URLs
                        if absolute_url.scheme() == "http" || absolute_url.scheme() == "https" {
                            links.push(url_str);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to resolve URL {}: {}", href, e);
                    }
                }
            }
        }

        // Remove duplicates while preserving order
        links.sort();
        links.dedup();

        Ok((title, links))
    }

    pub fn get_crawled_count(&self) -> usize {
        self.pages_crawled.load(Ordering::Relaxed)
    }

    pub fn get_visited_count(&self) -> usize {
        self.visited_urls.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawler_creation() {
        let config = CrawlerConfig {
            max_concurrency: 10,
            rate_limit: None,
            max_retries: 3,
            timeout: Duration::from_secs(30),
            user_agent: "test-agent".to_string(),
            max_pages: Some(100),
            max_depth: Some(3),
            respect_robots: true,
            follow_redirects: true,
            proxy: None,
        };

        let crawler = Crawler::new(config);
        assert!(crawler.is_ok());
    }

    #[test]
    fn test_parse_html_basic() {
        let config = CrawlerConfig {
            max_concurrency: 10,
            rate_limit: None,
            max_retries: 3,
            timeout: Duration::from_secs(30),
            user_agent: "test-agent".to_string(),
            max_pages: Some(100),
            max_depth: Some(3),
            respect_robots: true,
            follow_redirects: true,
            proxy: None,
        };

        let crawler = Crawler::new(config).unwrap();
        let html = r#"
            <html>
                <head><title>Test Page</title></head>
                <body>
                    <a href="/page1">Link 1</a>
                    <a href="https://example.com/page2">Link 2</a>
                </body>
            </html>
        "#;

        let base_url = Url::parse("https://example.com").unwrap();
        let (title, links) = crawler.parse_html(html, &base_url).unwrap();

        assert_eq!(title, Some("Test Page".to_string()));
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com/page1".to_string()));
        assert!(links.contains(&"https://example.com/page2".to_string()));
    }
}
