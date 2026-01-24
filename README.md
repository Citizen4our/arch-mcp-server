# Arch MCP Server

MCP server that provides code agents with architectural context - ERD diagrams, code policies, API documentation, service relationships, and best practices.

## Overview

This is a Model Context Protocol (MCP) server built with Rust that serves as an architectural context provider for AI coding agents. It offers tools and resources to help agents understand system architecture, relationships, and coding standards.

## Supported Document Types

The server supports scanning and providing access to various types of architectural and technical documentation:

### Architecture Diagrams
- **C4 Diagrams**: Context (C1), Container (C2), Component (C3), and Service (C4) diagrams
- **ERD Diagrams**: Entity Relationship Diagrams for database schemas
- **ADR Documents**: Architecture Decision Records
- **OpenAPI Specifications**: REST API documentation in YAML format

### Document Categories
- **Agreements**: API contracts, service agreements, and technical specifications
- **Architecture**: C4 diagrams, ERD diagrams, and ADR documents
- **OpenAPI**: REST API specifications and endpoint documentation
- **Backend**: PHP, Go, and other backend documentation
- **Frontend**: JavaScript, TypeScript, and frontend documentation

### Supported File Formats
- **Markdown**: `.md` files
- **MDX**: `.mdx` files (Markdown with JSX components)
- **Text**: `.txt` files
- **YAML**: `.yaml` files (OpenAPI specifications)

## Available MCP Tools

The server provides 5 powerful tools for architectural documentation analysis:

### 1. `get_resource_content`
**📄 Get Documentation Resource Content**
- **Purpose**: Retrieves content from specific documentation files using `docs://` paths
- **Parameters**: 
  - `path` (string): Resource path in format `docs://path/to/file`
- **Use Cases**: Reading specific architecture docs, API specs, guides, and technical documentation
- **Examples**: 
  - `docs://architecture/prj-1/c1.mdx` (C4 diagram)
  - `docs://openapi/mpa/activation/v2/public/get-customer-activation-info.yaml` (OpenAPI spec)

### 2. `get_docs_list`
**📋 Get Documentation List with Filters**
- **Purpose**: Lists documentation resources with advanced filtering and pagination
- **Parameters**:
  - `area` (optional): Filter by area (e.g., "architecture", "backend", "frontend", "openapi") - supports OR with `|` separator
  - `lang` (optional): Filter by language (e.g., "php", "go", "js", "ts") - supports OR with `|` separator
  - `category` (optional): Filter by category (e.g., "c1", "c2", "c3", "c4", "erd", "agreements", "openapi") - supports OR with `|` separator
  - `page` (optional): Page number for pagination (default: 1)
  - `limit` (optional): Items per page (default: 50, max: 200)
- **Use Cases**: Document discovery, architecture analysis, technical documentation research
- **Example Filters**:
  - `area=architecture&category=c4` - Find all C4 diagrams
  - `area=openapi&category=activation` - Find all OpenAPI specs for activation service
  - `area=backend&lang=php` - Find all PHP backend documentation
  - `category=agreements` - Find all agreement documents

### 3. `get_all_adr_documents`
**📋 Get All ADR Documents**
- **Purpose**: Retrieves all Architecture Decision Record (ADR) documents sorted by ADR number
- **Parameters**: None
- **Use Cases**: Discovering and analyzing architectural decisions across the project
- **Returns**: List of ADR documents with metadata including URI, description, and file paths

### 4. `get_project_overview`
**📊 Get Project Overview**
- **Purpose**: Provides comprehensive overview of a project with all document types, grouped by categories
- **Parameters**:
  - `project` (required): Project name
- **Use Cases**: Project analysis, documentation statistics, understanding project structure
- **Returns**: Structured JSON with project statistics and all ResourceInfo objects organized by type, area, and language
- **Features**:
  - Total document count and size
  - Documents grouped by type (C1, C2, C3, C4, ERD, ADR, agreements)
  - Documents grouped by area (architecture, backend, frontend)
  - Documents grouped by language (PHP, Go, JS, TS, etc.)
  - Complete list of all documents with full metadata

