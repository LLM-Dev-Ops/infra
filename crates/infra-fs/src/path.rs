//! Path utilities.

use std::path::{Path, PathBuf};

/// Extension trait for paths
pub trait PathExt {
    /// Get the file extension as a string
    fn extension_str(&self) -> Option<&str>;

    /// Get the file name without extension
    fn stem_str(&self) -> Option<&str>;

    /// Check if the path has a specific extension
    fn has_extension(&self, ext: &str) -> bool;

    /// Ensure the path has a specific extension
    fn with_extension_if_missing(&self, ext: &str) -> PathBuf;
}

impl<P: AsRef<Path>> PathExt for P {
    fn extension_str(&self) -> Option<&str> {
        self.as_ref().extension().and_then(|e| e.to_str())
    }

    fn stem_str(&self) -> Option<&str> {
        self.as_ref().file_stem().and_then(|e| e.to_str())
    }

    fn has_extension(&self, ext: &str) -> bool {
        self.extension_str().map_or(false, |e| e.eq_ignore_ascii_case(ext))
    }

    fn with_extension_if_missing(&self, ext: &str) -> PathBuf {
        if self.has_extension(ext) {
            self.as_ref().to_path_buf()
        } else {
            self.as_ref().with_extension(ext)
        }
    }
}

/// Normalize a path (resolve . and ..)
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => components.push(c),
        }
    }

    components.iter().collect()
}

/// Join multiple paths
pub fn join_paths<I, P>(paths: I) -> PathBuf
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut result = PathBuf::new();
    for path in paths {
        result.push(path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_str() {
        let path = Path::new("file.txt");
        assert_eq!(path.extension_str(), Some("txt"));

        let path = Path::new("file");
        assert_eq!(path.extension_str(), None);
    }

    #[test]
    fn test_has_extension() {
        let path = Path::new("file.TXT");
        assert!(path.has_extension("txt"));
        assert!(path.has_extension("TXT"));
        assert!(!path.has_extension("json"));
    }

    #[test]
    fn test_normalize_path() {
        let path = normalize_path("a/b/../c/./d");
        assert_eq!(path, PathBuf::from("a/c/d"));
    }

    #[test]
    fn test_join_paths() {
        let result = join_paths(["a", "b", "c"]);
        assert_eq!(result, PathBuf::from("a/b/c"));
    }
}
