use clap::{Parser, Subcommand};
use glean_mcp_test::{GleanConfig, GleanMcpError, Result, run_validation};

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
        /// Glean instance name (default: glean-dev-be)
        #[arg(short, long, default_value = "glean-dev-be")]
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
        /// Glean instance name (default: glean-dev-be)
        #[arg(short, long, default_value = "glean-dev-be")]
        instance: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { instance, format } => {
            println!("🚀 Starting Glean MCP Inspector validation...");
            println!("📋 Instance: {}", instance);

            match run_validation(Some(&instance)) {
                Ok(result) => {
                    if format == "json" {
                        let json_output = serde_json::to_string_pretty(&result)
                            .map_err(|e| GleanMcpError::Json(e))?;
                        println!("{}", json_output);
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
                            println!("Error: {}", error);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to run MCP Inspector: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Config { verbose } => {
            let config = GleanConfig::default();
            if verbose {
                let config_yaml = serde_yaml::to_string(&config).map_err(|e| {
                    GleanMcpError::Config(format!("Failed to serialize config: {}", e))
                })?;
                println!("📋 Current Configuration:\n{}", config_yaml);
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
            }
            Ok(())
        }

        Commands::Prerequisites => check_prerequisites(),

        Commands::Auth { instance } => {
            println!("🔐 Testing authentication for Glean instance: {}", instance);

            // Check GLEAN_AUTH_TOKEN environment variable
            println!("\n🔍 Checking GLEAN_AUTH_TOKEN environment variable:");
            let found_token = match std::env::var("GLEAN_AUTH_TOKEN") {
                Ok(value) => {
                    let masked = if value.len() > 8 {
                        format!("{}...{}", &value[..4], &value[value.len() - 4..])
                    } else {
                        "***".to_string()
                    };
                    println!("  ✅ GLEAN_AUTH_TOKEN: {}", masked);
                    true
                }
                Err(_) => {
                    println!("  ❌ GLEAN_AUTH_TOKEN: not set");
                    false
                }
            };

            if !found_token {
                println!("\n💡 No authentication token found.");
                println!("   Set the Glean auth token:");
                println!("   export GLEAN_AUTH_TOKEN=your_token_here");
                println!("\n🔗 For mise users:");
                println!("   mise set GLEAN_AUTH_TOKEN=your_token_here");
                return Ok(());
            }

            println!("\n🚀 Running authentication test...");
            match run_validation(Some(&instance)) {
                Ok(result) => {
                    if result.success {
                        println!("\n✅ Authentication test successful!");
                    } else {
                        println!("\n❌ Authentication test failed!");
                        if let Some(error) = &result.error {
                            println!("Error: {}", error);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Failed to run authentication test: {}", e);
                    std::process::exit(1);
                }
            }

            Ok(())
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
            println!("  {} {}", status, tool);
        }
    }

    if let Some(error) = &result.error {
        println!("\n⚠️  Error Details: {}", error);
    }
}

fn check_prerequisites() -> Result<()> {
    println!("🔍 Checking system prerequisites...");

    // Check if npx is available
    match std::process::Command::new("npx").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("✅ npx available: {}", version.trim());
            } else {
                println!("❌ npx command failed");
                return Err(GleanMcpError::Config("npx not available".to_string()));
            }
        }
        Err(_) => {
            println!("❌ npx not found");
            println!("Please install Node.js and npm to use MCP Inspector");
            return Err(GleanMcpError::Config("npx not found".to_string()));
        }
    }

    // Check if MCP Inspector package is available
    println!("🔍 Checking MCP Inspector availability...");
    match std::process::Command::new("npx")
        .args(&["@modelcontextprotocol/inspector", "--help"])
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

    println!("\n🎯 Prerequisites check completed!");
    println!("Run 'glean-mcp-test inspect' to test MCP server connection");

    Ok(())
}
