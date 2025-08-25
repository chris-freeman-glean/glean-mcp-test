use clap::{Parser, Subcommand};
use console::{Emoji, Term, style};
use glean_mcp_test::{
    GleanConfig, GleanMcpError, HostController, HostOperationResult, Result,
    claude_code::ClaudeCodeController, run_list_tools, run_test_all, run_tool_test, run_validation,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// Define consistent emojis with fallbacks
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", ">> ");
static CHECKMARK: Emoji<'_, '_> = Emoji("✅ ", "[OK] ");
static CROSS_MARK: Emoji<'_, '_> = Emoji("❌ ", "[FAIL] ");
static MAGNIFYING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "[SEARCH] ");
static CLIPBOARD: Emoji<'_, '_> = Emoji("📋 ", "[INFO] ");
static GEAR: Emoji<'_, '_> = Emoji("🔧 ", "[TOOL] ");
static LOCK: Emoji<'_, '_> = Emoji("🔐 ", "[AUTH] ");
static PARTY: Emoji<'_, '_> = Emoji("🎉 ", "[SUCCESS] ");
static WARNING: Emoji<'_, '_> = Emoji("⚠️ ", "[WARN] ");

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

        /// Debug output (show full tool response data)
        #[arg(short, long)]
        debug: bool,

        /// Number of retry attempts for failed tests (default: 4)
        #[arg(long, default_value = "4")]
        retry_attempts: u32,

        /// Initial backoff time in seconds for retries with jitter (default: 5)
        #[arg(long, default_value = "5")]
        retry_backoff: u64,

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
        let term = Term::stderr();
        let _ = term.write_line(&format!(
            "{}{}",
            CROSS_MARK,
            style(format!("Command failed: {e}")).red().bold()
        ));
        std::process::exit(1);
    }
}

