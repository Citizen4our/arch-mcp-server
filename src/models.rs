use std::{collections::BTreeMap, path::Path};

use crate::utils::file_reader::FileReader;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentKey(String); // resource URI

impl DocumentKey {
    pub fn new(uri: String) -> Self {
        Self(uri)
    }
}

/// Document resource metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ResourceInfo {
    pub uri: String,
    pub file_path: String,
    pub area: String,
    pub lang: String,
    pub category: Vec<String>,
    pub project: String,
    pub mime_type: String,
    pub size: u32,
    pub description: String,
}

/// Document types with extensibility
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DocumentType {
    Agreements,
    C1Diagram(String),
    C2Diagram(String),
    C3Diagram(String),
    C4Diagram(String),
    ErdDiagram(String),
    AdrDocument(String),
    OpenApiSpec(String),
    GuideDoc(String),
}

impl DocumentType {
    /// Gets URI prefix for document type
    pub fn get_uri_prefix(&self) -> String {
        match self {
            DocumentType::Agreements => "docs://agreements/".to_string(),
            DocumentType::ErdDiagram(project) => format!("docs://architecture/erd/{}/", project),
            DocumentType::C1Diagram(project)
            | DocumentType::C2Diagram(project)
            | DocumentType::C3Diagram(project) => format!("docs://architecture/{}/", project),
            DocumentType::C4Diagram(project) => format!("docs://architecture/{}/c4/", project),
            DocumentType::AdrDocument(project) => format!("docs://architecture/{}/adr/", project),
            DocumentType::OpenApiSpec(project) => format!("docs://openapi/{}/", project),
            DocumentType::GuideDoc(product) => format!("docs://guides/{}/", product),
        }
    }

    /// Generates resource description based on metadata
    pub fn generate_description(
        &self,
        area: &str,
        lang: &str,
        categories: &[String],
        _filename: &str,
    ) -> String {
        match self {
            DocumentType::Agreements => {
                let category_str = categories.join(", ");
                format!("Agreement document: {} - {} ({})", category_str, area, lang)
            }
            DocumentType::ErdDiagram(project) => {
                // Use filename as the diagram name, remove .mdx extension
                let diagram_name = _filename.trim_end_matches(".mdx");
                format!("ERD diagram for {} project: {}", project, diagram_name)
            }
            DocumentType::C1Diagram(project) => {
                format!("C1 diagram for {} project", project)
            }
            DocumentType::C2Diagram(project) => {
                format!("C2 diagram for {} project", project)
            }
            DocumentType::C3Diagram(project) => {
                format!("C3 diagram for {} project", project)
            }
            DocumentType::C4Diagram(project) => {
                // Use filename as service name
                let service_name = _filename.trim_end_matches(".mdx");
                format!(
                    "C4 diagram for {} service in {} project",
                    service_name, project
                )
            }
            DocumentType::AdrDocument(project) => {
                // Extract ADR number from filename (e.g., "001-temporal-transactionality.mdx" -> "001")
                let adr_number = _filename
                    .split('-')
                    .next()
                    .unwrap_or("unknown")
                    .trim_end_matches(".mdx");
                format!("ADR-{} for {} project", adr_number, project)
            }
            DocumentType::OpenApiSpec(project) => {
                let endpoint_name = _filename.trim_end_matches(".yaml");
                format!(
                    "OpenAPI specification for {} endpoint in {} project",
                    endpoint_name, project
                )
            }
            DocumentType::GuideDoc(product) => {
                let stem = _filename
                    .rsplit('.')
                    .nth(1)
                    .unwrap_or(_filename)
                    .replace('_', " ");
                format!("Guide: {} - {}", product, stem)
            }
        }
    }
}

/// Document scanner for populating BTreeMap
pub struct DocumentScanner;

impl DocumentScanner {
    /// Scans documents and populates BTreeMap
    pub fn scan_documents(
        document_type: DocumentType,
        area_paths: Vec<String>,
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) {
        if matches!(document_type, DocumentType::Agreements) {
            for target in area_paths {
                if let Err(e) = Self::scan_target_with_extensions(
                    &document_type,
                    &target,
                    &[],
                    file_reader,
                    resources,
                ) {
                    tracing::warn!("Failed to scan target '{}': {}", target, e);
                }
            }
            return;
        }

        for area_path in area_paths {
            if let Err(e) = Self::scan_area(&document_type, &area_path, file_reader, resources) {
                tracing::warn!("Failed to scan area '{}': {}", area_path, e);
            }
        }
    }

    pub fn scan_documents_with_extensions(
        document_type: DocumentType,
        scan_targets: Vec<String>,
        allowed_extensions: &[String],
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) {
        for target in scan_targets {
            if let Err(e) = Self::scan_target_with_extensions(
                &document_type,
                &target,
                allowed_extensions,
                file_reader,
                resources,
            ) {
                tracing::warn!("Failed to scan target '{}': {}", target, e);
            }
        }
    }

    /// Scans one area folder recursively
    fn scan_area(
        document_type: &DocumentType,
        area_path: &str,
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = Path::new(file_reader.docs_root()).join(area_path);

        if !full_path.exists() {
            return Err(format!("Area path does not exist: {}", area_path).into());
        }

        if !full_path.is_dir() {
            return Err(format!("Area path is not a directory: {}", area_path).into());
        }

        Self::scan_directory_recursive(
            document_type,
            &full_path,
            area_path,
            file_reader,
            resources,
        )?;

        Ok(())
    }

    fn scan_target_with_extensions(
        document_type: &DocumentType,
        target: &str,
        allowed_extensions: &[String],
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = Path::new(file_reader.docs_root()).join(target);

        if !full_path.exists() {
            tracing::warn!("Scan target does not exist: {}", target);
            return Ok(());
        }

        if full_path.is_file() {
            Self::process_file_universal(
                document_type,
                &full_path,
                target,
                allowed_extensions,
                file_reader,
                resources,
            )?;
            return Ok(());
        }

        if !full_path.is_dir() {
            tracing::warn!("Scan target is not a directory: {}", target);
            return Ok(());
        }

        Self::scan_directory_recursive_universal(
            document_type,
            &full_path,
            target,
            allowed_extensions,
            file_reader,
            resources,
        )?;

        Ok(())
    }

