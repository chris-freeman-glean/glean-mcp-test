# Glean MCP Testing Framework - Implementation Plan

## Project Overview

Rust-based testing framework for validating Glean's MCP (Model Context Protocol) server functionality across multiple host applications.

### Core Objectives

- Test all Glean MCP tools across supported host applications
- Verify OAuth Native and Bridge authentication methods
- Test enterprise workflows using Glean's tools
- Provide automated testing with minimal manual intervention

## Technical Environment

### Target Instances

- **Primary**: `scio-prod` (`https://scio-prod-be.glean.com/mcp/default`)
- **Secondary**: `glean-dev` (`https://glean-dev-be.glean.com/mcp/default`)
- **ChatGPT Endpoint**: `https://{instance}-be.glean.com/mcp/chatgpt`
- **Transport**: HTTP JSON-RPC via curl
- **Authentication**: OAuth 2.0 Bearer token via `GLEAN_AUTH_TOKEN`

### MCP Tools

#### Core Tools

1. **search** - Search Glean's content index
2. **chat** - Interact with Glean's AI assistant
3. **read_document** - Read documents by ID/URL

#### Enterprise Tools

4. **code_search** - Search code repositories
5. **employee_search** - Search people directory
6. **gemini_web_search** - Web search capability
7. **gmail_search** - Search Gmail messages
8. **meeting_lookup** - Find meeting information
9. **outlook_search** - Search Outlook messages
10. **web_browser** - Web browsing capability

### Host Applications

| Host           | Auth Method         | Config Type       | Status |
| -------------- | ------------------- | ----------------- | ------ |
| Cursor         | Bridge (mcp-remote) | Local             | P0     |
| VS Code        | Native OAuth        | Local             | P0     |
| Claude Desktop | Native OAuth        | Local             | P0     |
| ChatGPT        | Native OAuth        | Centrally-managed | P1     |
| Claude Code    | Native OAuth        | Command-line      | P1     |
| Windsurf       | Bridge (mcp-remote) | Local             | P1     |
| Goose          | Bridge (mcp-remote) | Command-line      | P2     |

## Implementation Status

### ✅ Completed: Core MCP Testing Infrastructure

**Components:**

- Direct HTTP MCP client integration
- Authentication system with `GLEAN_AUTH_TOKEN` support
- Tool discovery via `tools/list` JSON-RPC
- Tool execution via `tools/call` JSON-RPC
- Multi-endpoint testing (default + ChatGPT endpoints)
- Parallel test execution with progress tracking
- Multiple output formats (text, JSON)
- Comprehensive error handling and exit codes

**CLI Commands:**

```bash
# Core testing commands
cargo run -- test --instance scio-prod                       # Test core tools (default)
cargo run -- test --instance scio-prod --all                 # Test all tools
cargo run -- test --instance scio-prod --tools search,chat   # Test specific tools
cargo run -- test --instance scio-prod --json                # JSON output
cargo run -- test --instance scio-prod --parallel            # Parallel execution

# Utility commands
cargo run -- prerequisites                                   # Check system requirements
cargo run -- auth --instance scio-prod                       # Test authentication
cargo run -- inspect --instance scio-prod                    # Test server connectivity
cargo run -- list-tools --instance scio-prod                 # List available tools
cargo run -- config                                          # Show configuration
```

### 🔄 In Progress: Host Application Integration

**Status**: Not started
**Priority**: P0

**Tasks:**

- [ ] AppleScript automation for host configuration
- [ ] Cursor configuration (Bridge authentication via mcp-remote)
- [ ] VS Code configuration (Native OAuth)
- [ ] Claude Desktop configuration (Native OAuth)
- [ ] Host application detection and validation
- [ ] Configuration rollback functionality

**Implementation approach:**

```rust
pub struct HostController {
    pub fn configure_cursor(&self, glean_instance: &str) -> Result<bool, Error>
    pub fn configure_vscode(&self, glean_instance: &str) -> Result<bool, Error>
    pub fn configure_claude_desktop(&self, glean_instance: &str) -> Result<bool, Error>
    pub fn verify_connection(&self, host: &str) -> Result<bool, Error>
    pub fn rollback_configuration(&self, host: &str) -> Result<(), Error>
}
```

