//! Host application controllers for MCP testing
//!
//! This module contains controllers for testing MCP server functionality
//! across different host applications. It assumes MCP servers are already
//! configured and authenticated in each host application.

pub mod claude_code;

use crate::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Result of a host application testing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostOperationResult {
    pub success: bool,
    pub host: String,
    pub operation: String,
    pub details: String,
    pub error: Option<String>,
    pub duration: Option<Duration>,
}

impl HostOperationResult {
    #[must_use]
    pub fn new_success(host: &str, operation: &str, details: &str) -> Self {
        Self {
            success: true,
            host: host.to_string(),
            operation: operation.to_string(),
            details: details.to_string(),
            error: None,
            duration: None,
        }
    }

    #[must_use]
    pub fn new_error(host: &str, operation: &str, error: &str) -> Self {
        Self {
            success: false,
            host: host.to_string(),
            operation: operation.to_string(),
            details: String::new(),
            error: Some(error.to_string()),
            duration: None,
        }
    }

    #[must_use]
    pub const fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
}

/// Trait for all host application testing controllers
/// Assumes MCP servers are already configured and authenticated
pub trait HostController {
    /// Verify that MCP server connection is working and list available tools
    fn verify_mcp_server(
        &self,
    ) -> impl std::future::Future<Output = Result<HostOperationResult>> + Send;

    /// Test a specific Glean tool through the host application
    fn test_glean_tool(
        &self,
        tool_name: &str,
        query: &str,
    ) -> impl std::future::Future<Output = Result<HostOperationResult>> + Send;

    /// Test all available Glean tools with sample queries
    fn test_all_glean_tools(
        &self,
    ) -> impl std::future::Future<Output = Result<HostOperationResult>> + Send;

    /// Check if the host application is installed and available
    fn check_availability(&self) -> Result<bool>;

    /// Get the host application name
    fn host_name(&self) -> &'static str;

    /// List all configured MCP servers in the host
    fn list_mcp_servers(
        &self,
    ) -> impl std::future::Future<Output = Result<HostOperationResult>> + Send;
}