    /// Recursive directory scanning
    fn scan_directory_recursive(
        document_type: &DocumentType,
        dir_path: &Path,
        area_path: &str,
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entries = std::fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // For C4 diagrams, don't scan subdirectories - only scan the top-level c4/ directory
                // For C4 service diagrams, scan the services/ subdirectory
                let is_c4_diagram = matches!(
                    document_type,
                    DocumentType::C1Diagram(_)
                        | DocumentType::C2Diagram(_)
                        | DocumentType::C3Diagram(_)
                );
                let is_c4_service_diagram = matches!(document_type, DocumentType::C4Diagram(_));
                let is_erd_diagram = matches!(document_type, DocumentType::ErdDiagram(_));
                let is_adr_document = matches!(document_type, DocumentType::AdrDocument(_));

                if !is_c4_diagram {
                    // For C4 service diagrams, only scan services/ subdirectory
                    if is_c4_service_diagram {
                        if path.file_name().and_then(|n| n.to_str()) == Some("services") {
                            Self::scan_directory_recursive(
                                document_type,
                                &path,
                                area_path,
                                file_reader,
                                resources,
                            )?;
                        }
                    } else if is_erd_diagram {
                        // For ERD diagrams, scan recursively
                        Self::scan_directory_recursive(
                            document_type,
                            &path,
                            area_path,
                            file_reader,
                            resources,
                        )?;
                    } else if is_adr_document {
                        // For ADR documents, scan recursively
                        Self::scan_directory_recursive(
                            document_type,
                            &path,
                            area_path,
                            file_reader,
                            resources,
                        )?;
                    } else {
                        Self::scan_directory_recursive(
                            document_type,
                            &path,
                            area_path,
                            file_reader,
                            resources,
                        )?;
                    }
                }
            } else if path.is_file() {
                Self::process_file(document_type, &path, area_path, file_reader, resources)?;
            }
        }

        Ok(())
    }

    fn scan_directory_recursive_universal(
        document_type: &DocumentType,
        dir_path: &Path,
        scan_root: &str,
        allowed_extensions: &[String],
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entries = std::fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::scan_directory_recursive_universal(
                    document_type,
                    &path,
                    scan_root,
                    allowed_extensions,
                    file_reader,
                    resources,
                )?;
            } else if path.is_file() {
                Self::process_file_universal(
                    document_type,
                    &path,
                    scan_root,
                    allowed_extensions,
                    file_reader,
                    resources,
                )?;
            }
        }

        Ok(())
    }

    /// Processes a single file and adds to resources
    #[allow(clippy::too_many_lines)]
    fn process_file(
        document_type: &DocumentType,
        file_path: &Path,
        _area_path: &str,
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filename = file_path
            .file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy()
            .to_string();

        // Check if file should be processed based on document type
        if !Self::should_process_file(document_type, &filename) {
            return Ok(());
        }

        // Get relative path from docs_root to understand the structure
        let relative_path = file_path
            .strip_prefix(file_reader.docs_root())
            .map_err(|e| format!("Failed to get relative path: {}", e))?
            .to_string_lossy()
            .to_string();

        // Parse the relative path to extract area, lang, category, and project
        // Expected structure: content/docs/architecture/project/c4/filename.mdx
        // or: content/docs/architecture/project/c4/services/service.mdx
        // or: content/docs/backend/lang/category/filename.mdx
        let path_parts: Vec<&str> = relative_path.split('/').collect();
        let (uri, area, lang, categories, project) = match path_parts.as_slice() {
            ["content", "docs", "architecture", project, "c4", filename] => {
                // C4 diagram structure: content/docs/architecture/project/c4/filename.mdx
                let uri = format!("docs://architecture/{}/{}", project, filename);
                let category = match document_type {
                    DocumentType::C1Diagram(_) => "c1".to_string(),
                    DocumentType::C2Diagram(_) => "c2".to_string(),
                    DocumentType::C3Diagram(_) => "c3".to_string(),
                    _ => "c4".to_string(),
                };
                (
                    uri,
                    "architecture".to_string(),
                    String::new(),
                    vec![category],
                    (*project).to_string(),
                )
            }
            [
                "content",
                "docs",
                "architecture",
                project,
                "c4",
                "services",
                filename,
            ] => {
                // C4 service diagram structure: content/docs/architecture/project/c4/services/service.mdx
                let uri = format!("docs://architecture/{}/c4/{}", project, filename);
                (
                    uri,
                    "architecture".to_string(),
                    String::new(),
                    vec!["c4".to_string()],
                    (*project).to_string(),
                )
            }
            [
                "content",
                "docs",
                "architecture",
                project,
                "erd",
                "services",
                filename,
            ] => {
                // ERD diagram structure: content/docs/architecture/project/erd/services/filename.mdx
                let uri = format!("docs://architecture/erd/{}/{}", project, filename);
                (
                    uri,
                    "architecture".to_string(),
                    String::new(),
                    vec!["erd".to_string()],
                    (*project).to_string(),
                )
            }
            ["content", "docs", "architecture", project, "erd", filename] => {
                // ERD diagram structure: content/docs/architecture/project/erd/filename.mdx
                let uri = format!("docs://architecture/erd/{}/{}", project, filename);
                (
                    uri,
                    "architecture".to_string(),
                    String::new(),
                    vec!["erd".to_string()],
                    (*project).to_string(),
                )
            }
            ["content", "docs", "architecture", project, "adr", filename] => {
                // ADR document structure: content/docs/architecture/project/adr/filename.mdx
                let uri = format!("docs://architecture/{}/adr/{}", project, filename);
                // Extract ADR number from filename for category
                let adr_number = filename
                    .split('-')
                    .next()
                    .unwrap_or("unknown")
                    .trim_end_matches(".mdx");
                let category = format!("ADR-{}", adr_number);
                (
                    uri,
                    "architecture".to_string(),
                    String::new(),
                    vec!["adr".to_string(), category],
                    (*project).to_string(),
                )
            }
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                "endpoints",
                filename,
            ] => {
                // OpenAPI spec structure: content/docs/openapi-spec/project/service/version/access_level/endpoints/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}",
                    project, service, version, access_level, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                filename,
            ] => {
                // OpenAPI spec structure without endpoints subfolder: content/docs/openapi-spec/project/service/version/access_level/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}",
                    project, service, version, access_level, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                sub_category,
                "endpoints",
                filename,
            ] => {
                // OpenAPI spec structure with subcategory: content/docs/openapi-spec/project/service/version/access_level/sub_category/endpoints/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}/{}",
                    project, service, version, access_level, sub_category, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                        (*sub_category).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                sub_category,
                filename,
            ] => {
                // OpenAPI spec structure with subcategory without endpoints: content/docs/openapi-spec/project/service/version/access_level/sub_category/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}/{}",
                    project, service, version, access_level, sub_category, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                        (*sub_category).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                "endpoints",
                filename,
            ] => {
                // OpenAPI spec structure: openapi-spec/project/service/version/access_level/endpoints/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}",
                    project, service, version, access_level, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                filename,
            ] => {
                // OpenAPI spec structure without endpoints subfolder: openapi-spec/project/service/version/access_level/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}",
                    project, service, version, access_level, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                sub_category,
                "endpoints",
                filename,
            ] => {
                // OpenAPI spec structure with subcategory: openapi-spec/project/service/version/access_level/sub_category/endpoints/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}/{}",
                    project, service, version, access_level, sub_category, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                        (*sub_category).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            [
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                sub_category,
                filename,
            ] => {
                // OpenAPI spec structure with subcategory without endpoints: openapi-spec/project/service/version/access_level/sub_category/filename.yaml
                let uri = format!(
                    "docs://openapi/{}/{}/{}/{}/{}/{}",
                    project, service, version, access_level, sub_category, filename
                );
                (
                    uri,
                    "openapi".to_string(),
                    String::new(),
                    vec![
                        "openapi".to_string(),
                        (*service).to_string(),
                        (*version).to_string(),
                        (*access_level).to_string(),
                        (*sub_category).to_string(),
                    ],
                    (*project).to_string(),
                )
            }
            ["content", "docs", area, lang, category, ..] => {
                // Standard structure: content/docs/backend/lang/category/filename.mdx
                let uri = format!(
                    "{}{}/{}/{}/{}",
                    document_type.get_uri_prefix(),
                    area,
                    lang,
                    category,
                    filename
                );
                let categories = if matches!(document_type, DocumentType::Agreements) {
                    vec!["agreements".to_string(), (*category).to_string()]
                } else {
                    vec![(*category).to_string()]
                };
                (
                    uri,
                    (*area).to_string(),
                    (*lang).to_string(),
                    categories,
                    String::new(),
                )
            }
            ["content", "docs", _area, ..] => {
                // Skip files directly in area - they should not be processed
                return Ok(());
            }
            _ => return Err(format!("Invalid path structure: {}", relative_path).into()),
        };

        let mime_type = Self::get_mime_type(&filename);

        let metadata = std::fs::metadata(file_path)?;
        let size = metadata.len().try_into().unwrap_or(u32::MAX);

        let key = DocumentKey::new(uri.clone());
        let description = document_type.generate_description(&area, &lang, &categories, &filename);

        let resource_info = ResourceInfo {
            uri,
            file_path: relative_path.clone(),
            area,
            lang,
            category: categories,
            project,
            mime_type,
            size,
            description,
        };

        resources.insert(key, resource_info);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn process_file_universal(
        document_type: &DocumentType,
        file_path: &Path,
        scan_root: &str,
        allowed_extensions: &[String],
        file_reader: &FileReader,
        resources: &mut BTreeMap<DocumentKey, ResourceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filename = file_path
            .file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy()
            .to_string();

        if !Self::should_process_file_with_extensions(document_type, &filename, allowed_extensions)
        {
            return Ok(());
        }

        let relative_path = file_path
            .strip_prefix(file_reader.docs_root())
            .map_err(|e| format!("Failed to get relative path: {}", e))?
            .to_string_lossy()
            .to_string();

        let subpath = relative_under_target(&relative_path, scan_root);
        let uri = match document_type {
            DocumentType::Agreements => {
                let area = guess_agreements_area(scan_root);
                let uri_subpath = if area.is_empty() {
                    subpath.clone()
                } else {
                    format!("{}/{}", area.trim_end_matches('/'), subpath)
                };
                format!("{}{}", document_type.get_uri_prefix(), uri_subpath)
            }
            _ => format!("{}{}", document_type.get_uri_prefix(), subpath),
        };

        let (area, lang, categories, project) = match document_type {
            DocumentType::C1Diagram(project) => (
                "architecture".to_string(),
                String::new(),
                vec!["c1".to_string()],
                project.clone(),
            ),
            DocumentType::C2Diagram(project) => (
                "architecture".to_string(),
                String::new(),
                vec!["c2".to_string()],
                project.clone(),
            ),
            DocumentType::C3Diagram(project) => (
                "architecture".to_string(),
                String::new(),
                vec!["c3".to_string()],
                project.clone(),
            ),
            DocumentType::C4Diagram(project) => (
                "architecture".to_string(),
                String::new(),
                vec!["c4".to_string()],
                project.clone(),
            ),
            DocumentType::ErdDiagram(project) => (
                "architecture".to_string(),
                String::new(),
                vec!["erd".to_string()],
                project.clone(),
            ),
            DocumentType::AdrDocument(project) => {
                let adr_number = filename
                    .split('-')
                    .next()
                    .unwrap_or("unknown")
                    .trim_end_matches(".mdx");
                (
                    "architecture".to_string(),
                    String::new(),
                    vec!["adr".to_string(), format!("ADR-{}", adr_number)],
                    project.clone(),
                )
            }
            DocumentType::OpenApiSpec(project) => (
                "openapi".to_string(),
                String::new(),
                vec!["openapi".to_string()],
                project.clone(),
            ),
            DocumentType::GuideDoc(product) => (
                "guides".to_string(),
                String::new(),
                vec!["guides".to_string()],
                product.clone(),
            ),
            DocumentType::Agreements => {
                let area = guess_agreements_area(scan_root);
                let mut categories: Vec<String> = vec!["agreements".to_string()];

                let (lang, extra_categories) = parse_agreements_subpath(&subpath, &area);
                categories.extend(extra_categories);

                (
                    if area.is_empty() {
                        "agreements".to_string()
                    } else {
                        area
                    },
                    lang,
                    categories,
                    String::new(),
                )
            }
        };

        let mime_type = Self::get_mime_type(&filename);
        let metadata = std::fs::metadata(file_path)?;
        let size = metadata.len().try_into().unwrap_or(u32::MAX);
        let key = DocumentKey::new(uri.clone());
        let description = document_type.generate_description(&area, &lang, &categories, &filename);

        let resource_info = ResourceInfo {
            uri,
            file_path: relative_path,
            area,
            lang,
            category: categories,
            project,
            mime_type,
            size,
            description,
        };

        resources.insert(key, resource_info);
        Ok(())
    }

    fn should_process_file_with_extensions(
        document_type: &DocumentType,
        filename: &str,
        allowed_extensions: &[String],
    ) -> bool {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let is_allowed_ext = if allowed_extensions.is_empty() {
            true
        } else {
            allowed_extensions.iter().any(|e| e == &extension)
        };

        let file_stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match document_type {
            DocumentType::C1Diagram(_) => file_stem == "c1" && is_allowed_ext,
            DocumentType::C2Diagram(_) => file_stem == "c2" && is_allowed_ext,
            DocumentType::C3Diagram(_) => file_stem == "c3" && is_allowed_ext,
            DocumentType::C4Diagram(_)
            | DocumentType::ErdDiagram(_)
            | DocumentType::AdrDocument(_)
            | DocumentType::OpenApiSpec(_)
            | DocumentType::GuideDoc(_) => is_allowed_ext,
            DocumentType::Agreements => Self::should_process_file(document_type, filename),
        }
    }

    /// Determines if a file should be processed based on document type and filename
    fn should_process_file(document_type: &DocumentType, filename: &str) -> bool {
        match document_type {
            // C4 diagrams: only process specific files (c1.mdx, c2.mdx, c3.mdx)
            DocumentType::C1Diagram(_) => filename == "c1.mdx",
            DocumentType::C2Diagram(_) => filename == "c2.mdx",
            DocumentType::C3Diagram(_) => filename == "c3.mdx",

            // C4 service diagrams, ERD diagrams, ADR documents: process all .mdx files
            DocumentType::C4Diagram(_)
            | DocumentType::ErdDiagram(_)
            | DocumentType::AdrDocument(_) => Path::new(filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("mdx")),

            // OpenAPI specs: process all .yaml files
            DocumentType::OpenApiSpec(_) => Path::new(filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml")),

            // Agreements: process all supported files
            DocumentType::Agreements => Path::new(filename).extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("md")
                    || ext.eq_ignore_ascii_case("mdx")
                    || ext.eq_ignore_ascii_case("txt")
            }),
            // GuideDoc uses extension-based scanning only (process_file_universal)
            DocumentType::GuideDoc(_) => false,
        }
    }

    /// Determines MIME type by file extension
    pub fn get_mime_type(filename: &str) -> String {
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "md" | "mdx" => "text/markdown".to_string(),
            "yaml" | "yml" => "application/x-yaml".to_string(),
            "rst" => "text/x-rst".to_string(),
            _ => "text/plain".to_string(),
        }
    }
}

