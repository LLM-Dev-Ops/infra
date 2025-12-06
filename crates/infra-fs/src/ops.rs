//! Basic file operations.

use infra_errors::{InfraError, InfraResult, IoOperation};
use std::fs;
use std::path::Path;

/// Read a file as bytes
pub fn read(path: impl AsRef<Path>) -> InfraResult<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| InfraError::Io {
        operation: IoOperation::Read,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Read a file as a string
pub fn read_string(path: impl AsRef<Path>) -> InfraResult<String> {
    let path = path.as_ref();
    fs::read_to_string(path).map_err(|e| InfraError::Io {
        operation: IoOperation::Read,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Write bytes to a file
pub fn write(path: impl AsRef<Path>, contents: &[u8]) -> InfraResult<()> {
    let path = path.as_ref();

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| InfraError::Io {
                operation: IoOperation::Create,
                path: Some(parent.to_path_buf()),
                message: e.to_string(),
                context: None,
            })?;
        }
    }

    fs::write(path, contents).map_err(|e| InfraError::Io {
        operation: IoOperation::Write,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Append bytes to a file
pub fn append(path: impl AsRef<Path>, contents: &[u8]) -> InfraResult<()> {
    use std::io::Write;

    let path = path.as_ref();
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| InfraError::Io {
            operation: IoOperation::Write,
            path: Some(path.to_path_buf()),
            message: e.to_string(),
            context: None,
        })?;

    file.write_all(contents).map_err(|e| InfraError::Io {
        operation: IoOperation::Write,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Copy a file
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> InfraResult<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    // Create parent directories if needed
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| InfraError::Io {
                operation: IoOperation::Create,
                path: Some(parent.to_path_buf()),
                message: e.to_string(),
                context: None,
            })?;
        }
    }

    fs::copy(from, to).map_err(|e| InfraError::Io {
        operation: IoOperation::Copy,
        path: Some(from.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Remove a file or directory
pub fn remove(path: impl AsRef<Path>) -> InfraResult<()> {
    let path = path.as_ref();

    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|e| InfraError::Io {
        operation: IoOperation::Delete,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Check if a path exists
pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Create a directory
pub fn create_dir(path: impl AsRef<Path>) -> InfraResult<()> {
    let path = path.as_ref();
    fs::create_dir(path).map_err(|e| InfraError::Io {
        operation: IoOperation::Create,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

/// Create a directory and all parent directories
pub fn create_dir_all(path: impl AsRef<Path>) -> InfraResult<()> {
    let path = path.as_ref();
    fs::create_dir_all(path).map_err(|e| InfraError::Io {
        operation: IoOperation::Create,
        path: Some(path.to_path_buf()),
        message: e.to_string(),
        context: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temp::TempDir;

    #[test]
    fn test_read_write() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");

        write(&path, b"test content").unwrap();
        let content = read(&path).unwrap();

        assert_eq!(content, b"test content");
    }

    #[test]
    fn test_append() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");

        write(&path, b"hello").unwrap();
        append(&path, b" world").unwrap();
        let content = read_string(&path).unwrap();

        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_copy() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src.txt");
        let dst = temp.path().join("dst.txt");

        write(&src, b"copy me").unwrap();
        copy(&src, &dst).unwrap();

        assert!(exists(&dst));
        assert_eq!(read(&dst).unwrap(), b"copy me");
    }
}
