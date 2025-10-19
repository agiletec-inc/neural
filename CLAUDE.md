# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚨 Critical Rules (絶対厳守)

**DOCKER-FIRST MANDATORY**:
```bash
# ❌ FORBIDDEN - Mac host pollution
npm install
pnpm install  # Outside Docker
cd src-tauri && cargo build

# ✅ CORRECT - Docker-only development
make workspace
pnpm install --frozen-lockfile
pnpm dev
```

**Why Docker-First**:
- Dependencies isolated in named volumes
- Mac environment stays clean
- Consistent dev environment across team
- No version conflicts

**📖 Global Rules**: See `~/.claude/CLAUDE.md` and `~/github/CLAUDE.md`

---

## Project Overview

**NeuraL Translator** - Tauri v2 desktop app for real-time translation using local Ollama LLM (llama3.1:8b).

**Key Features**:
- Desktop translation interface with clipboard monitoring
- Global keyboard shortcut (⌘+Shift+T) for quick access
- Auto-translation mode for clipboard changes
- Translation caching (100 entries, FIFO)
- Always-on-top window mode

**Stack**: Tauri 2.x (Rust) + React 18 + TypeScript + Vite + Ollama API

---

## 🐳 Docker-First Development

### Standard Workflow

```bash
# 1. Start workspace container
make up

# 2. Enter workspace shell
make workspace

# 3. Install dependencies (first time only)
pnpm install --frozen-lockfile

# 4. Start Vite dev server
pnpm dev  # Access at http://localhost:1420
```

### Common Commands

```bash
make up              # Start workspace container
make down            # Stop all services
make workspace       # Enter workspace shell
make install         # Install dependencies (Docker)
make dev             # Start Vite dev server (Docker)
make build           # Build frontend (Docker)
make clean           # Remove Mac host pollution
make clean-all       # Stop + clean + remove volumes
make ps              # Show container status
make logs            # Show logs
```

**Inside workspace** (`make workspace` first):
```bash
pnpm install --frozen-lockfile   # Install dependencies
pnpm dev                          # Vite dev server
pnpm build                        # Build frontend
pnpm tauri --help                 # Tauri CLI
```

---

## 🎯 Tauri-Specific Workflow

### Frontend Development (Docker)

**All frontend work happens in Docker**:
```bash
make workspace
pnpm dev  # Vite dev server at http://localhost:1420
```

**Access**:
- Browser: `http://localhost:1420`
- HMR: port 1421
- Test React components without desktop app

### Desktop App (Mac Host)

**Tauri desktop app requires Mac GUI**:
```bash
# Prerequisites (Mac host):
# 1. Rust toolchain installed
# 2. Ollama running: ollama serve
# 3. Dependencies installed via Docker

# Run desktop app (Mac)
make tauri-dev  # OR: pnpm tauri dev
```

**Why Mac Host?**:
- **Ollama GPU 使用**: Mac の Metal GPU で推論高速化（Docker は GPU 不可）
- **Tauri GUI**: デスクトップアプリの表示が必要
- **システム統合**: クリップボード、グローバルショートカット
- **ファイルシステム**: 直接アクセスが必要

**Development Split**:
- **Frontend changes**: Docker → Vite dev server → hot reload
- **Rust changes**: Mac → Tauri dev → desktop app restart
- **Dependencies**: Always Docker → never Mac

---

## Architecture

### Project Structure

```
neural/
├── src/                    # React frontend
│   ├── App.tsx            # Main UI + translation logic
│   ├── App.css            # Styles
│   └── main.tsx           # React entry
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Tauri app setup
│   │   ├── ollama.rs      # Ollama API client
│   │   └── main.rs        # Binary entry
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri config
├── docker-compose.yml     # Docker environment
└── Makefile               # Standard commands
```

### Communication Flow

```
User Action
  → React UI (src/App.tsx)
  → invoke("translate", {text, fromLang, toLang})
  → Rust Command Handler (src-tauri/src/lib.rs)
  → OllamaClient (src-tauri/src/ollama.rs)
  → HTTP Request → Ollama API (localhost:11434)
  ← JSON Response
  ← TranslateResponse
  ← React State Update
```