fn relative_under_target(relative_path_from_docs_root: &str, scan_root: &str) -> String {
    let scan_root = scan_root.trim_end_matches('/');
    let prefix = format!("{}/", scan_root);
    let rest = relative_path_from_docs_root
        .strip_prefix(&prefix)
        .unwrap_or(relative_path_from_docs_root);
    rest.trim_start_matches('/').to_string()
}

fn guess_agreements_area(scan_root: &str) -> String {
    let normalized = scan_root.trim_matches('/').to_ascii_lowercase();
    if normalized == "backend"
        || normalized.ends_with("/backend")
        || normalized.contains("/backend/")
    {
        return "backend".to_string();
    }
    if normalized == "frontend"
        || normalized.ends_with("/frontend")
        || normalized.contains("/frontend/")
    {
        return "frontend".to_string();
    }
    if normalized == "quality-assurance"
        || normalized.ends_with("/quality-assurance")
        || normalized.contains("/quality-assurance/")
    {
        return "quality-assurance".to_string();
    }

    // Best-effort fallback: use the last path component, unless it is a generic container like "docs".
    let last = normalized.rsplit('/').next().unwrap_or("").trim();
    if last == "docs" || last.is_empty() {
        String::new()
    } else {
        last.to_string()
    }
}

fn parse_agreements_subpath(subpath: &str, area: &str) -> (String, Vec<String>) {
    let parts: Vec<&str> = subpath.split('/').filter(|p| !p.is_empty()).collect();
    if (area == "backend" || area == "frontend") && parts.len() >= 2 {
        let lang = parts[0].to_string();
        let categories = parts[1..parts.len() - 1]
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>();
        return (lang, categories);
    }

    // If we cannot infer language, keep it empty and map the directory tree (excluding filename) as categories.
    let categories = parts[..parts.len().saturating_sub(1)]
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    (String::new(), categories)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_document_type_uri_prefixes() {
        assert_eq!(
            DocumentType::Agreements.get_uri_prefix(),
            "docs://agreements/"
        );
        assert_eq!(
            DocumentType::ErdDiagram("proj-a".to_string()).get_uri_prefix(),
            "docs://architecture/erd/proj-a/"
        );
        assert_eq!(
            DocumentType::C1Diagram("proj-a".to_string()).get_uri_prefix(),
            "docs://architecture/proj-a/"
        );
        assert_eq!(
            DocumentType::C2Diagram("proj-a".to_string()).get_uri_prefix(),
            "docs://architecture/proj-a/"
        );
        assert_eq!(
            DocumentType::C3Diagram("proj-a".to_string()).get_uri_prefix(),
            "docs://architecture/proj-a/"
        );
        assert_eq!(
            DocumentType::GuideDoc("eva4".to_string()).get_uri_prefix(),
            "docs://guides/eva4/"
        );
    }

    #[test]
    fn test_document_type_generate_description() {
        let agreements_desc = DocumentType::Agreements.generate_description(
            "backend",
            "php",
            &["agreements".to_string(), "api".to_string()],
            "test.md",
        );
        assert_eq!(
            agreements_desc,
            "Agreement document: agreements, api - backend (php)"
        );

        let erd_desc = DocumentType::ErdDiagram("proj-a".to_string()).generate_description(
            "frontend",
            "js",
            &["erd".to_string()],
            "diagram.mdx",
        );
        assert_eq!(erd_desc, "ERD diagram for proj-a project: diagram");

        let c1_desc = DocumentType::C1Diagram("proj-a".to_string()).generate_description(
            "architecture",
            "",
            &["c1".to_string()],
            "c1.mdx",
        );
        assert_eq!(c1_desc, "C1 diagram for proj-a project");

        let c2_desc = DocumentType::C2Diagram("proj-b".to_string()).generate_description(
            "architecture",
            "",
            &["c2".to_string()],
            "c2.mdx",
        );
        assert_eq!(c2_desc, "C2 diagram for proj-b project");

        let c3_desc = DocumentType::C3Diagram("proj-a".to_string()).generate_description(
            "architecture",
            "",
            &["c3".to_string()],
            "c3.mdx",
        );
        assert_eq!(c3_desc, "C3 diagram for proj-a project");

        let guide_desc = DocumentType::GuideDoc("eva4".to_string()).generate_description(
            "guides",
            "",
            &["guides".to_string()],
            "eva-repl.rst",
        );
        assert_eq!(guide_desc, "Guide: eva4 - eva-repl");

        let guide_desc_install = DocumentType::GuideDoc("psrt".to_string()).generate_description(
            "guides",
            "",
            &["guides".to_string()],
            "install_guide.rst",
        );
        assert_eq!(guide_desc_install, "Guide: psrt - install guide");
    }

    #[test]
    fn test_document_key_creation() {
        let uri = "docs://agreements/backend/php/api/test.md".to_string();
        let key = DocumentKey::new(uri.clone());
        // DocumentKey wraps the URI string, so we can compare by creating a new key
        assert_eq!(key, DocumentKey::new(uri));
    }

    #[test]
    fn test_resource_info_creation() {
        let resource_info = ResourceInfo {
            uri: "docs://test/uri".to_string(),
            file_path: "test/path.md".to_string(),
            area: "backend".to_string(),
            lang: "php".to_string(),
            category: vec!["api".to_string()],
            project: "proj-a".to_string(),
            mime_type: "text/markdown".to_string(),
            size: 1024,
            description: "Test document".to_string(),
        };

        assert_eq!(resource_info.uri, "docs://test/uri");
        assert_eq!(resource_info.area, "backend");
        assert_eq!(resource_info.project, "proj-a");
        assert_eq!(resource_info.size, 1024);
        assert_eq!(resource_info.category, vec!["api"]);
    }

    #[test]
    fn test_document_scanner_get_mime_type() {
        assert_eq!(DocumentScanner::get_mime_type("test.md"), "text/markdown");
        assert_eq!(DocumentScanner::get_mime_type("test.mdx"), "text/markdown");
        assert_eq!(DocumentScanner::get_mime_type("test.txt"), "text/plain");
        assert_eq!(
            DocumentScanner::get_mime_type("test.yaml"),
            "application/x-yaml"
        );
        assert_eq!(
            DocumentScanner::get_mime_type("test.yml"),
            "application/x-yaml"
        );
        assert_eq!(DocumentScanner::get_mime_type("install.rst"), "text/x-rst");
        assert_eq!(DocumentScanner::get_mime_type("test.unknown"), "text/plain");
    }

    #[test]
    fn test_should_process_file() {
        // C4 diagrams - only specific files
        assert!(DocumentScanner::should_process_file(
            &DocumentType::C1Diagram("proj-a".to_string()),
            "c1.mdx"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::C1Diagram("proj-a".to_string()),
            "c2.mdx"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::C1Diagram("proj-a".to_string()),
            "other.mdx"
        ));

        // C4 service diagrams - all .mdx files
        assert!(DocumentScanner::should_process_file(
            &DocumentType::C4Diagram("proj-a".to_string()),
            "activation.mdx"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::C4Diagram("proj-a".to_string()),
            "activation.yaml"
        ));

        // ERD diagrams - all .mdx files
        assert!(DocumentScanner::should_process_file(
            &DocumentType::ErdDiagram("proj-a".to_string()),
            "user-entities.mdx"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::ErdDiagram("proj-a".to_string()),
            "user-entities.yaml"
        ));

        // ADR documents - all .mdx files
        assert!(DocumentScanner::should_process_file(
            &DocumentType::AdrDocument("proj-a".to_string()),
            "001-temporal-transactionality.mdx"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::AdrDocument("proj-a".to_string()),
            "001-temporal-transactionality.yaml"
        ));

        // OpenAPI specs - all .yaml files
        assert!(DocumentScanner::should_process_file(
            &DocumentType::OpenApiSpec("mpa".to_string()),
            "get-customer-activation-info.yaml"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::OpenApiSpec("mpa".to_string()),
            "get-customer-activation-info.mdx"
        ));

        // Agreements - all supported files
        assert!(DocumentScanner::should_process_file(
            &DocumentType::Agreements,
            "api.md"
        ));
        assert!(DocumentScanner::should_process_file(
            &DocumentType::Agreements,
            "api.mdx"
        ));
        assert!(DocumentScanner::should_process_file(
            &DocumentType::Agreements,
            "api.txt"
        ));
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::Agreements,
            "api.yaml"
        ));

        // GuideDoc uses extension-based path only, not should_process_file
        assert!(!DocumentScanner::should_process_file(
            &DocumentType::GuideDoc("eva4".to_string()),
            "install.rst"
        ));
    }

    #[test]
    fn test_c4_diagram_uri_generation() {
        // Test C1 diagram URI generation
        let c1_proj_a = DocumentType::C1Diagram("proj-a".to_string());
        assert_eq!(c1_proj_a.get_uri_prefix(), "docs://architecture/proj-a/");

        // Test C2 diagram URI generation
        let c2_proj_b = DocumentType::C2Diagram("proj-b".to_string());
        assert_eq!(c2_proj_b.get_uri_prefix(), "docs://architecture/proj-b/");

        // Test C3 diagram URI generation
        let c3_proj_a = DocumentType::C3Diagram("proj-a".to_string());
        assert_eq!(c3_proj_a.get_uri_prefix(), "docs://architecture/proj-a/");
    }

    #[test]
    fn test_c4_diagram_description_generation() {
        // Test C1 diagram description
        let c1_desc = DocumentType::C1Diagram("proj-a".to_string()).generate_description(
            "architecture",
            "",
            &["c1".to_string()],
            "c1.mdx",
        );
        assert_eq!(c1_desc, "C1 diagram for proj-a project");

        // Test C2 diagram description
        let c2_desc = DocumentType::C2Diagram("proj-b".to_string()).generate_description(
            "architecture",
            "",
            &["c2".to_string()],
            "c2.mdx",
        );
        assert_eq!(c2_desc, "C2 diagram for proj-b project");

        // Test C3 diagram description
        let c3_desc = DocumentType::C3Diagram("proj-a".to_string()).generate_description(
            "architecture",
            "",
            &["c3".to_string()],
            "c3.mdx",
        );
        assert_eq!(c3_desc, "C3 diagram for proj-a project");
    }

    #[test]
    fn test_c4_diagram_file_filtering() {
        // Test that C4 diagrams only process specific files
        let c1_proj_a = DocumentType::C1Diagram("proj-a".to_string());
        let c2_proj_b = DocumentType::C2Diagram("proj-b".to_string());
        let c3_proj_a = DocumentType::C3Diagram("proj-a".to_string());
        let c4_activation = DocumentType::C4Diagram("proj-a".to_string());

        // These should be valid C4 diagram types
        assert!(matches!(c1_proj_a, DocumentType::C1Diagram(_)));
        assert!(matches!(c2_proj_b, DocumentType::C2Diagram(_)));
        assert!(matches!(c3_proj_a, DocumentType::C3Diagram(_)));
        assert!(matches!(c4_activation, DocumentType::C4Diagram(_)));

        // Test that we can identify C4 diagrams correctly
        let is_c4_diagram = matches!(
            c1_proj_a,
            DocumentType::C1Diagram(_) | DocumentType::C2Diagram(_) | DocumentType::C3Diagram(_)
        );
        assert!(is_c4_diagram);

        // Test that we can identify C4 service diagrams correctly
        let is_c4_service_diagram = matches!(c4_activation, DocumentType::C4Diagram(_));
        assert!(is_c4_service_diagram);
    }

    #[test]
    fn test_c4_service_diagram_uri_generation() {
        // Test C4 service diagram URI generation
        let c4_activation = DocumentType::C4Diagram("proj-a".to_string());
        assert_eq!(
            c4_activation.get_uri_prefix(),
            "docs://architecture/proj-a/c4/"
        );
    }

    #[test]
    fn test_c4_service_diagram_description_generation() {
        // Test C4 service diagram description
        let c4_desc = DocumentType::C4Diagram("proj-a".to_string()).generate_description(
            "architecture",
            "",
            &["c4".to_string()],
            "activation.mdx",
        );
        assert_eq!(
            c4_desc,
            "C4 diagram for activation service in proj-a project"
        );
    }

    #[test]
    fn test_c4_diagram_metadata_validation() {
        // Test that C1-C4 diagrams have correct metadata
        let c1_doc = DocumentType::C1Diagram("proj-a".to_string());
        let c2_doc = DocumentType::C2Diagram("proj-b".to_string());
        let c3_doc = DocumentType::C3Diagram("proj-a".to_string());
        let c4_doc = DocumentType::C4Diagram("proj-b".to_string());

        // Test URI prefixes
        assert_eq!(c1_doc.get_uri_prefix(), "docs://architecture/proj-a/");
        assert_eq!(c2_doc.get_uri_prefix(), "docs://architecture/proj-b/");
        assert_eq!(c3_doc.get_uri_prefix(), "docs://architecture/proj-a/");
        assert_eq!(c4_doc.get_uri_prefix(), "docs://architecture/proj-b/c4/");

        // Test descriptions
        assert_eq!(
            c1_doc.generate_description("architecture", "", &["c1".to_string()], "c1.mdx"),
            "C1 diagram for proj-a project"
        );
        assert_eq!(
            c2_doc.generate_description("architecture", "", &["c2".to_string()], "c2.mdx"),
            "C2 diagram for proj-b project"
        );
        assert_eq!(
            c3_doc.generate_description("architecture", "", &["c3".to_string()], "c3.mdx"),
            "C3 diagram for proj-a project"
        );
        assert_eq!(
            c4_doc.generate_description("architecture", "", &["c4".to_string()], "activation.mdx"),
            "C4 diagram for activation service in proj-b project"
        );
    }

    #[test]
    fn test_c4_diagram_category_mapping() {
        // Test that categories are correctly mapped for C1-C4 diagrams
        let c1_doc = DocumentType::C1Diagram("proj-a".to_string());
        let c2_doc = DocumentType::C2Diagram("proj-b".to_string());
        let c3_doc = DocumentType::C3Diagram("proj-a".to_string());
        let c4_doc = DocumentType::C4Diagram("proj-b".to_string());

        // These should match the category logic in process_file
        assert!(matches!(c1_doc, DocumentType::C1Diagram(_)));
        assert!(matches!(c2_doc, DocumentType::C2Diagram(_)));
        assert!(matches!(c3_doc, DocumentType::C3Diagram(_)));
        assert!(matches!(c4_doc, DocumentType::C4Diagram(_)));
    }

    #[test]
    fn test_erd_diagram_uri_prefixes() {
        // Test ERD diagram URI generation for proj-a project
        let erd_proj_a = DocumentType::ErdDiagram("proj-a".to_string());
        assert_eq!(
            erd_proj_a.get_uri_prefix(),
            "docs://architecture/erd/proj-a/"
        );

        // Test ERD diagram URI generation for proj-b project
        let erd_proj_b = DocumentType::ErdDiagram("proj-b".to_string());
        assert_eq!(
            erd_proj_b.get_uri_prefix(),
            "docs://architecture/erd/proj-b/"
        );
    }

    #[test]
    fn test_erd_diagram_description_generation() {
        // Test ERD diagram description for proj-a project
        let erd_proj_a_desc = DocumentType::ErdDiagram("proj-a".to_string()).generate_description(
            "backend",
            "php",
            &["erd".to_string()],
            "user_entities.mdx",
        );
        assert_eq!(
            erd_proj_a_desc,
            "ERD diagram for proj-a project: user_entities"
        );

        // Test ERD diagram description for proj-b project
        let erd_proj_b_description = DocumentType::ErdDiagram("proj-b".to_string())
            .generate_description("frontend", "js", &["erd".to_string()], "data_schema.mdx");
        assert_eq!(
            erd_proj_b_description,
            "ERD diagram for proj-b project: data_schema"
        );
    }

    #[test]
    fn test_erd_diagram_metadata_validation() {
        // Test that ERD diagrams have correct metadata for both projects
        let erd_proj_a = DocumentType::ErdDiagram("proj-a".to_string());
        let erd_proj_b = DocumentType::ErdDiagram("proj-b".to_string());

        // Test URI prefixes
        assert_eq!(
            erd_proj_a.get_uri_prefix(),
            "docs://architecture/erd/proj-a/"
        );
        assert_eq!(
            erd_proj_b.get_uri_prefix(),
            "docs://architecture/erd/proj-b/"
        );

        // Test descriptions
        assert_eq!(
            erd_proj_a.generate_description("backend", "php", &["erd".to_string()], "entities.mdx"),
            "ERD diagram for proj-a project: entities"
        );
        assert_eq!(
            erd_proj_b.generate_description("frontend", "js", &["erd".to_string()], "schema.mdx"),
            "ERD diagram for proj-b project: schema"
        );
    }

    #[test]
    fn test_erd_diagram_project_initialization() {
        // Test ERD diagram initialization for proj-a project
        let erd_proj_a = DocumentType::ErdDiagram("proj-a".to_string());
        assert!(matches!(erd_proj_a, DocumentType::ErdDiagram(_)));

        // Test ERD diagram initialization for proj-b project
        let erd_proj_b = DocumentType::ErdDiagram("proj-b".to_string());
        assert!(matches!(erd_proj_b, DocumentType::ErdDiagram(_)));

        // Verify both projects are properly initialized
        assert_eq!(
            erd_proj_a.get_uri_prefix(),
            "docs://architecture/erd/proj-a/"
        );
        assert_eq!(
            erd_proj_b.get_uri_prefix(),
            "docs://architecture/erd/proj-b/"
        );
    }

    #[test]
    fn test_erd_diagram_scanning_logic() {
        // Test that ERD diagrams are identified correctly for scanning
        let erd_proj_a = DocumentType::ErdDiagram("proj-a".to_string());
        let erd_proj_b = DocumentType::ErdDiagram("proj-b".to_string());

        // Test ERD diagram identification
        let is_erd_diagram_proj_a = matches!(erd_proj_a, DocumentType::ErdDiagram(_));
        let is_erd_diagram_proj_b = matches!(erd_proj_b, DocumentType::ErdDiagram(_));

        assert!(is_erd_diagram_proj_a);
        assert!(is_erd_diagram_proj_b);

        // Test that ERD diagrams are not C4 diagrams
        let is_c4_diagram_proj_a = matches!(
            erd_proj_a,
            DocumentType::C1Diagram(_) | DocumentType::C2Diagram(_) | DocumentType::C3Diagram(_)
        );
        let is_c4_service_diagram_proj_a = matches!(erd_proj_a, DocumentType::C4Diagram(_));

        assert!(!is_c4_diagram_proj_a);
        assert!(!is_c4_service_diagram_proj_a);
    }

    #[test]
    fn test_erd_diagram_path_processing() {
        // Test ERD diagram path processing for proj-a project
        let erd_proj_a = DocumentType::ErdDiagram("proj-a".to_string());

        // Test that ERD diagrams process .mdx files
        let test_filename = "user_entities.mdx";
        let is_erd_diagram = matches!(erd_proj_a, DocumentType::ErdDiagram(_));

        assert!(is_erd_diagram);
        assert!(
            Path::new(test_filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("mdx"))
        );

        // Test URI generation for ERD diagrams
        let expected_uri = format!("{}{}", erd_proj_a.get_uri_prefix(), test_filename);
        assert_eq!(
            expected_uri,
            "docs://architecture/erd/proj-a/user_entities.mdx"
        );
    }

    #[test]
    fn test_erd_diagram_path_matching() {
        // Test that ERD diagram paths are matched correctly
        let path = "content/docs/architecture/proj-b/erd/mail-transport.mdx";
        let path_parts: Vec<&str> = path.split('/').collect();

        // This should match the ERD pattern, not the standard pattern
        match path_parts.as_slice() {
            ["content", "docs", "architecture", project, "erd", filename] => {
                assert_eq!(*project, "proj-b");
                assert_eq!(*filename, "mail-transport.mdx");

                // Test URI generation
                let erd_diagram = DocumentType::ErdDiagram((*project).to_string());
                let uri = format!("{}{}", erd_diagram.get_uri_prefix(), filename);
                assert_eq!(uri, "docs://architecture/erd/proj-b/mail-transport.mdx");
            }
            _ => panic!("ERD diagram path should match the specific pattern"),
        }
    }

    #[test]
    fn test_adr_document_uri_prefix() {
        // Test ADR document URI generation for proj-a project
        let adr_proj_a = DocumentType::AdrDocument("proj-a".to_string());
        assert_eq!(
            adr_proj_a.get_uri_prefix(),
            "docs://architecture/proj-a/adr/"
        );

        // Test ADR document URI generation for proj-b project
        let adr_proj_b = DocumentType::AdrDocument("proj-b".to_string());
        assert_eq!(
            adr_proj_b.get_uri_prefix(),
            "docs://architecture/proj-b/adr/"
        );
    }

    #[test]
    fn test_adr_document_description_generation() {
        // Test ADR document description for proj-a project
        let adr_proj_a_desc = DocumentType::AdrDocument("proj-a".to_string()).generate_description(
            "architecture",
            "",
            &["ADR-001".to_string()],
            "001-temporal-transactionality.mdx",
        );
        assert_eq!(adr_proj_a_desc, "ADR-001 for proj-a project");

        // Test ADR document description for proj-b project
        let adr_proj_b_description = DocumentType::AdrDocument("proj-b".to_string())
            .generate_description(
                "architecture",
                "",
                &["ADR-002".to_string()],
                "002-microservices-communication.mdx",
            );
        assert_eq!(adr_proj_b_description, "ADR-002 for proj-b project");
    }

    #[test]
    fn test_adr_document_path_matching() {
        // Test that ADR document paths are matched correctly
        let path = "content/docs/architecture/proj-a/adr/001-temporal-transactionality.mdx";
        let path_parts: Vec<&str> = path.split('/').collect();

        // This should match the ADR pattern
        match path_parts.as_slice() {
            ["content", "docs", "architecture", project, "adr", filename] => {
                assert_eq!(*project, "proj-a");
                assert_eq!(*filename, "001-temporal-transactionality.mdx");

                // Test URI generation
                let adr_document = DocumentType::AdrDocument((*project).to_string());
                let uri = format!("{}{}", adr_document.get_uri_prefix(), filename);
                assert_eq!(
                    uri,
                    "docs://architecture/proj-a/adr/001-temporal-transactionality.mdx"
                );
            }
            _ => panic!("ADR document path should match the specific pattern"),
        }
    }

    #[test]
    fn test_erd_path_vs_standard_path() {
        // Test different path patterns to understand the issue
        let erd_path = "content/docs/architecture/proj-b/erd/mail-transport.mdx";
        let standard_path = "content/docs/backend/php/api/test.md";

        // Test ERD path
        let erd_parts: Vec<&str> = erd_path.split('/').collect();
        println!("ERD path parts: {:?}", erd_parts);

        // Test standard path
        let standard_parts: Vec<&str> = standard_path.split('/').collect();
        println!("Standard path parts: {:?}", standard_parts);

        // ERD path should have 6 parts
        assert_eq!(erd_parts.len(), 6);
        assert_eq!(
            erd_parts,
            [
                "content",
                "docs",
                "architecture",
                "proj-b",
                "erd",
                "mail-transport.mdx"
            ]
        );

        // Standard path should have 6 parts too
        assert_eq!(standard_parts.len(), 6);
        assert_eq!(
            standard_parts,
            ["content", "docs", "backend", "php", "api", "test.md"]
        );

        // Test that ERD path matches ERD pattern
        match erd_parts.as_slice() {
            ["content", "docs", "architecture", project, "erd", filename] => {
                let erd_diagram = DocumentType::ErdDiagram((*project).to_string());
                let uri = format!("{}{}", erd_diagram.get_uri_prefix(), filename);
                assert_eq!(uri, "docs://architecture/erd/proj-b/mail-transport.mdx");
                println!("ERD URI: {}", uri);
            }
            ["content", "docs", area, lang, category, filename] => {
                panic!(
                    "ERD path should not match standard pattern, but got area={}, lang={}, category={}, filename={}",
                    area, lang, category, filename
                );
            }
            _ => panic!("ERD path should match one of the patterns"),
        }
    }

    #[test]
    fn test_openapi_spec_uri_prefix() {
        let openapi_proj_a = DocumentType::OpenApiSpec("proj-a".to_string());
        assert_eq!(openapi_proj_a.get_uri_prefix(), "docs://openapi/proj-a/");

        let openapi_mpa = DocumentType::OpenApiSpec("mpa".to_string());
        assert_eq!(openapi_mpa.get_uri_prefix(), "docs://openapi/mpa/");
    }

    #[test]
    fn test_openapi_spec_description_generation() {
        let openapi_desc = DocumentType::OpenApiSpec("mpa".to_string()).generate_description(
            "openapi",
            "",
            &[
                "openapi".to_string(),
                "activation".to_string(),
                "v2".to_string(),
                "public".to_string(),
            ],
            "get-customer-activation-info.yaml",
        );
        assert_eq!(
            openapi_desc,
            "OpenAPI specification for get-customer-activation-info endpoint in mpa project"
        );
    }

    #[test]
    fn test_openapi_spec_path_matching() {
        let path = "content/docs/openapi-spec/mpa/activation/v2/public/endpoints/get-customer-activation-info.yaml";
        let path_parts: Vec<&str> = path.split('/').collect();

        match path_parts.as_slice() {
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                "endpoints",
                filename,
            ] => {
                assert_eq!(*project, "mpa");
                assert_eq!(*service, "activation");
                assert_eq!(*version, "v2");
                assert_eq!(*access_level, "public");
                assert_eq!(*filename, "get-customer-activation-info.yaml");

                let openapi_spec = DocumentType::OpenApiSpec((*project).to_string());
                let uri = format!(
                    "{}{}/{}/{}/{}",
                    openapi_spec.get_uri_prefix(),
                    service,
                    version,
                    access_level,
                    filename
                );
                assert_eq!(
                    uri,
                    "docs://openapi/mpa/activation/v2/public/get-customer-activation-info.yaml"
                );
            }
            _ => panic!("OpenAPI spec path should match the specific pattern"),
        }
    }

    #[test]
    fn test_openapi_spec_metadata_validation() {
        let openapi_mpa = DocumentType::OpenApiSpec("mpa".to_string());
        assert_eq!(openapi_mpa.get_uri_prefix(), "docs://openapi/mpa/");

        assert_eq!(
            openapi_mpa.generate_description(
                "openapi",
                "",
                &[
                    "openapi".to_string(),
                    "activation".to_string(),
                    "v2".to_string(),
                    "public".to_string(),
                ],
                "get-customer-activation-info.yaml"
            ),
            "OpenAPI specification for get-customer-activation-info endpoint in mpa project"
        );
    }

    #[test]
    fn test_openapi_spec_initialization() {
        let openapi_mpa = DocumentType::OpenApiSpec("mpa".to_string());
        assert!(matches!(openapi_mpa, DocumentType::OpenApiSpec(_)));

        let openapi_proj_a = DocumentType::OpenApiSpec("proj-a".to_string());
        assert!(matches!(openapi_proj_a, DocumentType::OpenApiSpec(_)));

        assert_eq!(openapi_mpa.get_uri_prefix(), "docs://openapi/mpa/");
        assert_eq!(openapi_proj_a.get_uri_prefix(), "docs://openapi/proj-a/");
    }

    #[test]
    fn test_openapi_spec_path_without_endpoints() {
        let path = "content/docs/openapi-spec/mpa/activation/v2/public/seat-activation.yaml";
        let path_parts: Vec<&str> = path.split('/').collect();

        match path_parts.as_slice() {
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                filename,
            ] => {
                assert_eq!(*project, "mpa");
                assert_eq!(*service, "activation");
                assert_eq!(*version, "v2");
                assert_eq!(*access_level, "public");
                assert_eq!(*filename, "seat-activation.yaml");

                let openapi_spec = DocumentType::OpenApiSpec((*project).to_string());
                let uri = format!(
                    "{}{}/{}/{}/{}",
                    openapi_spec.get_uri_prefix(),
                    service,
                    version,
                    access_level,
                    filename
                );
                assert_eq!(
                    uri,
                    "docs://openapi/mpa/activation/v2/public/seat-activation.yaml"
                );
            }
            _ => panic!("OpenAPI spec path without endpoints should match the specific pattern"),
        }
    }

    #[test]
    fn test_openapi_spec_path_with_subcategory() {
        let path = "content/docs/openapi-spec/mpa/oauth/v2/internal/activation/endpoints/get-customer-by-activation-id.yaml";
        let path_parts: Vec<&str> = path.split('/').collect();

        match path_parts.as_slice() {
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                sub_category,
                "endpoints",
                filename,
            ] => {
                assert_eq!(*project, "mpa");
                assert_eq!(*service, "oauth");
                assert_eq!(*version, "v2");
                assert_eq!(*access_level, "internal");
                assert_eq!(*sub_category, "activation");
                assert_eq!(*filename, "get-customer-by-activation-id.yaml");

                let openapi_spec = DocumentType::OpenApiSpec((*project).to_string());
                let uri = format!(
                    "{}{}/{}/{}/{}/{}",
                    openapi_spec.get_uri_prefix(),
                    service,
                    version,
                    access_level,
                    sub_category,
                    filename
                );
                assert_eq!(
                    uri,
                    "docs://openapi/mpa/oauth/v2/internal/activation/get-customer-by-activation-id.yaml"
                );
            }
            _ => panic!(
                "OpenAPI spec path with subcategory and endpoints should match the specific pattern"
            ),
        }
    }

    #[test]
    fn test_openapi_spec_path_with_subcategory_without_endpoints() {
        let path = "content/docs/openapi-spec/mpa/product/v2/internal/gateway/file.yaml";
        let path_parts: Vec<&str> = path.split('/').collect();

        match path_parts.as_slice() {
            [
                "content",
                "docs",
                "openapi-spec",
                project,
                service,
                version,
                access_level,
                sub_category,
                filename,
            ] => {
                assert_eq!(*project, "mpa");
                assert_eq!(*service, "product");
                assert_eq!(*version, "v2");
                assert_eq!(*access_level, "internal");
                assert_eq!(*sub_category, "gateway");
                assert_eq!(*filename, "file.yaml");

                let openapi_spec = DocumentType::OpenApiSpec((*project).to_string());
                let uri = format!(
                    "{}{}/{}/{}/{}/{}",
                    openapi_spec.get_uri_prefix(),
                    service,
                    version,
                    access_level,
                    sub_category,
                    filename
                );
                assert_eq!(
                    uri,
                    "docs://openapi/mpa/product/v2/internal/gateway/file.yaml"
                );
            }
            _ => panic!(
                "OpenAPI spec path with subcategory without endpoints should match the specific pattern"
            ),
        }
    }

    #[test]
    fn scan_with_extensions_and_missing_target_does_not_fail() {
        let temp_dir = TempDir::new().expect("temp dir");
        let docs_root = temp_dir.path();

        let c4_dir = docs_root.join("arch/c4");
        fs::create_dir_all(&c4_dir).expect("create c4 dir");
        fs::write(c4_dir.join("c1.puml"), "@startuml\n@enduml\n").expect("write c1.puml");

        let openapi_dir = docs_root.join("openapi");
        fs::create_dir_all(&openapi_dir).expect("create openapi dir");
        fs::write(openapi_dir.join("service.yml"), "openapi: 3.0.0\n").expect("write service.yml");

        let guide_dir = docs_root.join("eva4");
        fs::create_dir_all(guide_dir.join("svc")).expect("create guide dir");
        fs::write(
            docs_root.join("eva4/svc/eva-repl.rst"),
            "Replication service\n*******************\n",
        )
        .expect("write eva-repl.rst");

        let file_reader = FileReader::new(docs_root.to_string_lossy().to_string()).expect("reader");
        let mut resources: BTreeMap<DocumentKey, ResourceInfo> = BTreeMap::new();

        DocumentScanner::scan_documents_with_extensions(
            DocumentType::C1Diagram("proj-a".to_string()),
            vec!["arch/c4".to_string(), "missing/path".to_string()],
            &["puml".to_string(), "dot".to_string(), "mdx".to_string()],
            &file_reader,
            &mut resources,
        );

        DocumentScanner::scan_documents_with_extensions(
            DocumentType::OpenApiSpec("proj-a".to_string()),
            vec!["openapi".to_string()],
            &["yaml".to_string(), "yml".to_string()],
            &file_reader,
            &mut resources,
        );

        DocumentScanner::scan_documents_with_extensions(
            DocumentType::GuideDoc("eva4".to_string()),
            vec!["eva4".to_string()],
            &["rst".to_string()],
            &file_reader,
            &mut resources,
        );

        assert!(resources.contains_key(&DocumentKey::new(
            "docs://architecture/proj-a/c1.puml".to_string()
        )));
        assert!(resources.contains_key(&DocumentKey::new(
            "docs://openapi/proj-a/service.yml".to_string()
        )));
        assert!(resources.contains_key(&DocumentKey::new(
            "docs://guides/eva4/svc/eva-repl.rst".to_string()
        )));
    }
}
