# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Glean MCP Testing Framework - A Rust-based CLI tool for validating Glean's MCP (Model Context Protocol) server functionality across multiple host applications like Cursor, VS Code, Claude Desktop, and Claude Code.

## Essential Commands

### Building and Running
```bash
# Build the project
cargo build --release

# Run with specific commands
cargo run -- prerequisites    # Check system requirements
cargo run -- auth            # Test authentication setup
cargo run -- inspect         # Validate MCP server connection
cargo run -- list-tools      # List available MCP tools
cargo run -- test-tool -t glean_search -q "query"  # Test specific tool

# Test host applications (currently supports claude-code)
cargo run -- verify-host -H claude-code
cargo run -- test-host-tool -H claude-code -t glean_search -q "test query"
cargo run -- test-all-host-tools -H claude-code
```

### Linting and Code Quality
```bash
# Format code
cargo fmt

# Run Clippy lints (strict configuration in Cargo.toml)
cargo clippy

# Check code without building
cargo check
```

### Testing
```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- inspect
```

## Architecture Overview

### Core Components

1. **CLI Interface** (`src/main.rs`)
   - Clap-based command-line interface with comprehensive subcommands
   - Async operations using smol runtime
   - Support for both text and JSON output formats

2. **MCP Inspector** (`src/mcp_inspector/validator.rs`)
   - `GleanMCPInspector` - Core validation engine
   - Direct HTTP MCP protocol implementation using curl
   - Handles authentication via `GLEAN_AUTH_TOKEN` environment variable
   - JSON-RPC request generation for MCP tools (`tools/call`, `tools/list`)

3. **Host Controllers** (`src/host_controllers/`)
   - Modular system for testing different host applications
   - Currently implements `ClaudeCodeController` for Claude Code testing
   - Future support for Cursor, VS Code, Claude Desktop

4. **Configuration System** (`src/utils/config.rs`)
   - `GleanConfig` with defaults for different Glean instances
   - Host application configurations with auth methods and endpoints
   - Tool validation lists (core tools vs enterprise tools)

### Key Design Patterns

- **Async Runtime**: Uses `smol` for lightweight async operations
- **Error Handling**: Custom `GleanMcpError` with comprehensive error types
- **Structured Results**: `InspectorResult` and `HostOperationResult` for consistent output
- **Environment-Based Auth**: Reads `GLEAN_AUTH_TOKEN` from environment

### MCP Protocol Implementation

- **Server URLs**: `https://{instance}-be.glean.com/mcp/default` pattern
- **Authentication**: Bearer token auth with proper 401 handling for unauthenticated requests
- **Core Tools**: `glean_search`, `chat`, `read_document` (always available)
- **Enterprise Tools**: `code_search`, `employee_search`, etc. (configurable)

## Development Guidelines

### From Cursor Rules

- **Error Handling**: Use `anyhow::Result` and `thiserror::Error` patterns
- **Async Patterns**: Prefer `smol` runtime over tokio for lightweight operations
- **CLI Patterns**: Use clap derive macros with comprehensive help text
- **Code Quality**: Strict Clippy configuration enforces pedantic, nursery, and performance lints

### Environment Setup

- **Required**: Rust (latest stable), Node.js & npm for MCP Inspector
- **Authentication**: Set `GLEAN_AUTH_TOKEN` environment variable
- **Mise Support**: Configuration files present for mise users

### Testing Approach

- **Prerequisites Check**: Always verify npx and MCP Inspector availability
- **Connectivity Testing**: Basic HTTP connectivity before tool testing
- **Tool Validation**: Direct MCP JSON-RPC calls for realistic testing
- **Host Integration**: Test tools through actual host applications

## Configuration

### Default Glean Instance
- Production: `scio-prod.glean.com`
- Development: `glean-dev-be.glean.com`

### Host Application Support
- **Claude Code**: Native OAuth, command-line interface (P1 priority)
- **Cursor**: Bridge auth via mcp-remote (P0 priority)  
- **VS Code**: Native OAuth with streamable HTTP (P0 priority)
- **Claude Desktop**: Native OAuth via config file (P0 priority)

## Important Notes

- Always run `prerequisites` command before testing
- Use `--format json` for programmatic output consumption
- Set authentication token in environment for full testing capabilities
- Default instance is `scio-prod` - use `--instance` flag to override