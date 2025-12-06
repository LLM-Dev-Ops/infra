//! Session management.

use crate::identity::Identity;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use infra_errors::{AuthErrorKind, InfraError, InfraResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User identity
    pub identity: Identity,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Session data
    pub data: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session
    pub fn new(id: impl Into<String>, identity: Identity, duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            identity,
            created_at: now,
            last_activity: now,
            expires_at: now + duration,
            data: HashMap::new(),
        }
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Refresh the session
    pub fn refresh(&mut self, duration: Duration) {
        self.last_activity = Utc::now();
        self.expires_at = self.last_activity + duration;
    }

    /// Set session data
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.data.insert(key.into(), value.into());
    }

    /// Get session data
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Remove session data
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }
}

/// Session store trait
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a session
    async fn create(&self, session: Session) -> InfraResult<()>;

    /// Get a session by ID
    async fn get(&self, id: &str) -> InfraResult<Option<Session>>;

    /// Update a session
    async fn update(&self, session: Session) -> InfraResult<()>;

    /// Delete a session
    async fn delete(&self, id: &str) -> InfraResult<()>;

    /// Clean up expired sessions
    async fn cleanup(&self) -> InfraResult<usize>;
}

/// In-memory session store
pub struct MemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl MemorySessionStore {
    /// Create a new memory session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for MemorySessionStore {
    async fn create(&self, session: Session) -> InfraResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn get(&self, id: &str) -> InfraResult<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn update(&self, session: Session) -> InfraResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    async fn delete(&self, id: &str) -> InfraResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }

    async fn cleanup(&self) -> InfraResult<usize> {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let before = sessions.len();

        sessions.retain(|_, session| !session.is_expired());

        Ok(before - sessions.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_session_store() {
        let store = MemorySessionStore::new();
        let identity = Identity::user("user123");
        let session = Session::new("sess123", identity, Duration::hours(1));

        store.create(session).await.unwrap();

        let retrieved = store.get("sess123").await.unwrap().unwrap();
        assert_eq!(retrieved.identity.id, "user123");

        store.delete("sess123").await.unwrap();
        assert!(store.get("sess123").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let store = MemorySessionStore::new();
        let identity = Identity::user("user123");

        // Create an expired session
        let mut session = Session::new("expired", identity.clone(), Duration::hours(-1));
        session.expires_at = Utc::now() - Duration::hours(1);
        store.create(session).await.unwrap();

        // Create a valid session
        let session = Session::new("valid", identity, Duration::hours(1));
        store.create(session).await.unwrap();

        let cleaned = store.cleanup().await.unwrap();
        assert_eq!(cleaned, 1);

        assert!(store.get("valid").await.unwrap().is_some());
        assert!(store.get("expired").await.unwrap().is_none());
    }
}
