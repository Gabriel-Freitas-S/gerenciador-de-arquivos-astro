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
pub struct IdPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub id: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Validate)]
pub struct IdsPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(length(min = 1, message = "Lista de IDs não pode ser vazia"))]
    pub ids: Vec<i64>,
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

// ------------------------------ Departments ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentRecord {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DepartmentPayload {
    #[validate(length(min = 2, max = 200, message = "Nome deve ter entre 2 e 200 caracteres"))]
    pub name: String,
    #[validate(length(max = 50, message = "Código deve ter no máximo 50 caracteres"))]
    pub code: Option<String>,
    #[validate(length(max = 500, message = "Descrição deve ter no máximo 500 caracteres"))]
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DepartmentUpsertPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: DepartmentPayload,
}

// ------------------------------ Employees ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeRecord {
    pub id: i64,
    pub full_name: String,
    pub registration: String,
    pub cpf: Option<String>,
    pub department_id: Option<i64>,
    pub department_name: Option<String>,
    pub admission_date: String,
    pub termination_date: Option<String>,
    pub status: String,
    pub drawer_position_id: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeDetail {
    pub basic: EmployeeRecord,
    pub documents: Vec<DocumentRecord>,
    pub active_loans: Vec<LoanRecord>,
    pub drawer_position: Option<DrawerPositionRecord>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmployeePayload {
    #[validate(length(
        min = 3,
        max = 200,
        message = "Nome completo deve ter entre 3 e 200 caracteres"
    ))]
    pub full_name: String,
    #[validate(length(
        min = 3,
        max = 50,
        message = "Matrícula deve ter entre 3 e 50 caracteres"
    ))]
    pub registration: String,
    #[validate(length(min = 11, max = 14, message = "CPF inválido"))]
    pub cpf: Option<String>,
    pub department_id: Option<i64>,
    #[validate(length(min = 4, message = "Data de admissão é obrigatória"))]
    pub admission_date: String,
    pub termination_date: Option<String>,
    #[validate(length(min = 4, max = 20, message = "Status deve ter entre 4 e 20 caracteres"))]
    pub status: Option<String>,
    pub drawer_position_id: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmployeeCreatePayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: EmployeePayload,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmployeeUpdatePayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
    #[validate(nested)]
    pub data: EmployeePayload,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmployeeFilterPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub status: Option<String>,
    pub department_id: Option<i64>,
    #[allow(dead_code)]
    pub drawer_position_id: Option<i64>,
    #[validate(range(min = 1, max = 500, message = "Page size inválido"))]
    pub page_size: Option<i64>,
    #[validate(range(min = 1, message = "Página deve ser positiva"))]
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SearchPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(length(
        min = 2,
        max = 200,
        message = "Busca deve ter entre 2 e 200 caracteres"
    ))]
    pub query: String,
    #[validate(range(min = 1, max = 50, message = "Limite deve estar entre 1 e 50"))]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct TerminationPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
    #[validate(length(min = 4, message = "Data de demissão é obrigatória"))]
    pub termination_date: String,
    #[validate(length(max = 500, message = "Motivo deve ter no máximo 500 caracteres"))]
    pub reason: Option<String>,
    #[allow(dead_code)]
    pub transfer_to_box_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminationResult {
    pub employee: EmployeeRecord,
    pub archive_item: Option<ArchiveItemRecord>,
    pub label: Option<LabelData>,
}

