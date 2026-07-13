# Monogatari v0.9.5

A low-code game development engine for building LLM-driven visual novels and text adventures across Web/PWA and Windows packages.

## What It Is

Monogatari is an authoring workbench and runtime toolkit, not a game itself. Developers compose characters, scenes, dialogue graphs, story events, endings, knowledge, audio, and model behavior through visual editors and structured project files. Web packages run verified local models through WebGPU. Desktop and server deployments use a conservative backend planner across compatible local runtimes and generation-verified OpenAI-compatible services; the linked Windows DirectML executor remains limited to compatible full-sequence ONNX graphs.

## Key Features

- **Low-Code Story Flow** - Drag-and-drop nodes compose dialogue, conditions, model generation, scoring, authored events, state changes, and scene transitions.
- **Character and Ensemble Tests** - Developers inspect character prompts, streamed model output, evaluation traces, safety guards, relationships, and authored event decisions before packaging.
- **Authored Event System** - Projects may define relationship, score, evaluation, and cumulative-progress rules that trigger scenes, dialogue, endings, or flags; the engine adds no global gamification layer.
- **Quality Suites** - Offline regression scenarios validate character stability, prompt-injection resistance across structured role blocks, English, Chinese, Japanese, Korean, and Unicode-obfuscated player text, group chat runtime trace evidence, relationship and fallback scoring side-channel containment, memory-poisoning resistance, memory prompt replay safety, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, knowledge-boundary stability against player-supplied retcons, evaluation-summary safety, workflow output safety, workflow guard-only fallback, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, story-event trigger thresholds/idempotence, and SHA-256 event-rule fingerprint snapshots without requiring live model calls.
- **Web/PWA Distribution** - Browser builds include a versioned WebGPU inference contract, Transformers.js text generation, install metadata, offline shell, and project-content caching.
- **Web Bundle Budgets** - Production builds verify small entry assets while allowing bounded lazy chunks for Three.js, GLTF loading, OrbitControls, and Live2D.
- **Dialogue Editor** - Project-backed branching graph editor with atomic saves, node/edge validation, choice conditions, relationship effects, LLM prompts, protected deletion, browser drafts, and Playtest preview.
- **Visual Workflow Editor** - Drag-and-drop node-based editor for designing dialogue flows, branching conditions, LLM generation nodes, evaluation triggers, and scene transitions.
- **Workflow Validation** - Import/export and project-scoped save/load paths validate node ids, start/end structure, missing config fields, broken links, duplicate links, and unreachable nodes.
- **Scene Asset Library** - Project scene metadata and background files are scanned, validated, listed, and selectable as the active runtime scene.
- **Ending Route Editor** - Creators bind validated scenes and dialogues into versioned endings with event coverage diagnostics, optimistic concurrency, rollback-safe saves, reference-protected deletion, and author preview.
- **Scene Catalog Editor** - Creators promote background-inferred scenes into metadata, preview real assets, diagnose event access, and save or delete metadata with optimistic concurrency and cross-catalog reference protection.
- **Renderer Fallback Pipeline** - Playtest resolves project assets across Tauri and Web builds, preferring Live2D, then GLB/GLTF 3D models, then 2D sprites or portraits, with runtime load-failure fallback and a generated 3D stage placeholder when no art is available.
- **Real 3D Fixture** - A licensed animated glTF 2.0 Fox fixture verifies GLB loading, texture color space, automatic unit normalization, responsive camera framing, animation playback, and renderer fallback in packaged Web/PWA and desktop data roots.
- **Project Control Panel** - Project settings, path readiness, AI backend selection, and runtime initialization are managed from one production-oriented console.
- **Verified Project Packages** - Desktop authors export and import complete `.monogatari` ZIP packages with sanitized settings, deterministic inventories, SHA-256 verification, portable-path enforcement, bounded extraction, and transactional installation into a new project directory.
- **Agent Authoring Skill and MCP** - A repository Skill defines dependency-ordered visual-novel production, while the standard stdio MCP server exposes seven schema-backed inspect/validate/read/plan/apply tools over the same headless authoring core. Writes are disabled by default and require exact SHA preconditions plus a reviewed plan fingerprint. See [MCP_SERVER.md](docs/MCP_SERVER.md).
- **Character System** - Full personality model (Big Five traits), memory system, emotion tracking, and relationship scores per character.
- **Knowledge Base** - Keyword-indexed world lore, pinned character references, and release-verified context anchors that feed into AI prompts for consistent storytelling.
- **Branching Dialogue** - Pre-scripted dialogue trees with choices, relationship changes, and flag-based conditional branching.
- **Live2D Support** - Animated character models via PixiJS + pixi-live2d-display.
- **Save/Load System** - Full game state persistence including character states, flags, variables, and chat history.
- **Rhai Scripting** - Embedded scripting engine for custom game logic, conditions, and triggers.
- **Knowledge Base Manager** - Full CRUD interface for world lore, character backgrounds, and AI context entries with category filtering, tag cloud, and keyword search.
- **Professional Character Editor** - 5-tab editor with Big Five personality sliders, radar chart visualization, emotion configuration, relationship management, knowledge entries, renderer asset diagnostics, Playtest preview, emotion sprite mapping, and JSON export.
- **Audio Manager** - Manage background music, ambient sounds, and sound effects with per-track volume control and master mixer.
- **Plugin System** - Register and manage custom workflow node types, event triggers, and action handlers through a dedicated management UI.
- **Cloud Save Sync** - Project-scoped save manifests track local changes, pending uploads/downloads, cross-device conflicts, and remote preflight readiness without persisting sync tokens.
- **Multi-Language Support** - i18n scaffold with zh-CN, ja-JP, and ko-KR locale files for international deployment.
- **Template Marketplace** - Browse, import, and export community-created templates, characters, and story modules.

