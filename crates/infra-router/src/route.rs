//! Route definitions.

use crate::handler::Handler;
use crate::matcher::{MatchResult, PathMatcher};
use std::collections::HashMap;
use std::sync::Arc;

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Any,
}

impl Method {
    /// Check if this method matches another
    pub fn matches(&self, other: &Method) -> bool {
        *self == Method::Any || *self == *other
    }
}

/// A route definition
pub struct Route {
    /// Route path pattern
    path: String,
    /// HTTP method
    method: Method,
    /// Path matcher
    matcher: PathMatcher,
    /// Handler
    handler: Option<Arc<dyn Handler>>,
    /// Middleware
    middleware: Vec<Arc<dyn Handler>>,
    /// Route name
    name: Option<String>,
}

impl Route {
    /// Create a new route
    pub fn new(path: impl Into<String>) -> Self {
        let path = path.into();
        let matcher = PathMatcher::new(&path);
        Self {
            path,
            method: Method::Any,
            matcher,
            handler: None,
            middleware: Vec::new(),
            name: None,
        }
    }

    /// Get the path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the method
    pub fn method(&self) -> Method {
        self.method
    }

    /// Match a path
    pub fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        self.matcher.match_path(path)
    }

    /// Check if this route matches a request
    pub fn matches(&self, method: Method, path: &str) -> Option<HashMap<String, String>> {
        if !self.method.matches(&method) {
            return None;
        }
        self.match_path(path)
    }

    /// Get the handler
    pub fn handler(&self) -> Option<&Arc<dyn Handler>> {
        self.handler.as_ref()
    }
}

/// Route builder
pub struct RouteBuilder {
    route: Route,
}

impl RouteBuilder {
    /// Create a new builder
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            route: Route::new(path),
        }
    }

    /// Set the method
    pub fn method(mut self, method: Method) -> Self {
        self.route.method = method;
        self
    }

    /// Set as GET
    pub fn get(self) -> Self {
        self.method(Method::Get)
    }

    /// Set as POST
    pub fn post(self) -> Self {
        self.method(Method::Post)
    }

    /// Set as PUT
    pub fn put(self) -> Self {
        self.method(Method::Put)
    }

    /// Set as DELETE
    pub fn delete(self) -> Self {
        self.method(Method::Delete)
    }

    /// Set the handler
    pub fn handler<H: Handler + 'static>(mut self, handler: H) -> Self {
        self.route.handler = Some(Arc::new(handler));
        self
    }

    /// Add middleware
    pub fn middleware<H: Handler + 'static>(mut self, middleware: H) -> Self {
        self.route.middleware.push(Arc::new(middleware));
        self
    }

    /// Set the name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.route.name = Some(name.into());
        self
    }

    /// Build the route
    pub fn build(self) -> Route {
        self.route
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_matching() {
        let route = Route::new("/api/users/:id");
        let params = route.match_path("/api/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_route_method_matching() {
        let route = RouteBuilder::new("/api/users")
            .get()
            .build();

        assert!(route.matches(Method::Get, "/api/users").is_some());
        assert!(route.matches(Method::Post, "/api/users").is_none());
    }

    #[test]
    fn test_any_method() {
        let route = Route::new("/api/users");

        assert!(route.matches(Method::Get, "/api/users").is_some());
        assert!(route.matches(Method::Post, "/api/users").is_some());
    }
}