// ------------------------------ File Cabinets ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCabinetRecord {
    pub id: i64,
    pub number: String,
    pub location: Option<String>,
    pub num_drawers: i64,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FileCabinetPayload {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Identificador deve ter entre 1 e 50 caracteres"
    ))]
    pub number: String,
    #[validate(length(max = 200, message = "Localização deve ter no máximo 200 caracteres"))]
    pub location: Option<String>,
    #[validate(range(
        min = 1,
        max = 20,
        message = "Número de gavetas deve ficar entre 1 e 20"
    ))]
    pub num_drawers: Option<i64>,
    #[validate(length(max = 500, message = "Descrição deve ter no máximo 500 caracteres"))]
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct FileCabinetCreatePayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: FileCabinetPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawerRecord {
    pub id: i64,
    pub file_cabinet_id: i64,
    pub number: i64,
    pub capacity: i64,
    pub label: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DrawerPayload {
    pub file_cabinet_id: i64,
    #[validate(range(
        min = 1,
        max = 50,
        message = "Número da gaveta deve ficar entre 1 e 50"
    ))]
    pub number: i64,
    #[validate(range(min = 1, max = 200, message = "Capacidade deve ficar entre 1 e 200"))]
    pub capacity: i64,
    #[validate(length(max = 100, message = "Label deve ter no máximo 100 caracteres"))]
    pub label: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DrawerCreatePayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: DrawerPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawerPositionRecord {
    pub id: i64,
    pub drawer_id: i64,
    pub position: i64,
    pub employee_id: Option<i64>,
    pub is_occupied: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DrawerAssignmentPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
    pub drawer_id: i64,
    #[validate(range(min = 1, max = 500, message = "Posição deve ficar entre 1 e 500"))]
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawerWithOccupancy {
    pub drawer: DrawerRecord,
    pub occupied: i64,
    pub capacity: i64,
    pub occupancy_rate: f32,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCabinetWithOccupancy {
    pub cabinet: FileCabinetRecord,
    pub drawers: Vec<DrawerWithOccupancy>,
    pub total_positions: i64,
    pub occupied_positions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupationMap {
    pub cabinets: Vec<CabinetOccupationNode>,
    pub totals: OccupationTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CabinetOccupationNode {
    pub cabinet_id: i64,
    pub cabinet_label: String,
    pub occupancy_rate: f32,
    pub status: String,
    pub drawers: Vec<DrawerWithOccupancy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupationTotals {
    pub total_positions: i64,
    pub occupied_positions: i64,
    pub warnings: i64,
    pub critical: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorganizationPlan {
    pub total_moves: usize,
    pub suggestions: Vec<ReorganizationSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorganizationSuggestion {
    pub employee_id: i64,
    pub employee_name: String,
    pub from_drawer: String,
    pub to_drawer: String,
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReorganizationRequestPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(range(
        min = 50,
        max = 100,
        message = "Limite crítico deve ficar entre 50 e 100"
    ))]
    pub critical_threshold: Option<i64>,
    #[validate(range(
        min = 1,
        max = 50,
        message = "Máximo de realocações deve ficar entre 1 e 50"
    ))]
    pub max_moves: Option<i64>,
}

// ------------------------------ Documents ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCategoryRecord {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTypeRecord {
    pub id: i64,
    pub category_id: i64,
    pub name: String,
    pub retention_years: i64,
    pub is_required: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: i64,
    pub employee_id: i64,
    pub category_id: i64,
    pub type_id: i64,
    pub description: Option<String>,
    pub document_date: Option<String>,
    pub filing_date: String,
    pub expiration_date: Option<String>,
    pub notes: Option<String>,
    pub filed_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EmployeeDocumentsPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DocumentPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
    pub category_id: i64,
    pub type_id: i64,
    #[validate(length(max = 500, message = "Descrição deve ter no máximo 500 caracteres"))]
    pub description: Option<String>,
    pub document_date: Option<String>,
    pub expiration_date: Option<String>,
    #[validate(length(max = 500, message = "Notas deve ter no máximo 500 caracteres"))]
    pub notes: Option<String>,
    #[validate(length(max = 100, message = "Responsável deve ter no máximo 100 caracteres"))]
    pub filed_by: Option<String>,
}

// ------------------------------ Loans ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanRecord {
    pub id: i64,
    pub employee_id: i64,
    pub requester_name: String,
    pub requester_department_id: Option<i64>,
    pub reason: String,
    pub loan_date: String,
    pub expected_return_date: String,
    pub actual_return_date: Option<String>,
    pub status: String,
    pub return_notes: Option<String>,
    pub loaned_by: String,
    pub returned_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanWithEmployee {
    pub loan: LoanRecord,
    pub employee: EmployeeRecord,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoanPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
    #[validate(length(
        min = 3,
        max = 200,
        message = "Solicitante deve ter entre 3 e 200 caracteres"
    ))]
    pub requester_name: String,
    pub requester_department_id: Option<i64>,
    #[validate(length(
        min = 5,
        max = 500,
        message = "Motivo deve ter entre 5 e 500 caracteres"
    ))]
    pub reason: String,
    #[validate(length(min = 4, message = "Data prevista é obrigatória"))]
    pub expected_return_date: String,
    #[validate(length(max = 500, message = "Observações deve ter no máximo 500 caracteres"))]
    pub return_notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoanReturnPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub loan_id: i64,
    pub actual_return_date: Option<String>,
    #[validate(length(max = 500, message = "Observações deve ter no máximo 500 caracteres"))]
    pub return_notes: Option<String>,
}