- **Project Export** - Export project as distributable JSON manifest with content inventory for packaging.
- **Targeted Model Runtimes** - Web/PWA packages use WebGPU; desktop diagnostics report WebGPU, llama.cpp, WinML GenAI, DirectML ONNX, MLX-LM, and OpenAI-compatible readiness; server plans cover vLLM and SGLang without bundling those services into the app.
- **Title Preview** - Inspect the authored title experience and launch the current project Playtest.
- **Visual Review** - Review scene and character art, runtime metadata, and unlock-state presentation before release.
- **Transcript** - Inspect runtime conversation history by character and role.
- **Full i18n Internationalization** - Strictly verified English, Simplified Chinese, Japanese, and Korean catalogs cover all production UI surfaces.
- **Commercial Workbench UI** - Desktop-first dashboard, authoring editors, test surfaces, diagnostics, and packaging controls designed for repeated production use.

## Current Development Status

Verified on 2026-07-11:

- Frontend production build passes with `npm run build`.
- Web/PWA production build passes with `npm run build:web`, including static-hosting SPA fallback assets, dedicated install/maskable icons, copied project sample assets, and bundle-budget verification.
- Qwen3.5 0.8B Text ONNX Q4 initializes and generates a complete streamed Chinese response through the packaged Transformers.js WebGPU runtime on the verified Windows host.
- The versioned inference backend planner separates host/runtime detection from exact-model generation readiness and refuses to auto-select unprobed backends.
- Qwen3.5 0.8B is explicitly blocked on the current linked DirectML executor and tested WinML GenAI DirectML profiles; the hybrid recurrent state contract does not fit the existing full-sequence ONNX loop, and the newer WinML profile reproduced a DML graph-partition/capture failure.
- Qwen3.5 0.8B Q4_K_M generates through `llama.cpp` on WSL2 Ubuntu CPU, and `llama-server` passes health, non-streaming OpenAI-compatible completion, SSE streaming, and terminal `[DONE]` probes.
- The packaged `renderer_fox` fixture loads a real animated GLB with three clips; desktop 1440x900 and mobile 375x812 pixel probes confirm a nonblank, nonuniform, fully framed canvas whose signature changes over time.
- Playtest exposes a project-backed Story Library whose scene, dialogue, and ending entries consume the persistent event-unlock ledger. Content not referenced by an `unlock_*` action remains open for backward compatibility.
- The Story Event workbench edits trigger thresholds, character scopes, repeat behavior, typed actions, and metadata, with fingerprint conflict detection and rollback-safe project catalog replacement.
- The Ending Route workbench binds real scenes and dialogues into versioned assets, detects mismatched event coverage, rejects stale writes, protects referenced deletions, and previews valid routes without changing player unlock progress.
- Scene and Dialogue workbenches now edit the active project catalog instead of temporary UI state. Saves use catalog fingerprints and rollback-capable JSON transactions; scene deletion scans Story Events, endings, and workflows, while dialogue deletion scans Story Events and endings.
- Dialogue loading validates strict fields, authoritative node-map IDs, all transition targets, reachability, character references, relationship deltas, terminal metadata, script bounds, and LLM prompt requirements before runtime activation.
- Versioned ending assets bind a gated ending ID to an existing scene and dialogue, and real player launch validates all three access decisions before playback.
- Web/PWA builds package scene, dialogue, ending, event, and renderer assets in `project-assets.json`, cache them for offline use, execute checked-in branching dialogue nodes through a tested browser state machine, filter condition-gated choices with stable authored indices, skip false linear conditional nodes, carry dialogue variables/flags, apply validated choice relationship effects, and stop explicitly on unsupported browser scripts or expressions. Scene/Dialogue/Ending authoring drafts feed Playtest preview without modifying packaged source files. Desktop playback uses the bounded shared Rhai engine for the same authored conditions and scripts, preflights every relationship target before advancing the dialogue cursor, and applies bounded deltas through CharacterManager.
- Mobile shell readiness passes with `npm run verify:mobile-readiness`, covering viewport safe-area support, iOS/PWA metadata, compact Tauri shell limits, and bottom navigation safe-area padding.
- Responsive shell verification runs during `npm run build:web`, covering built 375px mobile and 768px tablet Web/PWA layout signals.
- Tauri mobile deployment preflight passes with `node scripts/verify-tauri-mobile-preflight.mjs`, covering Android/iOS command readiness, Vite `TAURI_DEV_HOST` binding, Tauri shell config, and mobile release documentation.
- Full frontend dependency audit passes with `npm audit`.
- The transport-neutral authoring core passes `cargo test --locked -p llm-authoring`, independently proving atomic rollback, portable path containment, project diagnostics, and credential-free settings persistence.
- The standard `monogatari-mcp` adapter passes real stdio child-process tests for handshake, seven schema-backed tools, scrubbed inspection, read-only core and delivery validation, exact JSON fingerprints, read-only refusal, reviewed-plan confirmation, one-writer exclusion, successful application, and rollback. Candidate application reports `core_runtime` only after real character/dialogue/knowledge managers load; strict scene/ending/Story Event/Workflow/Quality Suite catalogs load; inferred background scenes resolve; Workflow graphs and Quality expectations pass; and their project references pass. Delivery validation additionally proves declared asset readiness without claiming rendered visual quality.
- Frontend Vitest coverage independently exercises authoring validation, renderer fallback order, story access derivation, the pure browser workflow preview state machine, Pinia command/loading state, and shared Vue component interactions instead of relying only on production builds and source scans.
- Playwright Chromium coverage independently exercises workspace navigation, browser character-draft persistence, and dialogue authoring through browser Playtest; CI installs the pinned browser runtime before the frontend module matrix runs.
- Rust Tauri app crate passes `cargo check --locked -p llm-galgame-app`.
- Character quality suite regression tests pass inside `cargo test --locked -p llm-galgame-app`.
- Single-character and group chat prompts use the shared character mind contract and guarded response path for private reasoning leaks, identity drift, and tool-style response drift.
- The shared Rust AI prompt builder sanitizes embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings in message history and context sections so reusable integrations cannot accidentally reintroduce `[System]`/`[User]`/`[Assistant]` prompt-boundary injection.
- The legacy C# AI path mirrors role-boundary sanitization for bracket, fullwidth, XML/header, attributed XML-like, Markdown role-code-fence, comment-wrapped, punctuation-free heading, and JSON-shaped role spoofing, and redacts provider-error API secrets while the legacy solution remains part of the release gate.
- OpenAI-compatible API configuration debug output and API error surfaces redact API keys, bearer tokens, sensitive custom headers, and provider-echoed secret assignments before logs or frontend error reports can expose them.
- Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields so provider credentials remain runtime-only instead of landing in `settings.json`; bounded atomic replacement rejects symlinked or non-regular settings targets.
- Azure and ElevenLabs TTS provider errors redact token-shaped values, API-key assignments, authorization headers, and provider response bodies before Settings or speech-generation failures expose credentials.
- TTS synthesis runtime logs record spoken-text length metadata instead of raw dialogue, prompt text, or token-shaped content.
- Frontend runtime source ships without `console.log`/`console.debug` debug output, with release-gate coverage to keep production browser/Tauri consoles clean.
- Frontend shell navigation renders icons as escaped text rather than raw HTML, and release verification blocks `v-html` plus direct raw-HTML assignments in runtime source.
- Packaged Tauri desktop builds declare a production Content Security Policy covering local app assets, Tauri asset URLs, blob/data media, HTTPS connections, and localhost dev tooling while blocking object/frame/form surfaces and `unsafe-eval`.
- Web/PWA browser builds declare a matching app-shell Content Security Policy meta tag in both `index.html` and static-hosting fallback output, with release-gate coverage for required asset/connectivity sources and blocked object/frame/form/`unsafe-eval` surfaces.
- Web/PWA dist preparation emits a static-hosting `_headers` file with CSP, nosniff, referrer, and permissions policy headers for Netlify/Cloudflare-style hosts while keeping GitHub Pages fallback compatibility.
- Web/PWA dist preparation emits a static-hosting `_redirects` file with project asset passthrough rules and an SPA fallback rewrite for Netlify/Cloudflare-style hosts.
- Web/PWA dist preparation emits an Azure Static Web Apps `staticwebapp.config.json` with SPA navigation fallback, 404 rewrite, and matching global security headers, all tracked by release verification and artifact manifests.
- Web/PWA dist preparation emits a Vercel `vercel.json` with SPA rewrite and matching global security headers, so Vercel static deployments share the same route and browser-security baseline.
- Prompt-injection detection now covers structured role-control blocks, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, punctuation-free role headings, English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control phrases before scoring, memory writes, relationship deltas, and runtime safety traces consume player text; explicit XML, Markdown fence, and comment-wrapped role-control block bodies are omitted with their markers across Tauri, shared Rust AI, and legacy C# prompt builders.
- Chat, group chat, and quality-suite runtime traces now prove when the character mind contract and creator-pinned knowledge context were applied, including resolved pinned knowledge ref IDs for audit.
- OpenAI-compatible API streaming uses buffered SSE delta parsing so split JSON lines, split UTF-8 content, `[DONE]` markers, and final unterminated lines do not drop character response chunks.
- OpenAI-compatible API streaming rejects provider error frames and malformed SSE data frames before partial stream text can be finalized as successful dialogue.
- Streaming chat errors replace partial assistant bubbles with a stable failure message before surfacing the provider/runtime error.
- The shared AI inference pipeline retries or rejects unsuccessful provider result envelopes before chat, streaming, or workflow LLM callers can consume empty generated text.
- OpenAI-compatible API responses must include non-blank generated text before being reported as successful, including both standard and streaming completions.
- Prompt-injection detection covers player-authored memory writes such as "remember this as official canon" so long-term character knowledge cannot be casually poisoned by dialogue text.
- Player-authored meta-instructions are omitted from character memory writes, and legacy unsafe recent memories are replaced with guarded prompt placeholders before they can influence future replies.
- Prompt-injection text cannot advance local relationship sentiment deltas, so positive words inside meta-instructions cannot silently unlock relationship milestone events.
- Chat runtime responses emit safety trace evidence for player input wrapping, prompt-injection detection, memory guarding, response guarding, stream replacement, and relationship side-channel containment.
- Group chat character responses reuse the same runtime safety trace contract so multi-character scenes expose prompt-injection, response guard, memory guard, and relationship side-channel evidence per reply.
- Group chat per-character generation failures surface as stable system messages, are omitted from future prompt transcripts, and record response length metadata instead of raw dialogue in debug logs.
- Group chat command boundaries trim participant IDs, reject empty or duplicate participant sets, reject inactive sessions, and refuse blank messages before they can advance a multi-character scene.
- Settings-configured AI backends register through async-safe Tauri command paths. OpenAI-compatible API configuration validates local credentials and endpoint shape before activation, while backend auto-selection still requires a real model-level generation probe.
- OpenAI-compatible API configuration rejects blank runtime credentials or model IDs, embedded URL credentials, provider URL query strings/fragments, and non-local plaintext HTTP before a backend can become active.
- Windows ONNX configuration resolves model and tokenizer references under the active project data root, rejects raw filesystem paths, creates a required DirectML execution provider, validates compatible full-sequence causal-LM graphs, and activates the engine only after initialization succeeds. Qwen3.5 hybrid graphs are outside this executor contract.
- Chat sessions now expose a restorable audit report with the latest safety trace, evaluation, story-event decisions, event-rule fingerprints, and triggerable events so author diagnostics survive character switching.
- Quality Suites now export runtime safety trace evidence and guard-note count summaries, and include group chat plus block-body prompt-injection scenarios that require concrete guard notes for input wrapping, response guarding, memory guarding, relationship side-channel containment, and score/event containment.
- Local fallback scoring ignores prompt-injection text for engagement and creativity boosts, so long meta-instructions cannot unlock score-gated story events when model evaluation is unavailable.
- Local fallback scoring now recognizes Chinese, Japanese, and Korean friendly, question, and creative-story signals so offline scoring remains useful when model evaluation is unavailable in international builds.
- Evaluation score parsing clamps overrange, above-scale, and negative model scores before quality reports or event triggers consume them.
- Workflow LLM nodes wrap runtime inputs as untrusted data, guard generated output, and replace blank or guard-only output with stable failure text before it can enter story node results.
- Workflow output safety now covers tool-role/function-call shaped text so generated node output cannot masquerade as a runtime event command.
- Workflow save/load commands resolve JSON files under the active project `workflows/` directory and reject absolute, URI-like, and traversal-shaped paths before touching disk.
- Rhai script commands and direct `ScriptEngine` callers validate payload size and hidden control characters before execution or parsing, condition expressions use shared 2,000-character/control-character validation and run through a read-only Rhai engine, workflow conditions can read relationship/evaluation context variables without mutating story state, desktop run-context previews and Web/PWA workflow previews mirror local variable, flag, signed relationship, emotion, scene, and weighted random-branch behavior for later branches without mutating persistent runtime state, workflow validation catches invalid condition and script state-key config during authoring/import, shared script state keys are normalized to portable save-friendly names, and the shared script engine caps operations, recursion, expression depth, variables, functions, and module imports.
- Character, dialogue, and knowledge loader commands resolve directory references under the active project `characters/`, `dialogue/`, and `knowledge/` roots and reject absolute, URI-like, and traversal-shaped paths before touching disk.
- Character authoring create/delete commands resolve through the active or discovered default project data root, validate portable character IDs before writing or removing JSON files under `characters/`, and deletion removes the runtime character from memory.
- Plugin listing, registration, and removal commands resolve through the active or discovered default project data root, validate portable plugin IDs before touching manifest JSON files under `plugins/`, normalize optional `.rhai` script references under that plugin root, and the Plugin workbench sends the backend manifest contract directly.
- Marketplace import/export commands resolve template references under the active project `templates/` directory, reject raw filesystem paths, and allow built-in catalog entries to import by safe catalog ID.
- Live2D model commands load only project-relative `.model3.json`/`.json` files under the active project data root, and Playtest validates renderer asset paths before handing them to the runtime.
- Engine initialization validates that the selected project root is an existing local directory before loading managers or rebinding active project state.
- Engine project reloads stage character, dialogue, and knowledge managers before activation, replace rather than merge prior content, and clear chat, scene, script, and event runtime state even when reloading the same project root.
- Story event rules are versioned project assets under `events/*.json`. Chat scoring, workflow execution, local Web/PWA previews, and Quality Suites consume the same catalog, with validated score/relationship/evaluation thresholds, optional character scope, repeat behavior, stable rule/catalog fingerprints, and atomic author hot reloads.
- Checked-in score-gate workflow fixtures prove conversation evaluation can drive visual workflow branches and score-aware story-event unlocks.
- Chat runtime responses emit story-event trigger decisions with actual relationship values, score metrics, evaluation counts, stable SHA-256 event-rule fingerprints, and blocker reasons.
- Manual Chat scoring returns an atomic evaluation report with matching story-event trigger decisions and triggerable events so authors can debug score gates without waiting for periodic evaluation.
- Quality Suites reuse the same story-event decision contract as live chat, pin event-rule snapshot fingerprints, and use the same guarded workflow story-output finalization as Workflow LLM nodes so offline QA and runtime event audits stay aligned.
- Workflow Run traces expose evaluation metrics, thresholds, score sources, event trigger state, and blocker reasons for author debugging.
- Workflow canvas nodes show compact run badges for executed, pass/fail, blocked, completed, and waiting-choice states.
- Workflow Run preview context lets authors simulate scores, relationship values, evaluation counts, already-triggered events, and workflow state-node effects without live chat/model calls or persistent state mutation.
- Workflow Run preview context clamps author-provided scores and relationship values in both the editor payload and Rust execution path before score-gated branches consume them.
- Workflow Run preview presets cover unlock, low-score block, and repeat-trigger block branches for quick score-gate QA.
- Workflow Run reports graph coverage and unvisited nodes so authors can see which score/story branches still need testing.
- Workflow Run preset matrix executes all score-gate preview presets and merges graph coverage to confirm branch coverage quickly.
- Quality Suites can now pin workflow branch coverage snapshots, including the checked-in score-gate fixture's unlock, low-score, and repeat-trigger branches.
- Quality Suite lists, reports, and exports show versioned audit evidence with run metadata, suite source paths, SHA-256 suite fingerprints, build commit ids, failed-scenario ids, category summaries, safety-signal counts, finalized guarded workflow output text, and workflow coverage summaries for release QA, customer review, and branch-coverage audits.
- Quality suite schema validation rejects contradictory expectations such as out-of-range score bounds or events marked both expected and forbidden.
- Character prompts pin creator-declared knowledge references before keyword search results so core lore stays stable.
- Character content loading accepts single-object JSON files, legacy sprite field names, and optional renderer asset fields.
- Core sample characters Sakura, Luna, and Kenji ship with checked-in portrait and sprite SVG assets in both Web and bundled Tauri data roots.
- Audio Manager now controls real browser/Tauri audio elements for BGM, ambient loops, and SFX previews, with persisted track lists, per-track gain, and master/channel mixer state.
- C# legacy solution exits successfully with `dotnet test LLMAssistant.sln --no-restore`.
- Locale JSON files validate across project data and frontend fallback directories, including key coverage and Web/PWA fallback parity.
- i18n locale commands validate portable locale IDs before loading or listing JSON files under the active project `locales/` directory.
- Live2D remains on `pixi-live2d-display@0.4.0`; its transitive `gh-pages` dependency is pinned to the safe `6.3.0` line through npm overrides.
- Rust desktop dependencies are pinned through `rust-engine/Cargo.lock`, and the compiler/linter/formatter toolchain is fixed to `nightly-2026-07-03`, for reproducible Tauri builds.
- Tauri desktop packaging configuration declares Windows MSI/NSIS targets, a pinned WiX upgrade identity, installer metadata, icons, WebView2 bootstrap behavior, and bundled sample project data, all checked by the release verifier.
- Installed Tauri builds discover bundled sample `data/` resources at startup and bind them as the default project root when no development data root is available.
- Windows release-candidate audits query MSI/NSIS metadata, the stable MSI upgrade identity, and Authenticode status for both installers plus the extracted application, administratively extract the MSI, compare every bundled project file by SHA-256, and run the production executable's headless runtime verifier with build-commit provenance.
- Analytics logs, cloud-sync manifests, and generated system/API TTS assets are written under the active project data root with sanitized filenames for portable installed desktop builds.
- Rust and legacy C# asset managers validate project-relative asset paths before file access, and save managers validate save IDs before save/load/delete/list operations, so local asset and save APIs cannot escape the active project data root through traversal-shaped input.
- Story-event rules now execute bounded typed actions for scene, dialogue, ending, and script-flag unlocks. Applied effects live in a project-scoped, fingerprinted progress ledger; chat and real workflow runs share the executor while Quality Suites and author previews remain side-effect free.
- Rust runtime saves use the backward-compatible `monogatari-game-save/v3` schema to restore scene history, dialogue cursor/local state, typed Rhai variables, character emotion/relationships/full memory, chat history, evaluations, safety traces, triggered-event state, and persistent story unlocks. V1/v2 saves migrate known triggered events into the new progress ledger. Quick-save and auto-save use bounded stable slots with staged overwrite recovery instead of accumulating random files, and save reads/writes enforce a 32 MiB limit.
- Settings Cloud Sync status now consumes the backend manifest contract directly, showing local save file counts, pending upload/download work, cross-device conflicts, and remote preflight readiness while keeping sync tokens runtime-only.
- Project export emits a versioned manifest with engine/build provenance, content category summaries and fingerprints, an explicit whole-package SHA-256 fingerprint algorithm, file inventory, per-file SHA-256 and legacy MD5 checksums, generated asset coverage, and redacted sensitive settings. Installed apps can embed that manifest and its exact sanitized content in a verified `.monogatari` package for transactional handoff and recovery.
- The current human and Agent development-test baseline, reproduction commands, acceptance levels, and environment-dependent release boundaries are recorded in `docs/DEVELOPER_TEST_HANDOFF.md`.
- Release artifact manifests can be generated with `node scripts/create-release-manifest.mjs` to capture Web/PWA and desktop installer artifact paths, SHA-256 checksums, checked-in Quality Suite source evidence plus aggregate suite-set fingerprints, checked-in workflow source evidence plus aggregate workflow-set fingerprints, checked-in project content source evidence plus aggregate and per-category content-set fingerprints, checked-in release channel policy metadata, git source-state evidence, missing installer expectations, and verified installer signing evidence.
- One-command release verification passes with `node scripts/verify-release.mjs`, including all quality suite files, Rust core/state-key tests, Rust AI prompt/API/pipeline tests, Rust scripting and asset management tests, legacy C# AI prompt/API invariants, AI backend config, engine project root, asset/save-manager, script command, i18n locale, workflow command, content loader, character manager, plugin manager, marketplace, Live2D model, and TTS output/error/log-privacy invariants, structured role-block prompt-injection regressions, renderer asset contract checks, pinned knowledge-ref checks, locale coverage, frontend UI text artifact scanning, cloud-sync status contract checks, frontend source invariants, frontend route/sidebar coverage, Tauri packaging preflight, root and subpath Web/PWA builds, Web/PWA dist asset checks, release artifact manifest checks, and preview route smoke checks.
- Commercial release gates are tracked in `docs/RELEASE_CHECKLIST.md`.
- Runtime evidence, blockers, selection order, and staged platform adaptation are tracked in `docs/INFERENCE_BACKEND_MATRIX.md`.

