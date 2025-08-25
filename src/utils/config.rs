use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GleanConfig {
    pub glean_instance: GleanInstance,
    pub mcp_inspector: McpInspectorConfig,
    pub authentication: AuthConfig,
    pub tools_to_test: ToolsConfig,
    pub host_applications: HashMap<String, HostConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GleanInstance {
    pub name: String,
    pub environment: String,
    pub server_url: String,
    pub chatgpt_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInspectorConfig {
    pub package: String,
    pub validation_required: bool,
    pub tools_to_validate: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: String,
    pub oauth_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub core_tools: Vec<String>,
    pub enterprise_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub auth_method: String,
    pub config_type: String,
    pub mcp_config_path: Option<String>,
    pub server_url: String,
    pub priority: String,
}

impl Default for GleanConfig {
    fn default() -> Self {
        let mut host_applications = HashMap::new();

        host_applications.insert(
            "cursor".to_string(),
            HostConfig {
                auth_method: "bridge".to_string(),
                config_type: "local".to_string(),
                mcp_config_path: Some("~/.cursor/mcp.json".to_string()),
                server_url: "https://glean-dev-be.glean.com/mcp/default".to_string(),
                priority: "P0".to_string(),
            },
        );

        host_applications.insert(
            "vscode".to_string(),
            HostConfig {
                auth_method: "native".to_string(),
                config_type: "global".to_string(),
                mcp_config_path: Some("~/.vscode/settings.json".to_string()),
                server_url: "https://glean-dev-be.glean.com/mcp/default".to_string(),
                priority: "P0".to_string(),
            },
        );

        host_applications.insert(
            "claude_desktop".to_string(),
            HostConfig {
                auth_method: "native".to_string(),
                config_type: "local".to_string(),
                mcp_config_path: Some(
                    "~/Library/Application Support/Claude/claude_desktop_config.json".to_string(),
                ),
                server_url: "https://glean-dev-be.glean.com/mcp/default".to_string(),
                priority: "P0".to_string(),
            },
        );

        host_applications.insert(
            "claude_code".to_string(),
            HostConfig {
                auth_method: "native".to_string(),
                config_type: "command_line".to_string(),
                mcp_config_path: None, // Command-line tool, no config file
                server_url: "https://scio-prod.glean.com/mcp/default".to_string(),
                priority: "P1".to_string(),
            },
        );

        Self {
            glean_instance: GleanInstance {
                name: "scio-prod".to_string(),
                environment: "production".to_string(),
                server_url: "https://scio-prod.glean.com/mcp/default".to_string(),
                chatgpt_url: "https://scio-prod.glean.com/mcp/chatgpt".to_string(),
            },
            mcp_inspector: McpInspectorConfig {
                package: "@modelcontextprotocol/inspector".to_string(),
                validation_required: true,
                tools_to_validate: vec![
                    "glean_search".to_string(),
                    "chat".to_string(),
                    "read_document".to_string(),
                ],
            },
            authentication: AuthConfig {
                method: "oauth".to_string(),
                oauth_scopes: vec![
                    "MCP".to_string(),
                    "SEARCH".to_string(),
                    "TOOLS".to_string(),
                    "ENTITIES".to_string(),
                ],
            },
            tools_to_test: ToolsConfig {
                core_tools: vec![
                    "glean_search".to_string(),
                    "chat".to_string(),
                    "read_document".to_string(),
                ],
                enterprise_tools: vec![
                    "code_search".to_string(),
                    "employee_search".to_string(),
                    "gmail_search".to_string(),
                    "meeting_lookup".to_string(),
                    "outlook_search".to_string(),
                    "web_browser".to_string(),
                ],
            },
            host_applications,
        }
    }
}
