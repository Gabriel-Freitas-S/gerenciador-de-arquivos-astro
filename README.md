# Arquivo Inteligente (Astro + Tauri)

> Aplicativo desktop para Windows que combina Astro, Tauri e SQLite para gerenciar pastas, envelopes, gaveteiros e movimentações de um arquivo físico com autenticação local.

## 🚧 Stack principal

- [Astro 5](https://astro.build/) para a camada de interface.
- [astro-min](https://github.com/advanced-astro/min#readme) para minificar HTML/CSS/JS/SVG estáticos no build.
- [Tauri 2](https://tauri.app/) como runtime desktop (Rust + WebView2 no Windows).
- `@tauri-apps/api` para comunicação renderer ↔️ backend via `invoke`.
- Backend Rust com `rusqlite` (SQLite embarcado), `bcrypt` e gerenciamento de sessões em memória.

## ✅ Pré-requisitos

- Node.js 20 LTS e npm.
- Rust toolchain via [rustup](https://www.rust-lang.org/tools/install) + componentes MSVC/Clang (no Windows instale "Desktop development with C++").
- WebView2 Runtime (já incluído no Windows 11; no Windows 10 instale a versão Evergreen).
- Variáveis definidas em `.env` (`ARCHIVE_DEFAULT_ADMIN_LOGIN`, `ARCHIVE_DEFAULT_ADMIN_PASSWORD`).
- (Opcional, recomendado) [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) para baixar binários pré-compilados do `tauri-cli`.

### Instalando cargo-binstall rapidamente

```powershell
# Windows
irm https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-via-powershell.ps1 | iex
```

```bash
# macOS / Linux
curl -L https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
```

Depois de instalado, execute um dos scripts auxiliares para baixar o `tauri-cli` já compilado:

```powershell
pwsh ./scripts/setup-tauri.ps1 -NoConfirm
```

```bash
chmod +x ./scripts/setup-tauri.sh
./scripts/setup-tauri.sh
```

Os scripts verificam se `cargo-binstall` está disponível e exibem instruções caso precise instalá-lo manualmente.

## ⚙️ Configuração inicial

1. Copie o arquivo de variáveis e defina o login/senha padrão do primeiro administrador:

	```powershell
	Copy-Item .env.example .env
	# edite ARCHIVE_DEFAULT_ADMIN_LOGIN e ARCHIVE_DEFAULT_ADMIN_PASSWORD
	```

2. Instale as dependências JavaScript e Rust (cargo é instalado junto com o rustup):

	```powershell
	npm install
	```

3. Execute o modo desenvolvimento. O script roda `astro dev` (frontend) e abre a janela Tauri conectada ao servidor de desenvolvimento:

	```powershell
	npm run dev
	```

	O primeiro usuário é criado a partir das variáveis definidas em `.env` (`ARCHIVE_DEFAULT_ADMIN_LOGIN` / `ARCHIVE_DEFAULT_ADMIN_PASSWORD`) e tem a senha armazenada com bcrypt.

4. Gere o build de produção (renderização estática + binário Tauri):

	```powershell
	npm run build
	```

	O frontend é compilado para `dist/` e o executável final fica em `src-tauri/target/release/` (ou `debug/` durante testes).

5. (Opcional) Personalize os ícones nativos usados no Windows substituindo `src-tauri/icons/icon.ico` e `src-tauri/icons/icon.png`. Esses arquivos já atendem aos requisitos mínimos do Tauri, mas você pode sobrescrevê-los por ícones reais da sua marca antes de gerar instaladores.

## 🗂️ Estrutura relevante

```
src/
├── components/app/*      # UI do dashboard
├── layouts/Layout.astro  # shell principal
├── pages/index.astro     # tela única
├── scripts/
│   ├── app.ts            # controla estado da UI
│   └── archive-api.ts    # wrapper de invoke do Tauri
├── styles/global.css     # Tailwind tokens + estilos globais
└── types/archive.ts      # tipos compartilhados no front

src-tauri/
├── src/db.rs             # SQLite, migrações e queries
├── src/sessions.rs       # gerenciamento de sessões em memória
├── src/main.rs           # bootstrap do Tauri + comandos IPC
├── src/types.rs          # tipos usados pelas respostas do backend
├── Cargo.toml / build.rs # projeto Rust
└── tauri.conf.json       # configuração de build (dev/build commands)
```

Outros arquivos importantes:

- `.env.example`: modelo com as variáveis sensíveis necessárias.
- `astro.config.mjs`: integrações Astro (minificação e Tailwind via Vite).
- `package.json`: scripts (`npm run dev`, `npm run build`, `npm run dev:astro`, etc.) e dependências JS.
- `src-tauri/tauri.conf.json`: conecta Astro com o ciclo de vida do Tauri.

## 🎨 Tailwind 4 pronto para uso

- Tailwind foi instalado via `astro add tailwind`, então nenhuma configuração manual adicional é necessária.
- `src/styles/global.css` importa `tailwindcss`, registra tokens via `@theme` e mantém os estilos do dashboard.
- Ao criar novos componentes, basta usar classes utilitárias (ex.: `class="flex gap-4"`) ou adicionar regras dedicadas nesse arquivo.
- Caso precise extender o tema, adicione variáveis em `@theme { ... }` e utilize-as em utilitários como `bg-[color:var(--color-app-surface)]`.

## 🔐 Fluxo atual

1. O backend Rust abre `archive.sqlite` em `AppData` usando SQLite embarcado (modo WAL).
2. Comandos Tauri (`auth_login`, `storage_create`, etc.) validam payloads, conferem sessões em memória e hitam o banco.
3. O frontend chama esses comandos por meio de `archiveApi` (`@tauri-apps/api/core` + `invoke`).
4. `src/scripts/app.ts` mantém o estado do dashboard (login, cadastros, timeline) com as respostas `ApiResponse<T>` retornadas pelo backend.

## ▶️ Próximos passos sugeridos

1. Configurar pipeline de distribuição (MSIX/Inno Setup) com base nos artefatos `tauri build`.
2. Expandir o modelo do banco (ex.: anexos, auditoria detalhada, permissões avançadas).
3. Implementar telas auxiliares (busca global, relatórios, múltiplos arquivos físicos).
4. Adicionar testes unitários/integrados no backend Rust (commands e camada SQL).

## 🧰 Scripts úteis

- `npm run dev`: executa `tauri dev` (Astro em modo dev + janela Tauri).
- `npm run dev:astro`: executa apenas `astro dev` (útil para trabalhar só no front).
- `npm run build`: executa `tauri build` (gera `dist/` + binário assinado pelo Tauri).
- `npm run build:astro`: compila somente o frontend estático.
- `npm run preview`: pré-visualiza o build do Astro sem subir o backend (útil para inspeção de layout).

Sinta-se à vontade para adaptar os componentes conforme os fluxos do seu arquivo físico.