## Architecture

```
monogatari/
+-- rust-engine/           # Rust backend (Tauri desktop app)
|   +-- crates/
|   |   +-- core/          # EventBus, ServiceLocator, GameClock, error handling
|   |   +-- ai/            # Backend planner, API/ONNX engines, inference pipeline
|   |   +-- game/          # Characters, Dialogue, Knowledge, Scenes, Script parser
|   |   +-- assets/        # Asset management, save/load
|   |   +-- scripting/     # Rhai scripting engine
|   |   +-- authoring/     # Headless project services shared by Tauri and agents
|   |   +-- mcp-server/    # Standard schema-backed stdio Agent transport
|   |   +-- tauri-app/     # Tauri commands (AI, Chat, Dialogue, Workflow, etc.)
|   +-- data/              # Example characters, dialogues, knowledge, events, scenes, assets
+-- frontend/              # Vue 3 + Vite + Pinia
|   +-- src/
|   |   +-- views/         # 21 production views including chat, story runtime, editors, galleries, analytics, and quality gates
|   |   +-- components/    # Live2DCanvas, search, toasts, progress, dialogs, and shared UI
|   |   +-- stores/        # Pinia game store
|   |   +-- styles/        # Design system (CSS variables, components)
+-- .agents/skills/        # Agent-discoverable visual novel authoring workflow
+-- scripts/               # Module matrix, release, package, and installer verification
+-- src/                   # C# implementation (legacy, SDL2-based)
+-- tests/                 # C# tests
```

