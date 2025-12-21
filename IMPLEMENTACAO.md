# Sistema de Gerenciamento de Arquivos Físicos - Implementação Completa

Sistema para gestão de arquivos de funcionários em hospital, com controle de
gaveteiros, empréstimos e arquivo morto.

---

## 📊 Sumário de Mudanças

| Categoria        | Novos Arquivos   | Arquivos Modificados |
| ---------------- | ---------------- | -------------------- |
| Backend (Rust)   | 6                | 3                    |
| Frontend (Astro) | 15               | 2                    |
| Database         | 10 tabelas novas | -                    |

---

## 🗄️ Banco de Dados - Novas Tabelas

### Tabela: `departments`

```sql
CREATE TABLE departments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    code TEXT,
    description TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Tabela: `employees`

```sql
CREATE TABLE employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL,
    registration TEXT NOT NULL UNIQUE,  -- Matrícula
    cpf TEXT UNIQUE,
    department_id INTEGER REFERENCES departments(id),
    admission_date TEXT NOT NULL,
    termination_date TEXT,
    status TEXT DEFAULT 'ACTIVE',  -- ACTIVE, TERMINATED
    drawer_position_id INTEGER REFERENCES drawer_positions(id),
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_employees_registration ON employees(registration);
CREATE INDEX idx_employees_status ON employees(status);
CREATE INDEX idx_employees_name ON employees(full_name);
```

### Tabela: `file_cabinets` (Gaveteiros)

```sql
CREATE TABLE file_cabinets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number TEXT NOT NULL UNIQUE,
    location TEXT,  -- Sala, corredor
    num_drawers INTEGER NOT NULL DEFAULT 4,
    description TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Tabela: `drawers` (Gavetas)

```sql
CREATE TABLE drawers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_cabinet_id INTEGER NOT NULL REFERENCES file_cabinets(id),
    number INTEGER NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 30,
    label TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(file_cabinet_id, number)
);
```

### Tabela: `drawer_positions` (Posições)

```sql
CREATE TABLE drawer_positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    drawer_id INTEGER NOT NULL REFERENCES drawers(id),
    position INTEGER NOT NULL,
    employee_id INTEGER REFERENCES employees(id),
    is_occupied INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(drawer_id, position)
);
```

### Tabela: `document_categories`

```sql
CREATE TABLE document_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    code TEXT NOT NULL,  -- PESSOAL, MEDICINA, SEGURANCA, TREINAMENTO
    description TEXT,
    icon TEXT,
    color TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Dados iniciais
INSERT INTO document_categories (name, code, description) VALUES
('Pessoal', 'PESSOAL', 'Documentos pessoais, contratos, admissão'),
('Medicina do Trabalho', 'MEDICINA', 'Exames, ASOs, atestados'),
('Segurança do Trabalho', 'SEGURANCA', 'EPIs, treinamentos de segurança'),
('Treinamento', 'TREINAMENTO', 'Certificados, capacitações');
```

### Tabela: `document_types`

```sql
CREATE TABLE document_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL REFERENCES document_categories(id),
    name TEXT NOT NULL,
    retention_years INTEGER DEFAULT 5,  -- Prazo de guarda
    is_required INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(category_id, name)
);

-- Dados iniciais
INSERT INTO document_types (category_id, name, retention_years) VALUES
(1, 'Contrato de Trabalho', 10),
(1, 'RG', 5),
(1, 'CPF', 5),
(1, 'Comprovante de Residência', 2),
(1, 'Certidão de Nascimento/Casamento', 5),
(2, 'ASO Admissional', 20),
(2, 'ASO Periódico', 20),
(2, 'ASO Demissional', 20),
(2, 'Atestado Médico', 5),
(3, 'Ficha de EPI', 5),
(3, 'Treinamento NR', 5),
(4, 'Certificado de Curso', 5);
```

### Tabela: `documents`

