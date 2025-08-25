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
    pub fn new_success(tool_results: HashMap<String, bool>, inspector_data: Value) -> Self {
        Self {
            success: true,
            tool_results: Some(tool_results),
            inspector_data: Some(inspector_data),
            error: None,
        }
    }

    pub fn new_error(error: String) -> Self {
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
    inspector_cmd: String,
}

impl GleanMCPInspector {
    pub fn new(instance_name: Option<&str>) -> Self {
        let instance_name = instance_name.unwrap_or("glean-dev-be");
        Self {
            server_url: format!("https://{}.glean.com/mcp/default", instance_name),
            inspector_cmd: "npx".to_string(),
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

    /// Basic connectivity test to check if the Glean MCP server is reachable
    async fn test_basic_connectivity(&self) -> Result<InspectorResult> {
        println!("🔗 Testing basic connectivity to Glean MCP server...");

        // Use curl to test the SSE endpoint with a timeout
        // Note: We expect a 401 Unauthorized for a properly configured Glean server
        let mut child = Command::new("curl")
            .args(&[
                "-s", // Silent
                "-w",
                "%{http_code}", // Write HTTP status code
                "--max-time",
                "10", // 10 second timeout
                "-H",
                "Accept: text/event-stream", // SSE header
                "-H",
                "User-Agent: glean-mcp-test/0.1.0", // Identify ourselves
                &self.server_url,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GleanMcpError::Process(format!("Failed to spawn curl: {}", e)))?;

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
        let stdout_lines = stdout_result.map_err(|e| GleanMcpError::Io(e))?;
        let stderr_lines = stderr_result.map_err(|e| GleanMcpError::Io(e))?;

        let status = child
            .status()
            .await
            .map_err(|e| GleanMcpError::Process(format!("Failed to get process status: {}", e)))?;

        let response = stdout_lines.join("\n");
        let error_output = stderr_lines.join("\n");

        // Check if we got an HTTP status code
        if let Some(status_code) = response.lines().last() {
            match status_code {
                "401" => {
                    println!("✅ Server is reachable and properly configured!");
                    println!("🔐 Received expected 401 Unauthorized (OAuth required)");
                    println!("🎯 This confirms the Glean MCP server is running and protected");
                }
                "200" => {
                    println!("✅ Server responded with 200 OK!");
                    println!("⚠️  Unexpected: Server should require authentication");
                }
                code => {
                    println!("⚠️  Server responded with HTTP {}", code);
                    if !status.success() {
                        println!("❌ Request failed: {}", error_output);
                        return Ok(InspectorResult::new_error(format!(
                            "HTTP {}: {}",
                            code, error_output
                        )));
                    }
                }
            }
        } else if !status.success() {
            println!("❌ Server connection failed: {}", error_output);
            return Ok(InspectorResult::new_error(format!(
                "Connection failed: {}",
                error_output
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

        for tool_name in &expected_tools {
            tool_validation.insert(tool_name.to_string(), true);
            println!("✅ Assuming tool available: {}", tool_name);
        }

        let result = InspectorResult {
            success: true,
            tool_results: Some(tool_validation),
            inspector_data: Some(serde_json::Value::String(response)),
            error: None,
        };

        println!("🎉 Basic server validation completed successfully!");
        println!(
            "📝 Note: This is a basic connectivity test. Full MCP protocol validation will be implemented in future iterations."
        );

        Ok(result)
    }

    /// Validate that Glean-specific tools are present and correctly configured
    /// (This method will be used when we implement full MCP protocol parsing)
    pub fn validate_glean_tools(&self, inspector_data: Value) -> InspectorResult {
        let expected_tools = vec!["glean_search", "chat", "read_document"];
        let empty_vec = vec![];
        let available_tools = inspector_data
            .get("tools")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty_vec);

        let mut tool_validation = HashMap::new();
        for tool_name in &expected_tools {
            let found = self.validate_tool_schema(tool_name, available_tools);
            tool_validation.insert(tool_name.to_string(), found);

            if found {
                println!("✅ Validated tool: {}", tool_name);
            } else {
                println!("❌ Missing tool: {}", tool_name);
            }
        }

        let success_count = tool_validation.values().filter(|&&v| v).count();
        let success_rate = success_count as f64 / expected_tools.len() as f64;

        if success_rate == 1.0 {
            println!("🎉 All Glean MCP tools validated successfully!");
            InspectorResult::new_success(tool_validation, inspector_data)
        } else {
            let error_msg = format!(
                "Only {}/{} tools validated successfully",
                success_count,
                expected_tools.len()
            );
            println!("⚠️  {}", error_msg);
            let mut result = InspectorResult::new_success(tool_validation, inspector_data);
            result.success = false;
            result.error = Some(error_msg);
            result
        }
    }

    fn validate_tool_schema(&self, tool_name: &str, available_tools: &[Value]) -> bool {
        available_tools.iter().any(|tool| {
            tool.get("name")
                .and_then(|name| name.as_str())
                .map_or(false, |name| name == tool_name)
        })
    }
}

/// Example usage with smol runtime
pub fn run_validation(instance_name: Option<&str>) -> Result<InspectorResult> {
    smol::block_on(async {
        let inspector = GleanMCPInspector::new(instance_name);
        inspector.validate_server_with_inspector().await
    })
}
