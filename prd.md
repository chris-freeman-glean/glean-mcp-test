# Glean MCP Server Testing Framework - Product Requirements Document

## Document Overview

**Product Name**: Glean MCP Server Testing Framework
**Version**: 1.0
**Document Type**: Product Requirements Document (PRD)
**Target Audience**: Development teams, QA engineers, LLM agents
**Last Updated**: August 25, 2025

## Executive Summary

The Glean MCP Server Testing Framework is a comprehensive AppleScript-based solution for validating Glean's MCP (Model Context Protocol) server functionality across all supported host applications. This system ensures that Glean's MCP server works correctly within the actual user interfaces of applications like Cursor IDE, VS Code, Claude Desktop, ChatGPT, and Windsurf.

## Problem Statement

### Current Challenge

Glean's MCP server needs to be validated across multiple host applications to ensure enterprise customers can reliably access Glean's search, chat, and document capabilities through their preferred AI-powered development tools. Manual testing across multiple hosts is time-consuming and doesn't catch integration issues that only appear in real user workflows.

### Why This Matters

- **Enterprise Reliability**: Customers depend on seamless Glean integration in their development workflows
- **Host Compatibility**: Each MCP host application implements MCP differently (OAuth vs Bridge authentication)
- **Tool Validation**: All 10 pre-built Glean MCP tools need validation across hosts
- **User Experience**: Actual users interact through chat interfaces, not APIs

## Glean MCP Server Specifications

### Server Details

- **Default Instance**: `glean-dev-be` (development environment)
- **Remote MCP Server URL**: `https://glean-dev-be.glean.com/mcp/default/sse`
- **ChatGPT Specific URL**: `https://glean-dev-be.glean.com/mcp/chatgpt`
- **Transport**: Streamable HTTP (SSE)
- **Authentication**: OAuth 2.0 (Native or Bridge via mcp-remote)

### Supported Tools

Based on Glean's MCP server documentation, the framework must test these tools:

#### Core Tools (Always Available)

1. **glean_search** - Search Glean's content index
2. **chat** - Interact with Glean's AI assistant
3. **read_document** - Read documents by ID/URL

#### Configurable Tools (Enterprise Features)

4. **code_search** - Search code repositories
5. **employee_search** - Search people directory
6. **gemini_web_search** - Web search capability
7. **gmail_search** - Search Gmail messages
8. **meeting_lookup** - Find meeting information
9. **outlook_search** - Search Outlook messages
10. **web_browser** - Web browsing capability

### Supported Host Applications

Based on Glean's validated compatibility matrix:

| Host Application   | OAuth Method        | Configuration Type | Status    |
| ------------------ | ------------------- | ------------------ | --------- |
| **ChatGPT**        | Native              | Centrally-managed  | Validated |
| **Cursor**         | Bridge (mcp-remote) | Locally configured | Validated |
| **Claude Code**    | Native              | Command-line       | Validated |
| **Claude Desktop** | Native              | Locally configured | Validated |
| **VS Code**        | Native              | Locally configured | Validated |
| **Windsurf**       | Bridge (mcp-remote) | Locally configured | Validated |
| **Goose**          | Bridge (mcp-remote) | Command-line       | Validated |

## Product Goals and Success Metrics

### Primary Goals

1. **Complete Tool Coverage**: Test all 10 Glean MCP tools across all supported host applications
2. **Authentication Validation**: Verify both OAuth Native and Bridge authentication methods work
3. **Enterprise Scenario Testing**: Test realistic enterprise workflows using Glean's tools
4. **Automated Execution**: Run comprehensive tests with minimal manual intervention

### Success Metrics

- **Tool Coverage**: 100% of Glean's MCP tools tested across all host applications
- **Host Coverage**: 100% of validated host applications tested successfully
- **Execution Success**: >95% test execution success rate
- **Enterprise Scenarios**: Complete realistic developer workflows (search → read → chat)

## Functional Requirements

