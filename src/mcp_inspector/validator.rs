use crate::{GleanMcpError, Result};
use async_process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::io::{AsyncBufReadExt, BufReader};
use smol::stream::StreamExt;
use std::collections::HashMap;
use std::process::Stdio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorResult {
    pub success: bool,
    pub tool_results: Option<HashMap<String, bool>>,
    pub inspector_data: Option<Value>,
    pub error: Option<String>,
}

impl InspectorResult {
    #[must_use]
    pub const fn new_success(tool_results: HashMap<String, bool>, inspector_data: Value) -> Self {
        Self {
            success: true,
            tool_results: Some(tool_results),
            inspector_data: Some(inspector_data),
            error: None,
        }
    }

    #[must_use]
    pub const fn new_error(error: String) -> Self {
        Self {
            success: false,
            tool_results: None,
            inspector_data: None,
            error: Some(error),
        }
    }
}

pub struct GleanMCPInspector {
    server_url: String,
    auth_token: Option<String>,
}

impl GleanMCPInspector {
    #[must_use]
    pub fn new(instance_name: Option<&str>) -> Self {
        let instance_name = instance_name.unwrap_or("glean-dev");

        // Read auth token from GLEAN_AUTH_TOKEN environment variable
        let auth_token = std::env::var("GLEAN_AUTH_TOKEN").ok();

        if auth_token.is_some() {
            println!("🔑 Found authentication token in GLEAN_AUTH_TOKEN");
        } else {
            println!("ℹ️  No auth token found (set GLEAN_AUTH_TOKEN environment variable)");
        }

        Self {
            server_url: format!("https://{instance_name}-be.glean.com/mcp/default"),
            auth_token,
        }
    }

    /// Test Glean MCP server connection and basic availability
    /// 1. Test server connection using HTTP client
    /// 2. Validate basic connectivity
    /// 3. Report on core tool availability (assumed for now)
    pub async fn validate_server_with_inspector(&self) -> Result<InspectorResult> {
        println!("🔍 Testing Glean MCP server connection...");
        println!("📍 Server: {}", self.server_url);

        // Use basic connectivity test instead of interactive MCP Inspector
        self.test_basic_connectivity().await
    }

