# Installation Guide for Mac

## Prerequisites

This guide assumes you're using macOS and have basic command-line knowledge.

## Step 1: Install Rust

Install Rust using the official installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts and select the default installation options. After installation, restart your terminal.

Verify the installation:

```bash
rustc --version
cargo --version
```

## Step 2: Setup Environment Variables

Copy the environment configuration file:

```bash
cp .env.example .env
```

Edit the `.env` file to configure your settings (optional, defaults will be used if not configured).

This project requires `--docs-root` at runtime. Use `.env` only for optional settings like `BIND_ADDRESS` and `RUST_LOG`.

## Step 3: Build the Project

Build the project in release mode:

```bash
cargo build --release
```

## Step 4: Run the Server

Start the MCP server:

```bash
cargo run --release -- --docs-root ./example_docs/docs/content
```

The server will start on `http://127.0.0.1:8010/mcp`

## Step 5: Test the Connection

You can test the MCP connection by connecting to `http://127.0.0.1:8010/mcp` using an MCP client.

## Step 6: Configure Cursor MCP

To use this MCP server with Cursor, you need to configure it in your Cursor MCP settings.

### Locate Cursor MCP Configuration

The Cursor MCP configuration file is located at:
- **macOS/Linux**: `~/.cursor/mcp.json`
- **Windows**: `%APPDATA%\Cursor\mcp.json`

You can also configure it through the Cursor UI: Settings → Features → Model Context Protocol.

### Add MCP Server Configuration

**Important**: This MCP server uses HTTP transport and must be running as a separate process. The stdio transport is not supported because the server outputs logs to stdout, which interferes with the JSON protocol.

Add the following configuration to your `mcp.json` file:

#### Using HTTP Server (Required)

**Important**: This MCP server uses HTTP transport and must be running as a separate process. The stdio transport will be added in future.

**1. Start the server manually:**

```bash
# Using cargo
cargo run --release -- --docs-root /path/to/your/docs/content

# Or using the built binary
./target/release/arch-mcp-server --docs-root /path/to/your/docs/content
```

**2. Configure Cursor to connect via HTTP:**

```json
{
  "mcpServers": {
    "arch-mcp": {
      "url": "http://127.0.0.1:8010/mcp",
      "headers": {}
    }
  }
}
```

**For remote server:**
```json
{
  "mcpServers": {
    "arch-mcp": {
      "url": "https://your-server.com/mcp",
      "headers": {}
    }
  }
}
```

### CLI Arguments Reference

The MCP server supports the following CLI arguments:

- **`--docs-root <path>`** (required): Path to the docs repository root (the directory that contains `arch-mcp.toml`)
  - Example: `--docs-root ./example_docs/docs/content`
  - Example: `--docs-root /Users/username/projects/docs/content`

- **`--config <path>`** (optional): Explicit config file path
  - Default: `<docs-root>/arch-mcp.toml`
  - Example: `--config /custom/path/arch-mcp.toml`

- **`--bind-address <addr>`** (optional): Server bind address and port
  - Default: `127.0.0.1:8010`
  - Example: `--bind-address 0.0.0.0:8080`
  - Example: `--bind-address 127.0.0.1:9000`

- **`--rust-log <level>`** (optional): Logging level
  - Default: `info`
  - Options: `error`, `warn`, `info`, `debug`, `trace`
  - Example: `--rust-log debug`

### Complete Configuration Examples

**Example 1: Local development (default port):**
```json
{
  "mcpServers": {
    "arch-mcp": {
      "url": "http://127.0.0.1:8010/mcp",
      "headers": {}
    }
  }
}
```

**Example 2: Custom port:**
```json
{
  "mcpServers": {
    "arch-mcp": {
      "url": "http://127.0.0.1:9000/mcp",
      "headers": {}
    }
  }
}
```

**Example 3: Remote server:**
```json
{
  "mcpServers": {
    "arch-mcp": {
      "url": "https://arch-mcp.example.com/mcp",
      "headers": {}
    }
  }
}
```

### Verify Configuration

After adding the configuration:

1. **Restart Cursor** to load the new MCP server configuration
2. **Check MCP status**: The MCP server should appear in Cursor's available tools
3. **Test the connection**: Ask Cursor to use the arch-mcp server:
   ```
   Get overview for proj-a Project. Use arch-mcp server.
   ```

If configured correctly, you should see:
- The arch-mcp tools being called in the chat interface
- Tool execution results showing architectural documentation
- Successful retrieval of project overview data

## Step 7: Install MCP Inspector (Optional)

For debugging and testing your MCP server, you can use the MCP Inspector tool:

### Prerequisites
- Node.js: ^22.7.5

### Install MCP Inspector

```bash
npx @modelcontextprotocol/inspector
```

### Using MCP Inspector

1. **Start your MCP server** (from Step 4):
   ```bash
   cargo run --release -- --docs-root ./example_docs/docs/content
   ```

2. **In another terminal, start the inspector**:
   ```bash
   npx @modelcontextprotocol/inspector
   ```

3. **Configure the inspector** to connect to your server:
   - Transport: `streamable-http`
   - Server URL: `http://127.0.0.1:8010/mcp`

4. **Test your MCP server** using the visual interface to:
   - List available tools
   - Test tool calls
   - Browse resources
   - Test prompts

For more information about MCP Inspector, visit: [https://github.com/modelcontextprotocol/inspector](https://github.com/modelcontextprotocol/inspector)

## Step 8: Install Just (Optional)

For convenient project management and running common commands, you can install Just:

### Install Just

```bash
# Using cargo (recommended)
cargo install just

# Or using homebrew on macOS
brew install just
```

### Using Just

Once installed, you can use Just to run project commands:

```bash
# List all available commands
just --list

# Run specific commands
just check    # Check code without building
just fmt      # Format code
just test     # Run tests
just cov      # Run coverage tests
just fix      # Fix code issues
```

For more information about Just, visit: [https://github.com/casey/just](https://github.com/casey/just)

## Troubleshooting

### Environment Variables

The project uses `.env` files for configuration. Make sure you have copied `.env.example` to `.env` as described in Step 2.

## Development Commands

- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Run tests**: `cargo test`
- **Run linter**: `cargo clippy`
- **Build debug**: `cargo build`
- **Build release**: `cargo build --release`

## Next Steps

Once the server is running, you can:

1. Connect to it using an MCP client
2. Use the available tools and prompts
3. Access the documentation resources
4. Develop additional MCP functionality

For more information about the project, see the main README.md file.