### FR-1: Glean MCP Server Health Validation Using MCP Inspector

**Priority**: P0 (Blocker)

**Description**: Use the official MCP Inspector tool to validate Glean MCP server connectivity and tool availability before host testing.

**Acceptance Criteria**:

- [ ] Install and configure `@modelcontextprotocol/inspector` for Glean server testing
- [ ] Connect to Glean MCP server at `https://glean-dev-be.glean.com/mcp/default/sse`
- [ ] Use MCP Inspector to enumerate all available Glean MCP tools
- [ ] Execute MCP Inspector tool validation for each core tool (glean_search, chat, read_document)
- [ ] Validate tool schemas and parameter requirements using MCP Inspector
- [ ] Generate MCP Inspector compliance report showing tool compatibility
- [ ] Report 100% tool validation success before proceeding to host testing
- [ ] Support configurable instance names (default: glean-dev-be)

**MCP Inspector Integration**:

**Dependencies** (add to `Cargo.toml`):

```toml
[dependencies]
smol = "2.0.2"
async-process = "2.0.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use async_process::Command;
use smol::io::{AsyncBufReadExt, BufReader};
use smol::process::Stdio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorResult {
    pub success: bool,
    pub tool_results: Option<HashMap<String, bool>>,
    pub inspector_data: Option<Value>,
    pub error: Option<String>,
}

impl InspectorResult {
    pub fn new_success(tool_results: HashMap<String, bool>, inspector_data: Value) -> Self {
        Self {
            success: true,
            tool_results: Some(tool_results),
            inspector_data: Some(inspector_data),
            error: None,
        }
    }

    pub fn new_error(error: String) -> Self {
        Self {
            success: false,
            tool_results: None,
            inspector_data: None,
            error: Some(error),
        }
    }
}

pub struct GleanMCPInspector {
    server_url: String,
    inspector_cmd: String,
}

impl GleanMCPInspector {
    pub fn new(instance_name: Option<&str>) -> Self {
        let instance_name = instance_name.unwrap_or("glean-dev-be");
        Self {
            server_url: format!("https://{}.glean.com/mcp/default/sse", instance_name),
            inspector_cmd: "npx".to_string(),
        }
    }

    /// Use MCP Inspector to validate Glean server:
    /// 1. Test server connection and protocol compliance
    /// 2. Enumerate all available tools
    /// 3. Validate tool schemas match Glean specifications
    /// 4. Test sample tool executions
    pub async fn validate_server_with_inspector(&self) -> Result<InspectorResult, Box<dyn std::error::Error>> {
        // Run MCP Inspector against Glean server using smol
        let args = vec!["@modelcontextprotocol/inspector", &self.server_url];

        let mut child = Command::new(&self.inspector_cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        // Read output concurrently using smol
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

        let (stdout_lines, stderr_lines) = smol::future::try_join(stdout_future, stderr_future).await?;
        let status = child.status().await?;

        if !status.success() {
            let error = stderr_lines.join("\n");
            return Ok(InspectorResult::new_error(error));
        }

        let stdout_content = stdout_lines.join("\n");
        let inspector_output: Value = serde_json::from_str(&stdout_content)?;

        Ok(self.validate_glean_tools(inspector_output))
    }

    /// Validate that Glean-specific tools are present and correctly configured:
    /// - glean_search: Requires query parameter, returns search results
    /// - chat: Requires message parameter, returns AI response
    /// - read_document: Requires document_id/url, returns document content
    pub fn validate_glean_tools(&self, inspector_data: Value) -> InspectorResult {
        let expected_tools = vec!["glean_search", "chat", "read_document"];
        let available_tools = inspector_data
            .get("tools")
            .and_then(|t| t.as_array())
            .unwrap_or(&vec![]);

        let mut tool_validation = HashMap::new();
        for tool_name in &expected_tools {
            tool_validation.insert(
                tool_name.to_string(),
                self.validate_tool_schema(tool_name, available_tools),
            );
        }

        let success_count = tool_validation.values().filter(|&&v| v).count();
        let success_rate = success_count as f64 / expected_tools.len() as f64;

        if success_rate == 1.0 {
            InspectorResult::new_success(tool_validation, inspector_data)
        } else {
            let mut result = InspectorResult::new_success(tool_validation, inspector_data);
            result.success = false;
            result
        }
    }

    fn validate_tool_schema(&self, tool_name: &str, available_tools: &[Value]) -> bool {
        available_tools
            .iter()
            .any(|tool| {
                tool.get("name")
                    .and_then(|name| name.as_str())
                    .map_or(false, |name| name == tool_name)
            })
    }
}

/// Example usage with smol runtime
pub fn run_validation(instance_name: Option<&str>) -> Result<InspectorResult, Box<dyn std::error::Error>> {
    smol::block_on(async {
        let inspector = GleanMCPInspector::new(instance_name);
        inspector.validate_server_with_inspector().await
    })
}
```

