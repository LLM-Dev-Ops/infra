//! HTTP server utilities.

use axum::Router as AxumRouter;
use infra_errors::{InfraError, InfraResult};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Server router wrapper
pub struct Router {
    inner: AxumRouter,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            inner: AxumRouter::new(),
        }
    }

    /// Merge with another router
    pub fn merge(mut self, other: Router) -> Self {
        self.inner = self.inner.merge(other.inner);
        self
    }

    /// Nest a router under a path
    pub fn nest(mut self, path: &str, other: Router) -> Self {
        self.inner = self.inner.nest(path, other.inner);
        self
    }

    /// Get the inner axum router
    pub fn into_inner(self) -> AxumRouter {
        self.inner
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl From<AxumRouter> for Router {
    fn from(router: AxumRouter) -> Self {
        Self { inner: router }
    }
}

/// Server builder
pub struct ServerBuilder {
    router: Router,
    addr: SocketAddr,
    enable_cors: bool,
    enable_tracing: bool,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new(router: Router) -> Self {
        Self {
            router,
            addr: SocketAddr::from(([0, 0, 0, 0], 3000)),
            enable_cors: true,
            enable_tracing: true,
        }
    }

    /// Set the address
    pub fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }

    /// Set the port
    pub fn port(mut self, port: u16) -> Self {
        self.addr.set_port(port);
        self
    }

    /// Enable/disable CORS
    pub fn cors(mut self, enabled: bool) -> Self {
        self.enable_cors = enabled;
        self
    }

    /// Enable/disable tracing
    pub fn tracing(mut self, enabled: bool) -> Self {
        self.enable_tracing = enabled;
        self
    }

    /// Build and run the server
    pub async fn serve(self) -> InfraResult<()> {
        let mut app = self.router.into_inner();

        if self.enable_cors {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);
            app = app.layer(cors);
        }

        if self.enable_tracing {
            app = app.layer(TraceLayer::new_for_http());
        }

        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .map_err(|e| InfraError::Http {
                status: None,
                message: format!("Failed to bind to {}: {}", self.addr, e),
                url: None,
                context: None,
            })?;

        axum::serve(listener, app)
            .await
            .map_err(|e| InfraError::Http {
                status: None,
                message: format!("Server error: {e}"),
                url: None,
                context: None,
            })
    }
}
