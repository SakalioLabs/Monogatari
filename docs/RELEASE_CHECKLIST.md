# Release Checklist

This checklist tracks the gates required before packaging Monogatari as a commercial desktop engine.

## Required Gates

- [ ] Confirm the branch has no unrelated local changes.
- [ ] Install frontend dependencies with `npm ci` from `frontend/`.
- [ ] Run `npm audit` and verify 0 vulnerabilities.
- [ ] Run `npm run build` from `frontend/`.
- [ ] Run `cargo check --locked -p llm-galgame-app` from `rust-engine/`.
- [ ] Run `dotnet test LLMAssistant.sln --no-restore` from the repository root.
- [ ] Launch the app in Tauri dev mode and smoke-test Dashboard, AI Chat, Story Mode, Workflow, Scene Assets, and Settings.
- [ ] Build the desktop package with `cargo tauri build` from `rust-engine/crates/tauri-app`.

## Packaging Policy

- Commit `frontend/package-lock.json` and `rust-engine/Cargo.lock`.
- Do not commit `frontend/node_modules/`, `frontend/dist/`, or `rust-engine/target/`.
- Keep npm `overrides` documented when they are used to remediate transitive dependency issues.
- Treat `npm audit` and `cargo check --locked` failures as release blockers.

## Signing And Distribution

- [ ] Choose release channel names for internal, beta, and stable builds.
- [ ] Configure platform-specific installer signing before public distribution.
- [ ] Store signing credentials outside the repository.
- [ ] Generate checksums for released installers.
- [ ] Archive the exact source revision, `package-lock.json`, and `Cargo.lock` used for each release.

## Manual QA

- [ ] Verify non-Tauri browser preview renders without runtime crashes.
- [ ] Verify Tauri runtime commands work in the desktop shell.
- [ ] Verify streaming chat emits chunks, completion, emotion, relationship, evaluation, and event notifications.
- [ ] Verify Workflow validation catches missing config, broken links, duplicate links, and unreachable nodes.
- [ ] Verify Workflow save/load rejects invalid graphs and preserves valid graphs.
- [ ] Verify Scene Assets lists metadata scenes, validates missing backgrounds, and sets the active runtime scene.
- [ ] Verify Settings saves project config, reports missing required directories, configures AI, and initializes the runtime.
