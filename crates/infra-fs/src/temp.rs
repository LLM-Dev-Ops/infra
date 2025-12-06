//! Temporary file and directory utilities.

use infra_errors::{InfraError, InfraResult, IoOperation};
use std::path::{Path, PathBuf};
use tempfile;

/// Temporary file that is deleted on drop
pub struct TempFile {
    path: PathBuf,
    _file: tempfile::NamedTempFile,
}

impl TempFile {
    /// Create a new temporary file
    pub fn new() -> InfraResult<Self> {
        let file = tempfile::NamedTempFile::new().map_err(|e| InfraError::Io {
            operation: IoOperation::Create,
            path: None,
            message: format!("Failed to create temp file: {e}"),
            context: None,
        })?;

        let path = file.path().to_path_buf();

        Ok(Self { path, _file: file })
    }

    /// Create a temporary file with a specific extension
    pub fn with_extension(ext: &str) -> InfraResult<Self> {
        let file = tempfile::Builder::new()
            .suffix(&format!(".{ext}"))
            .tempfile()
            .map_err(|e| InfraError::Io {
                operation: IoOperation::Create,
                path: None,
                message: format!("Failed to create temp file: {e}"),
                context: None,
            })?;

        let path = file.path().to_path_buf();

        Ok(Self { path, _file: file })
    }

    /// Get the path to the temporary file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write content to the temporary file
    pub fn write(&self, content: &[u8]) -> InfraResult<()> {
        std::fs::write(&self.path, content).map_err(|e| InfraError::Io {
            operation: IoOperation::Write,
            path: Some(self.path.clone()),
            message: e.to_string(),
            context: None,
        })
    }

    /// Read content from the temporary file
    pub fn read(&self) -> InfraResult<Vec<u8>> {
        std::fs::read(&self.path).map_err(|e| InfraError::Io {
            operation: IoOperation::Read,
            path: Some(self.path.clone()),
            message: e.to_string(),
            context: None,
        })
    }
}

/// Temporary directory that is deleted on drop
pub struct TempDir {
    path: PathBuf,
    _dir: tempfile::TempDir,
}

impl TempDir {
    /// Create a new temporary directory
    pub fn new() -> InfraResult<Self> {
        let dir = tempfile::tempdir().map_err(|e| InfraError::Io {
            operation: IoOperation::Create,
            path: None,
            message: format!("Failed to create temp directory: {e}"),
            context: None,
        })?;

        let path = dir.path().to_path_buf();

        Ok(Self { path, _dir: dir })
    }

    /// Create a temporary directory with a specific prefix
    pub fn with_prefix(prefix: &str) -> InfraResult<Self> {
        let dir = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .map_err(|e| InfraError::Io {
                operation: IoOperation::Create,
                path: None,
                message: format!("Failed to create temp directory: {e}"),
                context: None,
            })?;

        let path = dir.path().to_path_buf();

        Ok(Self { path, _dir: dir })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a file in the temporary directory
    pub fn create_file(&self, name: &str) -> InfraResult<PathBuf> {
        let path = self.path.join(name);
        std::fs::File::create(&path).map_err(|e| InfraError::Io {
            operation: IoOperation::Create,
            path: Some(path.clone()),
            message: e.to_string(),
            context: None,
        })?;
        Ok(path)
    }

    /// Create a subdirectory in the temporary directory
    pub fn create_dir(&self, name: &str) -> InfraResult<PathBuf> {
        let path = self.path.join(name);
        std::fs::create_dir(&path).map_err(|e| InfraError::Io {
            operation: IoOperation::Create,
            path: Some(path.clone()),
            message: e.to_string(),
            context: None,
        })?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_file() {
        let temp = TempFile::new().unwrap();
        temp.write(b"test content").unwrap();
        let content = temp.read().unwrap();
        assert_eq!(content, b"test content");
    }

    #[test]
    fn test_temp_file_with_extension() {
        let temp = TempFile::with_extension("json").unwrap();
        assert!(temp.path().to_string_lossy().ends_with(".json"));
    }

    #[test]
    fn test_temp_dir() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.create_file("test.txt").unwrap();
        assert!(file_path.exists());
    }

    #[test]
    fn test_temp_dir_cleanup() {
        let path = {
            let temp = TempDir::new().unwrap();
            temp.path().to_path_buf()
        };
        // Directory should be deleted after drop
        assert!(!path.exists());
    }
}
