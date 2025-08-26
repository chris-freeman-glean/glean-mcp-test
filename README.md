# Glean MCP Testing Framework

A comprehensive Rust-based testing framework for validating Glean's MCP (Model Context Protocol) server functionality across all supported tools and instances.

## Overview

The Glean MCP Testing Framework provides automated testing of all Glean MCP tools with comprehensive reporting, parallel execution, and detailed status analysis. Perfect for monitoring tool health, debugging issues, and ensuring production readiness.

## Installation

### Option 1: Install from GitHub (Recommended)

Install directly from the GitHub repository:

```bash
# Install from main branch
cargo install --git https://github.com/your-username/glean-mcp-test.git

# Install from a specific branch
cargo install --git https://github.com/your-username/glean-mcp-test.git --branch main

# Install with custom binary name
cargo install --git https://github.com/your-username/glean-mcp-test.git --name glean-mcp-test
```

After installation, you can use the tool directly:

```bash
glean-mcp-test test-all --instance scio-prod --parallel
```

### Required Environment Variable

**You must set the `GLEAN_AUTH_TOKEN` environment variable before running any tests:**

```bash
# Set your Glean authentication token
export GLEAN_AUTH_TOKEN=your_token_here

# Or set it for the current session only
GLEAN_AUTH_TOKEN=your_token_here glean-mcp-test test-all --instance scio-prod
```

**How to get your token:**

1. Go to your Glean instance (e.g., https://scio-prod.glean.com)
2. Click on the secret Glean debug menu
3. Create a new "Glean MCP" token
4. Copy the token and set it as the `GLEAN_AUTH_TOKEN` environment variable

**Note:** The token must be valid for the instance you're testing (e.g., scio-prod token for scio-prod instance).

### Option 2: Build from Source

#### Prerequisites

- Rust (latest stable)
- Node.js & npm
- Glean authentication token

#### Setup

1. Clone and build:

   ```bash
   git clone https://github.com/your-username/glean-mcp-test.git
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
# If installed via cargo install:
glean-mcp-test test-all --instance scio-prod

# If building from source:
cargo run -- test-all --instance scio-prod

# Test specific tool categories
glean-mcp-test test-all --instance scio-prod --tools core
glean-mcp-test test-all --instance scio-prod --tools enterprise
glean-mcp-test test-all --instance scio-prod --tools search,chat

# Performance and output options
glean-mcp-test test-all --instance scio-prod --parallel --timeout 60
glean-mcp-test test-all --instance scio-prod --json --output results.json
glean-mcp-test test-all --instance scio-prod --format summary

# Advanced options
glean-mcp-test test-all --instance scio-prod --parallel --max-concurrent 5 --verbose
```

### 🔧 Utility Commands

```bash
# System verification
glean-mcp-test prerequisites                    # Check system requirements
glean-mcp-test auth --instance scio-prod       # Test authentication

# Server validation
glean-mcp-test inspect --instance scio-prod    # Validate MCP server connection
glean-mcp-test list-tools --instance scio-prod # List available tools

# Individual tool testing
glean-mcp-test test-tool -t search -q "remote work policy" --instance scio-prod
glean-mcp-test test-tool -t chat -q "What are the main benefits of using Glean?"

# Configuration management
glean-mcp-test config                           # Show configuration
glean-mcp-test config --verbose                # Show detailed YAML config
```

### 📊 Output Formats

All commands support multiple output formats and **return proper exit codes** (0=success, 1=failure):

- **`--format text`** (default): Human-readable with emojis and progress
- **`--format json`** or **`--json`**: Structured data for programmatic use
- **`--format summary`**: Concise overview with key metrics

## Configuration

### Environment Variables

**⚠️ Required:**

- `GLEAN_AUTH_TOKEN`: Your Glean authentication token (required for all operations)

**How to set:**

```bash
# Permanent (add to your shell profile)
echo 'export GLEAN_AUTH_TOKEN=your_token_here' >> ~/.bashrc
source ~/.bashrc

# Temporary (current session only)
export GLEAN_AUTH_TOKEN=your_token_here

# Per-command (single use)
GLEAN_AUTH_TOKEN=your_token_here glean-mcp-test test-all --instance scio-prod
```

**Token Requirements:**

- Must be valid for the target instance (e.g., scio-prod token for scio-prod instance)
- Must have appropriate API permissions
- Can be obtained from Glean Settings → API Tokens

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
if ! glean-mcp-test prerequisites; then
    echo "❌ Prerequisites failed"
    exit 1
fi

# Test authentication
if ! glean-mcp-test auth --instance scio-prod; then
    echo "❌ Authentication failed"
    exit 1
fi

# Run comprehensive test
if glean-mcp-test test-all --instance scio-prod --parallel --json --output health-check.json; then
    echo "✅ All tools healthy"
else
    echo "❌ Some tools failed - check health-check.json"
    exit 1
fi
```

**Note:** If building from source, replace `glean-mcp-test` with `cargo run --` in the script above.

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

- **Authentication errors**:
  - Verify `GLEAN_AUTH_TOKEN` environment variable is set: `echo $GLEAN_AUTH_TOKEN`
  - Ensure token is valid for your target instance (e.g., scio-prod token for scio-prod)
  - Check token permissions in Glean Settings → API Tokens
  - Try regenerating the token if it's expired
- **Tool timeouts**: Increase `--timeout` for slower tools (especially `chat`)
- **Connection issues**: Check network connectivity and instance URL
- **Missing tools**: Tools vary by instance; use `list-tools` to see available tools
- **"Token not found" errors**: Ensure `GLEAN_AUTH_TOKEN` is exported in your shell session

### Debug Steps

1. **Verify environment variable**: `echo $GLEAN_AUTH_TOKEN` (should show your token)
2. Run `glean-mcp-test prerequisites` to verify system setup
3. Run `glean-mcp-test auth --instance scio-prod` to test authentication
4. Run `glean-mcp-test list-tools --instance scio-prod` to see available tools
5. Use `--verbose` flag for detailed output
6. Check `--format json` output for structured error details

**Note:** If building from source, replace `glean-mcp-test` with `cargo run --` in the commands above.
