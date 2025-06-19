use clap::Parser;
use std::time::Duration;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rustcrawler",
    version = "0.1.0",
    about = "A fast, concurrent web crawler built in Rust",
    long_about = "RustCrawler is a high-performance web crawler that respects robots.txt, supports concurrent crawling with rate limiting, and includes retry mechanisms with exponential backoff."
)]
pub struct CliArgs {
    #[arg(
        help = "Starting URL(s) to crawl",
        required = true,
        value_name = "URL"
    )]
    pub urls: Vec<String>,

    #[arg(
        short = 'c',
        long = "concurrency",
        help = "Maximum number of concurrent requests",
        default_value = "50",
        value_name = "NUM"
    )]
    pub concurrency: usize,

    #[arg(
        short = 'r',
        long = "rate",
        help = "Rate limit in requests per second",
        value_name = "NUM"
    )]
    pub rate_limit: Option<f64>,

    #[arg(
        short = 'p',
        long = "proxy",
        help = "Proxy URL (http://user:pass@host:port or socks5://host:port)",
        value_name = "URL"
    )]
    pub proxy: Option<String>,

    #[arg(
        short = 'm',
        long = "max-pages",
        help = "Maximum number of pages to crawl",
        value_name = "NUM"
    )]
    pub max_pages: Option<usize>,

    #[arg(
        short = 'd',
        long = "depth",
        help = "Maximum crawl depth",
        value_name = "NUM"
    )]
    pub depth: Option<usize>,

    #[arg(
        short = 'u',
        long = "user-agent",
        help = "User agent string to use",
        default_value = "rustcrawler/0.1.0",
        value_name = "STRING"
    )]
    pub user_agent: String,

    #[arg(
        long = "timeout",
        help = "Request timeout in seconds",
        default_value = "30",
        value_name = "SECONDS"
    )]
    pub timeout: u64,

    #[arg(
        long = "retries",
        help = "Maximum number of retries per request",
        default_value = "3",
        value_name = "NUM"
    )]
    pub max_retries: usize,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Enable verbose logging",
        action = clap::ArgAction::Count
    )]
    pub verbose: u8,

    #[arg(
        long = "respect-robots",
        help = "Respect robots.txt rules",
        default_value = "true",
        action = clap::ArgAction::SetTrue
    )]
    pub respect_robots: bool,

    #[arg(
        long = "follow-redirects",
        help = "Follow HTTP redirects",
        default_value = "true",
        action = clap::ArgAction::SetTrue
    )]
    pub follow_redirects: bool,
}

impl CliArgs {
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate URLs
        for url_str in &self.urls {
            Url::parse(url_str)
                .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", url_str, e))?;
        }

        // Validate proxy URL if provided
        if let Some(proxy_url) = &self.proxy {
            Url::parse(proxy_url)
                .map_err(|e| anyhow::anyhow!("Invalid proxy URL '{}': {}", proxy_url, e))?;
        }

        // Validate concurrency
        if self.concurrency == 0 {
            return Err(anyhow::anyhow!("Concurrency must be greater than 0"));
        }

        // Validate rate limit
        if let Some(rate) = self.rate_limit {
            if rate <= 0.0 {
                return Err(anyhow::anyhow!("Rate limit must be greater than 0"));
            }
        }

        // Validate max pages
        if let Some(max_pages) = self.max_pages {
            if max_pages == 0 {
                return Err(anyhow::anyhow!("Max pages must be greater than 0"));
            }
        }

        // Validate depth
        if let Some(depth) = self.depth {
            if depth == 0 {
                return Err(anyhow::anyhow!("Depth must be greater than 0"));
            }
        }

        Ok(())
    }

    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }

    pub fn get_rate_limit_interval(&self) -> Option<Duration> {
        self.rate_limit.map(|rate| {
            let requests_per_second = rate;
            let interval_ms = (1000.0 / requests_per_second) as u64;
            Duration::from_millis(interval_ms)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_args() {
        let args = CliArgs {
            urls: vec!["https://example.com".to_string()],
            concurrency: 10,
            rate_limit: Some(5.0),
            proxy: Some("http://proxy.example.com:8080".to_string()),
            max_pages: Some(100),
            depth: Some(3),
            user_agent: "test-agent".to_string(),
            timeout: 30,
            max_retries: 3,
            verbose: 0,
            respect_robots: true,
            follow_redirects: true,
        };

        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_url() {
        let args = CliArgs {
            urls: vec!["not-a-valid-url".to_string()],
            concurrency: 10,
            rate_limit: None,
            proxy: None,
            max_pages: None,
            depth: None,
            user_agent: "test-agent".to_string(),
            timeout: 30,
            max_retries: 3,
            verbose: 0,
            respect_robots: true,
            follow_redirects: true,
        };

        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_zero_concurrency() {
        let args = CliArgs {
            urls: vec!["https://example.com".to_string()],
            concurrency: 0,
            rate_limit: None,
            proxy: None,
            max_pages: None,
            depth: None,
            user_agent: "test-agent".to_string(),
            timeout: 30,
            max_retries: 3,
            verbose: 0,
            respect_robots: true,
            follow_redirects: true,
        };

        assert!(args.validate().is_err());
    }
}