//! File system utilities for LLM-Dev-Ops infrastructure.
//!
//! This crate provides unified file operations with proper error handling
//! and optional async support.

mod ops;
mod path;
mod temp;

#[cfg(feature = "watch")]
mod watch;

pub use ops::{read, read_string, write, append, copy, remove, exists, create_dir, create_dir_all};
pub use path::{PathExt, normalize_path, join_paths};
pub use temp::{TempFile, TempDir};

#[cfg(feature = "watch")]
pub use watch::{FileWatcher, WatchEvent};

#[cfg(feature = "async")]
pub mod async_ops;

use infra_errors::InfraResult;
use std::path::Path;

/// Read a file as bytes
pub fn read_bytes(path: impl AsRef<Path>) -> InfraResult<Vec<u8>> {
    ops::read(path)
}

/// Read a file as a string
pub fn read_text(path: impl AsRef<Path>) -> InfraResult<String> {
    ops::read_string(path)
}

/// Read and parse JSON from a file
pub fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> InfraResult<T> {
    let content = read_text(path)?;
    serde_json::from_str(&content).map_err(|e| infra_errors::InfraError::Serialization {
        format: infra_errors::SerializationFormat::Json,
        message: e.to_string(),
        location: None,
        context: None,
    })
}

/// Write JSON to a file
pub fn write_json<T: serde::Serialize>(path: impl AsRef<Path>, data: &T) -> InfraResult<()> {
    let content = serde_json::to_string_pretty(data).map_err(|e| {
        infra_errors::InfraError::Serialization {
            format: infra_errors::SerializationFormat::Json,
            message: e.to_string(),
            location: None,
            context: None,
        }
    })?;
    write(path, content.as_bytes())
}

/// List files in a directory matching a glob pattern
pub fn glob_files(pattern: &str) -> InfraResult<Vec<std::path::PathBuf>> {
    glob::glob(pattern)
        .map_err(|e| infra_errors::InfraError::Io {
            operation: infra_errors::IoOperation::Read,
            path: Some(std::path::PathBuf::from(pattern)),
            message: e.to_string(),
            context: None,
        })?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>()
        .pipe(Ok)
}

/// Walk a directory tree
pub fn walk_dir(path: impl AsRef<Path>) -> InfraResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(path.as_ref()) {
        let entry = entry.map_err(|e| infra_errors::InfraError::Io {
            operation: infra_errors::IoOperation::Read,
            path: Some(path.as_ref().to_path_buf()),
            message: e.to_string(),
            context: None,
        })?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_roundtrip() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");

        write(&path, b"Hello, World!").unwrap();
        let content = read_string(&path).unwrap();

        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_json_roundtrip() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.json");

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        write_json(&path, &data).unwrap();
        let loaded: TestData = read_json(&path).unwrap();

        assert_eq!(loaded, data);
    }
}
