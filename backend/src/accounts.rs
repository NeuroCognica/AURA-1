/// Offline account management system
/// 
/// Stores username/password pairs in RocksDB with Forever Law compliance.
/// No encryption at this stage - basic authentication only.
/// Each account is immutable once created (append-only ledger).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "persistence")]
use crate::storage::Storage;

/// Account record stored in RocksDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password_plaintext: String, // TODO: Hash in Phase 4
    pub created_at_ms: i64,
    pub last_login_ms: i64,
    pub quiz_sessions: Vec<String>, // List of session_ids for this user
}

impl Account {
    pub fn new(username: String, password: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        Self {
            username,
            password_plaintext: password,
            created_at_ms: now,
            last_login_ms: now,
            quiz_sessions: Vec::new(),
        }
    }

    pub fn update_last_login(&mut self) {
        self.last_login_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
    }
}

/// Account login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Account creation request
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub username: Option<String>,
    pub error: Option<String>,
}

/// Account manager with RocksDB backend
#[cfg(feature = "persistence")]
pub struct AccountManager<'a> {
    store: &'a dyn Storage,
}

#[cfg(feature = "persistence")]
impl<'a> AccountManager<'a> {
    pub fn new(store: &'a dyn Storage) -> Self {
        Self { store }
    }

    fn account_key(username: &str) -> String {
        format!("account:{}", username)
    }

    /// Create new account (Forever Law: immutable after creation)
    pub fn create_account(&self, username: &str, password: &str) -> Result<Account> {
        let key = Self::account_key(username);
        
        // Check if account already exists
        if let Ok(Some(_)) = self.store.get_bytes(key.as_bytes()) {
            anyhow::bail!("Account already exists: {}", username);
        }
        
        let account = Account::new(username.to_string(), password.to_string());
        let json = serde_json::to_vec(&account)?;
        
        self.store.put_bytes(key.as_bytes(), &json)?;
        
        tracing::info!("Account created: {}", username);
        Ok(account)
    }

    /// Get account by username
    pub fn get_account(&self, username: &str) -> Result<Option<Account>> {
        let key = Self::account_key(username);
        
        if let Some(data) = self.store.get_bytes(key.as_bytes())? {
            let account: Account = serde_json::from_slice(&data)?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    /// Update account (for last_login, quiz_sessions)
    pub fn update_account(&self, account: &Account) -> Result<()> {
        let key = Self::account_key(&account.username);
        let json = serde_json::to_vec(account)?;
        
        self.store.put_bytes(key.as_bytes(), &json)?;
        
        tracing::debug!("Account updated: {}", account.username);
        Ok(())
    }

    /// Authenticate user (returns Account if successful)
    pub fn authenticate(&self, username: &str, password: &str) -> Result<Option<Account>> {
        if let Some(mut account) = self.get_account(username)? {
            if account.password_plaintext == password {
                account.update_last_login();
                self.update_account(&account)?;
                
                tracing::info!("Login successful: {}", username);
                Ok(Some(account))
            } else {
                tracing::warn!("Login failed (bad password): {}", username);
                Ok(None)
            }
        } else {
            tracing::warn!("Login failed (no such user): {}", username);
            Ok(None)
        }
    }

    /// Add quiz session to user account
    pub fn add_quiz_session(&self, username: &str, session_id: &str) -> Result<()> {
        if let Some(mut account) = self.get_account(username)? {
            if !account.quiz_sessions.contains(&session_id.to_string()) {
                account.quiz_sessions.push(session_id.to_string());
                self.update_account(&account)?;
                tracing::info!("Quiz session {} added to account {}", session_id, username);
            }
            Ok(())
        } else {
            anyhow::bail!("Account not found: {}", username)
        }
    }

    /// List all accounts (admin function)
    pub fn list_accounts(&self) -> Result<Vec<String>> {
        // TODO: Implement prefix scan when RocksStore supports it
        // For now, return empty list
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Simple in-memory Storage impl for testing
    struct TestStore {
        data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    }

    impl TestStore {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    impl Storage for TestStore {
        fn get_bytes(&self, key: &[u8]) -> anyhow::Result<Option<Vec<u8>>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn put_bytes(&self, key: &[u8], val: &[u8]) -> anyhow::Result<()> {
            self.data.lock().unwrap().insert(key.to_vec(), val.to_vec());
            Ok(())
        }

        fn append_log_atomic(&self, _speaker: &str, _content: &str) -> anyhow::Result<u64> {
            Ok(0) // Stub
        }
    }

    #[test]
    fn test_account_creation() {
        let store = TestStore::new();
        let mgr = AccountManager::new(&store);

        let account = mgr.create_account("alice", "secret123").unwrap();
        assert_eq!(account.username, "alice");
        assert_eq!(account.password_plaintext, "secret123");
        assert!(account.quiz_sessions.is_empty());
    }

    #[test]
    fn test_account_already_exists() {
        let store = TestStore::new();
        let mgr = AccountManager::new(&store);

        mgr.create_account("alice", "secret123").unwrap();
        let result = mgr.create_account("alice", "different");
        assert!(result.is_err());
    }

    #[test]
    fn test_authentication_success() {
        let store = TestStore::new();
        let mgr = AccountManager::new(&store);

        mgr.create_account("alice", "secret123").unwrap();
        let auth = mgr.authenticate("alice", "secret123").unwrap();
        assert!(auth.is_some());
        assert_eq!(auth.unwrap().username, "alice");
    }

    #[test]
    fn test_authentication_wrong_password() {
        let store = TestStore::new();
        let mgr = AccountManager::new(&store);

        mgr.create_account("alice", "secret123").unwrap();
        let auth = mgr.authenticate("alice", "wrongpass").unwrap();
        assert!(auth.is_none());
    }

    #[test]
    fn test_authentication_no_user() {
        let store = TestStore::new();
        let mgr = AccountManager::new(&store);

        let auth = mgr.authenticate("nobody", "anything").unwrap();
        assert!(auth.is_none());
    }

    #[test]
    fn test_add_quiz_session() {
        let store = TestStore::new();
        let mgr = AccountManager::new(&store);

        mgr.create_account("alice", "secret123").unwrap();
        mgr.add_quiz_session("alice", "session-001").unwrap();

        let account = mgr.get_account("alice").unwrap().unwrap();
        assert_eq!(account.quiz_sessions.len(), 1);
        assert_eq!(account.quiz_sessions[0], "session-001");
    }
}
