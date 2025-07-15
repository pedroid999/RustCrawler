# RustCrawler 🕷️ with MCP Support

A fast, concurrent web crawler built in Rust with robots.txt compliance, rate limiting, robust error handling, and **Model Context Protocol (MCP) support** for AI assistant integration.

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
- **MCP Server**: Built-in Model Context Protocol server for AI assistant integration

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

## Model Context Protocol (MCP) Server

RustCrawler includes a built-in MCP server that allows AI assistants to interact with the web crawler programmatically.

### MCP Server Features

The MCP server provides:
- **Web Crawling Operations**: Perform crawls with configurable parameters
- **Robots.txt Analysis**: Fetch and analyze robots.txt files
- **Crawl Statistics**: Access performance metrics and crawl data
- **Session Management**: UUID-based session tracking for result retrieval

### Installation and Setup

#### 1. Build the MCP Server

```bash
# Build the MCP server binary
cargo build --release --bin rustcrawler-mcp

# Test the server functionality
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}' | ./target/debug/rustcrawler-mcp
```

#### 2. Configure Claude Desktop

Add the following to your Claude Desktop MCP configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "rustcrawler": {
      "command": "/path/to/your/project/target/release/rustcrawler-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

#### 3. Restart Claude Desktop

After adding the configuration, restart Claude Desktop to load the MCP server.

### Available MCP Tools

#### `crawl_website`
Crawl a website with specified parameters and return results.

**Parameters:**
- `url` (required): Starting URL to crawl
- `max_depth` (optional): Maximum crawl depth (default: 1)
- `max_pages` (optional): Maximum number of pages to crawl (default: 10)
- `rate_limit` (optional): Rate limit in requests per second (default: 1)
- `respect_robots` (optional): Whether to respect robots.txt (default: true)
- `follow_redirects` (optional): Whether to follow HTTP redirects (default: true)

**Example:**
```json
{
  "url": "https://example.com",
  "max_depth": 2,
  "max_pages": 50,
  "rate_limit": 2.0,
  "respect_robots": true,
  "follow_redirects": true
}
```

#### `get_robots_txt`
Fetch and parse robots.txt for a given domain.

**Parameters:**
- `domain` (required): The domain to fetch robots.txt from

**Example:**
```json
{
  "domain": "example.com"
}
```

#### `get_crawl_stats`
Get statistics about recent crawl operations.

**Parameters:** None

### Available MCP Resources

#### `crawl://results/{session_id}`
Access detailed crawl results by session ID. Returns JSON data with:
- Session information
- Configuration used
- List of crawled pages with metadata
- Performance metrics

#### `crawl://stats`
Current crawling statistics and metrics including:
- Total number of crawls performed
- Total pages crawled
- Average response times
- Status code distribution

### MCP Usage Examples

#### Basic Website Crawling
```
Please crawl the website https://httpbin.org with a maximum depth of 2 and respect robots.txt rules.
```

#### Robots.txt Analysis
```
Can you fetch and analyze the robots.txt file for github.com?
```

#### Performance Monitoring
```
Show me the current crawling statistics and performance metrics.
```

#### Advanced Crawling with Custom Parameters
```
Crawl https://example.com with the following settings:
- Maximum depth: 3
- Maximum pages: 100
- Rate limit: 0.5 requests per second
- Follow redirects: yes
- Respect robots.txt: yes
```

### MCP Technical Details

#### Architecture
The MCP server is built using:
- **Native JSON-RPC**: Direct protocol implementation for maximum compatibility
- **Tokio**: Async runtime for high-performance I/O
- **Stdio Transport**: Communication via stdin/stdout as per MCP specification

#### Data Flow
1. **Tool Invocation**: AI assistant calls a tool with parameters
2. **Crawler Execution**: MCP server creates/configures crawler instance
3. **Result Storage**: Results stored with unique session ID
4. **Response**: Summary returned to assistant with session reference
5. **Resource Access**: Detailed results available via MCP resources

#### Error Handling
The server implements comprehensive error handling:
- Invalid URLs and parameters
- Network timeouts and failures
- Robots.txt compliance violations
- Rate limiting and concurrency controls

#### Security Considerations
- **Robots.txt Compliance**: Enabled by default to respect website policies
- **Rate Limiting**: Prevents overwhelming target servers
- **Input Validation**: All parameters validated before execution
- **Resource Limits**: Configurable limits on pages and depth

### MCP Troubleshooting

#### Server Not Starting
1. Check that the binary path in configuration is correct
2. Verify the binary has execute permissions
3. Check Claude Desktop logs for error messages

#### Tool Calls Failing
1. Enable debug logging with `RUST_LOG=debug`
2. Check network connectivity
3. Verify target website accessibility
4. Review robots.txt compliance

#### Performance Issues
1. Adjust rate limiting parameters
2. Reduce maximum pages or depth
3. Check system resource usage
4. Consider proxy configuration for better routing

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
- Model Context Protocol (MCP) server support