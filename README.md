# Arquivo Inteligente (Astro + Electron)

> Aplicativo desktop para Windows que combina Astro, Electron e SQLite (SQLCipher) para gerenciar pastas, envelopes, gaveteiros e movimentações de um arquivo físico com autenticação local.

## 🚧 Stack principal

- [Astro 5](https://astro.build/) para a camada de interface.
- [astro-electron](https://github.com/Igloczek/astro-electron) para empacotar e executar o front-end dentro do Electron.
- [astro-min](https://github.com/advanced-astro/min#readme) para minificar HTML/CSS/JS/SVG estáticos no build.
- [Electron 31](https://www.electronjs.org/) como runtime desktop.
- [@journeyapps/sqlcipher](https://github.com/journeyapps/node-sqlcipher) + `bcryptjs` para banco local criptografado e hashing de senhas.

## ✅ Pré-requisitos

- Node.js 20 LTS ou superior (necessário para construir os binários do Electron).
- npm (Electron não funciona bem com pnpm ou yarn moderno).
- Ferramentas de build do Windows (instaladas automaticamente pelo instalador do Node, se solicitado) para compilar o SQLCipher.

## ⚙️ Configuração inicial

1. Copie o arquivo de variáveis e defina uma chave forte para o banco:

	```powershell
	Copy-Item .env.example .env
	# edite ARCHIVE_DB_KEY, ARCHIVE_DEFAULT_ADMIN_LOGIN e ARCHIVE_DEFAULT_ADMIN_PASSWORD
	```

2. Instale as dependências:

	```powershell
	npm install
	```

3. Execute o modo desenvolvimento (Astro + Electron sobem em conjunto pelo `astro-electron`; defina a chave do banco antes de iniciar):

	```powershell
	$env:ARCHIVE_DB_KEY = 'sua-chave-super-secreta'
	npm run dev
	```

	O primeiro usuário é criado a partir das variáveis definidas em `.env` (`ARCHIVE_DEFAULT_ADMIN_LOGIN` / `ARCHIVE_DEFAULT_ADMIN_PASSWORD`) e armazena a senha com bcrypt.

4. Para gerar o build de produção (renderização estática + bundle do processo principal/preload):

	```powershell
	npm run build
	```

	O resultado fica em `dist/` (renderer) e `dist-electron/` (main/preload). A publicação final pode ser feita com Electron Forge, Electron Builder ou outra ferramenta de empacotamento.

## 🗂️ Estrutura relevante

```
src/
├── components/
│   └── app/
│       ├── AppHeader.astro   # cabeçalho/status reutilizável
│       ├── AuthGate.astro    # formulário de login isolado
│       ├── MovementPanel.astro
│       ├── StoragePanel.astro
│       └── SummaryPanel.astro
├── electron/
│   ├── database.ts      # conexão com SQLCipher, migrações e consultas
│   ├── main.ts          # bootstrap do Electron + IPC
│   ├── preload.ts       # expõe API segura para o renderer
│   ├── sessions.ts      # gerenciamento de sessões em memória
│   └── types.ts         # tipos compartilhados entre os processos
├── layouts/Layout.astro # layout principal com carregamento do app.ts
├── pages/index.astro    # dashboard com login, cadastros e timeline
├── scripts/app.ts       # ponto de entrada da lógica de UI/IPC
└── styles/app.css       # estilos globais do shell do aplicativo
```

Outros arquivos importantes:

- `.env.example`: modelo com `ARCHIVE_DB_KEY` e credenciais padrão.
- `astro.config.mjs`: integrações Astro + Electron e pontos de entrada.
- `package.json`: scripts (`npm run dev`, `npm run build`) e dependências.

## 🔐 Fluxo atual

1. Preload expõe `window.archive.*` com canais IPC protegidos.
2. `src/scripts/app.ts` controla login, cadastros de unidades e registro de movimentações via `window.archive`.
3. O banco (`archive.sqlite`) é salvo em `app.getPath('userData')` e protegido por `PRAGMA key` com a chave definida em `.env`.
4. O primeiro usuário é criado automaticamente caso a tabela esteja vazia.

## ▶️ Próximos passos sugeridos

1. Adicionar empacotamento com Electron Forge/Electron Builder (atualmente não configurado).
2. Expandir o modelo de dados (itens detalhados, anexos, auditoria).
3. Implementar telas adicionais (busca, dashboards específicos, permissões múltiplas).
4. Evoluir o `app.ts` para uma classe modular caso o front-end cresça.

## 🧰 Scripts úteis

- `npm run dev`: liga o Astro em modo desenvolvimento e aciona automaticamente o Electron via `astro-electron`.
- `npm run build`: gera `dist/` (renderer) e `dist-electron/` (main/preload) em uma única etapa.
- `npm run preview`: pré-visualiza apenas o build estático do Astro (sem Electron).

Sinta-se à vontade para adaptar os componentes conforme os fluxos do seu arquivo físico.