```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL REFERENCES employees(id),
    category_id INTEGER NOT NULL REFERENCES document_categories(id),
    type_id INTEGER NOT NULL REFERENCES document_types(id),
    description TEXT,
    document_date TEXT,
    filing_date TEXT DEFAULT CURRENT_TIMESTAMP,
    expiration_date TEXT,
    notes TEXT,
    filed_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_documents_employee ON documents(employee_id);
```

### Tabela: `loans` (Empréstimos)

```sql
CREATE TABLE loans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL REFERENCES employees(id),
    requester_name TEXT NOT NULL,
    requester_department_id INTEGER REFERENCES departments(id),
    reason TEXT NOT NULL,
    loan_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expected_return_date TEXT NOT NULL,
    actual_return_date TEXT,
    status TEXT DEFAULT 'BORROWED',  -- BORROWED, RETURNED
    return_notes TEXT,
    loaned_by TEXT NOT NULL,
    returned_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_loans_status ON loans(status);
CREATE INDEX idx_loans_employee ON loans(employee_id);
```

### Tabela: `dead_archive_boxes` (Caixas Arquivo Morto)

```sql
CREATE TABLE dead_archive_boxes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    box_number TEXT NOT NULL UNIQUE,
    year INTEGER NOT NULL,
    period TEXT,  -- Ex: "Jan-Jun 2024"
    letter_range TEXT,  -- Ex: "A-F"
    location TEXT,  -- Prateleira, estante
    capacity INTEGER DEFAULT 50,
    current_count INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### Tabela: `dead_archive_items`

```sql
CREATE TABLE dead_archive_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_id INTEGER NOT NULL REFERENCES employees(id),
    box_id INTEGER NOT NULL REFERENCES dead_archive_boxes(id),
    transfer_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    disposal_eligible_date TEXT,  -- 5 anos após demissão
    disposed INTEGER DEFAULT 0,
    disposal_date TEXT,
    disposal_term_number TEXT,
    transferred_by TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_dead_archive_employee ON dead_archive_items(employee_id);
```

### Tabela: `audit_logs`

```sql
CREATE TABLE audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id),
    action TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id INTEGER,
    old_values TEXT,  -- JSON
    new_values TEXT,  -- JSON
    ip_address TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_audit_created ON audit_logs(created_at);
```

---

## 🦀 Backend Rust - Novos Comandos

### Arquivo: `src-tauri/src/commands/employees.rs` [NOVO]

```rust
// Comandos a implementar:

#[tauri::command]
pub async fn create_employee(
    state: State<'_, AppState>,
    payload: EmployeeCreatePayload,
) -> Result<ApiResponse<EmployeeRecord>, ()>

#[tauri::command]
pub async fn update_employee(
    state: State<'_, AppState>,
    payload: EmployeeUpdatePayload,
) -> Result<ApiResponse<EmployeeRecord>, ()>

#[tauri::command]
pub async fn terminate_employee(
    state: State<'_, AppState>,
    payload: TerminationPayload,
) -> Result<ApiResponse<TerminationResult>, ()>
// → Registra data demissão
// → Move automaticamente para arquivo morto
// → Libera posição no gaveteiro
// → Gera dados para etiqueta

#[tauri::command]
pub async fn list_employees(
    state: State<'_, AppState>,
    payload: EmployeeFilterPayload,
) -> Result<ApiResponse<Vec<EmployeeRecord>>, ()>

#[tauri::command]
pub async fn search_employees(
    state: State<'_, AppState>,
    payload: SearchPayload,
) -> Result<ApiResponse<Vec<EmployeeRecord>>, ()>

#[tauri::command]
pub async fn get_employee(
    state: State<'_, AppState>,
    payload: IdPayload,
) -> Result<ApiResponse<EmployeeDetail>, ()>
```

### Arquivo: `src-tauri/src/commands/departments.rs` [NOVO]

```rust
#[tauri::command]
pub async fn list_departments(...) -> Result<ApiResponse<Vec<DepartmentRecord>>, ()>

#[tauri::command]
pub async fn create_department(...) -> Result<ApiResponse<DepartmentRecord>, ()>

