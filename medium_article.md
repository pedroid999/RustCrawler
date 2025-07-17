# Building a High-Performance Web Crawler in Rust with MCP Support: A Senior Developer's Guide

*A comprehensive walkthrough of RustCrawler - a production-ready web crawler built with Rust, featuring Model Context Protocol integration for AI assistant workflows*

## Introduction

In the rapidly evolving landscape of web scraping and data collection, the need for efficient, scalable, and respectful web crawlers has never been greater. Today, I'm excited to share **RustCrawler**, a high-performance web crawler built in Rust that not only delivers exceptional speed and safety but also integrates seamlessly with modern AI assistant workflows through the Model Context Protocol (MCP).

This article will take you through the journey of building a production-ready web crawler, highlighting the architectural decisions, implementation details, and modern integrations that make RustCrawler a compelling choice for senior developers working on data collection and analysis projects.

## Project Overview

RustCrawler is a fast, concurrent web crawler designed with safety, performance, and compliance in mind. Built entirely in Rust, it leverages the language's memory safety guarantees and excellent async/await ecosystem to deliver reliable crawling capabilities at scale.

### Core Features

- **🚀 Async/Concurrent Architecture**: Built on Tokio runtime with configurable concurrency limits
- **🤖 Robots.txt Compliance**: Respects robots.txt rules and crawl-delay directives
- **⚡ Rate Limiting**: Global and per-domain rate limiting to be respectful to servers
- **🔄 Retry Logic**: Exponential backoff retry mechanism for resilient crawling
- **🌐 Proxy Support**: HTTP/HTTPS proxy integration for enterprise environments
- **🔒 URL Deduplication**: Thread-safe URL deduplication using DashMap
- **📊 MCP Integration**: Built-in Model Context Protocol server for AI assistant workflows

## Architecture Deep Dive

### Module Organization

The crawler is organized into four main modules, each with distinct responsibilities:

```
src/
├── cli.rs          # Command-line interface and argument parsing
├── crawler.rs      # Core crawling logic with concurrency control
├── robots.rs       # Robots.txt parsing and compliance engine
├── mcp/            # Model Context Protocol server implementation
│   ├── mod.rs      # MCP module exports
│   ├── server.rs   # Core MCP server logic
│   ├── tools.rs    # MCP tool implementations
│   └── resources.rs # MCP resource management
└── main.rs         # Application entry point
```

### Key Architectural Decisions

#### 1. Concurrency Model

The crawler uses Tokio's `Semaphore` to control concurrent requests, ensuring we don't overwhelm target servers:

```rust
// From the architecture - controlled concurrency
let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
```

#### 2. Memory-Safe URL Deduplication

Using `DashMap` for thread-safe URL deduplication without traditional locking overhead:

```rust
// Thread-safe URL tracking
let visited_urls = Arc::new(DashSet::new());
```

#### 3. Respectful Crawling

Built-in robots.txt compliance and rate limiting ensure ethical crawling practices:

```rust
// Robots.txt compliance built-in
let robots_manager = RobotsManager::new();
if !robots_manager.can_fetch(&url, &user_agent).await? {
    return Err(CrawlError::RobotsForbidden);
}
```

## Step-by-Step Implementation Guide

### Step 1: Project Setup and Dependencies

First, let's examine the project structure and dependencies:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
scraper = "0.20"
clap = { version = "4.0", features = ["derive"] }
dashmap = "6.0"
url = "2.5"
anyhow = "1.0"
thiserror = "1.0"
# ... additional dependencies for logging, JSON, and async traits
```

### Step 2: Command Line Interface Design

The CLI is built using Clap's derive macros for type-safe argument parsing:

```bash
# Basic crawling
rustcrawler https://example.com

# Advanced configuration
rustcrawler https://example.com \
    --concurrency 10 \
    --rate 2 \
    --max-pages 100 \
    --depth 3 \
    --proxy http://proxy.example.com:8080