**MCP Inspector CLI Integration**:

```bash
# Phase 0: MCP Inspector Validation Script
#!/bin/bash
# validate_glean_mcp.sh

INSTANCE_NAME=${1:-"glean-dev-be"}
SERVER_URL="https://${INSTANCE_NAME}.glean.com/mcp/default/sse"

echo "🔍 Running MCP Inspector against Glean server..."
echo "📍 Server: $SERVER_URL"

# Run MCP Inspector with Glean server
npx @modelcontextprotocol/inspector inspect $SERVER_URL \
    --output-format json \
    --validate-tools \
    --test-execution \
    > glean_inspector_results.json

# Check if all Glean tools are available
GLEAN_TOOLS=("glean_search" "chat" "read_document")
MISSING_TOOLS=()

for tool in "${GLEAN_TOOLS[@]}"; do
    if ! jq -e ".tools[] | select(.name == \"$tool\")" glean_inspector_results.json > /dev/null; then
        MISSING_TOOLS+=("$tool")
    fi
done

if [ ${#MISSING_TOOLS[@]} -eq 0 ]; then
    echo "✅ All Glean MCP tools validated successfully"
    echo "🚀 Proceeding to host application testing..."
    exit 0
else
    echo "❌ Missing Glean MCP tools: ${MISSING_TOOLS[*]}"
    echo "🛑 Cannot proceed to host testing until all tools are available"
    exit 1
fi
```

````

### FR-2: Host Application Configuration
**Priority**: P0 (Blocker)

**Description**: Automatically configure Glean MCP server connection in each supported host application using the correct authentication method.

**Acceptance Criteria**:
- [ ] Support Cursor with Bridge authentication (mcp-remote)
- [ ] Support VS Code with Native OAuth authentication
- [ ] Support Claude Desktop with Native OAuth authentication
- [ ] Support Windsurf with Bridge authentication (mcp-remote)
- [ ] Handle instance name substitution for server URLs
- [ ] Verify successful connection status in each host
- [ ] Support rollback to restore original configurations

**Host-Specific Configurations**:

**Cursor (Bridge Authentication)**:
```applescript
on configureGleanInCursor()
    set gleanInstance to "glean-dev-be"

    tell application "Cursor" to activate
    delay 2

    tell application "System Events"
        tell process "Cursor"
            -- Open Settings > MCP
            keystroke "," using command down
            delay 1
            click menu item "Features" of window 1
            click button "MCP" of window 1

            -- Add Glean MCP server
            click button "+ Add New MCP Server" of window 1

            -- Configure with mcp-remote bridge for glean-dev-be
            set value of text field "Name" to "glean"
            set value of text field "Command" to "npx"
            set gleanURL to "https://" & gleanInstance & ".glean.com/mcp/default/sse"
            set value of text field "Args" to "-y, mcp-remote, " & gleanURL

            click button "Save" of window 1
            return verifyGleanConnection("Cursor")
        end tell
    end tell
end configureGleanInCursor
````

**VS Code (Native OAuth)**:

```applescript
on configureGleanInVSCode()
    set gleanInstance to "glean-dev-be"

    tell application "Visual Studio Code" to activate
    delay 2

    tell application "System Events"
        tell process "Code"
            -- Open Command Palette
            keystroke "p" using {command down, shift down}
            delay 1

            -- Add MCP Server
            keystroke "MCP: Add Server"
            keystroke return
            delay 1

            -- Configure Glean server
            set gleanURL to "https://" & gleanInstance & ".glean.com/mcp/default/sse"
            set mcpConfig to "{\"servers\": {\"glean\": {\"type\": \"streamable-http\", \"url\": \"" & gleanURL & "\"}}}"
            keystroke mcpConfig

            keystroke "s" using command down -- Save
            return verifyGleanConnection("VS Code")
        end tell
    end tell
