# Monogatari Release Checklist

## Pre-Release Verification

### Automated Gate
- [ ] `node scripts/verify-release.mjs` passes from the repository root, covering JSON assets, workflow files, score-gate workflow execution regressions, renderer asset contracts, pinned knowledge-ref contracts, all quality suites, workflow branch coverage snapshots, locale coverage, sensitive token scans, frontend UI text artifact scans, frontend source invariants, frontend route/sidebar coverage, release-critical Rust checks/tests, frontend audit, root and subpath Web/PWA builds, Web/PWA dist assets, preview route smoke checks, and legacy C# tests

### Frontend
- [ ] `cd frontend && npm run build` passes with zero errors
- [ ] `cd frontend && npm run build:web` emits manifest, service worker, offline fallback, `404.html`, and `.nojekyll` assets
- [ ] Web/PWA manifest includes dedicated install and maskable icons, and `sw.js` precaches those icon assets for offline install surfaces
- [ ] `cd frontend && npm run verify:web-budget` passes with entry JS/CSS and lazy renderer chunks inside budget
- [ ] `npm audit` shows zero vulnerabilities
- [ ] All 21 views render correctly (Dashboard, Title, Story Mode, AI Chat, Workflow, Character Editor, Scene Assets, Settings, Characters, Group Chat, Analytics, Quality, Marketplace, Plugins, Audio, Knowledge, Dialogue Editor, Scene Editor, CG Gallery, Backlog, Achievements)
- [ ] Sidebar navigation works for all 20 items
- [ ] Responsive layout verified on mobile viewport (375px) and tablet (768px)

### Rust Backend
- [ ] `cargo check --locked -p llm-galgame-app` passes
- [ ] All 22 command modules register correctly in main.rs
- [ ] Chat streaming works with API backend
- [ ] Character personality/knowledge injection verified

### Content
- [ ] Example characters load correctly (Sakura, Luna, Kenji)
- [ ] Example dialogues play through with choices
- [ ] Knowledge base search returns relevant results
- [ ] Scene assets validate without missing file warnings
- [ ] Checked-in character renderer asset fields resolve to supported project-relative files or intentionally fall back to the generated 3D placeholder
- [ ] Character Editor renderer asset diagnostics flag unsupported extensions, absolute paths, external URLs, and parent traversal before saving/exporting character JSON
- [ ] Character Editor renderer preview follows Story Mode priority for Live2D, GLB/GLTF, sprite/portrait, and generated 3D fallback states
- [ ] Story Mode and Character Editor preview both derive renderer priority from the shared frontend renderer asset selector
- [ ] `npm run verify:renderer-assets` passes, proving shared renderer selector priority, expression sprite resolution, validation skips, and generated fallback behavior
- [ ] Story Mode renderer fallback verified for Live2D, GLB/GLTF, sprite/portrait, and assetless character states
- [ ] Checked-in character `knowledge_refs`, legacy `knowledge`, and `knowledgeRefs` resolve to existing knowledge entries in both project data roots

### AI Integration
- [ ] API mode: streaming chat with OpenAI-compatible endpoint
- [ ] ONNX mode: local model inference (if applicable)
- [ ] Evaluation triggers fire at correct intervals
- [ ] Relationship milestones unlock events correctly
- [ ] Workflow LLM nodes guard generated output before it is used by downstream story nodes
- [ ] Character prompts include creator-declared pinned knowledge references before keyword search results
- [ ] Quality Suites panel runs character stability, prompt-injection, relationship and fallback scoring side-channel containment, memory-poisoning containment, memory prompt replay containment, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, knowledge-boundary stability, evaluation-summary safety, workflow output safety, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, event-idempotence, and event-rule snapshot regression checks
- [ ] Quality suite files reject out-of-range score expectations and contradictory expected/forbidden events, markers, or workflow nodes before release reports run
- [ ] Quality Suites panel shows and exports versioned audit evidence with failed-scenario ids, category summaries, safety-signal counts, and workflow coverage summaries

### Workflow Editor
- [ ] All 21 node types render in palette
- [ ] Drag-and-drop creates nodes on canvas
- [ ] Validation catches missing fields and broken links
- [ ] Run panel emits a trace for checked-in sample workflows and stops at unresolved player choices
- [ ] Checked-in score-gate workflow fixture completes both fallback and unlocked branches from seeded evaluation state
- [ ] Run panel shows evaluation metric, threshold, score source, event trigger state, and blocker reasons for score-gated story branches
- [ ] Canvas nodes show run badges for executed, pass/fail, blocked event, completed, and waiting-choice states
- [ ] Run preview context can simulate character scores, relationship values, evaluation counts, and already-triggered events without mutating chat session state
- [ ] Run preview context clamps overrange author scores and relationship values before score-gated workflow branches consume them
- [ ] Run preview presets cover unlock, low-score block, and repeat-trigger block branches for score-gated workflows
- [ ] Run report shows graph coverage percentage, executed node count, and unvisited node ids for branch QA
- [ ] Run preset matrix executes all score-gate preview presets and reports merged graph coverage
- [ ] Quality Suite report shows the score-gate workflow coverage snapshot at 100% merged branch coverage
- [ ] Export produces valid JSON

### Audio
- [ ] BGM tracks list and volume controls work
- [ ] Ambient loop tracks play through the music transport and respond to the ambient mixer channel
- [ ] SFX preview plays correctly
- [ ] Master, BGM, ambient, SFX, and voice mixer channels respond to input and persist across reloads

### i18n
- [ ] Locale switching works (en, zh-CN, ja-JP, ko-KR)
- [ ] Nested key resolution works for all locale files
- [ ] Browser fallback locale JSON loads correctly under root and `VITE_BASE_PATH` subpath deployments

### Cloud Sync
- [ ] Push/pull commands execute without error
- [ ] Sync status displays correctly in Settings

## Distribution
- [ ] Version bumped in tauri.conf.json
- [ ] Version bumped in package.json
- [ ] Version bumped in rust-engine/Cargo.toml
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md version and features updated
- [ ] Git tag created: `git tag v0.9.5`
- [ ] Web/PWA preview verified with `npm run preview:web`
- [ ] Subpath web deployment verified with `VITE_BASE_PATH` when publishing to GitHub Pages or another non-root path
- [ ] Windows MSI installer built: `cargo tauri build`
- [ ] macOS DMG installer built (if applicable)
- [ ] Linux AppImage built (if applicable)
- [ ] Code signing applied to installers
- [ ] GitHub Release created with installers attached