```

### Step 3: Core Crawler Implementation

The crawler implementation focuses on:

1. **Configuration Management**: Centralized configuration structure
2. **Async Task Management**: Efficient task spawning and coordination
3. **Error Handling**: Comprehensive error types with context
4. **Result Aggregation**: Collecting and formatting crawl results

### Step 4: Robots.txt Compliance Engine

A dedicated module handles robots.txt parsing and compliance:

- Fetches and caches robots.txt files per domain
- Parses User-agent specific rules
- Honors Crawl-delay directives
- Gracefully handles malformed or missing robots.txt

### Step 5: HTML Processing and Link Extraction

The crawler processes HTML content to:
- Extract page titles from `<title>` tags
- Find and resolve all links (`<a href>` attributes)
- Convert relative URLs to absolute URLs
- Filter and validate discovered URLs

## Model Context Protocol Integration

### What is MCP?

The Model Context Protocol (MCP) is a standardized protocol that allows AI assistants to interact with external tools and services. RustCrawler's MCP integration makes it possible for AI assistants like Claude to perform web crawling operations programmatically.

### MCP Server Architecture

The MCP server is implemented as a separate binary (`rustcrawler-mcp`) that communicates via stdin/stdout:

```rust
// MCP server entry point
#[tokio::main]
async fn main() -> Result<()> {
    let server = RustCrawlerMcpServer::new();
    
    // Handle MCP protocol messages via stdin/stdout
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        match handle_mcp_message(&server, &line).await {
            Ok(response) => {
                writeln!(stdout, "{}", response)?;
                stdout.flush()?;
            }
            Err(e) => error!("MCP error: {}", e),
        }
    }
    
    Ok(())
}
```

### Available MCP Tools

1. **`crawl_website`**: Perform web crawling with configurable parameters
2. **`get_robots_txt`**: Fetch and analyze robots.txt files
3. **`get_crawl_stats`**: Access performance metrics and statistics

### MCP Resources

- **`crawl://results/{session_id}`**: Access detailed crawl results
- **`crawl://stats`**: Current crawling statistics and metrics

## Claude Code Configuration and Development Experience

### CLAUDE.md Configuration

The project includes a comprehensive `CLAUDE.md` file that defines development guidelines and workflow for AI-assisted development:

```markdown
# Rust-Specific Guidelines

### Build and Test Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo clippy` - Run linter (required before commits)
- `cargo fmt` - Format code (required before commits)
- `cargo check` - Type check without building