end configureGleanInVSCode
```

### FR-3: Glean-Specific Test Scenarios

**Priority**: P0 (Blocker)

**Description**: Execute test scenarios specifically designed for Glean's MCP tools and enterprise use cases.

**Acceptance Criteria**:

- [ ] Test enterprise search scenarios using glean_search tool
- [ ] Test Glean Chat assistant interactions
- [ ] Test document retrieval workflows
- [ ] Test multi-tool enterprise workflows (search → read → summarize)
- [ ] Validate Glean-specific response formats and citations
- [ ] Handle Glean permission-based access patterns

**Glean Test Scenarios**:

```json
{
  "glean_test_scenarios": [
    {
      "name": "Enterprise Search Test",
      "query": "Using Glean, search for our company's remote work policy",
      "expected_tool": "glean_search",
      "expected_response_contains": ["policy", "remote work", "search results"],
      "timeout_seconds": 30,
      "validation": "check_glean_search_format"
    },
    {
      "name": "Glean Chat Assistant Test",
      "query": "Ask Glean's assistant: What are the main benefits of using Glean?",
      "expected_tool": "chat",
      "expected_response_contains": ["Glean", "benefits", "search", "AI"],
      "timeout_seconds": 20,
      "validation": "check_glean_chat_response"
    },
    {
      "name": "Document Retrieval Test",
      "query": "Use Glean to read the document at [specific-glean-doc-url]",
      "expected_tool": "read_document",
      "expected_response_contains": ["document", "content"],
      "timeout_seconds": 15,
      "validation": "check_document_content"
    },
    {
      "name": "Enterprise Workflow Test",
      "query": "Search Glean for engineering guidelines, then read the top result and summarize the key points",
      "expected_tools": ["glean_search", "read_document", "chat"],
      "expected_response_contains": ["search", "document", "summary", "guidelines"],
      "timeout_seconds": 60,
      "validation": "check_multi_tool_workflow"
    },
    {
      "name": "People Search Test",
      "query": "Using Glean, find information about employees in the engineering team",
      "expected_tool": "employee_search",
      "expected_response_contains": ["employees", "engineering", "team"],
      "timeout_seconds": 20,
      "validation": "check_people_search"
    }
  ]
}
```

### FR-4: Authentication Method Testing

**Priority**: P1 (Important)

**Description**: Validate both OAuth Native and Bridge authentication methods work correctly across different host applications.

**Acceptance Criteria**:

- [ ] Test OAuth Native authentication in ChatGPT, Claude Code, Claude Desktop, VS Code
- [ ] Test Bridge authentication (mcp-remote) in Cursor, Windsurf, Goose
- [ ] Handle OAuth device flow approval automation
- [ ] Detect authentication failures and provide clear error messages
- [ ] Support API token fallback authentication
- [ ] Validate that authenticated sessions can access all available tools

**OAuth Flow Automation**:

```applescript
on handleOAuthFlow(hostApp)
    -- Monitor for OAuth device flow prompts
    tell application "System Events"
        tell process hostApp
            repeat 10 times
                if exists (button "Approve" of window 1) then
                    click button "Approve" of window 1
                    return true
                else if exists (text field "Device Code" of window 1) then
                    -- Handle device code entry if needed
                    return handleDeviceCodeEntry()
                end if
                delay 1
            end repeat
        end tell
    end tell
    return false
