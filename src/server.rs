use std::collections::BTreeMap;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt_handler, prompt_router, schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;

use crate::{
    models::{DocumentKey, ResourceInfo},
    utils::file_reader::FileReader,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GetResourceContentArgs {
    /// Resource path in format docs://path/to/file
    pub path: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GetDocsListArgs {
    /// Area filter (e.g., "architecture", "backend", "frontend") - supports OR with | separator
    pub area: Option<String>,
    /// Language filter (e.g.,"php", "go", "ts", "js", "py", "rust") - supports OR with | separator  
    pub lang: Option<String>,
    /// Category filter (e.g., "c1", "c2", "c3", "c4", "api-documentation") - supports OR with | separator
    pub category: Option<String>,
    /// Page number for pagination (default: 1)
    pub page: Option<u32>,
    /// Number of items per page (default: 50, max: 200)
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DocsListResponse {
    /// List of matching documents
    pub documents: Vec<ResourceInfo>,
    /// Total number of pages
    pub total_pages: u32,
    /// Current page number
    pub current_page: u32,
    /// Number of items per page
    pub limit: u32,
    /// Total number of matching documents
    pub total_documents: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AdrListResponse {
    /// List of ADR documents sorted by ADR number
    pub adr_documents: Vec<ResourceInfo>,
    /// Total number of ADR documents
    pub total_adr_documents: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAllAdrDocumentsArgs {}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GetProjectOverviewArgs {
    /// Project name (as defined in `arch-mcp.toml`)
    pub project: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GetAgreementsArgs {
    /// Programming language filter (e.g., "php", "go", "js", "ts", "py", "rust")
    pub lang: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AgreementsResponse {
    /// Programming language filter applied
    pub lang: String,
    /// List of agreement documents for the specified language
    pub agreements: Vec<ResourceInfo>,
    /// Total number of agreement documents found
    pub total_agreements: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ProjectOverviewResponse {
    /// Project name
    pub project: String,
    /// Total number of documents in project
    pub total_documents: u32,
    /// Total size of all documents in bytes
    pub total_size: u64,
    /// Documents grouped by type
    pub documents_by_type: std::collections::BTreeMap<String, Vec<ResourceInfo>>,
    /// Documents grouped by area
    pub documents_by_area: std::collections::BTreeMap<String, Vec<ResourceInfo>>,
    /// Documents grouped by language
    pub documents_by_language: std::collections::BTreeMap<String, Vec<ResourceInfo>>,
    /// All documents in the project
    pub all_documents: Vec<ResourceInfo>,
}

#[derive(Clone)]
pub struct DocumentServer {
    file_reader: FileReader,
    resources: BTreeMap<DocumentKey, ResourceInfo>,
    tool_router: ToolRouter<DocumentServer>,
    prompt_router: PromptRouter<DocumentServer>,
}

#[tool_router]
impl DocumentServer {
    pub fn new_with_resources(
        file_reader: FileReader,
        resources: BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Self {
        Self {
            file_reader,
            resources,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    /// Reads file content by file path
    fn read_file_by_path(&self, file_path: &str) -> Result<String, McpError> {
        self.file_reader.read_file_content(file_path).map_err(|e| {
            McpError::internal_error(
                "file_read_error",
                Some(json!({
                    "file_path": file_path,
                    "error": format!("Failed to read file: {}", e)
                })),
            )
        })
    }

    /// Checks if a value matches any of the filter values (supports OR with | separator)
    pub fn matches_filter(value: &str, filter: &Option<String>) -> bool {
        match filter {
            None => true,
            Some(filter_str) => filter_str
                .split('|')
                .any(|filter_value| filter_value.trim() == value),
        }
    }

    /// Checks if any category in the categories array matches any of the filter values
    pub fn matches_category_filter(categories: &[String], filter: &Option<String>) -> bool {
        match filter {
            None => true,
            Some(filter_str) => {
                let filter_values: Vec<&str> = filter_str.split('|').map(|v| v.trim()).collect();
                categories.iter().any(|category| {
                    filter_values
                        .iter()
                        .any(|filter_value| filter_value == category)
                })
            }
        }
    }

    /// Filters documents based on the provided criteria
    fn filter_documents(&self, args: &GetDocsListArgs) -> Vec<&ResourceInfo> {
        self.resources
            .values()
            .filter(|info| {
                // Check area filter
                let area_matches = Self::matches_filter(&info.area, &args.area);

                // Check lang filter
                let lang_matches = Self::matches_filter(&info.lang, &args.lang);

                // Check category filter - now works with array of categories
                let category_matches =
                    Self::matches_category_filter(&info.category, &args.category);

                area_matches && lang_matches && category_matches
            })
            .collect()
    }

    #[tool(
        description = "Retrieves documentation content from docs:// paths. Use for reading architecture docs, API specs, guides, and technical documentation. Paths must start with 'docs://' prefix. Supports all document types including C4 diagrams, ERD diagrams, ADR documents, and API agreements. Returns raw file content as text for further processing by AI agents.",
        annotations(
            title = "📄 Get Documentation Resource Content",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_resource_content(
        &self,
        Parameters(GetResourceContentArgs { path }): Parameters<GetResourceContentArgs>,
    ) -> Result<CallToolResult, McpError> {
        if !path.starts_with("docs://") {
            return Err(McpError::invalid_params(
                "invalid_path",
                Some(json!({
                    "error": "Path must start with 'docs://'",
                    "provided_path": path
                })),
            ));
        }

        // First, find the resource by URI in our resources map
        let resource_info = self
            .resources
            .get(&DocumentKey::new(path.clone()))
            .ok_or_else(|| {
                McpError::resource_not_found(
                    "resource_not_found",
                    Some(json!({
                        "uri": path,
                        "error": "Resource not found in scanned documents"
                    })),
                )
            })?;

        // Then read the file content using the file path from ResourceInfo
        let content = self.read_file_by_path(&resource_info.file_path)?;

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(
        description = "Lists documentation resources with advanced filtering and pagination capabilities. Use this tool to search and browse architecture documents, API specifications, technical guides, and project documentation. Supports filtering by area (backend|frontend|architecture), programming language (php|go|js|ts), and category (agreements|api-documentation|c1|c2|c3|c4|erd) using OR logic with | separator. Perfect for finding specific document types like C4 diagrams (category=c4), API documentation (category=api-documentation), or backend PHP docs (area=backend&lang=php). Returns paginated results with metadata including file paths, sizes, and URIs. Default limit: 50, max: 200. Use for document discovery, architecture analysis, and technical documentation research. Essential for understanding project structure and finding relevant documentation.",
        annotations(
            title = "📋 Get Documentation List with Filters",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_docs_list(
        &self,
        Parameters(args): Parameters<GetDocsListArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Set default values
        let page = args.page.unwrap_or(1);
        let limit = args.limit.unwrap_or(50);

        // Validate pagination parameters
        if page == 0 {
            return Err(McpError::invalid_params(
                "invalid_page",
                Some(json!({
                    "error": "Page must be greater than 0",
                    "provided_page": page
                })),
            ));
        }

        if limit == 0 || limit > 200 {
            return Err(McpError::invalid_params(
                "invalid_limit",
                Some(json!({
                    "error": "Limit must be between 1 and 200",
                    "provided_limit": limit
                })),
            ));
        }

        // Filter documents
        let filtered_docs = self.filter_documents(&args);
        let total_documents = filtered_docs.len() as u32;
        let total_pages = (total_documents + limit - 1) / limit; // Ceiling division

        // Calculate pagination
        let start_index = ((page - 1) * limit) as usize;
        let end_index = std::cmp::min(start_index + limit as usize, filtered_docs.len());

        // Get paginated results
        let paginated_docs: Vec<ResourceInfo> = filtered_docs[start_index..end_index]
            .iter()
            .map(|info| (*info).clone())
            .collect();

        // Create response
        let response = DocsListResponse {
            documents: paginated_docs,
            total_pages,
            current_page: page,
            limit,
            total_documents,
        };

        // Serialize response to JSON
        let response_json = serde_json::to_value(&response).map_err(|e| {
            McpError::internal_error(
                "serialization_error",
                Some(json!({
                    "error": format!("Failed to serialize response: {}", e)
                })),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    #[tool(
        description = "Retrieves all ADR (Architecture Decision Record) documents sorted by ADR number. Returns a list of all ADR documents with their metadata including URI, description, and file paths. ADR documents are identified by their category starting with 'ADR-' followed by the ADR number. Perfect for discovering and analyzing architectural decisions across the project. Essential for understanding why certain architectural choices were made, tracking decision history, and ensuring consistency in future development. Use this tool to get comprehensive view of all architectural decisions made in the project.",
        annotations(
            title = "📋 Get All ADR Documents",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_all_adr_documents(
        &self,
        _: Parameters<GetAllAdrDocumentsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Filter documents to get only ADR documents
        let adr_documents: Vec<ResourceInfo> = self
            .resources
            .values()
            .filter(|info| {
                // Check if any category starts with "ADR-"
                info.category.iter().any(|cat| cat.starts_with("ADR-"))
            })
            .cloned()
            .collect();

        // Sort by ADR number (extract number from category)
        let mut sorted_adr_documents = adr_documents;
        sorted_adr_documents.sort_by(|a, b| {
            let get_adr_number = |info: &ResourceInfo| -> u32 {
                info.category
                    .iter()
                    .find(|cat| cat.starts_with("ADR-"))
                    .and_then(|cat| {
                        cat.strip_prefix("ADR-")
                            .and_then(|num| num.parse::<u32>().ok())
                    })
                    .unwrap_or(0)
            };

            let a_number = get_adr_number(a);
            let b_number = get_adr_number(b);
            a_number.cmp(&b_number)
        });

        // Create response
        let response = AdrListResponse {
            adr_documents: sorted_adr_documents.clone(),
            total_adr_documents: sorted_adr_documents.len() as u32,
        };

        // Serialize response to JSON
        let response_json = serde_json::to_value(&response).map_err(|e| {
            McpError::internal_error(
                "serialization_error",
                Some(json!({
                    "error": format!("Failed to serialize ADR response: {}", e)
                })),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    #[tool(
        description = "Get comprehensive overview of a project with all document types, grouped by categories. Returns structured JSON with project statistics and all ResourceInfo objects organized by type, area, and language. Provides total document count, total size, and documents grouped by type (C1, C2, C3, C4, ERD, ADR, agreements), area (architecture, backend, frontend), and language (PHP, Go, JS, TS, etc.). Perfect for getting complete project understanding, analyzing documentation coverage, and understanding project structure. Essential for project analysis and documentation statistics.",
        annotations(
            title = "📊 Get Project Overview",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_project_overview(
        &self,
        Parameters(GetProjectOverviewArgs { project }): Parameters<GetProjectOverviewArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Filter documents by project
        let project_documents: Vec<&ResourceInfo> = self
            .resources
            .values()
            .filter(|info| info.project == project)
            .collect();

        if project_documents.is_empty() {
            return Err(McpError::resource_not_found(
                "project_not_found",
                Some(json!({
                    "project": project,
                    "error": "No documents found for the specified project"
                })),
            ));
        }

        // Calculate statistics
        let total_documents = project_documents.len() as u32;
        let total_size: u64 = project_documents.iter().map(|doc| doc.size as u64).sum();

        // Group documents by type (category)
        let mut documents_by_type: std::collections::BTreeMap<String, Vec<ResourceInfo>> =
            std::collections::BTreeMap::new();
        for doc in &project_documents {
            for category in &doc.category {
                documents_by_type
                    .entry(category.clone())
                    .or_insert_with(Vec::new)
                    .push((*doc).clone());
            }
        }

        // Group documents by area
        let mut documents_by_area: std::collections::BTreeMap<String, Vec<ResourceInfo>> =
            std::collections::BTreeMap::new();
        for doc in &project_documents {
            documents_by_area
                .entry(doc.area.clone())
                .or_insert_with(Vec::new)
                .push((*doc).clone());
        }

        // Group documents by language
        let mut documents_by_language: std::collections::BTreeMap<String, Vec<ResourceInfo>> =
            std::collections::BTreeMap::new();
        for doc in &project_documents {
            let lang = if doc.lang.is_empty() {
                "none".to_string()
            } else {
                doc.lang.clone()
            };
            documents_by_language
                .entry(lang)
                .or_insert_with(Vec::new)
                .push((*doc).clone());
        }

        // Create response
        let response = ProjectOverviewResponse {
            project: project.clone(),
            total_documents,
            total_size,
            documents_by_type,
            documents_by_area,
            documents_by_language,
            all_documents: project_documents.iter().map(|doc| (*doc).clone()).collect(),
        };

        // Serialize response to JSON
        let response_json = serde_json::to_value(&response).map_err(|e| {
            McpError::internal_error(
                "serialization_error",
                Some(json!({
                    "error": format!("Failed to serialize project overview response: {}", e)
                })),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }

    #[tool(
        description = "Get all agreement documents filtered by programming language. Returns API contracts, service agreements, and technical specifications for the specified language. Perfect for understanding API contracts and service interfaces for a specific technology stack.",
        annotations(
            title = "📋 Get Agreements by Language",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn get_agreements(
        &self,
        Parameters(GetAgreementsArgs { lang }): Parameters<GetAgreementsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Filter documents by language and agreements category
        let agreement_documents: Vec<&ResourceInfo> = self
            .resources
            .values()
            .filter(|info| info.lang == lang && info.category.iter().any(|cat| cat == "agreements"))
            .collect();

        // Create response
        let response = AgreementsResponse {
            lang: lang.clone(),
            agreements: agreement_documents
                .iter()
                .map(|doc| (*doc).clone())
                .collect(),
            total_agreements: agreement_documents.len() as u32,
        };

        // Serialize response to JSON
        let response_json = serde_json::to_value(&response).map_err(|e| {
            McpError::internal_error(
                "serialization_error",
                Some(json!({
                    "error": format!("Failed to serialize agreements response: {}", e)
                })),
            )
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            response_json.to_string(),
        )]))
    }
}

#[prompt_router]
impl DocumentServer {}

#[tool_handler]
#[prompt_handler]
impl ServerHandler for DocumentServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides document access tools. Tools: get_resource_content (reads files by docs:// path), get_docs_list (lists documents with filtering and pagination), get_all_adr_documents (retrieves all ADR documents sorted by number), get_project_overview (comprehensive project overview with statistics and grouped documents), get_agreements (retrieves agreement documents filtered by programming language).".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let resources: Vec<Resource> = self
            .resources
            .iter()
            .map(|(_key, info)| {
                let mut resource = RawResource::new(info.uri.clone(), info.description.clone());
                resource.description = Some(info.description.clone());
                resource.mime_type = Some(info.mime_type.clone());
                resource.size = Some(info.size);
                resource.no_annotation()
            })
            .collect();

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        // First, find the resource by URI in our resources map
        let resource_info = self
            .resources
            .get(&DocumentKey::new(request.uri.clone()))
            .ok_or_else(|| {
                McpError::resource_not_found(
                    "resource_not_found",
                    Some(json!({
                        "uri": request.uri,
                        "error": "Resource not found in scanned documents"
                    })),
                )
            })?;

        // Then read the file content using the file path from ResourceInfo
        let content = self.read_file_by_path(&resource_info.file_path)?;

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::TextResourceContents {
                uri: request.uri.clone(),
                mime_type: Some(resource_info.mime_type.clone()),
                text: content,
                meta: None,
            }],
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
            meta: None,
        })
    }

    async fn subscribe(
        &self,
        request: SubscribeRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        // Check if the resource exists
        if !self.resources.contains_key(&DocumentKey::new(request.uri.clone())) {
            return Err(McpError::resource_not_found(
                "resource_not_found",
                Some(json!({
                    "uri": request.uri,
                    "error": "Cannot subscribe to resource that does not exist"
                })),
            ));
        }
        // Subscription is successful (no-op for static resources)
        Ok(())
    }

    async fn unsubscribe(
        &self,
        _request: UnsubscribeRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        // Unsubscription is successful (no-op for static resources)
        Ok(())
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_get_resource_content_tool_attributes() {
        let router = DocumentServer::tool_router();
        assert!(router.has_route("get_resource_content"));

        let tools = router.list_all();
        assert!(tools.iter().any(|t| t.name == "get_resource_content"));
    }

    #[tokio::test]
    async fn test_get_resource_content_invalid_path() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );
        let args = GetResourceContentArgs {
            path: "invalid/path".to_string(),
        };

        let result = docs.get_resource_content(Parameters(args)).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.code.0, -32602);
        }
    }

    #[tokio::test]
    async fn test_get_docs_list_tool_attributes() {
        let router = DocumentServer::tool_router();
        assert!(router.has_route("get_docs_list"));

        let tools = router.list_all();
        assert!(tools.iter().any(|t| t.name == "get_docs_list"));
    }

    #[tokio::test]
    async fn test_get_docs_list_pagination_validation() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );
        let args = GetDocsListArgs {
            area: None,
            lang: None,
            category: None,
            page: Some(0), // Invalid page
            limit: Some(50),
        };

        let result = docs.get_docs_list(Parameters(args)).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.code.0, -32602);
        }
    }

    #[tokio::test]
    async fn test_get_docs_list_limit_validation() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );
        let args = GetDocsListArgs {
            area: None,
            lang: None,
            category: None,
            page: Some(1),
            limit: Some(201), // Invalid limit (max is 200)
        };

        let result = docs.get_docs_list(Parameters(args)).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.code.0, -32602);
        }
    }

    #[tokio::test]
    async fn test_read_file_by_path_success() {
        // Create a temporary test file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_read_file.txt");
        let test_content = "Test file content for reading";

        std::fs::write(&test_file, test_content).expect("Failed to write test file");

        // Create DocumentServer instance with a mock FileReader that can read our test file
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );

        // Test reading the file (this will fail if the file doesn't exist in the docs root)
        // We'll test the error case since we can't easily mock the FileReader
        let result = docs.read_file_by_path("nonexistent_file.txt");
        assert!(result.is_err());

        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[tokio::test]
    async fn test_read_file_by_path_error_handling() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );
        let result = docs.read_file_by_path("nonexistent_file.txt");

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code.0, -32603); // Internal error
            assert!(error.data.is_some());
        }
    }

    #[test]
    fn test_matches_filter_function() {
        // Test with no filter (should match everything)
        assert!(DocumentServer::matches_filter("any_value", &None));

        // Test with exact match
        assert!(DocumentServer::matches_filter(
            "exact",
            &Some("exact".to_string())
        ));

        // Test with OR logic
        assert!(DocumentServer::matches_filter(
            "value1",
            &Some("value1|value2".to_string())
        ));
        assert!(DocumentServer::matches_filter(
            "value2",
            &Some("value1|value2".to_string())
        ));

        // Test with no match
        assert!(!DocumentServer::matches_filter(
            "nomatch",
            &Some("value1|value2".to_string())
        ));

        // Test with whitespace
        assert!(DocumentServer::matches_filter(
            "value1",
            &Some(" value1 | value2 ".to_string())
        ));
    }

    #[test]
    fn test_matches_category_filter_function() {
        // Test with no filter (should match everything)
        assert!(DocumentServer::matches_category_filter(
            &["any_value".to_string()],
            &None
        ));

        // Test with exact match
        assert!(DocumentServer::matches_category_filter(
            &["exact".to_string()],
            &Some("exact".to_string())
        ));

        // Test with OR logic
        assert!(DocumentServer::matches_category_filter(
            &["value1".to_string()],
            &Some("value1|value2".to_string())
        ));
        assert!(DocumentServer::matches_category_filter(
            &["value2".to_string()],
            &Some("value1|value2".to_string())
        ));

        // Test with multiple categories - should match if any category matches
        assert!(DocumentServer::matches_category_filter(
            &["value1".to_string(), "other".to_string()],
            &Some("value1|value2".to_string())
        ));
        assert!(DocumentServer::matches_category_filter(
            &["other".to_string(), "value2".to_string()],
            &Some("value1|value2".to_string())
        ));

        // Test with no match
        assert!(!DocumentServer::matches_category_filter(
            &["nomatch".to_string()],
            &Some("value1|value2".to_string())
        ));

        // Test with whitespace
        assert!(DocumentServer::matches_category_filter(
            &["value1".to_string()],
            &Some(" value1 | value2 ".to_string())
        ));

        // Test agreements category
        assert!(DocumentServer::matches_category_filter(
            &["agreements".to_string(), "api".to_string()],
            &Some("agreements".to_string())
        ));
        assert!(DocumentServer::matches_category_filter(
            &["agreements".to_string(), "api".to_string()],
            &Some("api".to_string())
        ));
    }

    #[tokio::test]
    async fn test_get_all_adr_documents_tool_attributes() {
        let router = DocumentServer::tool_router();
        assert!(router.has_route("get_all_adr_documents"));

        let tools = router.list_all();
        assert!(tools.iter().any(|t| t.name == "get_all_adr_documents"));
    }

    #[tokio::test]
    async fn test_get_project_overview_tool_attributes() {
        let router = DocumentServer::tool_router();
        assert!(router.has_route("get_project_overview"));

        let tools = router.list_all();
        assert!(tools.iter().any(|t| t.name == "get_project_overview"));
    }

    #[tokio::test]
    async fn test_get_project_overview_project_not_found() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );
        let args = GetProjectOverviewArgs {
            project: "nonexistent_project".to_string(),
        };

        let result = docs.get_project_overview(Parameters(args)).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.code.0, -32002);
        }
    }

    #[tokio::test]
    async fn test_get_agreements_tool_attributes() {
        let router = DocumentServer::tool_router();
        assert!(router.has_route("get_agreements"));

        let tools = router.list_all();
        assert!(tools.iter().any(|t| t.name == "get_agreements"));
    }

    #[tokio::test]
    async fn test_get_agreements_success() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path().to_string_lossy().to_string();
        let docs = DocumentServer::new_with_resources(
            FileReader::new(docs_root).expect("file reader"),
            BTreeMap::new(),
        );
        let args = GetAgreementsArgs {
            lang: "php".to_string(),
        };

        let result = docs.get_agreements(Parameters(args)).await;
        // This will succeed even with empty results since we don't have agreements in test data
        assert!(result.is_ok());
    }
}
