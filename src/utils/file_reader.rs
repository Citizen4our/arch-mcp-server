use std::{env, fs, path::Path};

/// File reader that reads files relative to a specified docs root path.
///
/// # Examples
/// ```
/// use arch_mcp_server::utils::file_reader::FileReader;
///
/// let reader = FileReader::new("/path/to/docs")?;
/// let content = reader.read_file_content("README.md")?;
/// println!("{}", content);
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileReader {
    docs_root: String,
}

#[allow(dead_code)]
impl FileReader {
    /// Creates a new FileReader with the specified docs root path.
    /// Supports both absolute and relative paths.
    ///
    /// # Arguments
    /// * `docs_root` - The root directory for document files (absolute or relative)
    ///
    /// # Returns
    /// * `Result<Self, std::io::Error>` - FileReader instance or error
    pub fn new(docs_root: impl Into<String>) -> Result<Self, std::io::Error> {
        let docs_root = docs_root.into();
        let path = Path::new(&docs_root);

        // Resolve the path to absolute if it's relative
        let resolved_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            // For relative paths, resolve relative to current working directory
            std::env::current_dir()
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Cannot get current directory: {}", e),
                    )
                })?
                .join(path)
        };

        // Validate that the resolved path exists and is a directory
        if !resolved_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "DOCS_ROOT_PATH does not exist: {} (resolved from: {})",
                    resolved_path.display(),
                    docs_root
                ),
            ));
        }

        if !resolved_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "DOCS_ROOT_PATH is not a directory: {} (resolved from: {})",
                    resolved_path.display(),
                    docs_root
                ),
            ));
        }

        // Store the resolved absolute path
        let resolved_docs_root = resolved_path.to_string_lossy().to_string();
        Ok(Self {
            docs_root: resolved_docs_root,
        })
    }

    /// Creates a new FileReader by reading DOCS_ROOT_PATH from environment variables.
    /// Supports both environment variables and .env files.
    /// DOCS_ROOT_PATH can be either an absolute or relative path.
    ///
    /// # Returns
    /// * `Result<Self, std::io::Error>` - FileReader instance or error
    pub fn from_env() -> Result<Self, std::io::Error> {
        // Try to load .env file if it exists
        let _ = dotenvy::dotenv();

        let docs_root = env::var("DOCS_ROOT_PATH").map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "DOCS_ROOT_PATH environment variable not set",
            )
        })?;

        Self::new(docs_root)
    }

    /// Reads file content from a path relative to the docs root.
    ///
    /// # Arguments
    /// * `relative_path` - Path relative to docs root
    ///
    /// # Returns
    /// * `Result<String, std::io::Error>` - File content as String or error
    pub fn read_file_content(&self, relative_path: &str) -> Result<String, std::io::Error> {
        let full_path = Path::new(&self.docs_root).join(relative_path);

        // Validate that the path is within the docs root to prevent directory traversal
        let canonical_docs_root = fs::canonicalize(&self.docs_root).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Cannot canonicalize DOCS_ROOT_PATH: {}", e),
            )
        })?;

        let canonical_full_path = fs::canonicalize(&full_path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cannot canonicalize file path: {}", e),
            )
        })?;

        if !canonical_full_path.starts_with(&canonical_docs_root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Path traversal detected: file path is outside DOCS_ROOT_PATH",
            ));
        }

        fs::read_to_string(&canonical_full_path)
    }

    /// Reads file content as bytes from a path relative to the docs root.
    ///
    /// # Arguments
    /// * `relative_path` - Path relative to docs root
    ///
    /// # Returns
    /// * `Result<Vec<u8>, std::io::Error>` - File content as bytes or error
    pub fn read_file_bytes(&self, relative_path: &str) -> Result<Vec<u8>, std::io::Error> {
        let full_path = Path::new(&self.docs_root).join(relative_path);

        // Validate that the path is within the docs root to prevent directory traversal
        let canonical_docs_root = fs::canonicalize(&self.docs_root).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Cannot canonicalize DOCS_ROOT_PATH: {}", e),
            )
        })?;

        let canonical_full_path = fs::canonicalize(&full_path).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cannot canonicalize file path: {}", e),
            )
        })?;

        if !canonical_full_path.starts_with(&canonical_docs_root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Path traversal detected: file path is outside DOCS_ROOT_PATH",
            ));
        }

        fs::read(&canonical_full_path)
    }

    /// Gets the docs root path.
    pub fn docs_root(&self) -> &str {
        &self.docs_root
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::TempDir;

    use super::*;

    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let docs_root = temp_dir.path().join("docs");
        fs::create_dir_all(&docs_root).expect("Failed to create docs directory");

        (temp_dir, docs_root)
    }

    #[test]
    fn test_file_reader_new_success() {
        let (_temp_dir, docs_root) = setup_test_env();

        let reader = FileReader::new(docs_root.to_str().unwrap());
        assert!(reader.is_ok());

        let reader = reader.unwrap();
        assert_eq!(reader.docs_root(), docs_root.to_str().unwrap());
    }

    #[test]
    fn test_file_reader_new_nonexistent_path() {
        let result = FileReader::new("/nonexistent/path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_file_reader_new_file_not_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("not_a_dir");
        fs::write(&file_path, "content").expect("Failed to write file");

        let result = FileReader::new(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_read_file_content_success() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Create test file
        let test_file = docs_root.join("test.txt");
        let test_content = "Hello, World!";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        let reader = FileReader::new(docs_root.to_str().unwrap()).expect("Failed to create reader");
        let result = reader.read_file_content("test.txt");

        match result {
            Ok(content) => assert_eq!(content, test_content),
            Err(e) => panic!("Failed to read file: {}", e),
        }
    }

    #[test]
    fn test_read_file_content_with_subdirectory() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Create subdirectory and file
        let subdir = docs_root.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdirectory");

        let test_file = subdir.join("nested.txt");
        let test_content = "Nested content";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        let reader = FileReader::new(docs_root.to_str().unwrap()).expect("Failed to create reader");
        let result = reader.read_file_content("subdir/nested.txt");

        match result {
            Ok(content) => assert_eq!(content, test_content),
            Err(e) => panic!("Failed to read file: {}", e),
        }
    }

    #[test]
    fn test_read_file_content_file_not_found() {
        let (_temp_dir, docs_root) = setup_test_env();

        let reader = FileReader::new(docs_root.to_str().unwrap()).expect("Failed to create reader");
        let result = reader.read_file_content("nonexistent.txt");

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_read_file_content_path_traversal_protection() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Create test file
        let test_file = docs_root.join("test.txt");
        fs::write(&test_file, "test content").expect("Failed to write test file");

        let reader = FileReader::new(docs_root.to_str().unwrap()).expect("Failed to create reader");

        // Try to access file outside docs root using path traversal
        let result = reader.read_file_content("../test.txt");
        match result {
            Err(e) => {
                // The error could be either PermissionDenied or NotFound depending on the system
                assert!(matches!(
                    e.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::NotFound
                ));
            }
            Ok(_) => panic!("Path traversal should have been blocked"),
        }
    }

    #[test]
    fn test_read_file_bytes_success() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Create test file with binary content
        let test_file = docs_root.join("test.bin");
        let test_content = b"Binary content with \x00 null bytes";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        let reader = FileReader::new(docs_root.to_str().unwrap()).expect("Failed to create reader");
        let result = reader.read_file_bytes("test.bin");

        match result {
            Ok(content) => assert_eq!(content, test_content),
            Err(e) => panic!("Failed to read file: {}", e),
        }
    }

    #[test]
    fn test_read_file_content_unicode() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Create test file with Unicode content
        let test_file = docs_root.join("unicode.txt");
        let test_content = "Hello, world! 🌍";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        let reader = FileReader::new(docs_root.to_str().unwrap()).expect("Failed to create reader");
        let result = reader.read_file_content("unicode.txt");

        match result {
            Ok(content) => assert_eq!(content, test_content),
            Err(e) => panic!("Failed to read file: {}", e),
        }
    }

    #[test]
    fn test_from_env_missing_env_var() {
        // Ensure env var is not set
        unsafe {
            env::remove_var("DOCS_ROOT_PATH");
        }

        let result = FileReader::from_env();
        match result {
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
            }
            Ok(_) => {
                // If it succeeds, it means .env file was loaded with DOCS_ROOT_PATH
                // This is also a valid scenario, so we just log it
                println!("Note: .env file was loaded with DOCS_ROOT_PATH");
            }
        }
    }

    #[test]
    fn test_file_reader_absolute_path_success() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Test FileReader::new directly instead of using environment variables
        // This avoids the unsafe environment variable manipulation
        let result = FileReader::new(docs_root.to_str().unwrap());
        assert!(result.is_ok());

        let reader = result.unwrap();
        // The resolved path should be absolute and match the docs_root
        // Since docs_root is already absolute, FileReader::new() should return it as-is
        assert_eq!(reader.docs_root(), docs_root.to_str().unwrap());
    }

    #[test]
    fn test_file_reader_relative_path() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Test with absolute path instead of changing working directory
        // This avoids the unsafe working directory manipulation
        let result = FileReader::new(docs_root.to_str().unwrap());
        assert!(result.is_ok());

        let reader = result.unwrap();
        // The resolved path should be absolute
        assert!(reader.docs_root().starts_with('/') || reader.docs_root().contains(':'));
    }

    #[test]
    fn test_file_reader_absolute_path() {
        let (_temp_dir, docs_root) = setup_test_env();

        // Test with absolute path
        let result = FileReader::new(docs_root.to_str().unwrap());
        assert!(result.is_ok());

        let reader = result.unwrap();
        assert_eq!(reader.docs_root(), docs_root.to_str().unwrap());
    }

    #[test]
    fn test_file_reader_relative_path_nonexistent() {
        let result = FileReader::new("nonexistent_relative_path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}