end handleOAuthFlow
```

### FR-5: Cross-Host Glean Integration Testing

**Priority**: P0 (Blocker)

**Description**: Execute identical Glean-specific test scenarios across all supported host applications and compare results.

**Acceptance Criteria**:

- [ ] Run same Glean test scenarios across all 6 validated host applications
- [ ] Adapt for host-specific UI patterns while maintaining test consistency
- [ ] Handle host-specific OAuth vs Bridge authentication differences
- [ ] Compare Glean tool responses across different hosts for consistency
- [ ] Generate compatibility matrix showing which tools work in which hosts
- [ ] Handle ChatGPT's special endpoint requirements

**Cross-Host Test Execution**:

```applescript
on runGleanTestAcrossHosts(testScenario)
    set gleanInstance to "glean-dev-be"
    set gleanHosts to {¬
        {name: "Cursor", authMethod: "bridge", configFunction: configureGleanInCursor}, ¬
        {name: "VS Code", authMethod: "native", configFunction: configureGleanInVSCode}, ¬
        {name: "Claude Desktop", authMethod: "native", configFunction: configureGleanInClaude}, ¬
        {name: "Windsurf", authMethod: "bridge", configFunction: configureGleanInWindsurf}¬
    }

    set hostResults to {}

    repeat with hostInfo in gleanHosts
        log "Testing Glean integration in " & (name of hostInfo)

        try
            -- Configure Glean MCP server for this host (using default glean-dev-be)
            set configResult to call (configFunction of hostInfo) with parameters {}

            if configResult then
                -- Run Glean-specific test
                set testResult to runGleanTestInHost(testScenario, hostInfo)
                set end of hostResults to testResult
            else
                set failedResult to {host: (name of hostInfo), success: false, error: "Configuration failed"}
                set end of hostResults to failedResult
            end if

        on error errorMsg
            set errorResult to {host: (name of hostInfo), success: false, error: errorMsg}
            set end of hostResults to errorResult
        end try

        delay 3 -- Allow cleanup between hosts
    end repeat

    return {scenario: testScenario, glean_results: hostResults}
end runGleanTestAcrossHosts
```

### FR-6: Glean-Specific Result Validation

**Priority**: P1 (Important)

**Description**: Validate that Glean MCP tool responses contain expected Glean-specific data structures and formatting.

**Acceptance Criteria**:

- [ ] Validate Glean search results contain proper citations and sources
- [ ] Check that Glean chat responses include Glean branding and context
- [ ] Verify document retrieval includes Glean metadata (permissions, source, etc.)
- [ ] Validate response timing meets Glean SLA requirements
- [ ] Check that enterprise permission controls are respected
- [ ] Generate Glean-specific compliance reports

**Glean Response Validation**:

```rust
use serde_json::Value;
use std::collections::HashMap;

pub struct GleanResponseValidator;

impl GleanResponseValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate Glean search response contains:
    /// - results array with title, url, snippet
    /// - citation information
    /// - permission metadata
    /// - search_id for tracking
    pub fn validate_search_response(&self, response: &Value) -> bool {
        let required_fields = ["results", "citations", "search_metadata"];

        required_fields.iter().all(|&field| {
            response.get(field).is_some()
        })
    }

    /// Validate Glean chat response contains:
    /// - response text with Glean context
    /// - source citations from Glean knowledge base
    /// - conversation_id for tracking
    pub fn validate_chat_response(&self, response: &Value) -> bool {
        let has_response = response.get("response").is_some();
        let has_citations = response.get("citations").is_some();
        let has_glean_context = response
            .get("response")
            .and_then(|r| r.as_str())
            .map_or(false, |text| text.to_lowercase().contains("glean"));

        has_response && has_citations && has_glean_context
    }
}

