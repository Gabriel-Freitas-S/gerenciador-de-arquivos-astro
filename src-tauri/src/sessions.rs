use std::collections::HashMap;

use chrono::Utc;
use std::sync::RwLock;
use uuid::Uuid;

use crate::types::UserProfile;

#[derive(Clone)]
pub struct ActiveSession {
    pub token: String,
    pub profile: UserProfile,
    #[allow(dead_code)]
    pub(crate) issued_at: i64,
}

#[derive(Default)]
pub struct SessionStore {
    sessions: RwLock<HashMap<String, ActiveSession>>,
}

impl SessionStore {
    pub fn create(&self, profile: UserProfile) -> ActiveSession {
        let token = Uuid::new_v4().to_string();
        let session = ActiveSession {
            token: token.clone(),
            profile,
            issued_at: Utc::now().timestamp_millis(),
        };
        // Unwrap is safe here as we are not handling lock poisoning in this simple app
        self.sessions
            .write()
            .unwrap()
            .insert(token.clone(), session.clone());
        session
    }

    pub fn get(&self, token: &str) -> Option<ActiveSession> {
        self.sessions.read().unwrap().get(token).cloned()
    }

    pub fn require(&self, token: &str) -> Result<ActiveSession, &'static str> {
        self.get(token)
            .ok_or("Sessão inválida. Faça login novamente.")
    }

    pub fn revoke(&self, token: &str) {
        self.sessions.write().unwrap().remove(token);
    }
}