### Key Components

**Frontend (React)**:
- **App.tsx**: Main UI, translation logic, clipboard monitoring, state management
- **Translation cache**: Map with key `{text}|{fromLang}|{toLang}`, max 100 entries
- **Clipboard polling**: 1 second interval when auto-translate enabled

**Backend (Rust)**:
- **lib.rs**: Tauri commands, global shortcut registration (⌘+Shift+T)
- **ollama.rs**: Ollama API client (base URL: `http://localhost:11434`)
  - Model: `qwen2.5:3b` (高速・多言語対応優秀)
  - Temperature: 0.3, Top-p: 0.9
  - Endpoint: `/api/generate`
  - Health check: `/api/tags`

**Tauri Plugins**:
- `tauri-plugin-clipboard-manager`: Read/write clipboard
- `tauri-plugin-global-shortcut`: Global keyboard shortcuts
- `tauri-plugin-opener`: Open external URLs

---

## 🔧 Configuration

### Docker Environment

**docker-compose.yml**:
- Base image: `node:24-bookworm`
- Named volumes: `pnpm-store`, `node_modules`, `cargo_registry`
- Ports: 1420 (Vite), 1421 (HMR)
- Auto-install: Rust + pnpm on startup

**Why Named Volumes?**:
- Mac stays clean (no node_modules pollution)
- Fast installation (cached between runs)
- Isolated dependencies per project

### Tauri Config (src-tauri/tauri.conf.json)

```json
{
  "productName": "neural-translator",
  "identifier": "com.neural-translator.app",
  "app": {
    "windows": [{
      "width": 900,
      "height": 600,
      "minWidth": 800,
      "minHeight": 500,
      "alwaysOnTop": true  // Always visible
    }]
  }
}
```

### Vite Config (vite.config.ts)

```typescript
{
  server: {
    port: 1420,  // Tauri requirement
    strictPort: true,
    host: '0.0.0.0',  // Docker external access
    allowedHosts: ['neural.agiletec.traefik', 'localhost']
  }
}
```

---

## 🦙 Ollama Integration

### Prerequisites

**Install Ollama** (Mac host):

⚠️ **重要**: Ollama は **Mac ローカルで実行必須**（Docker 不可）
- **理由**: Mac の GPU (Metal) を使用するため
- Docker だと GPU が使えず、推論速度が激遅になる
- Mac ネイティブなら GPU 加速で **3-5倍高速**

```bash
# Install: https://ollama.ai
brew install ollama  # OR download installer

# Pull model (Qwen2.5:3b - 軽量・超高速・GPU最適化)
ollama pull qwen2.5:3b

# Start server (GPU 使用)
ollama serve  # Runs on port 11434
```

### Check Ollama Status

```bash
make ollama-check  # Verify Ollama running
make ollama-pull   # Pull qwen2.5:3b model
```

### API Configuration

**Base URL**: `http://localhost:11434`
**Model**: `qwen2.5:3b` (Qwen2.5 - 高速・多言語対応)
**Prompt Template**:
```
Translate the following text from {from_lang} to {to_lang}.
Only provide the translation without any explanations or additional text:

{text}
```

**Request Parameters**:
- `temperature`: 0.3 (deterministic)
- `top_p`: 0.9
- `stream`: false

---

## 🧪 Testing Translation Flow

```bash
# 1. Start Ollama (Mac)
ollama serve

# 2. Verify Ollama running
make ollama-check

# 3. Start workspace (Docker)
make up
make workspace

# 4. Install dependencies (first time)
pnpm install --frozen-lockfile

# 5. Start Vite dev server
pnpm dev  # Access at http://localhost:1420

# 6. Test in browser
# - Check status indicator (should show "オンライン")
# - Enter text and click "翻訳する"
# - Or press ⌘+Enter

# 7. Test desktop app (Mac)
make tauri-dev
# - Copy text
# - Press ⌘+Shift+T
# - Should auto-translate clipboard
```

---

## 🚨 Common Issues

**"pnpm: command not found"**
→ Must run inside Docker: `make workspace`

