# Rust Development Tools

Comprehensive Rust development workflow and tooling for the RustCrawler project.

## Purpose

This command provides Rust-specific development workflows, build commands, and best practices for working with the RustCrawler web crawler project.

## Usage

```
/rust-tools
```

## What this command does

1. **Provides Rust build commands** for development and production
2. **Manages testing workflows** including unit and integration tests
3. **Handles code quality** with formatting and linting
4. **Manages dependencies** and crate ecosystem
5. **Supports async/concurrent** development patterns

## Build Commands

### Development Build
```bash
# Fast type checking (no compilation)
cargo check

# Build in debug mode (faster compilation, slower runtime)
cargo build

# Build and run the binary
cargo run

# Run with specific arguments
cargo run -- --url https://example.com --depth 2
```

### Production Build
```bash
# Optimized build for production
cargo build --release

# Run optimized binary
cargo run --release

# Build specific binary target
cargo build --bin rustcrawler --release
```

## Testing Workflows

### Unit Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_robots_parser

# Run tests in specific module
cargo test robots::

# Run doc tests
cargo test --doc
```

### Integration Tests
```bash
# Run only integration tests
cargo test --test integration

# Run specific integration test file
cargo test --test crawler_integration
```

### Async Testing
```bash
# Using tokio-test for async tests
#[tokio::test]
async fn test_concurrent_crawling() {
    // Test async functionality
}
```

## Code Quality Tools

### Formatting
```bash
# Format all code
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Format specific file
cargo fmt src/crawler.rs
```

### Linting with Clippy
```bash
# Run clippy (linter)
cargo clippy

# Clippy with all targets
cargo clippy --all-targets

# Clippy with strict mode
cargo clippy -- -D warnings

# Fix auto-fixable clippy issues
cargo clippy --fix
```

### Documentation
```bash
# Generate documentation
cargo doc

# Generate and open documentation
cargo doc --open

# Include private items in docs
cargo doc --document-private-items
```

## Dependency Management

### Adding Dependencies
```bash
# Add runtime dependency
cargo add reqwest

# Add dev dependency
cargo add --dev tokio-test

# Add dependency with features
cargo add tokio --features full

# Add dependency with specific version
cargo add serde --version "1.0"
```

### Updating Dependencies
```bash
# Update all dependencies
cargo update

# Update specific dependency
cargo update -p reqwest

# Show outdated dependencies (requires cargo-outdated)
cargo outdated
```

### Dependency Information
```bash
# Show dependency tree
cargo tree

# Show licenses of dependencies
cargo tree --format "{p} {l}"

# Check for security advisories (requires cargo-audit)
cargo audit
```

## Project-Specific Patterns

### Web Crawler Architecture
```rust
use tokio::sync::Semaphore;
use dashmap::DashMap;
use anyhow::Result;

// Concurrent crawler with rate limiting
pub struct Crawler {
    client: reqwest::Client,
    visited: DashMap<String, bool>,
    semaphore: Semaphore,
}

impl Crawler {
    pub async fn crawl_url(&self, url: &str) -> Result<CrawlResult> {
        let _permit = self.semaphore.acquire().await?;
        // Implementation here
    }
}
```

### Error Handling Patterns
```rust
use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrawlerError {
    #[error("HTTP request failed")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
    
    #[error("Robots.txt parsing failed")]
    RobotsParseError,
}

// Using anyhow for application errors
pub async fn fetch_page(url: &str) -> Result<String> {
    let response = reqwest::get(url)
        .await
        .context("Failed to fetch page")?;
    
    response.text()
        .await
        .context("Failed to read response body")
}
```

### Async Best Practices
```rust
use tokio::task::JoinSet;
use std::sync::Arc;

// Spawn concurrent tasks
let mut tasks = JoinSet::new();
for url in urls {
    let crawler = Arc::clone(&crawler);
    tasks.spawn(async move {
        crawler.crawl_url(&url).await
    });
}

// Collect results
while let Some(result) = tasks.join_next().await {
    match result? {
        Ok(crawl_result) => println!("Success: {:?}", crawl_result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Performance and Debugging

### Profiling
```bash
# Build with debug symbols for profiling
cargo build --release --debug

# Run with Valgrind (Linux)
valgrind --tool=callgrind target/release/rustcrawler

# Performance testing
cargo bench
```

### Debugging
```bash
# Build with debug info
cargo build

# Run with environment variables for logging
RUST_LOG=debug cargo run

# Run with backtrace on panic
RUST_BACKTRACE=1 cargo run

# Full backtrace
RUST_BACKTRACE=full cargo run
```

## CI/CD Integration

### GitHub Actions Workflow
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

### Pre-commit Hooks
```bash
# Install pre-commit hooks
#!/bin/sh
# .git/hooks/pre-commit

# Format code
cargo fmt --check
if [ $? -ne 0 ]; then
    echo "Code formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
cargo clippy -- -D warnings
if [ $? -ne 0 ]; then
    echo "Clippy check failed. Fix warnings before committing."
    exit 1
fi

# Run tests
cargo test
if [ $? -ne 0 ]; then
    echo "Tests failed. Fix failing tests before committing."
    exit 1
fi
```

## Project-Specific Commands

### RustCrawler Development
```bash
# Run crawler with debug logging
RUST_LOG=rustcrawler=debug cargo run -- --url https://example.com

# Test robots.txt parsing
cargo test robots::tests

# Benchmark crawler performance
cargo bench --bench crawler_bench

# Run integration tests
cargo test --test integration -- --test-threads=1
```

## Common Troubleshooting

### Compilation Issues
```bash
# Clean build artifacts
cargo clean

# Update Rust toolchain
rustup update

# Check for toolchain issues
rustup show

# Verbose compilation
cargo build -v
```

### Dependency Conflicts
```bash
# Show dependency resolution
cargo tree

# Update specific problematic dependency
cargo update -p dependency-name

# Use specific version
# Edit Cargo.toml and run cargo update
```

## Best Practices Checklist

- [ ] Run `cargo fmt` before every commit
- [ ] Ensure `cargo clippy` passes without warnings
- [ ] Write tests for new functionality
- [ ] Use `anyhow::Result<T>` for error handling
- [ ] Add proper logging with the `log` crate
- [ ] Document public APIs with doc comments
- [ ] Use `Arc` and async patterns for concurrency
- [ ] Handle timeouts and rate limiting in network code
- [ ] Validate and sanitize external inputs (URLs)
- [ ] Keep dependencies updated for security