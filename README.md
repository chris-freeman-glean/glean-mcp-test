# Glean MCP Testing Framework

A testing framework for validating Glean's MCP (Model Context Protocol) server functionality.

## Overview

Test Glean's MCP server connectivity, authentication, and tool functionality. Supports both human-readable text and JSON output formats.

## Quick Start

### Prerequisites

- Rust (latest stable)
- Node.js & npm
- Glean authentication token

### Setup

1. Clone and build:

   ```bash
   git clone <repository-url>
   cd glean-mcp-test
   cargo build --release
   ```

2. Set authentication token:

   ```bash
   export GLEAN_AUTH_TOKEN=your_token_here
   ```

3. Verify setup:
   ```bash
   cargo run -- prerequisites
   cargo run -- auth
   ```

## Commands

### Basic Usage

```bash
# Check system requirements
cargo run -- prerequisites

# Test authentication
cargo run -- auth

# Validate MCP server connection
cargo run -- inspect

# List available tools
cargo run -- list-tools

# Test individual tools
cargo run -- test-tool -t glean_search -q "your query"
cargo run -- test-tool -t chat -q "your question"

# Show configuration
cargo run -- config
```

All commands support `--format json` for programmatic use.

## Configuration

### Environment Variables

- `GLEAN_AUTH_TOKEN`: Your Glean authentication token (required)

### Available Tools

- **glean_search**: Search Glean's content index
- **chat**: Interact with Glean's AI assistant
- **read_document**: Read documents by ID/URL
- Additional enterprise tools (code_search, employee_search, etc.)

## Troubleshooting

### Common Issues

**Authentication errors**: Verify `GLEAN_AUTH_TOKEN` is set correctly

**MCP Inspector not found**: Install Node.js and npm

**Connection timeout**: Check network connectivity and token validity

### Debug Steps

1. Run `cargo run -- prerequisites` to check system requirements
2. Run `cargo run -- auth` to test authentication
3. Use `RUST_LOG=debug cargo run -- inspect` for detailed logging