#[tauri::command]
pub async fn update_department(...) -> Result<ApiResponse<DepartmentRecord>, ()>
```

### Arquivo: `src-tauri/src/commands/file_cabinets.rs` [NOVO]

```rust
#[tauri::command]
pub async fn create_file_cabinet(...) -> Result<ApiResponse<FileCabinetRecord>, ()>

#[tauri::command]
pub async fn create_drawer(...) -> Result<ApiResponse<DrawerRecord>, ()>

#[tauri::command]
pub async fn list_file_cabinets(...) -> Result<ApiResponse<Vec<FileCabinetWithOccupancy>>, ()>

#[tauri::command]
pub async fn get_occupation_map(...) -> Result<ApiResponse<OccupationMap>, ()>
// → Retorna estrutura visual com indicadores de capacidade

#[tauri::command]
pub async fn assign_employee_position(...) -> Result<ApiResponse<DrawerPositionRecord>, ()>

#[tauri::command]
pub async fn suggest_reorganization(...) -> Result<ApiResponse<ReorganizationPlan>, ()>
// → Algoritmo que identifica espaços vazios
// → Sugere realocações mantendo ordem alfabética
```

### Arquivo: `src-tauri/src/commands/documents.rs` [NOVO]

```rust
#[tauri::command]
pub async fn list_document_categories(...) -> Result<ApiResponse<Vec<CategoryRecord>>, ()>

#[tauri::command]
pub async fn list_document_types(...) -> Result<ApiResponse<Vec<TypeRecord>>, ()>

#[tauri::command]
pub async fn create_document(...) -> Result<ApiResponse<DocumentRecord>, ()>

#[tauri::command]
pub async fn list_employee_documents(...) -> Result<ApiResponse<Vec<DocumentRecord>>, ()>
```

### Arquivo: `src-tauri/src/commands/loans.rs` [NOVO]

```rust
#[tauri::command]
pub async fn create_loan(...) -> Result<ApiResponse<LoanRecord>, ()>
// → Registra empréstimo
// → Define previsão de devolução

#[tauri::command]
pub async fn return_loan(...) -> Result<ApiResponse<LoanRecord>, ()>
// → Atualiza data devolução
// → Muda status para RETURNED

#[tauri::command]
pub async fn list_loans(...) -> Result<ApiResponse<Vec<LoanRecord>>, ()>

#[tauri::command]
pub async fn get_pending_loans(...) -> Result<ApiResponse<Vec<LoanRecord>>, ()>

#[tauri::command]
pub async fn get_overdue_loans(...) -> Result<ApiResponse<Vec<LoanWithEmployee>>, ()>
// → Retorna empréstimos vencidos para alertas
```

### Arquivo: `src-tauri/src/commands/dead_archive.rs` [NOVO]

```rust
#[tauri::command]
pub async fn create_archive_box(...) -> Result<ApiResponse<BoxRecord>, ()>

#[tauri::command]
pub async fn list_archive_boxes(...) -> Result<ApiResponse<Vec<BoxWithCount>>, ()>

#[tauri::command]
pub async fn transfer_to_archive(...) -> Result<ApiResponse<ArchiveItemRecord>, ()>
// → Chamado automaticamente ao demitir

#[tauri::command]
pub async fn get_disposal_candidates(...) -> Result<ApiResponse<Vec<DisposalCandidate>>, ()>
// → Lista documentos com 5+ anos após demissão

#[tauri::command]
pub async fn register_disposal(...) -> Result<ApiResponse<DisposalTerm>, ()>
// → Gera termo de descarte
```

### Arquivo: `src-tauri/src/commands/reports.rs` [NOVO]

```rust
#[tauri::command]
pub async fn get_dashboard_stats(...) -> Result<ApiResponse<DashboardStats>, ()>
// → Total funcionários ativos/demitidos
// → Pastas emprestadas
// → Alertas de capacidade crítica

#[tauri::command]
pub async fn get_movements_report(...) -> Result<ApiResponse<MovementsReport>, ()>