// ------------------------------ Dead Archive ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveBoxRecord {
    pub id: i64,
    pub box_number: String,
    pub year: i64,
    pub period: Option<String>,
    pub letter_range: Option<String>,
    pub location: Option<String>,
    pub capacity: i64,
    pub current_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ArchiveBoxPayload {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Identificador deve ter entre 1 e 50 caracteres"
    ))]
    pub box_number: String,
    #[validate(range(min = 1900, max = 3000, message = "Ano inválido"))]
    pub year: i64,
    #[validate(length(max = 50, message = "Período deve ter no máximo 50 caracteres"))]
    pub period: Option<String>,
    #[validate(length(max = 20, message = "Faixa deve ter no máximo 20 caracteres"))]
    pub letter_range: Option<String>,
    #[validate(length(max = 200, message = "Local deve ter no máximo 200 caracteres"))]
    pub location: Option<String>,
    #[validate(range(min = 1, max = 500, message = "Capacidade deve ficar entre 1 e 500"))]
    pub capacity: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ArchiveBoxCreatePayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(nested)]
    pub data: ArchiveBoxPayload,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxWithCount {
    #[serde(rename = "box")]
    pub r#box: ArchiveBoxRecord,
    pub occupants: Vec<ArchiveItemRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveItemRecord {
    pub id: i64,
    pub employee_id: i64,
    pub box_id: i64,
    pub transfer_date: String,
    pub disposal_eligible_date: Option<String>,
    pub disposed: bool,
    pub disposal_date: Option<String>,
    pub disposal_term_number: Option<String>,
    pub transferred_by: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ArchiveTransferPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub employee_id: i64,
    pub box_id: i64,
    pub disposal_eligible_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisposalCandidate {
    pub archive_item: ArchiveItemRecord,
    pub employee: EmployeeRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisposalTerm {
    pub term_number: String,
    pub generated_at: String,
    pub items: Vec<ArchiveItemRecord>,
    pub generated_by: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DisposalRegisterPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    #[validate(length(min = 1, message = "Selecione pelo menos um item"))]
    pub item_ids: Vec<i64>,
    #[validate(length(
        max = 100,
        message = "Número do termo deve ter no máximo 100 caracteres"
    ))]
    pub term_number: Option<String>,
}

// ------------------------------ Reports & Labels ------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub active_employees: i64,
    pub terminated_employees: i64,
    pub open_loans: i64,
    pub overdue_loans: i64,
    pub critical_cabinets: Vec<CabinetOccupationNode>,
    pub archive_boxes: i64,
    pub last_sync: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementsReport {
    pub total_movements: i64,
    pub by_action: HashMap<String, i64>,
    pub latest: Vec<MovementRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoansReport {
    pub total_loans: i64,
    pub open_loans: i64,
    pub overdue_loans: Vec<LoanWithEmployee>,
    pub returned_today: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExportResult {
    pub path: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelData {
    pub title: String,
    pub subtitle: Option<String>,
    pub details: HashMap<String, String>,
    pub generated_at: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LabelRequestPayload {
    #[validate(length(min = 1, message = "Token não pode ser vazio"))]
    pub token: String,
    pub entity_id: i64,
    #[validate(length(max = 20, message = "Formato deve ter no máximo 20 caracteres"))]
    pub format: Option<String>,
}
