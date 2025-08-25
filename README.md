# Glean MCP Testing Framework

A comprehensive Rust-based testing framework for validating Glean's MCP (Model Context Protocol) server functionality across all supported tools and instances.

## Overview

The Glean MCP Testing Framework provides automated testing of all Glean MCP tools with comprehensive reporting, parallel execution, and detailed status analysis. Perfect for monitoring tool health, debugging issues, and ensuring production readiness.

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
   cargo run -- auth --instance scio-prod
   ```

4. **Test all tools** (main feature):

   ```bash
   # Test all available tools with parallel execution
   cargo run -- test-all --instance scio-prod --parallel

   # Quick core tools test
   cargo run -- test-all --instance scio-prod --tools core --format summary
   ```

## Commands

### 🧪 Main Command: `test-all`

**Test all available MCP tools with comprehensive reporting:**

```bash
# Test all tools (discovers tools automatically)
cargo run -- test-all --instance scio-prod

# Test specific tool categories
cargo run -- test-all --instance scio-prod --tools core
cargo run -- test-all --instance scio-prod --tools enterprise
cargo run -- test-all --instance scio-prod --tools search,chat

# Performance and output options
cargo run -- test-all --instance scio-prod --parallel --timeout 60
cargo run -- test-all --instance scio-prod --json --output results.json
cargo run -- test-all --instance scio-prod --format summary

# Advanced options
cargo run -- test-all --instance scio-prod --parallel --max-concurrent 5 --verbose
```

### 🔧 Utility Commands

```bash
# System verification
cargo run -- prerequisites                    # Check system requirements
cargo run -- auth --instance scio-prod       # Test authentication

# Server validation
cargo run -- inspect --instance scio-prod    # Validate MCP server connection
cargo run -- list-tools --instance scio-prod # List available tools

# Individual tool testing
cargo run -- test-tool -t search -q "remote work policy" --instance scio-prod
cargo run -- test-tool -t chat -q "What are the main benefits of using Glean?"

# Configuration management
cargo run -- config                           # Show configuration
cargo run -- config --verbose                # Show detailed YAML config
```

### 📊 Output Formats

All commands support multiple output formats and **return proper exit codes** (0=success, 1=failure):

- **`--format text`** (default): Human-readable with emojis and progress
- **`--format json`** or **`--json`**: Structured data for programmatic use
- **`--format summary`**: Concise overview with key metrics

## Configuration

### Environment Variables

- `GLEAN_AUTH_TOKEN`: Your Glean authentication token (required)

### Available Tools

The framework automatically discovers available tools and categorizes them:

#### Core Tools (Always Available)

- **search**: Search Glean's content index
- **chat**: Interact with Glean's AI assistant
- **read_document**: Read documents by ID/URL

#### Enterprise Tools (Instance-Dependent)

- **code_search**: Search code repositories
- **employee_search**: Search people directory
- **gmail_search**: Search Gmail messages
- **outlook_search**: Search Outlook messages
- **meeting_lookup**: Find meeting information
- **web_browser**: Web browsing capability
- **gemini_web_search**: Web search capability

### Instances

- **scio-prod**: Production instance (recommended for testing)
- **glean-dev**: Development instance (limited token compatibility)

## Advanced Usage

### Scripting & Automation

All commands return proper exit codes for scripting:

```bash
#!/bin/bash
# CI/CD health check script

# Check prerequisites
if ! cargo run -- prerequisites; then
    echo "❌ Prerequisites failed"
    exit 1
fi

# Test authentication
if ! cargo run -- auth --instance scio-prod; then
    echo "❌ Authentication failed"
    exit 1
fi

# Run comprehensive test
if cargo run -- test-all --instance scio-prod --parallel --json --output health-check.json; then
    echo "✅ All tools healthy"
else
    echo "❌ Some tools failed - check health-check.json"
    exit 1
fi
```

### Performance Options

- **`--parallel`**: Run tests concurrently (3-5x faster)
- **`--max-concurrent N`**: Limit concurrent tests (default: 3)
- **`--timeout N`**: Per-tool timeout in seconds (default: 60)

### Example Results

```bash
🧪 Glean MCP Tools Test Results
==================================================
📊 Overall Status: ✅ SUCCESS
🔧 Tools Tested: 3/3 successful
📈 Success Rate: 100%

📋 Individual Tool Results:
------------------------------
  ✅ search (0.39s)
  ✅ chat (7.21s)
  ✅ read_document (0.16s)

⏱️  Execution Summary:
--------------------
   Total time: 7.85s
   Parallel: Yes
   Timeout per tool: 60s
```

## Troubleshooting

### Common Issues

- **Authentication errors**: Verify `GLEAN_AUTH_TOKEN` is valid for your instance
- **Tool timeouts**: Increase `--timeout` for slower tools (especially `chat`)
- **Connection issues**: Check network connectivity and instance URL
- **Missing tools**: Tools vary by instance; use `list-tools` to see available tools

### Debug Steps

1. Run `cargo run -- prerequisites` to verify system setup
2. Run `cargo run -- auth --instance scio-prod` to test authentication
3. Run `cargo run -- list-tools --instance scio-prod` to see available tools
4. Use `--verbose` flag for detailed output
5. Check `--format json` output for structured error details
