# Glean MCP Testing Framework - Agent Guidelines

## Build/Lint/Test Commands

```bash
# Build and run
cargo build --release
cargo run -- inspect --instance scio-prod

# Linting and code quality
cargo fmt                    # Format code
cargo clippy                 # Run all lints (pedantic, nursery, perf, style, complexity)
cargo check                  # Check without building

# Testing
cargo test                   # Run all tests
cargo test -- --nocapture   # Run tests with output
cargo test test_name        # Run specific test
RUST_LOG=debug cargo test   # Run tests with debug logging
```

## Code Style Guidelines

### Error Handling
- Use custom `GleanMcpError` enum with descriptive error messages
- Always use `Result<T>` type alias for fallible operations
- Use `?` operator for error propagation with context
- Categorize errors: Inspector, Config, Auth, Host, Network, Validation, Process, Io, Json

### Async Runtime
- Use `smol` runtime instead of tokio for async operations
- Use `async_process::Command` for async process execution
- Use `smol::block_on()` for executing async code from sync contexts
- Prefer smol utilities over tokio equivalents

### Imports and Dependencies
- Follow standard Rust import conventions
- Use `clap` derive macros for CLI interfaces
- Use `serde` for serialization with derive features
- Use `anyhow` and `thiserror` for error handling

### Naming Conventions
- Structs: PascalCase (e.g., `GleanMCPInspector`, `InspectorResult`)
- Functions: snake_case (e.g., `validate_server_with_inspector`)
- Variables: snake_case (e.g., `server_url`, `tool_results`)
- Enums: PascalCase with descriptive variants

### Documentation
- Use `//!` for module-level documentation
- Use `///` for public function documentation
- Include examples for complex functions
- Document error conditions where relevant

### CLI Patterns
- Use clap derive macros with comprehensive help text
- Support both text and JSON output formats
- Use emoji indicators for text output: ✅ ❌ 🚀 📋 🔍 ⚠️
- Provide structured JSON output for programmatic use
- Use appropriate exit codes (0 for success, 1 for failure)

### Code Quality Standards
- Strict Clippy configuration (pedantic, nursery, performance, style, complexity)
- Cognitive complexity threshold: 30
- Type complexity threshold: 250
- Function arguments threshold: 8
- Lines per function threshold: 150

## Cursor Rules Integration

Follow all guidelines from `.cursor/rules/`:
- **async-runtime.mdc**: Use smol runtime patterns
- **cli-patterns.mdc**: Follow CLI UX patterns with structured output
- **error-handling.mdc**: Use custom error types and proper propagation
- **rust-coding-standards.mdc**: Follow strict linting and documentation standards
- **testing-validation.mdc**: Use structured validation results and testing patterns

## Development Workflow

1. Run `cargo clippy` before committing changes
2. Run `cargo fmt` to format code
3. Run `cargo test` to verify functionality
4. Use `cargo check` for quick validation during development
5. Follow async patterns using smol runtime
6. Use structured error handling with `GleanMcpError`