### 5. `get_agreements`
**📋 Get Agreements by Language**
- **Purpose**: Retrieves all agreement documents filtered by programming language
- **Parameters**:
  - `lang` (required): Programming language (e.g., "php", "go", "js", "ts", "py", "rust")
- **Use Cases**: Understanding API contracts, service agreements, and technical specifications for specific technology stack
- **Returns**: List of agreement documents with metadata for the specified language
- **Features**:
  - Filtered by programming language
  - API contracts and service agreements
  - Technical specifications
  - Complete metadata for each agreement

## Document Scanning

The server scans and indexes documents from a docs repository root provided via `--docs-root`, using an `arch-mcp.toml` mapping file.

```
<docs-root>/
├── arch-mcp.toml
└── ... (any layout; paths are configured inside arch-mcp.toml)
```

## Resource URI Patterns

Documents are accessible via structured URIs:

- **C4 Diagrams**: `docs://architecture/{project}/{diagram}.mdx`
- **ERD Diagrams**: `docs://architecture/erd/{project}/{diagram}.mdx`
- **ADR Documents**: `docs://architecture/{project}/adr/{adr-number}-{title}.mdx`
- **Agreements**: `docs://agreements/{area}/{lang}/{category}/{file}`

## Quick Start

### Prerequisites

- Rust 1.70+ (2024 edition)
- Cargo

For detailed installation instructions, see [install.md](install.md).

Quick install:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or follow the official guide: https://rust-lang.org/tools/install/

### Installation

1. Build the project:
```bash
cargo build --release
```

2. Run the server:
```bash
cargo run --release -- --docs-root ./example_docs/docs/content
```

The server will start on `127.0.0.1:8010` by default.

### Configuring Cursor to Use the MCP Server

**Important**: This MCP server uses HTTP transport and must be running as a separate process. The stdio transport is not supported because the server outputs logs to stdout, which interferes with the JSON protocol.

To use this MCP server with Cursor:

1. **Start the MCP Server** (must be running before configuring Cursor):

   **Option A: Using Cargo**:
   ```bash
   cargo run --release -- --docs-root ./example_docs/docs/content
   ```

   **Option B: Using Docker Compose**:
   ```bash
   docker compose up
   ```

   **Option C: Using built binary**:
   ```bash
   ./target/release/arch-mcp-server --docs-root ./example_docs/docs/content
   ```

2. **Locate your Cursor MCP configuration file**:
   - macOS/Linux: `~/.cursor/mcp.json`
   - Windows: `%APPDATA%\Cursor\mcp.json`
   - or by Cursor UI: Settings → Features → Model Context Protocol
  
3. **Add the Arch MCP Server configuration**:

   **For local development** (server running on localhost):
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

   **For remote server**:
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

4. **Verify Connection**:
   - The MCP server should appear in Cursor's available tools
   - You can test it by asking Cursor to use architectural context in your conversations

5. **Test the MCP Server**:
   To verify that the MCP server is working correctly, you can test it by entering this prompt in the Cursor chat:
   ```
   get overview for proj-a Project.
   Use arch-mcp server.
   ```
   
   If the MCP server is properly connected, you should see:
   - The arch-mcp tools being called in the chat interface
   - Tool execution results showing architectural documentation
   - Successful retrieval of project overview data

