# AGENTS.md

## Project Overview

This is a Rust-based Model Context Protocol (MCP) server that provides tools, prompts, and resources for AI agents. The project uses the `rmcp` library to implement MCP server functionality with HTTP transport, supporting tools, prompts, and resource management for AI agent interactions.


## Build and Development Commands

### Standard Rust Commands
- Build: `cargo build`
- Build release: `cargo build --release`
- Check: `cargo check`
- Test: `cargo test`
- Run: `cargo run`
- Format: `cargo fmt`
- Clippy: `cargo clippy`

### Development Server
- Run server: `cargo run --release` (starts on 127.0.0.1:8010)
- Test MCP connection: Connect to `http://127.0.0.1:8010/mcp`

### Important: Do NOT run cargo run automatically
- **NEVER run `cargo run` or `cargo run --release` without explicit user request**
- Only run compilation commands: `cargo check`, `cargo build`, `cargo test`
- Server startup should only happen when user explicitly asks for it

### Testing Commands
- Run all tests: `cargo test`
- Run with output: `cargo test -- --nocapture`
- Test specific module: `cargo test counter`

## Code Quality Guidelines

### Critical Thinking Checklist (MANDATORY)
Before proposing any solution, complete this checklist:

1. **Question the problem itself**: Is the complexity actually necessary? Can we eliminate the need entirely?
2. **Find the simplest possible solution first**: What would require the least code changes to solve this?
3. **Challenge every assumption**: Why does the current code work this way? Is it still relevant?
4. **Look at actual usage**: How is this code really used? Can we optimize for the common cases?
5. **Design the API before implementation**: What would be the cleanest interface for this functionality?

### Problem-Solving Priority Order (MUST follow in sequence):
1. **ELIMINATE** - Can we remove the problem entirely by restructuring?
2. **SIMPLIFY** - Can we solve this trivially by changing the approach?
3. **REUSE** - Does existing code already solve this?
4. **Only then: CREATE** - Write new solution only if above options fail

### Complexity Warning Signs 🚨
If you find yourself doing ANY of these, STOP and reconsider:
- Using `Box<dyn Any>` or type erasure
- Creating new trait abstractions for a single use case
- Writing more than 20 lines to work around a problem
- Adding generic parameters that propagate through multiple layers
- Introducing runtime type checking or downcasting
- Making something generic "just in case"
- Creating abstractions on top of abstractions

### Code Style Requirements
- **Always strive to write idiomatic Rust code**
- Follow Rust naming conventions (snake_case for variables/functions, PascalCase for types)
- Use Rust's ownership system effectively (avoid unnecessary cloning)
- Prefer pattern matching over if-let chains
- Use iterator methods instead of manual loops when appropriate
- Leverage Rust's type system for safety (Option, Result, etc.)
- Write code that takes advantage of Rust's zero-cost abstractions
- Use appropriate error handling with Result types
- Follow Rust's module system and visibility rules
- Write code that is both safe and performant
- Verify information before presenting it
- Make changes file by file
- Never use apologies
- Avoid feedback about understanding
- Don't suggest whitespace changes
- Don't summarize changes made
- Don't invent changes other than what's explicitly requested
- Don't ask for confirmation of information already provided
- Preserve existing code and structures
- Provide all edits in a single chunk
- Don't ask to verify implementations that are visible in context
- Don't suggest updates when no modifications are needed
- Provide links to real files, not x.md
- Don't show current implementation unless specifically requested

### Code Formatting Requirements
- **Always run `just fmt` before committing code changes**
- **Run `just fmt` after making any code modifications**
- Ensure consistent code formatting across the entire codebase
- Format all Rust files before submitting pull requests
- Verify formatting is applied to all modified files


## Documentation Guidelines

### Language Requirements
- **All documentation must be in English only**
- This includes README.md files, code comments, API documentation, inline documentation, function descriptions, and variable naming explanations
- Maintain consistent English terminology throughout the codebase
- Use proper technical English terminology
- Keep documentation clear and concise
- Write all commit messages in English

### Comment Guidelines
- All comments must be in English
- **Comments should only be added where code is complex or non-obvious**
- Comments should only be written where code is difficult to understand or contains non-standard logic
- If code is obvious, comments only clutter it and provide no benefit
- The most valuable comments are those that answer "why" questions (why this solution was chosen, why this algorithm was used, why this code cannot be changed)
- Comments that simply repeat what is already visible in the code are unnecessary
- Keep comments short, laconic, and specific
- Avoid verbose or unnecessary comments
- Focus on explaining complex logic or non-obvious behavior
- Use clear, technical language
- Write brief, to-the-point explanations
- Avoid redundant comments that just repeat the code
- Prefer single-line comments for simple explanations
- Use multi-line comments only for complex algorithms or important context