#[tauri::command]
pub async fn get_loans_report(...) -> Result<ApiResponse<LoansReport>, ()>

#[tauri::command]
pub async fn export_to_excel(...) -> Result<ApiResponse<FilePath>, ()>
```

### Arquivo: `src-tauri/src/commands/labels.rs` [NOVO]

```rust
#[tauri::command]
pub async fn generate_folder_label(...) -> Result<ApiResponse<LabelData>, ()>
// → Nome, matrícula, localização física

#[tauri::command]
pub async fn generate_envelope_label(...) -> Result<ApiResponse<LabelData>, ()>

#[tauri::command]
pub async fn generate_box_label(...) -> Result<ApiResponse<LabelData>, ()>
// → Ano, período, funcionários A-Z
```

---

## 🎨 Frontend Astro - Novas Páginas

### Estrutura de Pastas

```
src/
├── pages/
│   ├── index.astro                 [MODIFICAR] Dashboard principal
│   ├── employees/
│   │   ├── index.astro             [NOVO] Lista de funcionários
│   │   ├── [id].astro              [NOVO] Detalhes do funcionário
│   │   └── form.astro              [NOVO] Cadastro/Edição
│   ├── cabinets/
│   │   ├── index.astro             [NOVO] Mapa de gaveteiros
│   │   └── [id].astro              [NOVO] Detalhes do gaveteiro
│   ├── documents/
│   │   ├── index.astro             [NOVO] Gestão de documentos
│   │   └── add.astro               [NOVO] Adicionar documento
│   ├── loans/
│   │   └── index.astro             [NOVO] Controle de empréstimos
│   ├── archive/
│   │   └── index.astro             [NOVO] Arquivo morto
│   ├── reports/
│   │   └── index.astro             [NOVO] Central de relatórios
│   ├── tools/
│   │   └── labels.astro            [NOVO] Gerador de etiquetas
│   └── settings/
│       ├── index.astro             [NOVO] Configurações gerais
│       ├── users.astro             [NOVO] Gestão de usuários
│       └── document-types.astro    [NOVO] Tipos de documentos
├── components/
│   └── app/
│       ├── Sidebar.astro           [NOVO] Menu lateral
│       ├── DashboardCards.astro    [NOVO] Cards de estatísticas
│       ├── Timeline.astro          [NOVO] Timeline de movimentações
│       ├── AlertBanner.astro       [NOVO] Alertas de capacidade
│       ├── OccupationMap.astro     [NOVO] Mapa visual de gaveteiros
│       ├── EmployeeTable.astro     [NOVO] Tabela de funcionários
│       ├── LoanTable.astro         [NOVO] Tabela de empréstimos
│       ├── Modal.astro             [NOVO] Modal reutilizável
│       └── Pagination.astro        [NOVO] Paginação
└── scripts/
    └── modules/
        ├── employees-api.ts        [NOVO]
        ├── cabinets-api.ts         [NOVO]
        ├── documents-api.ts        [NOVO]
        ├── loans-api.ts            [NOVO]
        ├── archive-api.ts          [NOVO]
        └── reports-api.ts          [NOVO]
