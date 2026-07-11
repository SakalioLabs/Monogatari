# Contributing to Monogatari

Thank you for your interest in contributing to Monogatari, an LLM-driven visual novel engine.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/Monogatari.git`
3. Install frontend dependencies: `cd frontend && npm install`
4. Run the dev server: `cd rust-engine/crates/tauri-app && cargo tauri dev`

## Project Structure

- `rust-engine/` - Rust backend (Tauri 2.x desktop app)
  - `crates/core/` - EventBus, ServiceLocator, GameClock
  - `crates/ai/` - API engine (OpenAI-compatible), ONNX engine (DirectML)
  - `crates/game/` - Characters, Dialogue, Knowledge, Scenes, Script parser
  - `crates/assets/` - Asset management, save/load
  - `crates/scripting/` - Rhai scripting engine
  - `crates/tauri-app/` - Tauri commands and state management
- `frontend/` - Vue 3 + TypeScript + Vite + Pinia
  - `src/views/` - 22 application views
  - `src/components/` - Reusable components (Live2D, 3D viewer, etc.)
  - `src/stores/` - Pinia state management
  - `src/lib/` - Utilities (Tauri bridge, i18n, toast)
- `.agents/skills/` - Repository-level Agent authoring workflows
- `scripts/module-test-matrix.json` - Module ownership and independent verification gates

## Development Guidelines

### Frontend
- Use Vue 3 Composition API with `<script setup lang="ts">`
- Follow the existing dark theme design system in `main.css`
- Ensure responsive layout on mobile (360px) and desktop
- Use `invokeCommand()` from `lib/tauri.ts` for Tauri calls with browser fallback

### Rust Backend
- Follow existing patterns for Tauri commands (see `commands/` modules)
- Register new commands in `main.rs` and `commands/mod.rs`
- Use `AppState` for shared state via `State<AppState>`
- Use `tokio::sync::RwLock` for concurrent access

### Commit Messages
- Use conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`
- Keep commits focused on a single change
- Update CHANGELOG.md and README.md for user-facing changes

## Testing

- List gates: `node scripts/verify-modules.mjs --list`
- One module: `node scripts/verify-modules.mjs --module rust-ai`
- One implementation group: `node scripts/verify-modules.mjs --group rust`
- All default module gates: `node scripts/verify-modules.mjs`
- Complete release gate: `node scripts/verify-release.mjs`

The module runner can write machine-readable evidence with `--report <path>`. Add every new implementation surface to `scripts/module-test-matrix.json` and add focused tests before relying on the integrated release gate.

The retained Windows x64 SDL2 application gate downloads official pinned SDL2, SDL2_image, and SDL2_ttf release archives, verifies their SHA-256 values, and writes generated native files under ignored `runtimes/`. Run it directly with `powershell -NoProfile -File scripts/prepare-legacy-sdl.ps1` when diagnosing that compatibility path.

## Pull Requests

1. Create a feature branch from `master`
2. Make your changes following the guidelines above
3. Ensure all builds pass
4. Submit a PR with a clear description of changes