## Agent Authoring

Agents can invoke `$author-visual-novel` from the repository Skill at `.agents/skills/author-visual-novel`. The Skill authors the canonical project graph in dependency order, requires Quality Suite evidence, and validates output through the same engine and release contracts used for human-authored projects.

The shared `llm-authoring` crate also defines `monogatari-agent-project-transaction/v1`: a JSON-only multi-file plan/apply contract with create-or-exact-SHA preconditions, portable path containment, candidate validation, and rollback. Its headless core-runtime loader owns real character/dialogue/knowledge manager loading plus shared scene/ending models, bounded catalog loading, inferred background-scene discovery, and cross-reference checks for both Tauri and Agent transports. `monogatari-mcp` exposes that core through seven standard stdio tools with a startup-fixed project root, read-only core/delivery validation, reviewed plan fingerprints, process-level project leases, document-level inspection, and `core_runtime` candidate application.

Use the module matrix for narrow feedback while authoring:

```bash
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs --module frontend-unit --module rust-authoring --module rust-mcp --module rust-game --module rust-tauri
```

Run `node scripts/verify-release.mjs` before describing generated content as a deliverable project. The current module coverage and remaining graph/runtime extraction are tracked in `docs/MODULE_VERIFICATION.md`.

## Quick Start

### Prerequisites

