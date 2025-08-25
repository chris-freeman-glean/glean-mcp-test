use clap::{Parser, Subcommand};
use glean_mcp_test::{
    GleanConfig, GleanMcpError, HostController, HostOperationResult, Result,
    claude_code::ClaudeCodeController, run_list_tools, run_test_all, run_tool_test, run_validation,
};

#[derive(Parser)]
#[command(name = "glean-mcp-test")]
#[command(
    about = "Glean MCP Testing Framework - Validate Glean's MCP server across host applications"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate Glean MCP server using MCP Inspector
    Inspect {
        /// Glean instance name (default: scio-prod)
        #[arg(short, long, default_value = "scio-prod")]
        instance: String,

        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show current configuration
    Config {
        /// Show full configuration details
        #[arg(short, long)]
        verbose: bool,
    },

    /// Check system prerequisites
    Prerequisites,

    /// Test authentication with current environment variables
    Auth {
        /// Glean instance name (default: scio-prod)
        #[arg(short, long, default_value = "scio-prod")]
        instance: String,
    },

    /// Test a specific MCP tool with a query
    TestTool {
        /// Tool name (search, chat, `read_document`, etc.)
        #[arg(short, long)]
        tool: String,

        /// Query to send to the tool
        #[arg(short, long)]
        query: String,

        /// Glean instance name (default: scio-prod)
        #[arg(short, long, default_value = "scio-prod")]
        instance: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// List available tools from the MCP server
    ListTools {
        /// Glean instance name (default: scio-prod)
        #[arg(short, long, default_value = "scio-prod")]
        instance: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Verify MCP servers are configured and list available tools in a host
    VerifyHost {
        /// Host application (claude-code, cursor, vscode, claude-desktop)
        #[arg(short = 'H', long)]
        host: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Test a specific Glean tool through a host application
    TestHostTool {
        /// Host application (claude-code, cursor, vscode, claude-desktop)
        #[arg(short = 'H', long)]
        host: String,

        /// Tool name (`glean_search`, chat, `read_document`, etc.)
        #[arg(short, long)]
        tool: String,

        /// Query to send to the tool
        #[arg(short, long)]
        query: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Test all available Glean tools through a host application
    TestAllHostTools {
        /// Host application (claude-code, cursor, vscode, claude-desktop)
        #[arg(short = 'H', long)]
        host: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Check if a host application is available
    CheckHost {
        /// Host application (claude-code, cursor, vscode, claude-desktop)
        #[arg(short = 'H', long)]
        host: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// List all configured MCP servers in a host application
    ListHostServers {
        /// Host application (claude-code, cursor, vscode, claude-desktop)
        #[arg(short = 'H', long)]
        host: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Test all available MCP tools and report status
    TestAll {
        /// Glean instance name (default: glean-dev)
        #[arg(short, long, default_value = "glean-dev")]
        instance: String,

        /// Output format (text, json, summary)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Tools to test (comma-separated list, or 'core'/'enterprise'/'all')
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Test scenario (quick, comprehensive, custom)
        #[arg(short, long, default_value = "quick")]
        scenario: String,

        /// Enable parallel testing
        #[arg(short, long)]
        parallel: bool,

        /// Maximum concurrent tests when parallel is enabled
        #[arg(long, default_value = "3")]
        max_concurrent: usize,

        /// Timeout per tool test in seconds
        #[arg(long, default_value = "60")]
        timeout: u64,

        /// Verbose output (show detailed results)
        #[arg(short, long)]
        verbose: bool,

        /// Output results as JSON (shortcut for --format json)
        #[arg(long)]
        json: bool,

        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // For async operations, use smol::block_on
    if let Err(e) = smol::block_on(async { handle_command(cli.command).await }) {
        eprintln!("❌ Command failed: {}", e);
        std::process::exit(1);
    }
}

#[allow(clippy::cognitive_complexity)]
async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Inspect { instance, format } => {
            println!("🚀 Starting Glean MCP Inspector validation...");
            println!("📋 Instance: {instance}");

            match run_validation(Some(&instance)) {
                Ok(result) => {
                    if format == "json" {
                        match serde_json::to_string_pretty(&result) {
                            Ok(json_output) => println!("{}", json_output),
                            Err(e) => {
                                eprintln!("❌ Failed to serialize JSON: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        print_text_result(&result);
                    }

                    if result.success {
                        println!("\n🎉 Validation completed successfully!");
                        println!("🚀 Ready to proceed to host application testing");
                        std::process::exit(0);
                    } else {
                        println!("\n❌ Validation failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to run MCP Inspector: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Config { verbose } => {
            let config = GleanConfig::default();

            if verbose {
                match serde_yaml::to_string(&config) {
                    Ok(config_yaml) => {
                        println!("📋 Current Configuration:\n{}", config_yaml);
                        println!("\n✅ Configuration displayed successfully!");
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to serialize config: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("📋 Glean Instance: {}", config.glean_instance.name);
                println!("🔗 Server URL: {}", config.glean_instance.server_url);
                println!("🔧 Inspector Package: {}", config.mcp_inspector.package);
                println!("🔑 Auth Method: {}", config.authentication.method);
                println!("📊 Core Tools: {}", config.tools_to_test.core_tools.len());
                println!(
                    "🏢 Enterprise Tools: {}",
                    config.tools_to_test.enterprise_tools.len()
                );
                println!("💻 Host Applications: {}", config.host_applications.len());
                println!("\n✅ Configuration displayed successfully!");
                std::process::exit(0);
            }
        }

        Commands::Prerequisites => match check_prerequisites() {
            Ok(_) => {
                println!("\n✅ Prerequisites check completed successfully!");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("\n❌ Prerequisites check failed: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Auth { instance } => {
            println!("🔐 Testing authentication for Glean instance: {instance}");

            // Check GLEAN_AUTH_TOKEN environment variable
            println!("\n🔍 Checking GLEAN_AUTH_TOKEN environment variable:");
            #[allow(clippy::option_if_let_else)]
            let found_token = if let Ok(value) = std::env::var("GLEAN_AUTH_TOKEN") {
                let masked = if value.len() > 8 {
                    format!("{}...{}", &value[..4], &value[value.len() - 4..])
                } else {
                    "***".to_string()
                };
                println!("  ✅ GLEAN_AUTH_TOKEN: {masked}");
                true
            } else {
                println!("  ❌ GLEAN_AUTH_TOKEN: not set");
                false
            };

            if !found_token {
                println!("\n💡 No authentication token found.");
                println!("   Set the Glean auth token:");
                println!("   export GLEAN_AUTH_TOKEN=your_token_here");
                println!("\n🔗 For mise users:");
                println!("   mise set GLEAN_AUTH_TOKEN=your_token_here");
                std::process::exit(1);
            }

            println!("\n🚀 Running authentication test...");
            match run_validation(Some(&instance)) {
                Ok(result) => {
                    if result.success {
                        println!("\n✅ Authentication test successful!");
                        std::process::exit(0);
                    } else {
                        println!("\n❌ Authentication test failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Failed to run authentication test: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::TestTool {
            tool,
            query,
            instance,
            format,
        } => {
            println!("🔧 Testing MCP tool: {tool} with query: \"{query}\"");
            println!("📋 Instance: {instance}");

            match run_tool_test(&tool, &query, Some(&instance), &format) {
                Ok(result) => {
                    if result.success {
                        if format == "json" {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&result)
                                    .unwrap_or_else(|_| "{}".to_string())
                            );
                        } else {
                            println!("\n🎉 Tool test completed successfully!");
                            if let Some(response_data) = &result.inspector_data {
                                println!("📄 Response:");
                                println!(
                                    "{}",
                                    serde_json::to_string_pretty(response_data)
                                        .unwrap_or_else(|_| "No response data".to_string())
                                );
                            }
                        }
                        std::process::exit(0);
                    } else {
                        println!("❌ Tool test failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to run tool test: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::ListTools { instance, format } => {
            println!("📋 Listing available tools from MCP server");
            println!("📋 Instance: {instance}");

            match run_list_tools(Some(&instance), &format) {
                Ok(result) => {
                    if result.success {
                        if format == "json" {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&result)
                                    .unwrap_or_else(|_| "{}".to_string())
                            );
                        } else {
                            println!("\n🎉 Tools listed successfully!");
                        }
                        std::process::exit(0);
                    } else {
                        println!("❌ Failed to list tools!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to list tools: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::VerifyHost { host, format } => {
            println!("🔍 Verifying MCP servers in host: {host}");

            match run_host_operation(&host, "verify", "", None, None, &format).await {
                Ok(result) => {
                    if result.success {
                        println!("✅ Host verification completed successfully!");
                        std::process::exit(0);
                    } else {
                        println!("❌ Host verification failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to verify host: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::TestHostTool {
            host,
            tool,
            query,
            format,
        } => {
            println!("🧪 Testing Glean tool '{tool}' on host '{host}' with query: \"{query}\"");

            match run_host_operation(&host, "test_tool", "", Some(&tool), Some(&query), &format)
                .await
            {
                Ok(result) => {
                    if result.success {
                        println!("✅ Glean tool test completed successfully!");
                        std::process::exit(0);
                    } else {
                        println!("❌ Glean tool test failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to test Glean tool on host: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::TestAllHostTools { host, format } => {
            println!("🧪 Testing all Glean tools on host: {host}");

            match run_host_operation(&host, "test_all", "", None, None, &format).await {
                Ok(result) => {
                    if result.success {
                        println!("✅ All Glean tools test completed successfully!");
                        std::process::exit(0);
                    } else {
                        println!("❌ Some Glean tools failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to test all Glean tools: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::CheckHost { host, format } => {
            println!("🔍 Checking if host application '{host}' is available");

            match check_host_availability(&host, &format) {
                Ok(available) => {
                    if available {
                        println!("✅ Host '{host}' is available and ready for testing");
                        std::process::exit(0);
                    } else {
                        println!("❌ Host '{host}' is not available");
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to check host availability: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::ListHostServers { host, format } => {
            println!("📋 Listing MCP servers in host: {host}");

            match run_host_operation(&host, "list", "", None, None, &format).await {
                Ok(result) => {
                    if result.success {
                        println!("✅ MCP servers listed successfully!");
                        std::process::exit(0);
                    } else {
                        println!("❌ Failed to list MCP servers!");
                        if let Some(error) = &result.error {
                            println!("Error: {error}");
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to list MCP servers: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::TestAll {
            instance,
            format,
            tools,
            scenario,
            parallel,
            max_concurrent,
            timeout,
            verbose,
            json,
            output,
        } => {
            // Determine the actual format to use (--json flag overrides --format)
            let actual_format = if json {
                "json".to_string()
            } else {
                format.clone()
            };

            println!("🧪 Testing all available MCP tools");
            println!("📋 Instance: {}", instance);
            println!("🔧 Tools filter: {}", tools);
            println!("📊 Scenario: {}", scenario);
            println!("⚡ Parallel: {}", parallel);

            if parallel {
                println!("🚀 Max concurrent: {}", max_concurrent);
            }
            println!("⏱️  Timeout per tool: {}s", timeout);

            let test_options = glean_mcp_test::TestAllOptions {
                tools_filter: tools.clone(),
                scenario: scenario.clone(),
                parallel,
                max_concurrent,
                timeout,
                verbose,
                format: actual_format.clone(),
            };

            match run_test_all(Some(&instance), &test_options) {
                Ok(result) => {
                    let output_content = result.format_output(&actual_format, verbose);

                    if let Some(output_file) = output {
                        match std::fs::write(&output_file, &output_content) {
                            Ok(_) => println!("📄 Results written to: {}", output_file),
                            Err(e) => {
                                eprintln!("❌ Failed to write output file: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        println!("{}", output_content);
                    }

                    if result.success {
                        println!("\n🎉 All tool testing completed successfully!");
                        std::process::exit(0);
                    } else {
                        println!("\n❌ Some tools failed testing!");
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to run test-all: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_text_result(result: &glean_mcp_test::InspectorResult) {
    println!("\n📊 MCP Inspector Results:");
    println!(
        "Status: {}",
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    if let Some(tool_results) = &result.tool_results {
        println!("\n🔧 Tool Validation Results:");
        for (tool, success) in tool_results {
            let status = if *success { "✅" } else { "❌" };
            println!("  {status} {tool}");
        }
    }

    if let Some(error) = &result.error {
        println!("\n⚠️  Error Details: {error}");
    }
}

fn check_prerequisites() -> Result<()> {
    println!("🔍 Checking system prerequisites...");

    // Check if npx is available
    if let Ok(output) = std::process::Command::new("npx").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ npx available: {}", version.trim());
        } else {
            println!("❌ npx command failed");
            return Err(GleanMcpError::Config("npx not available".to_string()));
        }
    } else {
        println!("❌ npx not found");
        println!("Please install Node.js and npm to use MCP Inspector");
        return Err(GleanMcpError::Config("npx not found".to_string()));
    }

    // Check if MCP Inspector package is available
    println!("🔍 Checking MCP Inspector availability...");
    match std::process::Command::new("npx")
        .args(["@modelcontextprotocol/inspector", "--help"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("✅ MCP Inspector available");
            } else {
                println!("⚠️  MCP Inspector may need to be installed");
                println!("Run: npx @modelcontextprotocol/inspector --help");
            }
        }
        Err(_) => {
            println!("⚠️  Could not check MCP Inspector");
        }
    }

    println!("🎯 Prerequisites check completed!");
    println!("Run 'glean-mcp-test inspect' to test MCP server connection");

    Ok(())
}

/// Create a Claude Code controller (only supported host for now)
fn create_claude_code_controller(host: &str) -> Result<ClaudeCodeController> {
    match host {
        "claude-code" => Ok(ClaudeCodeController::new()),
        _ => Err(GleanMcpError::Host(format!(
            "Unsupported host application: '{host}'. Supported hosts: claude-code"
        ))),
    }
}

/// Run a host operation (configure, verify, `test_tool`, rollback)
async fn run_host_operation(
    host: &str,
    operation: &str,
    instance: &str,
    tool: Option<&str>,
    query: Option<&str>,
    format: &str,
) -> Result<HostOperationResult> {
    let controller = create_claude_code_controller(host)?;

    // Note: Server URL generation no longer needed for testing approach
    let _server_url = format!("https://{instance}-be.glean.com/mcp/default");

    let result = match operation {
        "verify" => controller.verify_mcp_server().await?,
        "test_tool" => {
            let tool_name = tool.ok_or_else(|| {
                GleanMcpError::Host("Tool name is required for test_tool operation".to_string())
            })?;
            let query_text = query.ok_or_else(|| {
                GleanMcpError::Host("Query is required for test_tool operation".to_string())
            })?;
            controller.test_glean_tool(tool_name, query_text).await?
        }
        "test_all" => controller.test_all_glean_tools().await?,
        "list" => controller.list_mcp_servers().await?,
        _ => {
            return Err(GleanMcpError::Host(format!(
                "Unknown operation: {operation}. Available: verify, test_tool, test_all, list"
            )));
        }
    };

    // Print result based on format
    if format == "json" {
        let json_output = serde_json::to_string_pretty(&result).map_err(GleanMcpError::Json)?;
        println!("{json_output}");
    } else {
        print_host_result(&result);
    }

    Ok(result)
}

/// Check if a host application is available
fn check_host_availability(host: &str, format: &str) -> Result<bool> {
    let controller = create_claude_code_controller(host)?;
    let available = controller.check_availability()?;

    if format == "json" {
        let result = serde_json::json!({
            "host": host,
            "available": available,
            "operation": "check_availability"
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
        );
    } else if available {
        println!("✅ Host '{host}' is available");
    } else {
        println!("❌ Host '{host}' is not available");
    }

    Ok(available)
}

/// Print host operation result in text format
fn print_host_result(result: &HostOperationResult) {
    println!("\n📊 Host Operation Results:");
    println!("Host: {}", result.host);
    println!("Operation: {}", result.operation);
    println!(
        "Status: {}",
        if result.success {
            "✅ Success"
        } else {
            "❌ Failed"
        }
    );

    if !result.details.is_empty() {
        println!("Details: {}", result.details);
    }

    if let Some(error) = &result.error {
        println!("⚠️  Error: {error}");
    }

    if let Some(duration) = result.duration {
        println!("⏱️  Duration: {duration:?}");
    }
}