### Development Workflow
- Always run `cargo fmt` and `cargo clippy` before committing
- Use `cargo check` frequently during development
- Run full test suite with `cargo test` before pushing
```

### Claude Code Template Integration

This project leverages the excellent work from the [Claude Code Templates](https://github.com/davila7/claude-code-templates) repository, which provides:

- **Standardized Project Structure**: Consistent organization patterns
- **Development Workflow Templates**: Pre-configured development practices
- **Quality Assurance Integration**: Built-in linting and formatting rules
- **Documentation Standards**: Comprehensive documentation guidelines

The templates significantly accelerate development by providing:
1. **Consistent Code Style**: Automated formatting and linting
2. **Best Practice Enforcement**: Built-in quality checks
3. **Documentation Templates**: Structured documentation approach
4. **Development Workflow**: Standardized Git workflow and commit practices

## Performance Considerations for Production

### Memory Management

The crawler keeps visited URLs in memory using `DashSet`. For large-scale crawls, consider:

```rust
// Memory-efficient URL tracking for large crawls
let visited_urls = Arc::new(DashSet::with_capacity(expected_url_count));
```

### Rate Limiting Strategy

Implement respectful crawling with appropriate delays:

```rust
// Configurable rate limiting
let rate_limiter = RateLimiter::new(
    Duration::from_millis(1000 / config.requests_per_second)
);
```

### Concurrency Tuning

Balance performance with server respect:

```rust
// Optimal concurrency configuration
let config = CrawlerConfig {
    max_concurrency: 50,        // Adjust based on target server capacity
    rate_limit: Some(Duration::from_millis(500)), // 2 req/sec
    timeout: Duration::from_secs(30),
    max_retries: 3,
    // ... other configuration
};
```

## Integration with Modern AI Workflows

### Claude Desktop Integration

Configure Claude Desktop to use the MCP server:

```json
{
  "mcpServers": {
    "rustcrawler": {
      "command": "/path/to/rustcrawler-mcp",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### AI Assistant Usage Examples

```
# Natural language crawling requests
"Please crawl https://example.com with a depth of 2 and respect robots.txt"

# Advanced configuration
"Crawl the site with these settings: max 100 pages, 0.5 req/sec rate limit"

# Analysis requests
"Analyze the robots.txt file for github.com and show me the restrictions"
```

## Testing and Quality Assurance

### Comprehensive Testing Strategy

```bash
# Run all tests
cargo test

# Test with output for debugging
cargo test -- --nocapture

# Test specific modules
cargo test robots::tests
```

### Integration Testing

```bash
# Test against local server
python -m http.server 8000
rustcrawler http://localhost:8000 --max-pages 5 -v
```

### Code Quality Checks

```bash
# Required before commits
cargo fmt
cargo clippy
cargo test
```

## Security and Compliance

### Built-in Security Features

1. **Robots.txt Compliance**: Enabled by default
2. **Rate Limiting**: Prevents server overwhelming
3. **Input Validation**: All parameters validated
4. **Resource Limits**: Configurable bounds on pages and depth

### Best Practices Implementation

- No sensitive data in error messages
- Proper error context with `anyhow`
- Secure proxy configuration support
- Timeout protection for hanging requests

## Deployment and Distribution

### Building for Production

```bash
# Release build with optimizations
cargo build --release

# Install locally
cargo install --path .

# Build MCP server
cargo build --release --bin rustcrawler-mcp
```

### Docker Integration

The project structure supports containerization with proper multi-stage builds for optimal image sizes.

## Future Enhancements and Roadmap

### Planned Features

1. **Disk-based URL Storage**: For massive crawls exceeding memory limits
2. **Distributed Crawling**: Multi-node crawler coordination
3. **Advanced Content Processing**: JavaScript rendering capabilities
4. **Enhanced MCP Resources**: More detailed analytics and reporting
5. **Plugin Architecture**: Extensible processing pipeline

### Community Contributions

The project welcomes contributions in:
- Performance optimizations
- Additional MCP tools and resources
- Enhanced robots.txt compliance
- Documentation improvements
- Test coverage expansion

## Conclusion

RustCrawler represents a modern approach to web crawling, combining Rust's performance and safety guarantees with contemporary AI assistant integration. The project demonstrates how traditional tools can be enhanced with protocols like MCP to create seamless workflows for modern development practices.

For senior developers, RustCrawler offers:

1. **Production-Ready Architecture**: Robust error handling and concurrency management
2. **Modern Integration**: Native MCP support for AI assistant workflows
3. **Ethical Crawling**: Built-in compliance and rate limiting
4. **Extensible Design**: Clean module structure for future enhancements
5. **Quality Assurance**: Comprehensive testing and code quality tools

The combination of Rust's performance characteristics, thoughtful architecture decisions, and modern protocol support makes RustCrawler an excellent choice for data collection projects requiring both speed and reliability.

## References and Acknowledgments

### Project Repository
- **GitHub**: [RustCrawler Project](https://github.com/your-username/RustCrawler)
- **Documentation**: [Project README](https://github.com/your-username/RustCrawler/blob/main/README.md)

### Special Thanks
- **Claude Code Templates**: [davila7/claude-code-templates](https://github.com/davila7/claude-code-templates) - Invaluable templates and best practices that accelerated development
- **Rust Community**: For excellent crates and ecosystem support
- **Tokio Team**: For the exceptional async runtime
- **Model Context Protocol**: For the protocol specification and tooling

### Technical Resources
- [Tokio Documentation](https://tokio.rs/)
- [Reqwest HTTP Client](https://docs.rs/reqwest/)
- [Scraper HTML Parser](https://docs.rs/scraper/)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/)

---

*This article was written with assistance from Claude Code, demonstrating the practical integration of AI assistants in modern development workflows.*

**About the Author**: Pedro Nieto is a senior software engineer with expertise in system programming, web technologies, and AI integration. You can find more of his work on [GitHub](https://github.com/pedrete999).

**License**: This project is licensed under the MIT License - see the [LICENSE](https://github.com/your-username/RustCrawler/blob/main/LICENSE) file for details.