### 🔄 In Progress: Authentication System

**Status**: Not started
**Priority**: P1

**Tasks:**

- [ ] OAuth Native authentication handler
- [ ] Bridge authentication (mcp-remote) handler
- [ ] OAuth device flow automation
- [ ] Authentication failure detection
- [ ] API token fallback
- [ ] Authentication session validation

### 📋 Planned: Enterprise Test Scenarios

**Status**: Not started
**Priority**: P0

**Test Scenarios:**

1. **Enterprise Search**: "Using Glean, search for our company's remote work policy"
2. **Chat Assistant**: "Ask Glean's assistant: What are the main benefits of using Glean?"
3. **Document Retrieval**: "Use Glean to read the document at [specific-glean-doc-url]"
4. **Multi-tool Workflow**: "Search Glean for engineering guidelines, then read the top result and summarize"
5. **People Search**: "Using Glean, find information about employees in the engineering team"

**Implementation:**

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TestScenario {
    pub name: String,
    pub query: String,
    pub expected_tool: String,
    pub expected_tools: Option<Vec<String>>,
    pub timeout_seconds: u64,
}

pub struct ScenarioExecutor {
    pub fn execute_scenario(&self, scenario: &TestScenario, host: &str) -> Result<TestResult, Error>
    pub fn validate_response(&self, scenario: &TestScenario, response: &str) -> bool
}
```

### 📋 Planned: Cross-Host Execution Engine

**Status**: Not started
**Priority**: P0

**Tasks:**

- [ ] Cross-host test orchestration
- [ ] Result comparison system
- [ ] Compatibility matrix generation
- [ ] Host-specific UI adaptation
- [ ] Comprehensive reporting system

## Current Architecture

### Project Structure

```
src/
├── main.rs                    # CLI interface
├── lib.rs                     # Public API
├── mcp_inspector/             # Direct MCP testing
│   ├── mod.rs
│   └── validator.rs          # Core MCP client
├── host_controllers/          # Host app automation (TODO)
│   ├── mod.rs
│   ├── cursor.rs
│   ├── vscode.rs
│   └── claude.rs
├── test_scenarios/           # Enterprise scenarios (TODO)
│   ├── mod.rs
│   └── glean_scenarios.rs
└── utils/
    ├── mod.rs
    └── config.rs            # Configuration management
```

### Key Dependencies

```toml
[dependencies]
smol = "2.0.2"                # Async runtime
async-process = "2.0.0"       # Process execution
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
console = "0.15"              # Terminal UI
indicatif = "0.17"            # Progress bars
```

## Next Implementation Steps

### Phase 1: Host Integration (Immediate)

1. Implement AppleScript automation for host configuration
2. Start with Cursor (Bridge auth via mcp-remote)
3. Add VS Code and Claude Desktop (Native OAuth)
4. Build connection verification system

### Phase 2: Authentication (Parallel)

1. Implement OAuth device flow automation
2. Create authentication failure detection
3. Add session validation

### Phase 3: Enterprise Scenarios (Follow-up)

1. Implement test scenario framework
2. Create cross-host execution engine
3. Build compatibility matrix reporting

## Risk Factors

### High Risk

- **AppleScript Reliability**: UI automation fragility
- **OAuth Flow Automation**: Complex authentication flows
- **Host Application Updates**: UI changes breaking automation

### Medium Risk

- **Network Dependency**: Glean server availability
- **Performance Target**: Execution time constraints

## Configuration

### Environment Variables

- `GLEAN_AUTH_TOKEN`: Required OAuth token for MCP server access

### Configuration File

```yaml
# glean_config.yaml
glean_instance:
  name: 'scio-prod'
  server_url: 'https://scio-prod-be.glean.com/mcp/default'
  chatgpt_url: 'https://scio-prod-be.glean.com/mcp/chatgpt'

host_applications:
  cursor:
    auth_method: 'bridge'
    config_type: 'local'
  vscode:
    auth_method: 'native'
    config_type: 'global'
```
