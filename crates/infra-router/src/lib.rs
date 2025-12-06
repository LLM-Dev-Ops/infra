//! API routing and gateway for LLM-Dev-Ops infrastructure.
//!
//! This crate provides routing, load balancing, and API gateway
//! functionality.

mod route;
mod matcher;
mod handler;
mod gateway;
mod balancer;

pub use route::{Route, RouteBuilder};
pub use matcher::{PathMatcher, MatchResult};
pub use handler::{Handler, HandlerFn, HandlerResult};
pub use gateway::{Gateway, GatewayConfig, GatewayBuilder};
pub use balancer::{LoadBalancer, Backend, Strategy};

use infra_errors::InfraResult;

/// Create a new gateway builder
pub fn gateway() -> GatewayBuilder {
    GatewayBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_matching() {
        let route = Route::new("/api/users/:id");
        let result = route.match_path("/api/users/123");

        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }
}