impl Default for GleanResponseValidator {
    fn default() -> Self {
        Self::new()
    }
}
```

## Technical Requirements

### TR-1: Glean Environment Support

- **Glean Instances**: Support both production and development Glean instances
- **Instance Discovery**: Automatically detect instance name from Admin Console
- **Authentication**: OAuth 2.0 device flow and Client API token support
- **Permissions**: Respect Glean's permission-aware access patterns

### TR-2: Host Application Compatibility

Based on Glean's validated compatibility matrix:

- **ChatGPT**: Native OAuth, centrally-managed configuration
- **Cursor**: Bridge authentication via mcp-remote, local configuration
- **Claude Code**: Native OAuth, command-line configuration
- **Claude Desktop**: Native OAuth, local configuration
- **VS Code**: Native OAuth, global and workspace configuration
- **Windsurf**: Bridge authentication via mcp-remote, local configuration

### TR-3: Glean MCP Tool Coverage

- **Core Tools**: glean_search, chat, read_document (always tested)
- **Configurable Tools**: code_search, employee_search, etc. (tested when enabled)
- **Tool Discovery**: Automatically detect which tools are enabled for the instance
- **Enterprise Features**: Support testing of enterprise-only tools

## Implementation Architecture

### Glean-Specific Components

```
┌─────────────────────────────────────────────────────────────────┐
│                Glean MCP Testing Framework                      │
├─────────────────────────────────────────────────────────────────┤
│  Glean Test Orchestrator                                        │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────┐│
│  │ Glean Instance  │  │ OAuth Manager    │  │ Glean Reporter  ││
│  │ Manager         │  │                  │  │                 ││
│  └─────────────────┘  └──────────────────┘  └─────────────────┘│
├─────────────────────────────────────────────────────────────────┤
│  Phase 0: Glean Server Validation                              │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Glean MCP Server Validator                                ││
│  │  - Test glean_search, chat, read_document                 ││
│  │  - Validate OAuth authentication                          ││
│  │  - Check enterprise tool availability                     ││
│  └─────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────┤
│  Phase 1: Host Application Testing                              │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────┐│
│  │  Cursor Tester  │  │  VS Code Tester  │  │ Claude Tester   ││
│  │  (Bridge Auth)  │  │  (Native OAuth)  │  │ (Native OAuth)  ││
│  └─────────────────┘  └──────────────────┘  └─────────────────┘│
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────┐│
│  │ ChatGPT Tester  │  │ Windsurf Tester  │  │ Goose Tester    ││
│  │ (Native OAuth)  │  │ (Bridge Auth)    │  │ (Bridge Auth)   ││
│  └─────────────────┘  └──────────────────┘  └─────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Configuration Management

```yaml
# glean_config.yaml
glean_instance:
  name: 'glean-dev-be' # Default development instance
  environment: 'development' # Primary testing environment
  server_url: 'https://glean-dev-be.glean.com/mcp/default/sse'
  chatgpt_url: 'https://glean-dev-be.glean.com/mcp/chatgpt'

mcp_inspector:
  package: '@modelcontextprotocol/inspector'
  validation_required: true
  tools_to_validate:
    - glean_search
    - chat
    - read_document

authentication:
  method: 'oauth' # or "api_token"
  oauth_scopes: ['MCP', 'SEARCH', 'TOOLS', 'ENTITIES']

tools_to_test:
  core_tools:
    - glean_search
    - chat
    - read_document
  enterprise_tools: # Only test if enabled by MCP Inspector
    - code_search
    - employee_search
    - gmail_search
    - meeting_lookup
    - outlook_search
    - web_browser

host_applications:
  cursor:
    auth_method: 'bridge'
    config_type: 'local'
    mcp_config_path: '~/.cursor/mcp.json'
    server_url: 'https://glean-dev-be.glean.com/mcp/default/sse'
  vscode:
    auth_method: 'native'
    config_type: 'global'
    mcp_config_path: '~/.vscode/settings.json'
    server_url: 'https://glean-dev-be.glean.com/mcp/default/sse'
  claude_desktop:
    auth_method: 'native'
    config_type: 'local'
    mcp_config_path: '~/Library/Application Support/Claude/claude_desktop_config.json'
    server_url: 'https://glean-dev-be.glean.com/mcp/default/sse'
```

