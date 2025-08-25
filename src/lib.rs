//! Glean MCP Testing Framework
//!
//! A comprehensive testing framework for validating Glean's MCP (Model Context Protocol)
//! server functionality across all supported host applications.

pub mod mcp_inspector;
pub mod utils;

pub use mcp_inspector::*;
pub use utils::*;

/// Main error type for the framework
#[derive(thiserror::Error, Debug)]
pub enum GleanMcpError {
    #[error("MCP Inspector error: {0}")]
    Inspector(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Host application error: {0}")]
    Host(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Process error: {0}")]
    Process(String),
}

pub type Result<T> = std::result::Result<T, GleanMcpError>;
