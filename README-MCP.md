# RustCrawler MCP Server

This document describes the Model Context Protocol (MCP) server functionality for RustCrawler, which allows AI assistants to interact with the web crawler programmatically.

## Overview

The RustCrawler MCP Server provides a standardized interface for AI assistants to:
- Perform web crawling operations
- Access crawl results and statistics
- Fetch and analyze robots.txt files
- Monitor crawling performance

## Installation and Setup

### 1. Build the MCP Server

```bash
# Build the MCP server binary
cargo build --release --bin rustcrawler-mcp

# Test the server functionality
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}' | ./target/debug/rustcrawler-mcp
```

### 2. Configure Claude Desktop

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

### 3. Restart Claude Desktop

After adding the configuration, restart Claude Desktop to load the MCP server.

## Available Tools

### `crawl_website`

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

### `get_robots_txt`

Fetch and parse robots.txt for a given domain.

**Parameters:**
- `domain` (required): The domain to fetch robots.txt from

**Example:**
```json
{
  "domain": "example.com"
}
```

### `get_crawl_stats`

Get statistics about recent crawl operations.

**Parameters:** None

## Available Resources

### `crawl://results/{session_id}`

Access detailed crawl results by session ID. Returns JSON data with:
- Session information
- Configuration used
- List of crawled pages with metadata
- Performance metrics

### `crawl://stats`

Current crawling statistics and metrics including:
- Total number of crawls performed
- Total pages crawled
- Average response times
- Status code distribution

## Usage Examples

### Basic Website Crawling

```
Please crawl the website https://httpbin.org with a maximum depth of 2 and respect robots.txt rules.
```

### Robots.txt Analysis

```
Can you fetch and analyze the robots.txt file for github.com?
```

### Performance Monitoring

```
Show me the current crawling statistics and performance metrics.
```

### Advanced Crawling with Custom Parameters

```
Crawl https://example.com with the following settings:
- Maximum depth: 3
- Maximum pages: 100
- Rate limit: 0.5 requests per second
- Follow redirects: yes
- Respect robots.txt: yes
```

## Technical Details

### Architecture

The MCP server is built using:
- **RMCP SDK**: Official Rust Model Context Protocol implementation
- **Tokio**: Async runtime for high-performance I/O
- **Stdio Transport**: Communication via stdin/stdout as per MCP specification

### Data Flow

1. **Tool Invocation**: AI assistant calls a tool with parameters
2. **Crawler Execution**: MCP server creates/configures crawler instance
3. **Result Storage**: Results stored with unique session ID
4. **Response**: Summary returned to assistant with session reference
5. **Resource Access**: Detailed results available via MCP resources

### Error Handling

The server implements comprehensive error handling:
- Invalid URLs and parameters
- Network timeouts and failures
- Robots.txt compliance violations
- Rate limiting and concurrency controls

### Security Considerations

- **Robots.txt Compliance**: Enabled by default to respect website policies
- **Rate Limiting**: Prevents overwhelming target servers
- **Input Validation**: All parameters validated before execution
- **Resource Limits**: Configurable limits on pages and depth

## Troubleshooting

### Server Not Starting

1. Check that the binary path in configuration is correct
2. Verify the binary has execute permissions
3. Check Claude Desktop logs for error messages

### Tool Calls Failing

1. Enable debug logging with `RUST_LOG=debug`
2. Check network connectivity
3. Verify target website accessibility
4. Review robots.txt compliance

### Performance Issues

1. Adjust rate limiting parameters
2. Reduce maximum pages or depth
3. Check system resource usage
4. Consider proxy configuration for better routing

## Development

### Adding New Tools

1. Implement tool logic in `src/mcp/tools.rs`
2. Add tool definition to `list_tools()` in `src/mcp/server.rs`
3. Add tool execution case in `call_tool()`
4. Update documentation

### Custom Resources

1. Define resource structure in `src/mcp/resources.rs`
2. Add resource template to `list_resources()`
3. Implement resource retrieval in `get_resource()`
4. Update URI patterns and documentation

### Testing

```bash
# Run unit tests
cargo test

# Build and test MCP server
cargo build --bin rustcrawler-mcp
./target/debug/rustcrawler-mcp

# Test with sample MCP client
echo '{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}' | ./target/debug/rustcrawler-mcp
```

## Contributing

When contributing to the MCP server functionality:

1. Follow the existing async patterns
2. Maintain compatibility with MCP specification
3. Add appropriate error handling
4. Update documentation
5. Include tests for new functionality

## License

This MCP server implementation is licensed under the same terms as the main RustCrawler project (MIT License).