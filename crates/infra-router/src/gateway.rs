//! API gateway.

use crate::balancer::{Backend, LoadBalancer, Strategy};
use crate::handler::{Handler, HandlerResult, RequestContext};
use crate::route::{Method, Route, RouteBuilder};
use async_trait::async_trait;
use infra_errors::{InfraError, InfraResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Gateway name
    pub name: String,
    /// Request timeout
    pub timeout_ms: u64,
    /// Max request body size
    pub max_body_size: usize,
    /// Enable request logging
    pub logging: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            name: "gateway".to_string(),
            timeout_ms: 30000,
            max_body_size: 10 * 1024 * 1024, // 10MB
            logging: true,
        }
    }
}

/// API Gateway
pub struct Gateway {
    config: GatewayConfig,
    routes: Vec<Route>,
    middleware: Vec<Arc<dyn Handler>>,
    backends: HashMap<String, Arc<LoadBalancer>>,
}

impl Gateway {
    /// Create a new gateway
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            config,
            routes: Vec::new(),
            middleware: Vec::new(),
            backends: HashMap::new(),
        }
    }

    /// Add a route
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Add middleware
    pub fn add_middleware<H: Handler + 'static>(&mut self, handler: H) {
        self.middleware.push(Arc::new(handler));
    }

    /// Add a backend
    pub fn add_backend(&mut self, name: impl Into<String>, balancer: LoadBalancer) {
        self.backends.insert(name.into(), Arc::new(balancer));
    }

    /// Route a request
    pub async fn route(&self, method: Method, path: &str, ctx: RequestContext) -> InfraResult<HandlerResult> {
        // Find matching route
        for route in &self.routes {
            if let Some(params) = route.matches(method, path) {
                let mut route_ctx = ctx.clone();
                route_ctx.params = params;

                // Execute middleware
                for _mw in &self.middleware {
                    // In a real implementation, middleware could modify or short-circuit
                }

                // Execute handler
                if let Some(handler) = route.handler() {
                    return handler.handle(route_ctx).await;
                }
            }
        }

        Ok(HandlerResult::not_found())
    }

    /// Get a backend by name
    pub fn backend(&self, name: &str) -> Option<Arc<LoadBalancer>> {
        self.backends.get(name).cloned()
    }

    /// Get config
    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }
}

/// Gateway builder
pub struct GatewayBuilder {
    config: GatewayConfig,
    routes: Vec<Route>,
    middleware: Vec<Arc<dyn Handler>>,
    backends: HashMap<String, LoadBalancer>,
}

impl GatewayBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: GatewayConfig::default(),
            routes: Vec::new(),
            middleware: Vec::new(),
            backends: HashMap::new(),
        }
    }

    /// Set the name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set timeout
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.config.timeout_ms = ms;
        self
    }

    /// Set max body size
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.config.max_body_size = size;
        self
    }

    /// Enable/disable logging
    pub fn logging(mut self, enabled: bool) -> Self {
        self.config.logging = enabled;
        self
    }

    /// Add a route
    pub fn route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    /// Add middleware
    pub fn middleware<H: Handler + 'static>(mut self, handler: H) -> Self {
        self.middleware.push(Arc::new(handler));
        self
    }

    /// Add a backend
    pub fn backend(mut self, name: impl Into<String>, balancer: LoadBalancer) -> Self {
        self.backends.insert(name.into(), balancer);
        self
    }

    /// Build the gateway
    pub fn build(self) -> Gateway {
        let mut gateway = Gateway::new(self.config);
        gateway.routes = self.routes;
        gateway.middleware = self.middleware;

        for (name, balancer) in self.backends {
            gateway.backends.insert(name, Arc::new(balancer));
        }

        gateway
    }
}

impl Default for GatewayBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoHandler;

    #[async_trait]
    impl Handler for EchoHandler {
        async fn handle(&self, ctx: RequestContext) -> InfraResult<HandlerResult> {
            Ok(HandlerResult::ok(format!("Path: {}", ctx.path)))
        }
    }

    #[tokio::test]
    async fn test_gateway_routing() {
        let gateway = GatewayBuilder::new()
            .name("test-gateway")
            .route(
                RouteBuilder::new("/api/echo")
                    .get()
                    .handler(EchoHandler)
                    .build(),
            )
            .build();

        let ctx = RequestContext::new("/api/echo");
        let result = gateway.route(Method::Get, "/api/echo", ctx).await.unwrap();

        assert_eq!(result.status, 200);
    }

    #[tokio::test]
    async fn test_gateway_not_found() {
        let gateway = GatewayBuilder::new().build();

        let ctx = RequestContext::new("/unknown");
        let result = gateway.route(Method::Get, "/unknown", ctx).await.unwrap();

        assert_eq!(result.status, 404);
    }
}