### Code Examples Policy
- **Do not create standalone example files or directories**
- **Do not create an `examples/` directory**
- Avoid creating sample implementations that are not used in production
- **Always implement tests instead of examples** to demonstrate functionality
- Use test cases to show how components should be used
- Tests should serve as documentation for code usage
- Ensure tests cover all usage scenarios that would typically be shown in examples
- Document usage patterns directly in code comments or README files
- Use code snippets in documentation rather than separate example files
- Reference test cases in documentation when explaining usage

### Documentation Retrieval
- **Use Context7 MCP server whenever possible** for obtaining up-to-date documentation
- Context7 provides access to comprehensive library documentation and API references
- Prefer Context7 over static documentation when working with external libraries
- Use Context7 to get the latest version information and breaking changes
- Leverage Context7 for framework/library/crates-specific documentation
- Context7 helps ensure documentation accuracy and currency

## MCP Server Integration Guidelines

### Core Requirements
- **Always use the official rmcp library** from https://crates.io/crates/rmcp
- **Use tokio async runtime** for MCP server operations
- The library is designed to work with async/await patterns

### MCP Server Implementation
When implementing MCP servers:
- Follow the standard rmcp server patterns
- Use async/await code patterns throughout
- Implement ServerHandler trait for core functionality
- Use tool_router and prompt_router macros for routing

### Best Practices
- Use rmcp's native error handling mechanisms (McpError)
- Ensure all MCP interactions are thread-safe with Arc<Mutex<>>
- Use rmcp's built-in parameter validation
- Follow rmcp's recommended testing patterns
- Always specify and check version compatibility with ProtocolVersion

## Testing Instructions

### Test Commands
- Run all tests: `cargo test`
- Run with output: `cargo test -- --nocapture`
- Run specific test: `cargo test test_name`
- Run tests in module: `cargo test counter`

### Test Requirements
- All code changes must include corresponding tests
- Tests should demonstrate MCP functionality and serve as documentation
- Use integration tests for end-to-end MCP server functionality
- Use unit tests for component-level usage (tools, prompts, resources)
- Ensure tests pass before committing changes
- Test both tool and prompt router functionality

## Project Structure



### MCP Server Components
- **Tools**: get_resource_content, get_docs_list, get_agreements, get_project_overview, get_all_adr_documents
- **Resources**: Architecture diagrams (C4, ERD), ADR documents, OpenAPI specifications, API agreements
- **Transport**: HTTP server on 127.0.0.1:8010/mcp

## Deployment and Environment

### Target Platform
- Primary target: Native platform compilation
- Development: macOS/Linux/Windows
- No cross-compilation required for standard development

### Server Configuration
- Default bind address: `127.0.0.1:8010`
- MCP endpoint: `http://127.0.0.1:8010/mcp`
- Logging: Configured with tracing and env-filter
- Graceful shutdown: Ctrl+C signal handling

## Security Considerations

- Follow MCP security best practices
- Use proper error handling and validation with McpError
- Avoid exposing sensitive information in logs or documentation
- Validate all input parameters in tools and prompts
- Use proper authentication and authorization for production deployments
- Secure HTTP transport configuration for production use

## MCP-Specific Development Guidelines

### Tool Implementation
- Use `#[tool(description = "...")]` attribute for all tools
- Return `CallToolResult::success()` or `CallToolResult::failure()`
- Use `Parameters<T>` for type-safe parameter handling
- Implement proper error handling with `McpError`

### Prompt Implementation
- Use `#[prompt(name = "...")]` attribute for all prompts
- Return `GetPromptResult` with description and messages
- Use `PromptMessage` for structured prompt content
- Support both required and optional parameters

### Resource Management
- Implement `list_resources()` and `read_resource()` methods
- Use proper URI schemes (docs:// for documentation resources)
- Return `ResourceContents` for resource data
- Handle resource not found errors gracefully

### Server Configuration
- Set proper `ProtocolVersion` in server info
- Configure `ServerCapabilities` appropriately
- Provide clear `instructions` for AI agents
- Use `Implementation::from_build_env()` for version info

## Key Questions to Ask Before Implementation

- "What is the actual problem I'm trying to solve?"
- "What would the calling code look like?"
- "Can I solve this with existing standard library functions?"
- "Am I over-engineering this?"
- "Can I eliminate this complexity entirely?"
- "Is there existing code that already solves this?"
- "Does this tool/prompt provide clear value to AI agents?"
- "Are the parameters well-defined and type-safe?"

## Before Implementation Checklist

1. Write the calling code first (outside-in design)
2. Use the simplest types that could work
3. Prefer composition over complex inheritance/traits
4. Question every generic parameter and abstraction layer
5. Verify the solution follows the ELIMINATE → SIMPLIFY → REUSE → CREATE priority order
6. Ensure MCP tools have clear descriptions and type-safe parameters
7. Test both tool and prompt functionality with proper error handling
8. Verify server capabilities and protocol version compatibility