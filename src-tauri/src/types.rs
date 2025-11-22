use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUnitRecord {
    pub id: i64,
    pub label: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub section: Option<String>,
    pub capacity: i64,
    pub occupancy: i64,
    pub metadata: Option<Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementRecord {
    pub id: i64,
    pub reference: Option<String>,
    pub item_label: Option<String>,
    pub from_unit: Option<String>,
    pub to_unit: Option<String>,
    pub action: String,
    pub note: Option<String>,
    pub actor: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    pub total_units: i64,
    pub units_by_type: HashMap<String, i64>,
    pub movements_today: i64,
    pub last_movement: Option<MovementRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub token: String,
    pub profile: UserProfile,
    pub snapshot: SnapshotSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub name: String,
    pub login: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CredentialsPayload {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenPayload {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct StoragePayload {
    pub label: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub section: Option<String>,
    pub capacity: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct StorageCreatePayload {
    pub token: String,
    pub data: StoragePayload,
}

#[derive(Debug, Deserialize)]
pub struct MovementPayload {
    pub token: String,
    pub data: MovementData,
}

#[derive(Debug, Deserialize)]
pub struct MovementData {
    pub action: String,
    pub reference: Option<String>,
    pub item_label: Option<String>,
    pub from_unit: Option<String>,
    pub to_unit: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error<E: ToString>(error: E) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.to_string()),
        }
    }
}