**"オフライン" status in UI**
→ Ollama not running or not at localhost:11434
→ Start: `ollama serve`

**"Model not found" error**
→ Model not pulled
→ Run: `ollama pull qwen2.5:3b`

**"Clipboard not working"**
→ macOS permissions issue
→ Grant accessibility access to app in System Settings

**"Global shortcut not triggering"**
→ Shortcut conflict with another app
→ Or app not running with GUI

**"Module not found" after git pull**
→ Dependencies changed
→ Run: `make workspace` → `pnpm install --frozen-lockfile`

**React hot reload not working**
→ Check Vite dev server running on 1420
→ Check HMR port 1421 accessible
→ Restart: `make restart`

---

## 📝 Code Patterns

### Adding Tauri Commands

**1. Define Rust function** (src-tauri/src/lib.rs):
```rust
#[tauri::command]
async fn my_command(param: String) -> Result<Response, String> {
    // Implementation
    Ok(Response { data: param })
}
```

**2. Register in handler**:
```rust
.invoke_handler(tauri::generate_handler![
    my_command,  // Add here
    translate,
    // ... other commands
])
```

**3. Call from frontend**:
```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<Response>("my_command", { param: "value" });
```

### Frontend-Backend Type Safety

**Keep Rust and TypeScript types synchronized**:

**Rust** (src-tauri/src/ollama.rs):
```rust
#[derive(Serialize, Deserialize)]
pub struct TranslateResponse {
    pub translated_text: String,
}
```

**TypeScript** (src/App.tsx):
```typescript
interface TranslateResponse {
  translated_text: string;
}
```

### State Management Pattern

**OllamaClient state**:
```rust
// In lib.rs setup
let ollama_client = Arc::new(Mutex::new(OllamaClient::new()));

tauri::Builder::default()
    .manage(ollama_client)  // Manage state
    .invoke_handler(...)
```

**Access in commands**:
```rust
#[tauri::command]
async fn translate(
    state: State<'_, Arc<Mutex<OllamaClient>>>,
) -> Result<TranslateResponse, String> {
    let client = state.lock().await;
    client.translate(request).await
}
```

---

## 🔨 Build & Distribution

### Development Build

```bash
# Frontend only (Docker)
make workspace
pnpm build

# Desktop app (Mac)
pnpm tauri build --debug
```

### Production Build

```bash
# Prerequisites: Dependencies installed via Docker
make install

# Build (Mac - requires code signing)
pnpm tauri build

# Output: src-tauri/target/release/bundle/
# - macOS: .app, .dmg
# - Windows: .exe, .msi (if building on Windows)
# - Linux: .AppImage, .deb (if building on Linux)
```

---

## 🧹 Cleanup

### Regular Cleanup

```bash
# Remove Mac host pollution
make clean

# What it removes:
# - node_modules/ (should be in Docker volume)
# - dist/ (build artifacts)
# - .turbo/ (cache)
# - package-lock.json (npm forbidden, pnpm only)
# - .DS_Store (Mac junk)
```

### Complete Reset

```bash
# Stop + clean + remove volumes
make clean-all

# Removes everything:
# - All containers
# - All volumes (pnpm-store, node_modules, cargo_registry)
# - Mac host pollution

# Rebuild from scratch:
make up
make workspace
pnpm install --frozen-lockfile
```

---

## 📚 Related Documentation

**Global Rules**:
- `~/.claude/CLAUDE.md` - SuperClaude framework
- `~/github/CLAUDE.md` - Workspace-level rules

**Reference Implementation**:
- `~/github/agiletec/CLAUDE.md` - Turborepo monorepo example
- `~/github/makefile-global/templates/` - Makefile templates

**External Docs**:
- [Tauri v2 Docs](https://v2.tauri.app/)
- [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Vite](https://vitejs.dev/)
- [React 18](https://react.dev/)

---

**Version**: 1.0 (2025-10-14)
**Change Log**:
- Initial Docker-First setup
- Added Makefile standardization
- Documented Tauri-specific workflow split (frontend Docker, desktop Mac)
- Added Ollama integration guide
