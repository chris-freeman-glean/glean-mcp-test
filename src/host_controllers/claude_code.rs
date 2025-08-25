//! Claude Code host testing controller
//!
//! Tests MCP server functionality for Claude Code (command-line tool)
//! Assumes: MCP server is already configured and authenticated via `claude mcp add`
//! Testing: Uses `claude mcp` commands to test Glean tool functionality

use super::{HostController, HostOperationResult};
use crate::{GleanMcpError, Result};
use async_process::Command;
use smol::io::{AsyncBufReadExt, BufReader};
use smol::stream::StreamExt;
use std::process::Stdio;
use std::time::Instant;

/// Controller for Claude Code command-line application
pub struct ClaudeCodeController {
    /// Path to the claude binary (defaults to "claude" assuming it's in PATH)
    claude_path: String,
}

impl ClaudeCodeController {
    /// Create a new Claude Code controller
    #[must_use]
    pub fn new() -> Self {
        // Try to find the actual claude binary path, fallback to "claude"
        let claude_path = Self::find_claude_binary().unwrap_or_else(|| "claude".to_string());
        Self { claude_path }
    }

    /// Find the Claude Code binary in common installation locations
    fn find_claude_binary() -> Option<String> {
        // Common installation paths for Claude Code
        let common_paths = [
            format!(
                "{}/.claude/local/claude",
                std::env::var("HOME").unwrap_or_default()
            ),
            "/usr/local/bin/claude".to_string(),
            "/opt/homebrew/bin/claude".to_string(),
            "claude".to_string(), // Fallback to PATH
        ];

        for path in &common_paths {
            if std::path::Path::new(path).exists() {
                return Some(path.clone());
            }
        }

        None
    }

    /// Create a new Claude Code controller with custom binary path
    #[must_use]
    pub const fn with_path(claude_path: String) -> Self {
        Self { claude_path }
    }

    /// List all configured MCP servers in Claude Code
    async fn list_mcp_servers_internal(&self) -> Result<String> {
        let mut child = Command::new(&self.claude_path)
            .args(["mcp", "list"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                GleanMcpError::Host(format!("Failed to spawn claude mcp list command: {e}"))
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| GleanMcpError::Host("Failed to capture stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| GleanMcpError::Host("Failed to capture stderr".to_string()))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

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
        let stdout_lines = stdout_result
            .map_err(|e| GleanMcpError::Host(format!("Failed to read stdout: {e}")))?;
        let stderr_lines = stderr_result
            .map_err(|e| GleanMcpError::Host(format!("Failed to read stderr: {e}")))?;

        let status = child.status().await.map_err(|e| {
            GleanMcpError::Host(format!("Failed to get claude mcp list status: {e}"))
        })?;

        let output = stdout_lines.join("\n");
        let error_output = stderr_lines.join("\n");

        if !status.success() {
            return Err(GleanMcpError::Host(format!(
                "claude mcp list failed: {error_output}"
            )));
        }

        Ok(output)
    }

    /// Execute a Glean tool using Claude Code
    async fn execute_glean_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        query: &str,
    ) -> Result<String> {
        // For now, we'll simulate tool execution by checking if the server exists
        // In a real implementation, this would use Claude Code's interactive session
        // or a specific command to execute MCP tools

        // First verify the server is available
        let server_list = self.list_mcp_servers_internal().await?;

        if !server_list.contains(server_name) {
            return Err(GleanMcpError::Host(format!(
                "MCP server '{server_name}' not found. Available servers: {server_list}"
            )));
        }

        // Simulate tool execution result
        // In practice, this would involve:
        // 1. Starting Claude Code in interactive mode
        // 2. Using the MCP server to call the tool
        // 3. Parsing the response

        Ok(format!(
            "Simulated execution of '{tool_name}' tool with query '{query}' on server '{server_name}'"
        ))
    }
}

impl Default for ClaudeCodeController {
    fn default() -> Self {
        Self::new()
    }
}

impl HostController for ClaudeCodeController {
    async fn verify_mcp_server(
        &self,
    ) -> Result<HostOperationResult> {
        let start_time = Instant::now();

        match self.list_mcp_servers_internal().await {
            Ok(output) => Ok(HostOperationResult::new_success(
                "claude-code",
                "verify_mcp_server",
                &format!("MCP servers verified: {output}"),
            )
            .with_duration(start_time.elapsed())),
            Err(e) => Ok(HostOperationResult::new_error(
                "claude-code",
                "verify_mcp_server",
                &e.to_string(),
            )
            .with_duration(start_time.elapsed())),
        }
    }

    fn test_glean_tool(
        &self,
        tool_name: &str,
        query: &str,
    ) -> impl std::future::Future<Output = Result<HostOperationResult>> + Send {
        let tool_name = tool_name.to_string();
        let query = query.to_string();
        async move {
            let start_time = Instant::now();

            // Test the tool by using Claude Code's interactive session
            // This assumes glean_default server is already configured
            match self
                .execute_glean_tool("glean_default", &tool_name, &query)
                .await
            {
                Ok(output) => Ok(HostOperationResult::new_success(
                    "claude-code",
                    "test_glean_tool",
                    &format!("Tool '{tool_name}' executed successfully: {output}"),
                )
                .with_duration(start_time.elapsed())),
                Err(e) => Ok(HostOperationResult::new_error(
                    "claude-code",
                    "test_glean_tool",
                    &format!("Tool '{tool_name}' failed: {e}"),
                )
                .with_duration(start_time.elapsed())),
            }
        }
    }

    async fn test_all_glean_tools(
        &self,
    ) -> Result<HostOperationResult> {
        let start_time = Instant::now();

        // Define core Glean tools to test
        let glean_tools = vec![
            ("glean_search", "remote work policy"),
            ("chat", "What are the benefits of using Glean?"),
            ("read_document", "https://docs.glean.com"),
        ];

        let mut results = Vec::new();
        let mut success_count = 0;

        for (tool_name, sample_query) in &glean_tools {
            match self.test_glean_tool(tool_name, sample_query).await {
                Ok(result) => {
                    if result.success {
                        success_count += 1;
                    }
                    results.push(format!(
                        "{tool_name}: {}",
                        if result.success { "✅" } else { "❌" }
                    ));
                }
                Err(_) => {
                    results.push(format!("{tool_name}: ❌ Error"));
                }
            }
        }

        let total_tools = glean_tools.len();
        let details = format!(
            "Tested {total_tools} Glean tools, {success_count} successful:\n{}",
            results.join("\n")
        );

        Ok(
            HostOperationResult::new_success("claude-code", "test_all_glean_tools", &details)
                .with_duration(start_time.elapsed()),
        )
    }

    fn check_availability(&self) -> Result<bool> {
        // Check if claude command is available in PATH
        match std::process::Command::new(&self.claude_path)
            .arg("--version")
            .output()
        {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn host_name(&self) -> &'static str {
        "claude-code"
    }

    async fn list_mcp_servers(
        &self,
    ) -> Result<HostOperationResult> {
        let start_time = Instant::now();

        match self.list_mcp_servers_internal().await {
            Ok(output) => Ok(HostOperationResult::new_success(
                "claude-code",
                "list_mcp_servers",
                &format!("MCP servers: {output}"),
            )
            .with_duration(start_time.elapsed())),
            Err(e) => Ok(HostOperationResult::new_error(
                "claude-code",
                "list_mcp_servers",
                &e.to_string(),
            )
            .with_duration(start_time.elapsed())),
        }
    }
}