- Rust 1.70+ (with MSVC toolchain on Windows)
- Node.js 18+
- npm

### Install and Run

```bash
# Clone
git clone https://github.com/SakalioLabs/Monogatari.git
cd Monogatari

# Install frontend dependencies
cd frontend
npm install
cd ..

# Build and run the Tauri app
cd rust-engine/crates/tauri-app
cargo tauri dev
```

### Production Build

Run the automated pre-release gate first:

```bash
node scripts/verify-release.mjs
```

This verifies JSON assets, versioned story event catalogs and fingerprints, checked-in workflow event references, renderer asset contracts for characters and scenes, pinned character knowledge refs, all quality suite files, workflow branch coverage snapshots, locale coverage, sensitive token patterns, frontend UI text artifacts, frontend source invariants, legacy C# AI prompt/API invariants, AI backend config, engine project root, asset/save-manager, script command, i18n locale, workflow command, content loader, character manager, plugin manager, marketplace, Live2D model, and TTS output/error/log-privacy invariants, frontend route/sidebar coverage, Tauri desktop packaging configuration, Tauri mobile deployment preflight, Rust core/state-key, AI, scripting, game, assets, and Tauri checks and tests, root and subpath Web/PWA builds with bundle budgets, Web/PWA event assets, release artifact manifest checks, preview route smoke checks, frontend audit, and legacy C# tests.