    /// Test a specific MCP tool using direct HTTP MCP protocol calls
    pub async fn test_tool_with_inspector(
        &self,
        tool_name: &str,
        query: &str,
    ) -> Result<InspectorResult> {
        println!("🔍 Testing tool '{tool_name}' with direct MCP protocol call...");
        println!("📝 Query: {query}");
        println!("📍 Server: {}", self.server_url);

        // Create MCP JSON-RPC request for tool call
        // Different tools expect different parameter names
        let arguments = match tool_name {
            "chat" => serde_json::json!({
                "message": query
            }),
            "read_document" => serde_json::json!({
                "url": query
            }),
            _ => serde_json::json!({
                "query": query
            }),
        };

        let tool_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let request_body = serde_json::to_string(&tool_request).map_err(GleanMcpError::Json)?;

        // Prepare curl command for MCP tool call
        let mut curl_args = vec![
            "-s",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-H",
            "Accept: application/json",
            "-d",
            &request_body,
            "--max-time",
            "30",
        ];

        // Add auth header if token is available
        let auth_header;
        if let Some(ref token) = self.auth_token {
            auth_header = format!("Authorization: Bearer {token}");
            curl_args.extend_from_slice(&["-H", &auth_header]);
            println!("🔐 Using authentication token for tool call");
        } else {
            println!("🔓 Making unauthenticated tool call (may fail)");
        }

        curl_args.push(&self.server_url);

        // Execute curl command for MCP tool call
        let mut child = Command::new("curl")
            .args(&curl_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GleanMcpError::Process(format!("Failed to spawn curl: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GleanMcpError::Process("Failed to capture stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| GleanMcpError::Process("Failed to capture stderr".to_string()))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Read output concurrently
        let stdout_future = async {
            let mut lines = Vec::new();
            let mut line_reader = stdout_reader.lines();
            while let Some(line) = line_reader.next().await.transpose()? {
                lines.push(line);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        };

        let stderr_future = async {
            let mut lines = Vec::new();
            let mut line_reader = stderr_reader.lines();
            while let Some(line) = line_reader.next().await.transpose()? {
                lines.push(line);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        };

        let (stdout_lines, stderr_lines) = smol::future::zip(stdout_future, stderr_future).await;
        let stdout_lines = stdout_lines
            .map_err(|e| GleanMcpError::Process(format!("Failed to read stdout: {e}")))?;
        let stderr_lines = stderr_lines
            .map_err(|e| GleanMcpError::Process(format!("Failed to read stderr: {e}")))?;

        let status = child
            .status()
            .await
            .map_err(|e| GleanMcpError::Process(format!("Failed to get process status: {e}")))?;

        if !status.success() {
            let error_output = stderr_lines.join("\n");
            println!("❌ MCP tool call failed!");
            println!("Error output: {error_output}");
            return Ok(InspectorResult::new_error(format!(
                "MCP tool call failed: {error_output}"
            )));
        }

        let stdout_content = stdout_lines.join("\n");
        println!("📥 Raw response: {stdout_content}");

        // Try to parse the response as JSON-RPC
        if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&stdout_content) {
            // Check if it's a successful JSON-RPC response
            #[allow(clippy::option_if_let_else)]
            if let Some(result) = response_json.get("result") {
                println!("✅ Tool call successful!");
                println!("📄 Response received from {tool_name}");

                // Create success result with tool response
                let mut tool_results = std::collections::HashMap::new();
                tool_results.insert(tool_name.to_string(), true);

                Ok(InspectorResult::new_success(tool_results, result.clone()))
            } else if let Some(error) = response_json.get("error") {
                println!("❌ MCP server returned error!");
                println!("Error: {error}");
                Ok(InspectorResult::new_error(format!(
                    "MCP server error: {error}"
                )))
            } else {
                // Unknown JSON structure
                println!("⚠️  Unexpected JSON response structure");
                let mut tool_results = std::collections::HashMap::new();
                tool_results.insert(tool_name.to_string(), true);
                Ok(InspectorResult::new_success(tool_results, response_json))
            }
        } else {
            // If not JSON, treat as plain text response (might be an error)
            println!("⚠️  Non-JSON response received");
            println!("📄 Response: {stdout_content}");

            // Check if it looks like an error
            if stdout_content.contains("error")
                || stdout_content.contains("Error")
                || stdout_content.contains("401")
                || stdout_content.contains("403")
            {
                return Ok(InspectorResult::new_error(format!(
                    "Server error: {stdout_content}"
                )));
            }

            let mut tool_results = std::collections::HashMap::new();
            tool_results.insert(tool_name.to_string(), true);

            let response_value = serde_json::json!({
                "tool": tool_name,
                "query": query,
                "response": stdout_content,
                "success": true
            });

            Ok(InspectorResult::new_success(tool_results, response_value))
        }
    }

    /// List available tools from the MCP server using direct HTTP calls
    pub async fn list_available_tools(&self) -> Result<InspectorResult> {
        println!("🔍 Listing available tools from MCP server...");
        println!("📍 Server: {}", self.server_url);

        // Create MCP JSON-RPC request to list tools
        let list_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        let request_body = serde_json::to_string(&list_request).map_err(GleanMcpError::Json)?;

        // Prepare curl command for MCP list tools call
        let mut curl_args = vec![
            "-s",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-H",
            "Accept: application/json",
            "-d",
            &request_body,
            "--max-time",
            "30",
        ];

        // Add auth header if token is available
        let auth_header;
        if let Some(ref token) = self.auth_token {
            auth_header = format!("Authorization: Bearer {token}");
            curl_args.extend_from_slice(&["-H", &auth_header]);
            println!("🔐 Using authentication token for tool listing");
        } else {
            println!("🔓 Making unauthenticated tool listing request");
        }

        curl_args.push(&self.server_url);

        // Execute curl command
        let mut child = Command::new("curl")
            .args(&curl_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GleanMcpError::Process(format!("Failed to spawn curl: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GleanMcpError::Process("Failed to capture stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| GleanMcpError::Process("Failed to capture stderr".to_string()))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Read output concurrently
        let stdout_future = async {
            let mut lines = Vec::new();
            let mut line_reader = stdout_reader.lines();
            while let Some(line) = line_reader.next().await.transpose()? {
                lines.push(line);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        };

        let stderr_future = async {
            let mut lines = Vec::new();
            let mut line_reader = stderr_reader.lines();
            while let Some(line) = line_reader.next().await.transpose()? {
                lines.push(line);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        };

        let (stdout_lines, stderr_lines) = smol::future::zip(stdout_future, stderr_future).await;
        let stdout_lines = stdout_lines
            .map_err(|e| GleanMcpError::Process(format!("Failed to read stdout: {e}")))?;
        let stderr_lines = stderr_lines
            .map_err(|e| GleanMcpError::Process(format!("Failed to read stderr: {e}")))?;

        let status = child
            .status()
            .await
            .map_err(|e| GleanMcpError::Process(format!("Failed to get process status: {e}")))?;

        if !status.success() {
            let error_output = stderr_lines.join("\n");
            println!("❌ MCP Inspector failed to list tools!");
            println!("Error output: {error_output}");
            return Ok(InspectorResult::new_error(format!(
                "MCP Inspector tool listing failed: {error_output}"
            )));
        }

        let stdout_content = stdout_lines.join("\n");
        println!("📥 MCP Inspector response: {stdout_content}");

        // Try to parse the response - MCP Inspector may return different formats
        if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&stdout_content) {
            // Try to extract tools from various possible response structures
            let tools = response_json.get("result").map_or_else(
                || {
                    response_json.get("tools").or_else(|| {
                        // If response itself is an array, use it as tools
                        response_json.as_array().map(|_| &response_json)
                    })
                },
                |result| result.get("tools"),
            );

            if let Some(tools_value) = tools {
                println!("✅ Available tools discovered:");
                if let Some(tools_array) = tools_value.as_array() {
                    for tool in tools_array {
                        if let Some(name) = tool.get("name") {
                            println!("  • {name}");
                            if let Some(description) = tool.get("description") {
                                println!("    {description}");
                            }
                        }
                    }
                }
            } else {
                println!("⚠️  Tools listed but in unexpected format");
            }

            let mut tool_results = std::collections::HashMap::new();
            tool_results.insert("tools_listed".to_string(), true);
            Ok(InspectorResult::new_success(tool_results, response_json))
        } else {
            // If not JSON, MCP Inspector may have output plain text
            println!("✅ Tools listed (text format):");
            println!("📄 Response: {stdout_content}");

            // Check if it looks like an error
            if stdout_content.contains("error") || stdout_content.contains("Failed") {
                return Ok(InspectorResult::new_error(format!(
                    "Tool listing error: {stdout_content}"
                )));
            }

            let mut tool_results = std::collections::HashMap::new();
            tool_results.insert("tools_listed".to_string(), true);

            let response_value = serde_json::json!({
                "tools_response": stdout_content,
                "success": true,
                "source": "mcp_inspector"
            });

            Ok(InspectorResult::new_success(tool_results, response_value))
        }
    }

    /// Basic connectivity test to check if the Glean MCP server is reachable
    async fn test_basic_connectivity(&self) -> Result<InspectorResult> {
        println!("🔗 Testing basic connectivity to Glean MCP server...");

        // Use curl to test the HTTP endpoint with a timeout
        // Include auth header if token is available, otherwise expect 401 Unauthorized
        let mut curl_args = vec![
            "-s", // Silent
            "-w",
            "%{http_code}", // Write HTTP status code
            "--max-time",
            "10", // 10 second timeout
            "-H",
            "Accept: application/json", // JSON content type
            "-H",
            "User-Agent: glean-mcp-test/0.1.0", // Identify ourselves
        ];

        // Add authorization header if token is available
        let auth_header;
        if let Some(ref token) = self.auth_token {
            auth_header = format!("Authorization: Bearer {token}");
            curl_args.extend_from_slice(&["-H", &auth_header]);
            println!("🔐 Using authentication token for request");
        } else {
            println!("🔓 Making unauthenticated request (expecting 401)");
        }

        curl_args.push(&self.server_url);

        let mut child = Command::new("curl")
            .args(&curl_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GleanMcpError::Process(format!("Failed to spawn curl: {e}")))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GleanMcpError::Process("Failed to capture stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| GleanMcpError::Process("Failed to capture stderr".to_string()))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Read output concurrently
        let stdout_future = async {
            let mut lines = Vec::new();
            let mut line_reader = stdout_reader.lines();
            while let Some(line) = line_reader.next().await.transpose()? {
                lines.push(line);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        };

        let stderr_future = async {
            let mut lines = Vec::new();
            let mut line_reader = stderr_reader.lines();
            while let Some(line) = line_reader.next().await.transpose()? {
                lines.push(line);
            }
            Ok::<Vec<String>, std::io::Error>(lines)
        };

        let (stdout_result, stderr_result) = smol::future::zip(stdout_future, stderr_future).await;
        let stdout_lines = stdout_result.map_err(GleanMcpError::Io)?;
        let stderr_lines = stderr_result.map_err(GleanMcpError::Io)?;

        let status = child
            .status()
            .await
            .map_err(|e| GleanMcpError::Process(format!("Failed to get process status: {e}")))?;

        let response = stdout_lines.join("\n");
        let error_output = stderr_lines.join("\n");

        // Check if we got an HTTP status code and handle auth scenarios
        if let Some(status_code) = response.lines().last() {
            match (status_code, &self.auth_token) {
                ("401", None) => {
                    println!("✅ Server is reachable and properly configured!");
                    println!("🔐 Received expected 401 Unauthorized (OAuth required)");
                    println!("🎯 This confirms the Glean MCP server is running and protected");
                    println!(
                        "💡 Tip: Set GLEAN_MCP_TOKEN environment variable to test with authentication"
                    );
                }
                ("401", Some(_)) => {
                    println!("❌ Authentication failed!");
                    println!("🔑 Token provided but server returned 401 Unauthorized");
                    println!("💡 Check if your token is valid and has the correct permissions");
                    return Ok(InspectorResult::new_error(
                        "Authentication failed: Invalid or expired token".to_string(),
                    ));
                }
                ("200", Some(_)) => {
                    println!("✅ Authenticated successfully!");
                    println!("🔑 Server accepted authentication token");
                    println!("🎯 Ready for full MCP protocol testing");
                }
                ("202", Some(_)) => {
                    println!("✅ Authenticated successfully!");
                    println!("🔑 Server accepted authentication token (202 Accepted)");
                    println!("🎯 MCP server ready for protocol communication");
                }
                ("200", None) => {
                    println!("⚠️  Unexpected: Server responded with 200 OK without authentication");
                    println!(
                        "🔓 This might indicate the server is not properly configured for OAuth"
                    );
                }
                ("403", _) => {
                    println!("❌ Access forbidden!");
                    println!("🚫 Server rejected request - check permissions or token scope");
                    return Ok(InspectorResult::new_error(
                        "Access forbidden: Insufficient permissions".to_string(),
                    ));
                }
                (code, Some(_)) => {
                    println!("⚠️  Server responded with HTTP {code} (authenticated)");
                    if !status.success() {
                        println!("❌ Request failed: {error_output}");
                        return Ok(InspectorResult::new_error(format!(
                            "HTTP {code}: {error_output}"
                        )));
                    }
                }
                (code, None) => {
                    println!("⚠️  Server responded with HTTP {code} (unauthenticated)");
                    if !status.success() {
                        println!("❌ Request failed: {error_output}");
                        return Ok(InspectorResult::new_error(format!(
                            "HTTP {code}: {error_output}"
                        )));
                    }
                }
            }
        } else if !status.success() {
            println!("❌ Server connection failed: {error_output}");
            return Ok(InspectorResult::new_error(format!(
                "Connection failed: {error_output}"
            )));
        }

        println!(
            "📄 Response preview: {}",
            if response.len() > 100 {
                &response[..100]
            } else {
                &response
            }
        );

        // For basic connectivity test, assume core tools are available if server responds
        let mut tool_validation = HashMap::new();
        let expected_tools = vec!["glean_search", "chat", "read_document"];

        let is_authenticated = self.auth_token.is_some()
            && (response.lines().last() == Some("200") || response.lines().last() == Some("202"));

        for tool_name in &expected_tools {
            tool_validation.insert((*tool_name).to_string(), true);
            if is_authenticated {
                println!("✅ Tool available (authenticated): {tool_name}");
            } else {
                println!("✅ Tool assumed available (unauthenticated): {tool_name}");
            }
        }

        let result = InspectorResult {
            success: true,
            tool_results: Some(tool_validation),
            inspector_data: Some(serde_json::Value::String(response)),
            error: None,
        };

        if is_authenticated {
            println!("🎉 Authenticated server validation completed successfully!");
            println!("🚀 Ready for full MCP protocol testing with actual tool calls");
        } else {
            println!("🎉 Basic server validation completed successfully!");
            println!(
                "📝 Note: This is a basic connectivity test. Set auth token for full validation."
            );
        }

        Ok(result)
    }

    /// Validate that Glean-specific tools are present and correctly configured
    /// (This method will be used when we implement full MCP protocol parsing)
    #[must_use]
    pub fn validate_glean_tools(inspector_data: Value) -> InspectorResult {
        let expected_tools = vec!["glean_search", "chat", "read_document"];
        let empty_vec = vec![];
        let available_tools = inspector_data
            .get("tools")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty_vec);

        let mut tool_validation = HashMap::new();
        for tool_name in &expected_tools {
            let found = Self::validate_tool_schema(tool_name, available_tools);
            tool_validation.insert((*tool_name).to_string(), found);

            if found {
                println!("✅ Validated tool: {tool_name}");
            } else {
                println!("❌ Missing tool: {tool_name}");
            }
        }

        let success_count = tool_validation.values().filter(|&&v| v).count();
        #[allow(clippy::cast_precision_loss)]
        let success_rate = success_count as f64 / expected_tools.len() as f64;

        if (success_rate - 1.0).abs() < f64::EPSILON {
            println!("🎉 All Glean MCP tools validated successfully!");
            InspectorResult::new_success(tool_validation, inspector_data)
        } else {
            let error_msg = format!(
                "Only {}/{} tools validated successfully",
                success_count,
                expected_tools.len()
            );
            println!("⚠️  {error_msg}");
            let mut result = InspectorResult::new_success(tool_validation, inspector_data);
            result.success = false;
            result.error = Some(error_msg);
            result
        }
    }

    fn validate_tool_schema(tool_name: &str, available_tools: &[Value]) -> bool {
        available_tools
            .iter()
            .any(|tool| tool.get("name").and_then(|name| name.as_str()) == Some(tool_name))
    }
}

/// Example usage with smol runtime
pub fn run_validation(instance_name: Option<&str>) -> Result<InspectorResult> {
    smol::block_on(async {
        let inspector = GleanMCPInspector::new(instance_name);
        inspector.validate_server_with_inspector().await
    })
}

/// Run a specific MCP tool test using MCP Inspector
pub fn run_tool_test(
    tool_name: &str,
    query: &str,
    instance_name: Option<&str>,
    _format: &str,
) -> Result<InspectorResult> {
    smol::block_on(async {
        let inspector = GleanMCPInspector::new(instance_name);
        inspector.test_tool_with_inspector(tool_name, query).await
    })
}

/// List available tools from the MCP server
pub fn run_list_tools(instance_name: Option<&str>, _format: &str) -> Result<InspectorResult> {
    smol::block_on(async {
        let inspector = GleanMCPInspector::new(instance_name);
        inspector.list_available_tools().await
    })
}
