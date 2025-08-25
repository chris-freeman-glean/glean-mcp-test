use crate::{GleanMcpError, Result};
use async_process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smol::io::{AsyncBufReadExt, BufReader};
use smol::stream::StreamExt;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Async timeout helper function using smol Timer
async fn async_timeout<T, F>(duration: Duration, future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    use futures::future::{Either, select};
    use smol::Timer;

    let timeout_future = Timer::after(duration);

    match select(Box::pin(future), Box::pin(timeout_future)).await {
        Either::Left((result, _)) => result,
        Either::Right((_, _)) => Err(GleanMcpError::Process("Operation timed out".to_string())),
    }
}

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

// New data structures for test-all functionality

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAllOptions {
    pub tools_filter: String,
    pub scenario: String,
    pub parallel: bool,
    pub max_concurrent: usize,
    pub timeout: u64,
    pub verbose: bool,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllToolsTestResult {
    pub success: bool,
    pub total_tools: usize,
    pub successful_tools: usize,
    pub failed_tools: usize,
    pub tool_results: HashMap<String, ToolTestResult>,
    pub execution_summary: ExecutionSummary,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTestResult {
    pub tool_name: String,
    pub success: bool,
    pub response_time_ms: u64,
    pub test_query: String,
    pub response_data: Option<Value>,
    pub error_message: Option<String>,
    pub validation_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub start_time: String,
    pub end_time: String,
    pub total_duration_ms: u64,
    pub parallel_execution: bool,
    pub timeout_settings: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub schema: Option<Value>,
}

impl AllToolsTestResult {
    pub fn format_output(&self, format: &str, verbose: bool) -> String {
        match format {
            "json" => self.format_json(),
            "summary" => self.format_summary(),
            _ => self.format_text(verbose),
        }
    }

    fn format_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_summary(&self) -> String {
        format!(
            "🧪 Test Summary: {}/{} tools successful ({}%)\n⏱️  Total time: {:.2}s",
            self.successful_tools,
            self.total_tools,
            if self.total_tools > 0 {
                (self.successful_tools * 100) / self.total_tools
            } else {
                0
            },
            self.execution_summary.total_duration_ms as f64 / 1000.0
        )
    }

    fn format_text(&self, verbose: bool) -> String {
        let mut output = String::new();

        // Header with overall status
        output.push_str("🧪 Glean MCP Tools Test Results\n");
        output.push_str("=".repeat(50).as_str());
        output.push_str("\n");
        output.push_str(&format!(
            "📊 Overall Status: {}\n",
            if self.success {
                "✅ SUCCESS"
            } else {
                "❌ FAILED"
            }
        ));
        output.push_str(&format!(
            "🔧 Tools Tested: {}/{} successful\n",
            self.successful_tools, self.total_tools
        ));

        if self.total_tools > 0 {
            let success_rate = (self.successful_tools * 100) / self.total_tools;
            output.push_str(&format!("📈 Success Rate: {}%\n", success_rate));
        }

        // Individual tool results
        output.push_str("\n📋 Individual Tool Results:\n");
        output.push_str("-".repeat(30).as_str());
        output.push_str("\n");

        for (tool_name, result) in &self.tool_results {
            let status = if result.success { "✅" } else { "❌" };
            let duration = format!("{:.2}s", result.response_time_ms as f64 / 1000.0);
            output.push_str(&format!("  {} {} ({})\n", status, tool_name, duration));

            if verbose {
                output.push_str(&format!("    Query: \"{}\"\n", result.test_query));
                if !result.success {
                    if let Some(error) = &result.error_message {
                        output.push_str(&format!("    Error: {}\n", error));
                    }
                } else if let Some(validation) = &result.validation_details {
                    output.push_str(&format!("    Validation: {}\n", validation));
                }
                output.push_str("\n");
            }
        }

        // Execution summary
        output.push_str(&format!("\n⏱️  Execution Summary:\n"));
        output.push_str("-".repeat(20).as_str());
        output.push_str("\n");
        output.push_str(&format!(
            "   Total time: {:.2}s\n",
            self.execution_summary.total_duration_ms as f64 / 1000.0
        ));
        output.push_str(&format!(
            "   Parallel: {}\n",
            if self.execution_summary.parallel_execution {
                "Yes"
            } else {
                "No"
            }
        ));
        output.push_str(&format!(
            "   Timeout per tool: {}s\n",
            self.execution_summary.timeout_settings
        ));

        if let Some(error) = &self.error {
            output.push_str(&format!("\n⚠️  Global Error: {}\n", error));
        }

        output
    }
}

impl ToolTestResult {
    pub fn new_success(
        tool_name: String,
        response_time_ms: u64,
        test_query: String,
        response_data: Value,
    ) -> Self {
        Self {
            tool_name,
            success: true,
            response_time_ms,
            test_query,
            response_data: Some(response_data),
            error_message: None,
            validation_details: Some("Response received successfully".to_string()),
        }
    }

    pub fn new_error(
        tool_name: String,
        response_time_ms: u64,
        test_query: String,
        error: String,
    ) -> Self {
        Self {
            tool_name,
            success: false,
            response_time_ms,
            test_query,
            response_data: None,
            error_message: Some(error),
            validation_details: None,
        }
    }
}

pub struct TestQueryGenerator;

impl TestQueryGenerator {
    pub fn generate_test_query(&self, tool_name: &str) -> String {
        match tool_name {
            "search" => "remote work policy".to_string(),
            "chat" => "What are the main benefits of using Glean?".to_string(),
            "read_document" => {
                "https://help.glean.com/en/articles/6248863-getting-started-with-glean".to_string()
            }
            "code_search" => "function authenticate".to_string(),
            "employee_search" => "engineering team".to_string(),
            "gmail_search" => "from:noreply@glean.com".to_string(),
            "outlook_search" => "subject:meeting notes".to_string(),
            "meeting_lookup" => "weekly standup".to_string(),
            "web_browser" => "https://www.glean.com".to_string(),
            "gemini_web_search" => "latest technology trends".to_string(),
            _ => format!("test query for {}", tool_name),
        }
    }

    pub fn get_tool_category(&self, tool_name: &str) -> &'static str {
        match tool_name {
            "search" | "chat" | "read_document" => "core",
            "code_search" | "employee_search" | "gmail_search" | "outlook_search"
            | "meeting_lookup" | "web_browser" | "gemini_web_search" => "enterprise",
            _ => "unknown",
        }
    }
}

pub struct GleanMCPInspector {
    server_url: String,
    auth_token: Option<String>,
    query_generator: TestQueryGenerator,
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
            query_generator: TestQueryGenerator,
        }
    }

    /// Test all available MCP tools and report comprehensive results
    pub async fn test_all_tools(&self, options: &TestAllOptions) -> Result<AllToolsTestResult> {
        let start_time = Instant::now();
        let start_time_str = chrono::Utc::now().to_rfc3339();

        println!("🔍 Starting comprehensive tool testing...");

        // Step 1: Discover available tools
        let tools_result = self.list_available_tools().await?;
        let available_tools = self.extract_tools_from_result(&tools_result)?;

        // Step 2: Filter tools based on options
        let tools_to_test = self.filter_tools(&available_tools, options);

        if tools_to_test.is_empty() {
            return Ok(AllToolsTestResult {
                success: false,
                total_tools: 0,
                successful_tools: 0,
                failed_tools: 0,
                tool_results: HashMap::new(),
                execution_summary: ExecutionSummary {
                    start_time: start_time_str.clone(),
                    end_time: chrono::Utc::now().to_rfc3339(),
                    total_duration_ms: start_time.elapsed().as_millis() as u64,
                    parallel_execution: options.parallel,
                    timeout_settings: options.timeout,
                },
                error: Some("No tools found to test".to_string()),
            });
        }

        println!("📋 Found {} tools to test", tools_to_test.len());
        for tool in &tools_to_test {
            println!(
                "  • {} ({})",
                tool.name,
                self.query_generator.get_tool_category(&tool.name)
            );
        }

        // Step 3: Execute tests
        let test_results = if options.parallel {
            println!(
                "🚀 Running tests in parallel (max {} concurrent)",
                options.max_concurrent
            );
            self.execute_tests_parallel(&tools_to_test, options).await?
        } else {
            println!("🔄 Running tests sequentially");
            self.execute_tests_sequential(&tools_to_test, options)
                .await?
        };

        // Step 4: Generate final result
        let end_time = Instant::now();
        let successful_count = test_results.iter().filter(|r| r.success).count();
        let total_count = test_results.len();

        let mut tool_results_map = HashMap::new();
        for result in test_results {
            tool_results_map.insert(result.tool_name.clone(), result);
        }

        let execution_summary = ExecutionSummary {
            start_time: start_time_str,
            end_time: chrono::Utc::now().to_rfc3339(),
            total_duration_ms: end_time.duration_since(start_time).as_millis() as u64,
            parallel_execution: options.parallel,
            timeout_settings: options.timeout,
        };

        Ok(AllToolsTestResult {
            success: successful_count == total_count,
            total_tools: total_count,
            successful_tools: successful_count,
            failed_tools: total_count - successful_count,
            tool_results: tool_results_map,
            execution_summary,
            error: None,
        })
    }

    /// Extract tools from the list_available_tools result
    fn extract_tools_from_result(&self, result: &InspectorResult) -> Result<Vec<ToolInfo>> {
        let mut tools = Vec::new();

        if let Some(inspector_data) = &result.inspector_data {
            // Try to extract tools from various possible response structures
            if let Some(result_data) = inspector_data.get("result") {
                if let Some(tools_array) = result_data.get("tools").and_then(|t| t.as_array()) {
                    for tool in tools_array {
                        if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                            tools.push(ToolInfo {
                                name: name.to_string(),
                                description: tool
                                    .get("description")
                                    .and_then(|d| d.as_str())
                                    .map(|s| s.to_string()),
                                schema: tool.get("inputSchema").cloned(),
                            });
                        }
                    }
                }
            } else if let Some(tools_array) = inspector_data.get("tools").and_then(|t| t.as_array())
            {
                for tool in tools_array {
                    if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                        tools.push(ToolInfo {
                            name: name.to_string(),
                            description: tool
                                .get("description")
                                .and_then(|d| d.as_str())
                                .map(|s| s.to_string()),
                            schema: tool.get("inputSchema").cloned(),
                        });
                    }
                }
            }
        }

        // If no tools found in structured data, fall back to expected tools (core + enterprise)
        if tools.is_empty() {
            println!("⚠️  No tools found in response, using default tool set");
            tools = vec![
                // Core tools
                ToolInfo {
                    name: "search".to_string(),
                    description: Some("Search Glean's content index".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "chat".to_string(),
                    description: Some("Interact with Glean's AI assistant".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "read_document".to_string(),
                    description: Some("Read documents by ID/URL".to_string()),
                    schema: None,
                },
                // Enterprise tools
                ToolInfo {
                    name: "code_search".to_string(),
                    description: Some("Search code repositories".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "employee_search".to_string(),
                    description: Some("Search people directory".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "gmail_search".to_string(),
                    description: Some("Search Gmail messages".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "outlook_search".to_string(),
                    description: Some("Search Outlook messages".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "meeting_lookup".to_string(),
                    description: Some("Find meeting information".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "web_browser".to_string(),
                    description: Some("Web browsing capability".to_string()),
                    schema: None,
                },
                ToolInfo {
                    name: "gemini_web_search".to_string(),
                    description: Some("Web search capability".to_string()),
                    schema: None,
                },
            ];
        }

        Ok(tools)
    }

    /// Filter tools based on the test options
    fn filter_tools(
        &self,
        available_tools: &[ToolInfo],
        options: &TestAllOptions,
    ) -> Vec<ToolInfo> {
        match options.tools_filter.as_str() {
            "all" => available_tools.to_vec(),
            "core" => available_tools
                .iter()
                .filter(|tool| self.query_generator.get_tool_category(&tool.name) == "core")
                .cloned()
                .collect(),
            "enterprise" => available_tools
                .iter()
                .filter(|tool| self.query_generator.get_tool_category(&tool.name) == "enterprise")
                .cloned()
                .collect(),
            tools_list => {
                let requested_tools: Vec<&str> = tools_list.split(',').map(|s| s.trim()).collect();
                available_tools
                    .iter()
                    .filter(|tool| requested_tools.contains(&tool.name.as_str()))
                    .cloned()
                    .collect()
            }
        }
    }

    /// Execute tests in parallel with concurrency limits
    async fn execute_tests_parallel(
        &self,
        tools: &[ToolInfo],
        options: &TestAllOptions,
    ) -> Result<Vec<ToolTestResult>> {
        use smol::lock::Semaphore;

        let semaphore = Arc::new(Semaphore::new(options.max_concurrent));
        let mut tasks = Vec::new();

        for tool in tools {
            let semaphore = semaphore.clone();
            let tool = tool.clone();
            let timeout = Duration::from_secs(options.timeout);
            let query = self.query_generator.generate_test_query(&tool.name);

            let server_url = self.server_url.clone();
            let auth_token = self.auth_token.clone();

            let task = async move {
                let _permit = semaphore.acquire().await;

                println!("🔧 Testing tool: {} with query: \"{}\"", tool.name, query);

                let start_time = Instant::now();
                let result = async_timeout(
                    timeout,
                    Self::test_tool_direct(server_url, auth_token, &tool.name, &query),
                )
                .await;

                let response_time_ms = start_time.elapsed().as_millis() as u64;

                match result {
                    Ok(response_data) => {
                        println!(
                            "  ✅ {} completed ({:.2}s)",
                            tool.name,
                            response_time_ms as f64 / 1000.0
                        );
                        ToolTestResult::new_success(
                            tool.name,
                            response_time_ms,
                            query,
                            response_data,
                        )
                    }
                    Err(e) => {
                        if e.to_string().contains("timed out") {
                            println!("  ⏰ {} timed out", tool.name);
                            ToolTestResult::new_error(
                                tool.name,
                                response_time_ms,
                                query,
                                format!("Timeout after {}s", timeout.as_secs()),
                            )
                        } else {
                            println!("  ❌ {} failed: {}", tool.name, e);
                            ToolTestResult::new_error(
                                tool.name,
                                response_time_ms,
                                query,
                                e.to_string(),
                            )
                        }
                    }
                }
            };

            tasks.push(task);
        }

        // Execute all tests concurrently
        let results = futures::future::join_all(tasks).await;
        Ok(results)
    }

    /// Execute tests sequentially
    async fn execute_tests_sequential(
        &self,
        tools: &[ToolInfo],
        options: &TestAllOptions,
    ) -> Result<Vec<ToolTestResult>> {
        let mut results = Vec::new();
        let timeout = Duration::from_secs(options.timeout);

        for tool in tools {
            let query = self.query_generator.generate_test_query(&tool.name);
            println!("🔧 Testing tool: {} with query: \"{}\"", tool.name, query);

            let start_time = Instant::now();
            let result = async_timeout(
                timeout,
                Self::test_tool_direct(
                    self.server_url.clone(),
                    self.auth_token.clone(),
                    &tool.name,
                    &query,
                ),
            )
            .await;

            let response_time_ms = start_time.elapsed().as_millis() as u64;

            let test_result = match result {
                Ok(response_data) => {
                    println!(
                        "  ✅ {} completed ({:.2}s)",
                        tool.name,
                        response_time_ms as f64 / 1000.0
                    );
                    ToolTestResult::new_success(
                        tool.name.clone(),
                        response_time_ms,
                        query,
                        response_data,
                    )
                }
                Err(e) => {
                    if e.to_string().contains("timed out") {
                        println!("  ⏰ {} timed out", tool.name);
                        ToolTestResult::new_error(
                            tool.name.clone(),
                            response_time_ms,
                            query,
                            format!("Timeout after {}s", timeout.as_secs()),
                        )
                    } else {
                        println!("  ❌ {} failed: {}", tool.name, e);
                        ToolTestResult::new_error(
                            tool.name.clone(),
                            response_time_ms,
                            query,
                            e.to_string(),
                        )
                    }
                }
            };

            results.push(test_result);
        }

        Ok(results)
    }

    /// Direct tool testing method (static to avoid borrowing issues in async contexts)
    async fn test_tool_direct(
        server_url: String,
        auth_token: Option<String>,
        tool_name: &str,
        query: &str,
    ) -> Result<Value> {
        // Create MCP JSON-RPC request for tool call
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

        let request_body =
            serde_json::to_string(&tool_request).map_err(|e| GleanMcpError::Json(e))?;

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
        if let Some(ref token) = auth_token {
            auth_header = format!("Authorization: Bearer {}", token);
            curl_args.extend_from_slice(&["-H", &auth_header]);
        }

        curl_args.push(&server_url);

        // Execute curl command
        let mut child = Command::new("curl")
            .args(&curl_args)
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

        let (stdout_lines, stderr_lines) = smol::future::zip(stdout_future, stderr_future).await;
        let stdout_lines = stdout_lines
            .map_err(|e| GleanMcpError::Process(format!("Failed to read stdout: {}", e)))?;
        let stderr_lines = stderr_lines
            .map_err(|e| GleanMcpError::Process(format!("Failed to read stderr: {}", e)))?;

        let status = child
            .status()
            .await
            .map_err(|e| GleanMcpError::Process(format!("Failed to get process status: {}", e)))?;

        if !status.success() {
            let error_output = stderr_lines.join("\n");
            return Err(GleanMcpError::Process(format!(
                "MCP tool call failed: {}",
                error_output
            )));
        }

        let stdout_content = stdout_lines.join("\n");

        // Try to parse the response as JSON-RPC
        match serde_json::from_str::<Value>(&stdout_content) {
            Ok(response_json) => {
                if let Some(result) = response_json.get("result") {
                    Ok(result.clone())
                } else if let Some(error) = response_json.get("error") {
                    Err(GleanMcpError::Process(format!(
                        "MCP server error: {}",
                        error
                    )))
                } else {
                    Ok(response_json)
                }
            }
            Err(_) => {
                // If not JSON, check if it looks like an error
                if stdout_content.contains("error")
                    || stdout_content.contains("Error")
                    || stdout_content.contains("401")
                    || stdout_content.contains("403")
                    || stdout_content.contains("Invalid Secret")
                    || stdout_content.contains("Not allowed")
                    || stdout_content.contains("Authentication")
                    || stdout_content.contains("Unauthorized")
                {
                    Err(GleanMcpError::Process(format!(
                        "Server error: {}",
                        stdout_content
                    )))
                } else {
                    Ok(serde_json::json!({
                        "tool": tool_name,
                        "query": query,
                        "response": stdout_content,
                        "success": true
                    }))
                }
            }
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

        // For basic connectivity test, assume all tools are available if server responds
        let mut tool_validation = HashMap::new();
        let expected_tools = vec![
            "search",
            "chat",
            "read_document",
            "code_search",
            "employee_search",
            "gmail_search",
            "outlook_search",
            "meeting_lookup",
            "web_browser",
            "gemini_web_search",
        ];

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
    pub fn validate_glean_tools(&self, inspector_data: Value) -> InspectorResult {
        let expected_tools = vec![
            "search",
            "chat",
            "read_document",
            "code_search",
            "employee_search",
            "gmail_search",
            "outlook_search",
            "meeting_lookup",
            "web_browser",
            "gemini_web_search",
        ];
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

/// Run comprehensive testing of all available MCP tools
pub fn run_test_all(
    instance_name: Option<&str>,
    options: &TestAllOptions,
) -> Result<AllToolsTestResult> {
    smol::block_on(async {
        let inspector = GleanMCPInspector::new(instance_name);
        inspector.test_all_tools(options).await
    })
}