```bash
cd frontend
npm run build:web

cd rust-engine/crates/tauri-app
cargo tauri build
```

On Windows, audit the resulting MSI and NSIS before distribution:

```powershell
node scripts/verify-windows-installers.mjs --check
```

This requires valid Authenticode signatures. Internal or alpha QA may inspect an unsigned release candidate explicitly with `--allow-unsigned`; that flag does not make the audit release-ready and must not be used for stable or beta publication. When installers are present, `verify-release.mjs` runs this audit automatically and derives the unsigned policy from `MONOGATARI_RELEASE_CHANNEL`.

The installed production executable also supports a windowless resource and runtime check. The report path must be an absolute `.json` path:

```powershell
& 'C:\Program Files\Monogatari\llm-galgame-app.exe' --verify-installation 'C:\Temp\monogatari-installation-report.json'
```

After Web/PWA and installer builds are available, generate the distributable checksum manifest:

```bash
node scripts/create-release-manifest.mjs --channel=stable
```

The manifest is written under `release/` and records artifact SHA-256 hashes, Quality Suite source paths/fingerprints, aggregate Quality Suite and workflow source set fingerprints, aggregate plus per-category project content source set fingerprints, expected Windows MSI/NSIS installer presence, release channel policy, git source-state evidence, and installer signing evidence for GitHub Release handoff. Stable and beta final manifests require Windows MSI/NSIS installers plus verified signing evidence; `--allow-missing-installers` is only a policy-gated release-preflight exception. Final manifest writes also require a clean tracked git worktree by default, with `--allow-dirty-worktree` reserved for internal diagnostic manifests.

Installer signing evidence is stored next to each installer as `<installer>.sig.json`:

```json
{
  "schema": "monogatari-signature-evidence/v1",
  "artifact_sha256": "<installer sha256>",
  "status": "verified",
  "subject": "SakalioLabs",
  "verifier": "signtool verify /pa",
  "signed_at": "2026-07-08T00:00:00.000Z",
  "verified_at": "2026-07-08T00:00:00.000Z"
}
```

For static hosting under a subpath, set `VITE_BASE_PATH` before building:

```powershell
cd frontend
$env:VITE_BASE_PATH='/Monogatari/'
npm run build:web
Remove-Item Env:VITE_BASE_PATH
```

The web build emits `dist/404.html` for SPA fallback, `dist/.nojekyll` for GitHub Pages, `dist/_headers` and `dist/_redirects` for Netlify/Cloudflare-style hosts, `dist/staticwebapp.config.json` for Azure Static Web Apps, `dist/vercel.json` for Vercel, PWA assets, install/maskable icons, copied `data/assets` project sample assets, and `project-assets.json` so the service worker can precache sample renderer assets under the configured base path.

## Usage

### Character Test (Core Feature)

1. Configure your AI backend in Settings (API key, model, base URL)
2. Open Character Test from the dashboard
3. Select a character to talk with
4. Chat freely - the character responds in personality using LLM
5. Every 5 messages, the system evaluates your conversation quality
6. High scores unlock special events, scenes, and dialogue branches

### Workflow Editor (No-Code)

1. Open Workflow Editor from the dashboard
2. Drag nodes from the palette: Start, Dialogue, Choice, Condition, LLM Generate, Evaluation, Scene Change, Trigger Event, Relationship, Set Variable, Set Flag, End
3. Connect nodes to create story flows
4. Configure node properties in the side panel
5. Validate the graph before saving or exporting workflow JSON

### Scene Assets

1. Open Scene Assets from the sidebar
2. Review project scenes from `scenes/*.json` and backgrounds from `assets/backgrounds`
3. Fix any missing or invalid background paths shown by diagnostics
4. Set a scene active before testing Playtest or saving runtime state

### Project Control

1. Open Settings from the sidebar
2. Set the project data path, title, target FPS, and content directory mappings
3. Review readiness diagnostics for characters, dialogue, knowledge, story events, scenes, assets, and saves
4. Save `settings.json`, configure the AI backend with the runtime API key, then initialize the runtime
5. Use Export Package for a complete `.monogatari` handoff, or Import Package to verify and install a package into a new project directory