6. **Automatic MCP Integration (Optional)**:
   For automatic MCP tool usage without explicit "use arch-mcp" commands, you can add the following section to your `AGENTS.md` file:
   
   ```markdown
   ## MCP (Model Context Protocol)
   
   ### arch-mcp Integration
   Always use arch-mcp when you need to analyze project documentation, understand architectural decisions, or work with ADR (Architecture Decision Records). This tool provides comprehensive project analysis and documentation insights.
   
   **Key Features:**
   - **Project Overview**: Complete project statistics and documentation coverage
   - **ADR Analysis**: Architecture Decision Records for understanding design choices
   - **API Documentation**: Service agreements and API contracts
   - **C4 Diagrams**: Context, Container, Component, and Code level diagrams
   - **ERD Analysis**: Entity Relationship Diagrams for database design
   - **Technical Specifications**: Detailed specs by programming language and area
   - **Agreements**: Service contracts and technical agreements
   - **Architecture Documentation**: Comprehensive architectural documentation
   
   **When to Use ArchMCP:**
   - Understanding project architecture and design decisions
   - Analyzing existing documentation and specifications
   - Finding specific technical specifications and agreements
   - Reviewing architectural decisions and ADR history
   - Getting project statistics and documentation coverage
   - Working with database design and ERD diagrams
   - Understanding service agreements and API contracts
   - Analyzing C4 architecture diagrams
   ```
   
   This will enable the AI agent to automatically use arch-mcp tools when appropriate, without requiring explicit "use arch-mcp" commands in prompts.

**Note**: For remote server access, VPN connection is required.

## Usage

### Running the Server

```bash
cargo run --release -- --docs-root ./example_docs/docs/content
```

### MCP Inspector

To inspect and test the MCP server, use the official MCP inspector:

```bash
npx @modelcontextprotocol/inspector
```

This will open a web interface where you can:
- Connect to your MCP server
- Test available tools and resources
- Inspect server capabilities
- Debug MCP protocol interactions

### Example Usage

#### Get Project Overview
```json
{
  "tool": "get_project_overview",
  "parameters": {
    "project": "proj-a"
  }
}
```

**Response:**
```json
{
  "project": "proj-a",
  "total_documents": 25,
  "total_size": 1024000,
  "documents_by_type": {
    "c1": [ResourceInfo...],
    "c2": [ResourceInfo...],
    "c4": [ResourceInfo...],
    "erd": [ResourceInfo...],
    "ADR-001": [ResourceInfo...]
  },
  "documents_by_area": {
    "architecture": [ResourceInfo...],
    "backend": [ResourceInfo...]
  },
  "documents_by_language": {
    "php": [ResourceInfo...],
    "none": [ResourceInfo...]
  },
  "all_documents": [ResourceInfo...]
}
```

#### Get Agreements by Language
```json
{
  "tool": "get_agreements",
  "parameters": {
    "lang": "php"
  }
}
```

**Response:**
```json
{
  "lang": "php",
  "agreements": [
    {
      "uri": "docs://agreements/backend/php/api/user-service.md",
      "file_path": "content/docs/backend/php/api/user-service.md",
      "area": "backend",
      "lang": "php",
      "category": ["agreements", "api"],
      "project": "",
      "mime_type": "text/markdown",
      "size": 2048,
      "description": "Agreement document: agreements, api - backend (php)"
    }
  ],
  "total_agreements": 1
}
```

## Resources

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [rmcp Documentation](https://docs.rs/rmcp/)
- [MCP Inspector](https://www.npmjs.com/package/@modelcontextprotocol/inspector)

## TODO

### Planned Features

- **Stdio Transport**: Add stdio transport support for MCP protocol (currently only HTTP transport is supported)
- **Hot Reload**: Implement hot reload functionality to automatically rescan documents when files change without restarting the server
- **Extended Document Types**: Support more document types and file extensions (e.g., `.json`, `.xml`, `.csv`, `.rst`, `.asciidoc`)
- **Document Type Detection**: Automatic detection of document types based on content analysis
- **Caching**: Implement caching mechanism for frequently accessed documents to improve performance
- **Search Functionality**: Add full-text search capabilities across all documents
- **Configuration Validation**: Enhanced validation for `arch-mcp.toml` configuration file
- **Incremental Scanning**: Only rescan changed files instead of full directory scan
- **Document Metadata Extraction**: Extract and index metadata from documents (frontmatter, tags, etc.)

### Infrastructure & Build

- **CI/CD Pipeline**: Set up Continuous Integration pipeline for automated testing and building
- **Release Builds**: Automate creation of binary files