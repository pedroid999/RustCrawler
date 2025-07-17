use anyhow::Result;
use env_logger;
use log::{error, info};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use rustcrawler::mcp::RustCrawlerMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr so it doesn't interfere with MCP communication
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stderr)
        .init();

    info!("Starting RustCrawler MCP Server");

    // Create the MCP server
    let server = RustCrawlerMcpServer::new();

    info!("RustCrawler MCP Server is ready to accept connections via stdio");

    // Handle MCP protocol messages via stdin/stdout
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        };
        
        if line.trim().is_empty() {
            continue;
        }

        match handle_mcp_message(&server, &line).await {
            Ok(response) => {
                if !response.is_empty() {
                    if let Err(e) = writeln!(stdout, "{}", response) {
                        error!("Error writing response: {}", e);
                        break;
                    }
                    if let Err(e) = stdout.flush() {
                        error!("Error flushing stdout: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Error handling MCP message: {}", e);
                let request: Result<Value, _> = serde_json::from_str(&line);
                let id = request
                    .as_ref()
                    .ok()
                    .and_then(|r| r.get("id"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": e.to_string()
                    }
                });
                if let Err(e) = writeln!(stdout, "{}", error_response) {
                    error!("Error writing error response: {}", e);
                    break;
                }
                if let Err(e) = stdout.flush() {
                    error!("Error flushing stdout after error: {}", e);
                    break;
                }
            }
        }
    }

    info!("RustCrawler MCP Server shutdown");
    Ok(())
}

async fn handle_mcp_message(server: &RustCrawlerMcpServer, message: &str) -> Result<String> {
    let request: Value = serde_json::from_str(message)?;

    let id = request["id"].clone();
    let method = request["method"].as_str().unwrap_or("");
    let params = &request["params"];

    let result = match method {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "rustcrawler",
                    "version": "0.1.0"
                }
            })
        }
        "tools/list" => {
            json!({
                "tools": server.get_tools()
            })
        }
        "tools/call" => {
            let tool_name = params["name"].as_str().unwrap_or("");
            let arguments = params["arguments"].clone();

            match server.handle_tool_call(tool_name, arguments).await {
                Ok(content) => {
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": content
                            }
                        ]
                    })
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        "resources/list" => {
            json!({
                "resources": server.get_resources()
            })
        }
        "resources/read" => {
            let uri = params["uri"].as_str().unwrap_or("");

            match server.get_resource(uri).await {
                Ok(content) => {
                    json!({
                        "contents": [
                            {
                                "uri": uri,
                                "mimeType": "application/json",
                                "text": content
                            }
                        ]
                    })
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        "prompts/list" => {
            json!({
                "prompts": []
            })
        }
        "notifications/initialized" => {
            // Acknowledge initialization
            return Ok(String::new());
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown method: {}", method));
        }
    };

    let response = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });

    Ok(response.to_string())
}