### Node Types

| Node | Category | Description |
|------|----------|-------------|
| Start | Flow | Entry point of the workflow |
| Dialogue | Content | Show dialogue text from a character |
| Choice | Content | Present choices to the player |
| Condition | Flow | Branch based on a condition |
| LLM Generate | AI | Generate text using the LLM |
| Evaluation | AI | Evaluate conversation quality |
| Scene Change | Content | Change the background scene |
| Trigger Event | Flow | Trigger a special game event |
| Relationship | Character | Modify relationship score |
| Narration | Content | Display narrator text or inner monologue |
| BGM | Media | Control background music playback |
| SFX | Media | Play a sound effect |
| Wait | Flow | Pause workflow execution for a duration |
| Random Branch | Flow | Randomly select one of multiple branches |
| Sub Workflow | Flow | Execute another workflow as a subroutine |
| Camera | Media | Control camera position, zoom, and effects |
| Shake | Media | Screen shake effect for dramatic moments |
| Change Emotion | Character | Change a character's emotion |
| Set Variable | Logic | Set a game variable |
| Set Flag | Logic | Set a game flag |
| End | Flow | End of the workflow |

## Configuration

### API Mode (OpenAI-compatible)

```json
{
  "ai": {
    "provider": "api",
    "api": {
      "base_url": "https://api.openai.com/v1",
      "api_key": "",
      "model": "gpt-4o-mini"
    }
  }
}
```

API keys are runtime-only. Set them through Settings when configuring the AI backend; `settings.json` saves retain only non-secret provider metadata.

### ONNX Mode (Local)

ONNX configuration is project-scoped and uses the linked ONNX Runtime DirectML executor on Windows. It requires a compatible full-sequence causal-LM graph with standard tokenizer metadata and float32 logits; there is no implicit CPU fallback. Qwen3.5 uses hybrid attention, convolution, and recurrent state inputs and is therefore blocked on this executor. Use the verified WebGPU profile or a generation-probed OpenAI-compatible local/service backend for Qwen3.5. See [Inference Backend Matrix](docs/INFERENCE_BACKEND_MATRIX.md).

```json
{
  "ai": {
    "provider": "onnx",
    "onnx": {
      "model_path": "models/model.onnx",
      "tokenizer_path": "models/tokenizer.json",
      "use_directml": true
    }
  }
}
```

## Data Format

### Example Characters and Content

- **Sakura** (cheerful):: Nature-loving artist. Park walk dialogue with branching paths.
- **Luna** (thoughtful): Poetic stargazer. Stargazing dialogue with constellation lore.
- **Kenji** (honorable): Martial artist poet. Dojo visit dialogue with training themes.
- **Yuki** (mysterious): Ancient library guardian who speaks in riddles. Library encounter with branching paths.
- **Hiro** (enthusiastic): Young inventor beneath the observatory. Workshop dialogue with invention themes.
- **Aoi** (gentle): Village healer and herbal medicine expert. Clinic visit dialogue with 3 branching paths, herb lore knowledge.
- **Rin** (energetic): Chef with noodle_and_soul dialogue (12 nodes, 6 endings), Springtown cuisine knowledge.
- **Taro** (serene): Master woodcarver with woodcarver_workshop dialogue (12 nodes, 6 endings), woodcarving tradition knowledge.
- **Emi** (shy): Aspiring writer with writers_retreat dialogue (12 nodes, 7 endings), writing process knowledge.
- **Takeshi** (observant): Traveling photographer with through_the_lens dialogue (12 nodes, 7 endings), Springtown archive knowledge.
- **Nori** (serene): Postmaster with post_office_tales dialogue (12 nodes, 8 endings), Springtown history knowledge.
- **Sora** (focused): Astronomer with observatory_night dialogue (12 nodes, 8 endings), constellation knowledge.
- **Hana** (serene): Tea shop owner with Whispering Leaf dialogue (13 nodes, 8 endings), tea blends knowledge.
- **Kai** (contemplative): Wandering musician with cafe encounter dialogue (12 nodes, 5 endings), traveler lore knowledge.
- **Mio** (cheerful): Festival organizer with Starlight Festival dialogue (15 nodes, 4 endings), festival lore knowledge.
- **Springtown**: Shared world with cherry blossom park, observatory, dojo, the Great Library, and the Inventor's Workshop.

### Character

```json
{
  "id": "sakura",
  "name": "Sakura",
  "description": "A cheerful girl who loves nature",
  "background": "Grew up surrounded by cherry blossoms...",
  "personality": {
    "openness": 0.8,
    "extraversion": 0.7,
    "agreeableness": 0.9,
    "speech_style": "cheerful and friendly"
  },
  "live2d_model_path": "live2d/sakura/sakura.model3.json"
}
```

### Dialogue

```json
{
  "id": "meeting",
  "title": "First Meeting",
  "start_node_id": "start",
  "nodes": {
    "start": {
      "speaker_id": "sakura",
      "text": "Hello there!",
      "choices": [
        { "text": "Hi!", "next_node_id": "response", "relationship_changes": { "sakura": 0.2 } }
      ]
    }
  }
}
```

### Scene

```json
{
  "id": "sakura_park",
  "name": "Sakura Park",
  "background_path": "assets/backgrounds/sakura_park.svg",
  "bgm_path": null,
  "weather": "spring",
  "time_of_day": "day",
  "tags": ["outdoor", "calm", "demo"]
}
```

## Development Roadmap

### Completed