```

---

## 🔧 Modificações em Arquivos Existentes

### `src-tauri/src/db.rs`

Adicionar migrações para todas as novas tabelas na constante `MIGRATIONS`.

### `src-tauri/src/types.rs`

Adicionar novos tipos:

- `EmployeeRecord`, `EmployeePayload`, `EmployeeFilter`
- `DepartmentRecord`, `DepartmentPayload`
- `FileCabinetRecord`, `DrawerRecord`, `DrawerPositionRecord`
- `DocumentCategoryRecord`, `DocumentTypeRecord`, `DocumentRecord`
- `LoanRecord`, `LoanPayload`
- `ArchiveBoxRecord`, `ArchiveItemRecord`
- `DashboardStats`, `LabelData`

### `src-tauri/src/commands/mod.rs`

Adicionar exports dos novos módulos:

```rust
pub mod employees;
pub mod departments;
pub mod file_cabinets;
pub mod documents;
pub mod loans;
pub mod dead_archive;
pub mod reports;
pub mod labels;
```

### `src-tauri/src/main.rs`

Registrar novos comandos no `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // Existentes...
    // Novos:
    commands::employees::create_employee,
    commands::employees::update_employee,
    commands::employees::terminate_employee,
    commands::employees::list_employees,
    commands::employees::search_employees,
    // ... todos os outros
])
```

### `src/pages/index.astro`

Transformar em Dashboard principal com:

- Cards de estatísticas
- Alertas de capacidade crítica
- Timeline de movimentações
- Links rápidos para funções principais

---

## 📦 Componentes Visuais

### Mapa de Ocupação dos Gaveteiros

```
┌─────────────────────────────────────────────┐
│  Gaveteiro 01 - Sala Arquivos               │
├─────────────────────────────────────────────┤
│  Gaveta 1  🟢 [████████░░] 24/30            │
│  Gaveta 2  🟡 [██████████] 27/30            │
│  Gaveta 3  🔴 [██████████] 29/30            │
│  Gaveta 4  🟢 [██████░░░░] 18/30            │
└─────────────────────────────────────────────┘

Legenda:
🟢 Verde: < 70% ocupado
🟡 Amarelo: 70-90% ocupado
🔴 Vermelho: > 90% ocupado
```

### Cards do Dashboard

```
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ 👥 1.234     │ │ 📁 45        │ │ 🔴 3         │ │ 📦 89        │
│ Funcionários │ │ Empréstimos  │ │ Atrasados    │ │ Arquivo Morto│
│ Ativos       │ │ em Aberto    │ │              │ │              │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
```

---

## 🔐 Níveis de Permissão

| Ação                  | Admin | Operador | Consulta |
| --------------------- | ----- | -------- | -------- |
| Visualizar dados      | ✅    | ✅       | ✅       |
| Cadastrar funcionário | ✅    | ✅       | ❌       |
| Demitir funcionário   | ✅    | ❌       | ❌       |
| Registrar empréstimo  | ✅    | ✅       | ❌       |
| Devolver empréstimo   | ✅    | ✅       | ❌       |
| Gerenciar gaveteiros  | ✅    | ❌       | ❌       |
| Configurar tipos doc  | ✅    | ❌       | ❌       |
| Gerenciar usuários    | ✅    | ❌       | ❌       |
| Exportar relatórios   | ✅    | ✅       | ✅       |

---

## 📋 Ordem de Implementação Sugerida

### Sprint 1: Base

- [ ] Migrações de banco de dados
- [ ] Tipos Rust (types.rs)
- [ ] Comandos de departamentos
- [ ] Sidebar de navegação

### Sprint 2: Funcionários

- [ ] CRUD de funcionários
- [ ] Busca e filtros
- [ ] Páginas de listagem e formulário

### Sprint 3: Gaveteiros

- [ ] CRUD de gaveteiros/gavetas
- [ ] Alocação de posições
- [ ] Mapa de ocupação visual

### Sprint 4: Documentos

- [ ] Categorias e tipos
- [ ] Registro de documentos
- [ ] Listagem por funcionário

### Sprint 5: Empréstimos

- [ ] Emprestar/devolver
- [ ] Alertas de atraso
- [ ] Histórico

### Sprint 6: Arquivo Morto

- [ ] Transferência automática
- [ ] Gestão de caixas
- [ ] Controle de expurgo

### Sprint 7: Dashboard e Relatórios

- [ ] Dashboard principal
- [ ] Relatórios operacionais
- [ ] Exportação Excel

### Sprint 8: Ferramentas

- [ ] Gerador de etiquetas
- [ ] Trilha de auditoria
- [ ] Backup automático

---

## 🚀 Comandos para Iniciar

```bash
# Desenvolvimento
deno task dev

# Build
deno task build

# Backend (testes)
cd src-tauri && cargo test

# Gerar migrações Drizzle (se usar)
npx drizzle-kit generate
```
