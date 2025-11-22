use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use validator::Validate;

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

#[derive(Debug, Deserialize, Validate)]
pub struct CredentialsPayload {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Login deve ter entre 3 e 100 caracteres"
    ))]
    pub login: String,
    #[validate(length(
        min = 4,
        max = 100,
        message = "Senha deve ter entre 4 e 100 caracteres"
    ))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TokenPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StoragePayload {
    #[validate(length(
        min = 2,
        max = 200,
        message = "Label deve ter entre 2 e 200 caracteres"
    ))]
    pub label: String,
    #[serde(rename = "type")]
    #[validate(length(min = 1, max = 50, message = "Tipo não pode ser vazio"))]
    pub r#type: String,
    #[validate(length(max = 200, message = "Seção deve ter no máximo 200 caracteres"))]
    pub section: Option<String>,
    #[validate(range(min = 0, message = "Capacidade deve ser maior ou igual a 0"))]
    pub capacity: Option<i64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StorageCreatePayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: StoragePayload,
}

#[derive(Debug, Deserialize, Validate)]
pub struct MovementPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: MovementData,
}

#[derive(Debug, Deserialize, Validate)]
pub struct MovementData {
    #[validate(length(min = 3, max = 500, message = "Ação deve ter entre 3 e 500 caracteres"))]
    pub action: String,
    #[validate(length(max = 200, message = "Referência deve ter no máximo 200 caracteres"))]
    pub reference: Option<String>,
    #[validate(length(max = 200, message = "Label do item deve ter no máximo 200 caracteres"))]
    pub item_label: Option<String>,
    #[validate(length(
        max = 200,
        message = "Unidade de origem deve ter no máximo 200 caracteres"
    ))]
    pub from_unit: Option<String>,
    #[validate(length(
        max = 200,
        message = "Unidade de destino deve ter no máximo 200 caracteres"
    ))]
    pub to_unit: Option<String>,
    #[validate(length(max = 1000, message = "Nota deve ter no máximo 1000 caracteres"))]
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