- [x] Core engine architecture (EventBus, ServiceLocator, GameClock)
- [x] AI inference pipeline (API + linked project-scoped DirectML executor + conservative backend readiness planner)
- [x] Character system (personality, memory, emotions, relationships)
- [x] Dialogue system (branching, choices, flags, scripts)
- [x] Knowledge base (keyword search, category/tag indexing)
- [x] Scripting engine (Rhai-based)
- [x] Save/load system
- [x] Character and ensemble authoring test consoles
- [x] Conversation evaluation and scoring
- [x] Event trigger system (relationship milestones and project-authored progression events)
- [x] Visual workflow editor (drag-and-drop)
- [x] Frontend streaming chat integration via Tauri events
- [x] Streaming evaluation and event notifications (`chat-evaluation`, `chat-event-decisions`, `chat-events`)
- [x] Chat session lock optimization for slower LLM requests
- [x] Commercial workbench UI refresh with 21 engine-oriented sidebar destinations
- [x] Browser preview fallback for non-Tauri UI review
- [x] Frontend supply-chain audit remediation (Vite 8 + Live2D transitive override)
- [x] Rust lockfile policy for reproducible Tauri builds
- [x] Release checklist for packaging and QA gates
- [x] Workflow editor connection hit testing improvement
- [x] Workflow import/export validation and editor diagnostics panel
- [x] Scene/background asset management with catalog validation and active scene selection
- [x] Project configuration panel with settings persistence, path readiness, and runtime initialization
- [x] Story mode text, layout, and playback settings cleanup
- [x] Live2D rendering (PixiJS)
- [x] Tauri desktop application
- [x] Professional UI design system
- [x] Multi-character simultaneous group chat
- [x] TTS integration scaffold with voice assignment
- [x] 21 workflow node types with execution handlers for all types
- [x] Async-safe chat evaluation (blocking_read fix)
- [x] Cargo dev profile optimization for faster builds
- [x] Title Screen with cinematic animated particle effects and quick-access menu
- [x] Visual Review view with scene/character art collection and preview modal
- [x] Backlog viewer with conversation history replay and role-based filtering
- [x] Comprehensive i18n system with 280+ translation keys across all views
- [x] Full Simplified Chinese (zh-CN), Japanese (ja-JP), Korean (ko-KR) locale support
- [x] i18n-integrated sidebar with 22 navigation items using t() function
- [x] Quality Suites workbench for offline character stability and prompt-injection regression checks
- [x] Quality Suite workflow branch coverage snapshots for score-gated story QA
- [x] Quality Suite workbench and JSON export with stable audit summaries for QA evidence handoff
- [x] Runtime trace evidence for character mind contract and pinned knowledge context anchoring
- [x] Web/PWA distribution baseline with manifest, service worker, and offline fallback
- [x] Mobile Web/PWA shell readiness gate for safe-area viewport, install metadata, and compact Tauri shell limits
- [x] Responsive Web/PWA shell verification for built 375px mobile and 768px tablet layout signals
- [x] Tauri mobile deployment preflight gate for Android/iOS command readiness and Vite mobile dev host binding
- [x] Checked-in score-gate workflow fixture with backend execution regression for evaluation-driven story unlocks
- [x] Workflow Run score/event diagnostics for author-visible trigger debugging
- [x] Workflow canvas run badges for score-gated graph debugging
- [x] Workflow Run preview context for model-free score-gate branch testing
- [x] Workflow Run preview presets for unlock, low-score block, and repeat-trigger block QA
- [x] Workflow Run graph coverage summary with unvisited node reporting
- [x] Workflow Run preset matrix for merged score-gate branch coverage

### In Progress

- [x] Distribution channel policy and installer signing evidence gates
- [ ] Production installer code-signing credentials and final signed installer publication
- [x] Engine-level gamification removed; progression remains an authored project concern
- [ ] Mobile deployment (Tauri mobile)

### Planned

- [x] Voice synthesis integration (Windows SAPI TTS with emotion-based speech rate)
- [x] Music/ambient sound management
- [x] Multi-language support (i18n scaffold with locale loading and translation)
- [x] Plugin system for custom node types (scaffold with register/list/remove)
- [x] Cloud save sync with local manifest and checksum tracking
- [x] Analytics dashboard with engagement metrics and JSON export
- [x] Dialogue Editor view with visual branching node tree and inline editing
- [x] Verified `.monogatari` project package export, inspection, and transactional import
- [x] Knowledge Base Manager view with CRUD and filtering
- [x] Professional Character Editor with 5 tabs and radar chart
- [x] Frontend data sync with rust-engine content
- [x] Template marketplace scaffold (Rust backend + MarketplaceView frontend)
- [x] Plugin management frontend UI with register/list/remove
- [x] Cloud sync settings integration with project-scoped manifest status, push/pull preflight, and conflict counters
- [x] Multi-language locale files (zh-CN, ja-JP, ko-KR)
- [x] Enhanced group chat with streaming and emotion display
- [x] Release checklist document for packaging and QA gates
- [x] Mobile-responsive CSS with bottom tab navigation
- [x] Audio manager playback controls with BGM/ambient/SFX tracks, persisted lists, and master/channel mixer
- [x] Enhanced prompt engineering for character AI roleplay quality
- [x] GLTF/GLB 3D model loading with Three.js GLTFLoader, OrbitControls, animation playback
- [x] i18n composable with nested key support and locale persistence
- [x] i18n composable upgraded with nested key support and localStorage persistence
- [x] Sidebar navigation organized into 21 authoring, preview, quality, and project destinations
- [x] Title Screen, Visual Review, Backlog viewer views
- [x] Full i18n with 280+ keys and 4 locale files (en, zh-CN, ja-JP, ko-KR)
- [x] Project-authored progression through story-event conditions and workflow gates
- [ ] Mobile deployment (Tauri mobile)

## Tech Stack

- **Backend**: Rust, Tauri 2.x
- **Frontend**: Vue 3, TypeScript, Vite, Pinia
- **AI**: Transformers.js WebGPU, backend capability/readiness planner, linked ONNX Runtime DirectML for compatible Windows graphs, OpenAI-compatible local or managed services
- **Scripting**: Rhai
- **Rendering**: PixiJS, Live2D Cubism SDK
- **Desktop**: Tauri (WebView2 on Windows, WebKit on macOS/Linux)

## License

MIT
