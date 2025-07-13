mod cli;
mod crawler;
mod robots;

use crate::cli::CliArgs;
use crate::crawler::{Crawler, CrawlerConfig};
use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::process;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Parse command line arguments
    let args = CliArgs::parse();

    // Validate arguments
    if let Err(e) = args.validate() {
        error!("Invalid arguments: {}", e);
        process::exit(1);
    }

    // Set log level based on verbosity
    match args.verbose {
        0 => log::set_max_level(log::LevelFilter::Info),
        1 => log::set_max_level(log::LevelFilter::Debug),
        _ => log::set_max_level(log::LevelFilter::Trace),
    }

    info!("Starting RustCrawler v0.1.0");
    info!("Configuration:");
    info!("  URLs: {:?}", args.urls);
    info!("  Concurrency: {}", args.concurrency);
    info!("  User Agent: {}", args.user_agent);
    if let Some(rate) = args.rate_limit {
        info!("  Rate Limit: {} req/sec", rate);
    }
    if let Some(max_pages) = args.max_pages {
        info!("  Max Pages: {}", max_pages);
    }
    if let Some(depth) = args.depth {
        info!("  Max Depth: {}", depth);
    }
    if let Some(ref proxy) = args.proxy {
        info!("  Proxy: {}", proxy);
    }

    // Create crawler configuration
    let config = CrawlerConfig {
        max_concurrency: args.concurrency,
        rate_limit: args.get_rate_limit_interval(),
        max_retries: args.max_retries,
        timeout: args.get_timeout(),
        user_agent: args.user_agent.clone(),
        max_pages: args.max_pages,
        max_depth: args.depth,
        respect_robots: args.respect_robots,
        follow_redirects: args.follow_redirects,
        proxy: args.proxy.clone(),
    };

    // Create and run crawler
    match run_crawler(config, args.urls).await {
        Ok(results) => {
            info!("Crawling completed successfully!");
            info!("Total pages crawled: {}", results.len());

            // Print summary statistics
            let total_time: u128 = results.iter().map(|r| r.crawl_time.as_millis()).sum();
            let avg_time = if !results.is_empty() {
                total_time / results.len() as u128
            } else {
                0
            };

            info!("Average response time: {}ms", avg_time);

            // Count status codes
            let mut status_counts = std::collections::HashMap::new();
            for result in &results {
                *status_counts.entry(result.status_code).or_insert(0) += 1;
            }

            info!("Status code distribution:");
            for (status, count) in status_counts {
                info!("  {}: {}", status, count);
            }
        }
        Err(e) => {
            error!("Crawling failed: {}", e);
            process::exit(1);
        }
    }
}

async fn run_crawler(
    config: CrawlerConfig,
    start_urls: Vec<String>,
) -> Result<Vec<crawler::CrawlResult>> {
    // Create crawler
    let crawler = Crawler::new(config)?;

    // Start crawling
    info!("Starting crawl from {} URL(s)", start_urls.len());
    let results = crawler.crawl(start_urls).await?;

    info!("Crawl statistics:");
    info!("  Pages crawled: {}", crawler.get_crawled_count());
    info!("  URLs visited: {}", crawler.get_visited_count());

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawler_config_creation() {
        let config = CrawlerConfig {
            max_concurrency: 10,
            rate_limit: None,
            max_retries: 3,
            timeout: std::time::Duration::from_secs(30),
            user_agent: "test-agent".to_string(),
            max_pages: Some(100),
            max_depth: Some(3),
            respect_robots: true,
            follow_redirects: true,
            proxy: None,
        };

        let _urls = ["https://httpbin.org/html".to_string()];

        // This test just verifies the crawler can be created
        // Actual network tests would be more complex and require setup
        let result = Crawler::new(config);
        assert!(result.is_ok());
    }
}