#[allow(clippy::cognitive_complexity)]
async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Inspect { instance, format } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "{}{}",
                ROCKET,
                style("Starting Glean MCP Inspector validation...")
                    .cyan()
                    .bold()
            ));
            let _ = term.write_line(&format!(
                "{}{} {}",
                CLIPBOARD,
                style("Instance:").bold(),
                style(&instance).cyan()
            ));

            match run_validation(Some(&instance)) {
                Ok(result) => {
                    if format == "json" {
                        match serde_json::to_string_pretty(&result) {
                            Ok(json_output) => println!("{json_output}"),
                            Err(e) => {
                                let _ = term.write_line(&format!(
                                    "{}{}",
                                    CROSS_MARK,
                                    style(format!("Failed to serialize JSON: {e}")).red()
                                ));
                                std::process::exit(1);
                            }
                        }
                    } else {
                        print_enhanced_text_result(&result);
                    }

                    let _ = term.write_line("");
                    if result.success {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            PARTY,
                            style("Validation completed successfully!").green().bold()
                        ));
                        let _ = term.write_line(&format!(
                            "{}{}",
                            ROCKET,
                            style("Ready to proceed to host application testing").blue()
                        ));
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Validation failed!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to run MCP Inspector: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }

        Commands::Config { verbose } => {
            let config = GleanConfig::default();

            let term = Term::stdout();

            if verbose {
                match serde_yaml::to_string(&config) {
                    Ok(config_yaml) => {
                        let _ = term.write_line(&format!(
                            "📋 {}\n{}",
                            style("Current Configuration:").bold().underlined(),
                            config_yaml
                        ));
                        let _ = term.write_line("");
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CHECKMARK,
                            style("Configuration displayed successfully!")
                                .green()
                                .bold()
                        ));
                        std::process::exit(0);
                    }
                    Err(e) => {
                        let term = Term::stderr();
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style(format!("Failed to serialize config: {e}")).red()
                        ));
                        std::process::exit(1);
                    }
                }
            } else {
                let _ = term.write_line(&format!(
                    "📋 {}: {}",
                    style("Glean Instance").bold(),
                    style(&config.glean_instance.name).cyan()
                ));
                let _ = term.write_line(&format!(
                    "🔗 {}: {}",
                    style("Server URL").bold(),
                    style(&config.glean_instance.server_url).dim()
                ));
                let _ = term.write_line(&format!(
                    "🔧 {}: {}",
                    style("Inspector Package").bold(),
                    style(&config.mcp_inspector.package).cyan()
                ));
                let _ = term.write_line(&format!(
                    "🔑 {}: {}",
                    style("Auth Method").bold(),
                    style(&config.authentication.method).cyan()
                ));
                let _ = term.write_line(&format!(
                    "📊 {}: {}",
                    style("Core Tools").bold(),
                    style(config.tools_to_test.core_tools.len().to_string()).cyan()
                ));
                let _ = term.write_line(&format!(
                    "🏢 {}: {}",
                    style("Enterprise Tools").bold(),
                    style(config.tools_to_test.enterprise_tools.len().to_string()).cyan()
                ));
                let _ = term.write_line(&format!(
                    "💻 {}: {}",
                    style("Host Applications").bold(),
                    style(config.host_applications.len().to_string()).cyan()
                ));
                let _ = term.write_line("");
                let _ = term.write_line(&format!(
                    "{}{}",
                    CHECKMARK,
                    style("Configuration displayed successfully!")
                        .green()
                        .bold()
                ));
                std::process::exit(0);
            }
        }

        Commands::Prerequisites => match check_prerequisites_with_progress().await {
            Ok(()) => {
                let term = Term::stdout();
                let _ = term.write_line("");
                let _ = term.write_line(&format!(
                    "{}{}",
                    PARTY,
                    style("Prerequisites check completed successfully!")
                        .green()
                        .bold()
                ));
                std::process::exit(0);
            }
            Err(e) => {
                let term = Term::stderr();
                let _ = term.write_line("");
                let _ = term.write_line(&format!(
                    "{}{}",
                    CROSS_MARK,
                    style(format!("Prerequisites check failed: {e}")).red()
                ));
                std::process::exit(1);
            }
        },

        Commands::Auth { instance } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "{}{} {}",
                LOCK,
                style("Testing authentication for Glean instance:")
                    .cyan()
                    .bold(),
                style(&instance).yellow()
            ));

            // Create progress bar for authentication steps
            let auth_pb = ProgressBar::new(3);
            auth_pb.set_style(ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] {bar:30.cyan/blue} {pos:>1}/{len:1} {msg}"
            ).unwrap_or_else(|_| ProgressStyle::default_bar()));
            auth_pb.enable_steady_tick(Duration::from_millis(100));

            auth_pb.set_message("Checking environment variables...");

            // Check GLEAN_AUTH_TOKEN environment variable
            let _ = term.write_line("");
            let _ = term.write_line(&format!(
                "{}{}",
                MAGNIFYING_GLASS,
                style("Checking GLEAN_AUTH_TOKEN environment variable:").bold()
            ));

            #[allow(clippy::option_if_let_else)]
            let found_token = if let Ok(value) = std::env::var("GLEAN_AUTH_TOKEN") {
                let masked = if value.len() > 8 {
                    format!("{}...{}", &value[..4], &value[value.len() - 4..])
                } else {
                    "***".to_string()
                };
                let _ = term.write_line(&format!(
                    "  {}{} {}",
                    CHECKMARK,
                    style("GLEAN_AUTH_TOKEN:").green(),
                    style(masked).dim()
                ));
                true
            } else {
                let _ = term.write_line(&format!(
                    "  {}{}",
                    CROSS_MARK,
                    style("GLEAN_AUTH_TOKEN: not set").red()
                ));
                false
            };
            auth_pb.inc(1);

            if !found_token {
                auth_pb.finish_with_message(
                    style("❌ No authentication token found").red().to_string(),
                );
                let _ = term.write_line("");
                let _ = term.write_line(&format!(
                    "💡 {}",
                    style("No authentication token found.").yellow()
                ));
                let _ = term.write_line(&format!(
                    "   {}: {}",
                    style("Set the Glean auth token").bold(),
                    style("export GLEAN_AUTH_TOKEN=your_token_here").cyan()
                ));
                let _ = term.write_line("");
                let _ = term.write_line(&format!("🔗 {}", style("For mise users:").bold()));
                let _ = term.write_line(&format!(
                    "   {}",
                    style("mise set GLEAN_AUTH_TOKEN=your_token_here").cyan()
                ));
                std::process::exit(1);
            }

            auth_pb.set_message("Testing server connection...");
            auth_pb.inc(1);

            let _ = term.write_line("");
            let _ = term.write_line(&format!(
                "{}{}",
                ROCKET,
                style("Running authentication test...").cyan()
            ));

            match run_validation(Some(&instance)) {
                Ok(result) => {
                    auth_pb.inc(1);

                    if result.success {
                        auth_pb.finish_with_message(format!(
                            "{}{}",
                            CHECKMARK,
                            style("Authentication successful").green()
                        ));
                        let _ = term.write_line("");
                        let _ = term.write_line(&format!(
                            "{}{}",
                            PARTY,
                            style("Authentication test successful!").green().bold()
                        ));
                        std::process::exit(0);
                    } else {
                        auth_pb.finish_with_message(
                            style("❌ Authentication failed").red().to_string(),
                        );
                        let _ = term.write_line("");
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Authentication test failed!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    auth_pb
                        .finish_with_message(style("❌ Test execution failed").red().to_string());
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to run authentication test: {e}")).red()
                    ));
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
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "🔧 Testing MCP tool: {} with query: \"{}\"",
                style(&tool).cyan().bold(),
                style(&query).dim()
            ));
            let _ = term.write_line(&format!("📋 Instance: {}", style(&instance).cyan()));

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
                            let _ = term.write_line("");
                            let _ = term.write_line(&format!(
                                "{}{}",
                                PARTY,
                                style("Tool test completed successfully!").green().bold()
                            ));
                            if let Some(response_data) = &result.inspector_data {
                                let _ =
                                    term.write_line(&format!("📄 {}:", style("Response").bold()));
                                let response_json = serde_json::to_string_pretty(response_data)
                                    .unwrap_or_else(|_| "No response data".to_string());
                                let _ = term.write_line(&response_json);
                            }
                        }
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Tool test failed!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let term = Term::stderr();
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to run tool test: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }

        Commands::ListTools { instance, format } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "📋 {}",
                style("Listing available tools from MCP server")
                    .cyan()
                    .bold()
            ));
            let _ = term.write_line(&format!("📋 Instance: {}", style(&instance).cyan()));

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
                            let _ = term.write_line("");
                            let _ = term.write_line(&format!(
                                "{}{}",
                                PARTY,
                                style("Tools listed successfully!").green().bold()
                            ));
                        }
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Failed to list tools!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let term = Term::stderr();
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to list tools: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }

        Commands::VerifyHost { host, format } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "🔍 Verifying MCP servers in host: {}",
                style(&host).cyan().bold()
            ));

            match run_host_operation(&host, "verify", "", None, None, &format).await {
                Ok(result) => {
                    if result.success {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CHECKMARK,
                            style("Host verification completed successfully!")
                                .green()
                                .bold()
                        ));
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Host verification failed!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to verify host: {e}")).red()
                    ));
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
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "🧪 Testing Glean tool '{}' on host '{}' with query: \"{}\"",
                style(&tool).cyan(),
                style(&host).cyan(),
                style(&query).dim()
            ));

            match run_host_operation(&host, "test_tool", "", Some(&tool), Some(&query), &format)
                .await
            {
                Ok(result) => {
                    if result.success {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CHECKMARK,
                            style("Glean tool test completed successfully!")
                                .green()
                                .bold()
                        ));
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Glean tool test failed!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to test Glean tool on host: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }

        Commands::TestAllHostTools { host, format } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "🧪 Testing all Glean tools on host: {}",
                style(&host).cyan().bold()
            ));

            match run_host_operation(&host, "test_all", "", None, None, &format).await {
                Ok(result) => {
                    if result.success {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CHECKMARK,
                            style("All Glean tools test completed successfully!")
                                .green()
                                .bold()
                        ));
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Some Glean tools failed!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to test all Glean tools: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }

        Commands::CheckHost { host, format } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "🔍 Checking if host application '{}' is available",
                style(&host).cyan().bold()
            ));

            match check_host_availability(&host, &format) {
                Ok(available) => {
                    if available {
                        let _ = term.write_line(&format!(
                            "{}{} '{}' is available and ready for testing",
                            CHECKMARK,
                            style("Host").green(),
                            style(host).cyan()
                        ));
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{} '{}' is not available",
                            CROSS_MARK,
                            style("Host").red(),
                            style(host).cyan()
                        ));
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to check host availability: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }

        Commands::ListHostServers { host, format } => {
            let term = Term::stdout();
            let _ = term.write_line(&format!(
                "📋 Listing MCP servers in host: {}",
                style(&host).cyan().bold()
            ));

            match run_host_operation(&host, "list", "", None, None, &format).await {
                Ok(result) => {
                    if result.success {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CHECKMARK,
                            style("MCP servers listed successfully!").green().bold()
                        ));
                        std::process::exit(0);
                    } else {
                        let _ = term.write_line(&format!(
                            "{}{}",
                            CROSS_MARK,
                            style("Failed to list MCP servers!").red().bold()
                        ));
                        if let Some(error) = &result.error {
                            let _ = term.write_line(&format!("Error: {}", style(error).red()));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to list MCP servers: {e}")).red()
                    ));
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
            debug,
            retry_attempts,
            retry_backoff,
            json,
            output,
        } => {
            // Determine the actual format to use (--json flag overrides --format)
            let actual_format = if json { "json".to_string() } else { format };

            let term = Term::stdout();

            // Only show progress for non-JSON output
            if actual_format != "json" {
                // Clean header
                let _ = term.write_line(&format!(
                    "\n{} {}",
                    GEAR,
                    style("Glean MCP Tool Testing").cyan().bold()
                ));

                // Configuration summary - clean and compact
                let _ = term.write_line(&format!(
                    "📋 {} | 🔧 {} | ⚡ {} {}",
                    style(&instance).cyan(),
                    style(&tools).cyan(),
                    if parallel { "Parallel" } else { "Sequential" },
                    if parallel {
                        format!("({})", style(max_concurrent.to_string()).dim())
                    } else {
                        String::new()
                    }
                ));

                let _ = term.write_line("");
            }

            let test_options = glean_mcp_test::TestAllOptions {
                tools_filter: tools,
                scenario,
                parallel,
                max_concurrent,
                timeout,
                verbose,
                debug,
                retry_attempts,
                retry_backoff_seconds: retry_backoff,
                format: actual_format.clone(),
            };

            match run_test_all(Some(&instance), &test_options) {
                Ok(result) => {
                    let output_content = result.format_output(&actual_format, verbose, debug);

                    if let Some(output_file) = output {
                        match std::fs::write(&output_file, &output_content) {
                            Ok(()) => {
                                let _ = term.write_line(&format!(
                                    "📄 Results written to: {}",
                                    style(&output_file).cyan()
                                ));
                            }
                            Err(e) => {
                                let _ = term.write_line(&format!(
                                    "{}{}",
                                    CROSS_MARK,
                                    style(format!("Failed to write output file: {e}")).red()
                                ));
                                std::process::exit(1);
                            }
                        }
                    } else if actual_format == "json" {
                        // For JSON output, print directly without styling
                        println!("{output_content}");
                    } else {
                        // For text output, use console
                        let _ = term.write_line(&output_content);
                    }

                    if result.success {
                        if actual_format != "json" {
                            let _ = term.write_line(&format!(
                                "\n{}{}",
                                PARTY,
                                style("All tests completed successfully!").green().bold()
                            ));
                        }
                        std::process::exit(0);
                    } else {
                        if actual_format != "json" {
                            let _ = term.write_line(&format!(
                                "\n{}{}",
                                CROSS_MARK,
                                style("Some tools failed testing!").red().bold()
                            ));
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let _ = term.write_line(&format!(
                        "{}{}",
                        CROSS_MARK,
                        style(format!("Failed to run test-all: {e}")).red()
                    ));
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_enhanced_text_result(result: &glean_mcp_test::InspectorResult) {
    let term = Term::stdout();

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "📊 {}",
        style("MCP Inspector Results").bold().underlined()
    ));
    let _ = term.write_line(&style("─".repeat(50)).dim().to_string());

    // Status with enhanced styling
    let status_text = if result.success {
        format!("{}{}", CHECKMARK, style("SUCCESS").green().bold())
    } else {
        format!("{}{}", CROSS_MARK, style("FAILED").red().bold())
    };
    let _ = term.write_line(&format!("Status: {status_text}"));

    if let Some(tool_results) = &result.tool_results {
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}{}",
            GEAR,
            style("Tool Validation Results:").bold()
        ));
        let _ = term.write_line(&style("─".repeat(30)).dim().to_string());

        for (tool, success) in tool_results {
            let (emoji, tool_style) = if *success {
                (CHECKMARK, style(tool).green())
            } else {
                (CROSS_MARK, style(tool).red())
            };
            let _ = term.write_line(&format!("  {emoji}{tool_style}"));
        }
    }

    if let Some(error) = &result.error {
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}{}",
            WARNING,
            style("Error Details:").red().bold()
        ));
        let _ = term.write_line(&format!("  {}", style(error).dim()));
    }
}

