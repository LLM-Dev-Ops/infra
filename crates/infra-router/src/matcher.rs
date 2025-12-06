//! Path matching.

use regex::Regex;
use std::collections::HashMap;

/// Match result containing extracted parameters
pub type MatchResult = HashMap<String, String>;

/// Path matcher
pub struct PathMatcher {
    /// Original pattern
    pattern: String,
    /// Compiled regex
    regex: Regex,
    /// Parameter names in order
    params: Vec<String>,
}

impl PathMatcher {
    /// Create a new path matcher
    pub fn new(pattern: &str) -> Self {
        let mut regex_pattern = String::from("^");
        let mut params = Vec::new();

        for segment in pattern.split('/') {
            if segment.is_empty() {
                continue;
            }

            regex_pattern.push('/');

            if let Some(param) = segment.strip_prefix(':') {
                // Named parameter
                params.push(param.to_string());
                regex_pattern.push_str("([^/]+)");
            } else if segment == "*" {
                // Wildcard
                regex_pattern.push_str("(.*)");
            } else {
                // Literal segment
                regex_pattern.push_str(&regex::escape(segment));
            }
        }

        regex_pattern.push_str("/?$");

        let regex = Regex::new(&regex_pattern).unwrap_or_else(|_| Regex::new("^$").unwrap());

        Self {
            pattern: pattern.to_string(),
            regex,
            params,
        }
    }

    /// Match a path and extract parameters
    pub fn match_path(&self, path: &str) -> Option<MatchResult> {
        self.regex.captures(path).map(|caps| {
            let mut params = HashMap::new();

            for (i, name) in self.params.iter().enumerate() {
                if let Some(value) = caps.get(i + 1) {
                    params.insert(name.clone(), value.as_str().to_string());
                }
            }

            params
        })
    }

    /// Check if a path matches
    pub fn is_match(&self, path: &str) -> bool {
        self.regex.is_match(path)
    }

    /// Get the pattern
    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let matcher = PathMatcher::new("/api/users");
        assert!(matcher.is_match("/api/users"));
        assert!(matcher.is_match("/api/users/"));
        assert!(!matcher.is_match("/api/posts"));
    }

    #[test]
    fn test_parameter_extraction() {
        let matcher = PathMatcher::new("/api/users/:id");
        let params = matcher.match_path("/api/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_multiple_parameters() {
        let matcher = PathMatcher::new("/api/:resource/:id/comments/:comment_id");
        let params = matcher
            .match_path("/api/posts/456/comments/789")
            .unwrap();

        assert_eq!(params.get("resource"), Some(&"posts".to_string()));
        assert_eq!(params.get("id"), Some(&"456".to_string()));
        assert_eq!(params.get("comment_id"), Some(&"789".to_string()));
    }

    #[test]
    fn test_no_match() {
        let matcher = PathMatcher::new("/api/users/:id");
        assert!(matcher.match_path("/api/posts/123").is_none());
    }
}
