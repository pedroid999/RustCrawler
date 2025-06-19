# RustCrawler Requirements

This document contains the original requirements and specifications for the RustCrawler project.

## Original Prompt

**Code Generation Prompt (for an MVP implementation)**

Create a new Rust project (CLI application) called rustcrawler with the following specifications:

### 1. Async Runtime
Use Tokio (latest 1.x version) with the multi-threaded runtime (`#[tokio::main]`) as the foundation for concurrency.

### 2. HTTP Fetching
Use reqwest to perform HTTP GET requests asynchronously. Retrieve each page's HTTP status code and HTML content. Extract the page's `<title>` text from the HTML.

### 3. HTML Parsing
Use the scraper crate to parse HTML and extract all hyperlinks (`<a href="...">`). Resolve relative URLs using the url crate so they become absolute. Collect these URLs for crawling.

### 4. Robots.txt Compliance
Before crawling a domain, fetch and parse its robots.txt (if available) using a robots.txt parser (e.g., robotparser crate). Do not crawl URLs disallowed by robots.txt for our user agent (use "rustcrawler" or configurable agent string). If Crawl-Delay is specified, ensure delays between requests to that domain as per the value.

### 5. Concurrency & Rate Limiting
Support concurrent crawling with a limit (e.g., default 50 simultaneous requests, configurable via CLI). Use an asynchronous mechanism (such as a semaphore or FuturesUnordered) to enforce this limit. Also implement a global rate limit (requests per second, if specified) to avoid flooding. Ensure polite behavior (e.g., apply per-domain crawl delays from robots.txt).

### 6. Retry with Backoff
On transient failures (network errors or HTTP 5xx/429 responses), automatically retry the request a few times. Use an exponential backoff delay between retries. For example, wait 1s, then 2s, then 4s on subsequent retries. Limit the number of retries (e.g., 3 tries total per URL).

### 7. Proxy Support
If a proxy is specified (via CLI or environment), configure the reqwest client to route requests through the proxy. Support HTTP/HTTPS proxies (and optionally SOCKS5 if easily enabled). Ensure this is optional – no proxy by default.

### 8. URL Deduplication
Maintain a global set of visited URLs to avoid processing the same URL more than once. This set should be concurrency-safe. Use a structure like a DashMap or a Mutex-protected HashSet for thread-safe deduplication. Before scheduling a URL to crawl, check this set.

### 9. CLI Interface
Use Clap (derive API) to parse command-line arguments. Include options for:
- concurrency (`--concurrency`)
- rate limit (`--rate requests/sec`)
- proxy (`--proxy URL`)
- max pages or depth (`--max-pages` or `--depth` to limit crawl)
- user agent (`--user-agent`)

The only required positional argument should be the starting URL(s) to crawl. Provide clear `--help` text and version info.

### 10. Output & Logging
As the crawler runs, output each crawled page's URL, status code, and title to stdout (for example, one line per page: "  - "). Also, log warnings or info to stderr (use log or tracing crate) for events like hitting robots.txt rules or retrying a request. Ensure the output is easily readable. (Exact output format can be simple for MVP.)

### 11. Modular Design
Organize the code into modules for clarity:
- e.g., a cli module for argument parsing (Clap definitions),
- a crawler module for the crawl logic (functions to fetch page, parse links, handle robots, etc.),
- maybe a robots module for robots.txt handling,
- and main.rs to tie it together. 

Use idiomatic Rust error handling (Result with custom error types or anyhow). This makes the code easier to test (each module's functionality can be unit-tested in isolation).

### 12. Testing & Documentation
Include at least a simple test (for example, a unit test for the URL deduplication or a small robots.txt parsing). Use `#[tokio::test]` for async tests. Also, add comments in the code explaining key sections (especially where concurrency or synchronization is involved).

## Implementation Notes

Use idiomatic async/await style throughout. The code should handle graceful shutdown if possible (not mandatory, but try to clean up tasks on completion). Aim for clarity and correctness, then optimize where necessary (the chosen crates are already high-performance). Provide a Cargo.toml with the required dependencies (tokio, reqwest, scraper, clap, robotparser, dashmap, etc.), and ensure the project compiles and runs.

## Implementation Status

✅ **Completed**: All requirements have been successfully implemented in the RustCrawler project.

### Key Deviations from Original Requirements:

1. **Robots.txt Parser**: Due to compatibility issues with the `robotparser` crate, a custom robots.txt parser was implemented that handles the core functionality (User-agent matching, Disallow/Allow rules, and Crawl-delay parsing).

2. **Enhanced CLI**: Additional useful options were added beyond the minimum requirements:
   - `--timeout`: Request timeout configuration
   - `--retries`: Maximum retry attempts
   - `--verbose`: Logging verbosity levels
   - `--respect-robots`: Option to disable robots.txt compliance
   - `--follow-redirects`: Option to control redirect following

3. **Comprehensive Testing**: More extensive test coverage was implemented than the minimum requirement, including tests for CLI validation, robots.txt parsing, HTML parsing, and crawler configuration.

4. **Enhanced Output**: The output format includes additional useful information like response times and comprehensive crawl statistics.

All core requirements have been met or exceeded, providing a robust, production-ready web crawler.