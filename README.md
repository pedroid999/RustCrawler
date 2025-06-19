# RustCrawler 🕷️

A fast, concurrent web crawler built in Rust with robots.txt compliance, rate limiting, and robust error handling.

## Features

- **Async/Concurrent Crawling**: Built on Tokio with configurable concurrency limits
- **Robots.txt Compliance**: Respects robots.txt rules and crawl-delay directives
- **Rate Limiting**: Global and per-domain rate limiting capabilities
- **Retry Logic**: Exponential backoff retry mechanism for failed requests
- **Proxy Support**: HTTP/HTTPS proxy support
- **URL Deduplication**: Thread-safe URL deduplication to avoid crawling the same page twice
- **Configurable Depth**: Control crawl depth and maximum pages
- **HTML Parsing**: Extracts page titles and follows links
- **Comprehensive Logging**: Detailed logging with configurable verbosity levels

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Build from Source

```bash
git clone <repository-url>
cd RustCrawler
cargo build --release
```

### Install from Cargo

```bash
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Crawl a single website
rustcrawler https://example.com

# Crawl multiple starting URLs
rustcrawler https://example.com https://another-site.com
```

### Advanced Usage

```bash
# Limit concurrency and set rate limit
rustcrawler https://example.com --concurrency 10 --rate 2

# Set maximum pages and depth
rustcrawler https://example.com --max-pages 100 --depth 3

# Use a proxy
rustcrawler https://example.com --proxy http://proxy.example.com:8080

# Custom user agent and timeout
rustcrawler https://example.com --user-agent "MyBot/1.0" --timeout 60

# Verbose logging
rustcrawler https://example.com -vv
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--concurrency` | `-c` | Maximum concurrent requests | 50 |
| `--rate` | `-r` | Rate limit (requests/second) | None |
| `--proxy` | `-p` | Proxy URL | None |
| `--max-pages` | `-m` | Maximum pages to crawl | None |
| `--depth` | `-d` | Maximum crawl depth | None |
| `--user-agent` | `-u` | User agent string | rustcrawler/0.1.0 |
| `--timeout` | | Request timeout (seconds) | 30 |
| `--retries` | | Maximum retries per request | 3 |
| `--verbose` | `-v` | Verbose logging | Info level |
| `--respect-robots` | | Respect robots.txt | true |
| `--follow-redirects` | | Follow HTTP redirects | true |

## Architecture

The crawler is organized into several modules:

- **`cli`**: Command-line argument parsing using Clap
- **`crawler`**: Core crawling logic with concurrency control
- **`robots`**: Robots.txt parsing and compliance
- **`main`**: Application entry point and coordination

### Key Components

#### Concurrency Control
- Uses Tokio's `Semaphore` to limit concurrent requests
- Thread-safe URL deduplication with `DashSet`
- Rate limiting with configurable intervals

#### Robots.txt Compliance
- Fetches and caches robots.txt files per domain
- Respects `User-agent` specific rules
- Honors `Crawl-delay` directives
- Gracefully handles missing or malformed robots.txt

#### Error Handling & Retries
- Exponential backoff for failed requests
- Retries on 5xx status codes and network errors
- Comprehensive error context with `anyhow`

#### HTML Processing
- Extracts page titles from `<title>` tags
- Finds and resolves all links (`<a href>` attributes)
- Converts relative URLs to absolute URLs

## Examples

### Basic Web Crawling

```rust
use rustcrawler::{Crawler, CrawlerConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = CrawlerConfig {
        max_concurrency: 10,
        rate_limit: Some(Duration::from_millis(500)), // 2 req/sec
        max_retries: 3,
        timeout: Duration::from_secs(30),
        user_agent: "MyBot/1.0".to_string(),
        max_pages: Some(50),
        max_depth: Some(2),
        respect_robots: true,
        follow_redirects: true,
        proxy: None,
    };

    let crawler = Crawler::new(config)?;
    let results = crawler.crawl(vec!["https://example.com".to_string()]).await?;

    for result in results {
        println!("{}", result.format_output());
    }

    Ok(())
}
```

### Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test robots::tests
```

### Integration Testing

Test against a local server:

```bash
# Start a test server (if available)
python -m http.server 8000

# Test the crawler
rustcrawler http://localhost:8000 --max-pages 5 -v
```

## Performance Considerations

- **Memory Usage**: The crawler keeps track of visited URLs in memory. For very large crawls, consider implementing disk-based storage.
- **Rate Limiting**: Be respectful of target servers. Use appropriate rate limits and concurrency settings.
- **Network Timeouts**: Adjust timeout values based on target server response times.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

### v0.1.0
- Initial release
- Basic web crawling functionality
- Robots.txt compliance
- Concurrency control and rate limiting
- Retry logic with exponential backoff
- Proxy support
- Comprehensive CLI interface