async fn check_prerequisites_with_progress() -> Result<()> {
    let term = Term::stdout();
    let _ = term.write_line(&format!(
        "{}{}",
        MAGNIFYING_GLASS,
        style("Checking system prerequisites...").cyan().bold()
    ));

    // Create progress bar for prerequisites checking
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:30.cyan/blue} {pos:>1}/{len:1} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    // Check if npx is available
    pb.set_message("Checking Node.js/npm...");
    if let Ok(output) = std::process::Command::new("npx").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            let _ = term.write_line(&format!(
                "{}{} {}",
                CHECKMARK,
                style("npx available:").green(),
                style(version.trim()).dim()
            ));
        } else {
            pb.finish_with_message(style("❌ npx command failed").red().to_string());
            let _ = term.write_line(&format!(
                "{}{}",
                CROSS_MARK,
                style("npx command failed").red()
            ));
            return Err(GleanMcpError::Config("npx not available".to_string()));
        }
    } else {
        pb.finish_with_message(style("❌ npx not found").red().to_string());
        let _ = term.write_line(&format!("{}{}", CROSS_MARK, style("npx not found").red()));
        let _ = term.write_line(
            &style("Please install Node.js and npm to use MCP Inspector")
                .yellow()
                .to_string(),
        );
        return Err(GleanMcpError::Config("npx not found".to_string()));
    }
    pb.inc(1);

    // Add small delay for visual effect
    smol::Timer::after(Duration::from_millis(200)).await;

    // Check if MCP Inspector package is available
    pb.set_message("Checking MCP Inspector...");
    match std::process::Command::new("npx")
        .args(["@modelcontextprotocol/inspector", "--help"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let _ = term.write_line(&format!(
                    "{}{}",
                    CHECKMARK,
                    style("MCP Inspector available").green()
                ));
            } else {
                let _ = term.write_line(&format!(
                    "{}{}",
                    WARNING,
                    style("MCP Inspector may need to be installed").yellow()
                ));
                let _ = term.write_line(&format!(
                    "  {}: {}",
                    style("Run").bold(),
                    style("npx @modelcontextprotocol/inspector --help").cyan()
                ));
            }
        }
        Err(_) => {
            let _ = term.write_line(&format!(
                "{}{}",
                WARNING,
                style("Could not check MCP Inspector").yellow()
            ));
        }
    }
    pb.inc(1);

    // Add small delay for visual effect
    smol::Timer::after(Duration::from_millis(200)).await;

    // Check curl availability
    pb.set_message("Checking curl...");
    if let Ok(output) = std::process::Command::new("curl").arg("--version").output() {
        if output.status.success() {
            let _ = term.write_line(&format!("{}{}", CHECKMARK, style("curl available").green()));
        } else {
            let _ = term.write_line(&format!(
                "{}{}",
                WARNING,
                style("curl command failed").yellow()
            ));
        }
    } else {
        let _ = term.write_line(&format!("{}{}", WARNING, style("curl not found").yellow()));
        let _ = term.write_line(
            &style("curl is required for MCP server testing")
                .yellow()
                .to_string(),
        );
    }
    pb.inc(1);

    // Add small delay for visual effect
    smol::Timer::after(Duration::from_millis(200)).await;

    // Check environment variables
    pb.set_message("Checking environment...");
    if std::env::var("GLEAN_AUTH_TOKEN").is_ok() {
        let _ = term.write_line(&format!(
            "{}{}",
            CHECKMARK,
            style("GLEAN_AUTH_TOKEN configured").green()
        ));
    } else {
        let _ = term.write_line(&format!(
            "{}{}",
            WARNING,
            style("GLEAN_AUTH_TOKEN not set (optional)").yellow()
        ));
    }
    pb.inc(1);

    pb.finish_with_message(format!(
        "{}{}",
        CHECKMARK,
        style("Prerequisites check complete").green()
    ));

    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "💡 {}: {}",
        style("Next step").bold(),
        style("glean-mcp-test inspect").cyan()
    ));

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