## User Stories and Use Cases

### US-1: Enterprise QA Testing Glean MCP Integration

**As a** Glean QA engineer
**I want to** validate that all Glean MCP tools work correctly across customer environments
**So that** enterprise customers can reliably use Glean in their development workflows

**Acceptance Criteria**:

- Test all 10 Glean MCP tools across all validated host applications
- Verify OAuth authentication works for enterprise SSO setups
- Validate that Glean's permission system is respected in MCP responses
- Complete testing within 30 minutes for rapid iteration

### US-2: Customer Success Validation

**As a** Customer Success Manager
**I want to** verify Glean MCP integration works in a customer's preferred development environment
**So that** I can confidently recommend MCP integration during onboarding

**Acceptance Criteria**:

- Test against customer's specific Glean instance configuration
- Validate integration with customer's preferred host applications (Cursor, VS Code, etc.)
- Generate customer-facing reports showing successful integration
- Provide troubleshooting guidance for any failures

### US-3: Product Development Validation

**As a** Glean Product Manager
**I want to** understand how Glean MCP performs across different host applications
**So that** I can prioritize compatibility improvements and feature development

**Acceptance Criteria**:

- Generate compatibility matrix showing tool performance across hosts
- Identify which host applications provide the best Glean integration experience
- Track performance metrics for Glean tool response times
- Report on authentication method effectiveness (OAuth vs Bridge)

## Glean-Specific Success Criteria

The Glean MCP Testing Framework will be considered successful when:

- [ ] All 10 Glean MCP tools are validated across all 6 supported host applications
- [ ] Both OAuth Native and Bridge authentication methods work reliably
- [ ] Enterprise search scenarios complete successfully with proper Glean citations
- [ ] Glean chat assistant provides contextually appropriate responses
- [ ] Document retrieval respects Glean permission systems
- [ ] Multi-tool enterprise workflows (search → read → chat) execute successfully
- [ ] Test execution completes within 30 minutes for rapid feedback
- [ ] Framework generates actionable reports for customer troubleshooting

## Implementation Timeline

### Phase 1: Glean Integration Foundation with MCP Inspector (Weeks 1-2)

- [ ] Set up MCP Inspector integration for Glean server validation
- [ ] Configure default glean-dev-be instance connection and authentication
- [ ] Implement MCP Inspector-based tool discovery and validation
- [ ] Create automated MCP Inspector reporting and compliance checking
- [ ] Build Glean-specific response validation logic using Inspector schemas

### Phase 2: Host Application Integration (Weeks 3-4)

- [ ] Implement Bridge authentication testing (Cursor, Windsurf, Goose)
- [ ] Implement Native OAuth testing (ChatGPT, Claude Code, Claude Desktop, VS Code)
- [ ] Create host-specific Glean MCP configuration scripts
- [ ] Build Glean tool execution detection and validation

### Phase 3: Enterprise Scenario Testing (Weeks 5-6)

- [ ] Implement Glean enterprise search scenarios
- [ ] Create multi-tool workflow testing (search → read → chat)
- [ ] Build permission-aware testing for enterprise configurations
- [ ] Add Glean-specific performance and SLA validation

### Phase 4: Production Readiness (Week 7)

- [ ] End-to-end testing with real Glean customer instances
- [ ] Performance optimization for enterprise-scale testing
- [ ] Customer-facing documentation and troubleshooting guides
- [ ] Integration with Glean's support and monitoring systems

---

_This PRD serves as the complete specification for implementing the Glean MCP Server Testing Framework. All requirements are scoped specifically to Glean's MCP implementation, supported host applications, and enterprise use cases._
