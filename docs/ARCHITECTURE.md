# Monogatari Architecture

## Overview

Monogatari is a low-code game development engine built with Rust (Tauri 2.x) and Vue 3 + TypeScript. The workbench authors project content and validates it against the same runtime contracts used by Web/PWA and Windows packages.

## System Architecture

```
+--------------------------------------------------+
|       Human Frontend       |     Agent Clients     |
| Vue 3 workbench / PWA      | Skill / standard MCP  |
+--------------------------------------------------+
|       Tauri IPC Bridge     |   MCP stdio adapter   |
+--------------------------------------------------+
|                  Rust Backend                      |
|  25 Command Modules | Thin Tauri Adapters          |
+--------------------------------------------------+
|        Headless Authoring and Core Crates          |
| authoring | core | ai | game | assets | scripting |
+--------------------------------------------------+
|       Inference Planner and Runtime Profiles       |
| WebGPU | Local services | DirectML | Managed API   |
+--------------------------------------------------+
```

## Crate Structure

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `core` | Shared infrastructure | EventBus, ServiceLocator, GameClock |
| `ai` | AI inference and selection | InferencePipeline, BackendPlan, APIEngine, ONNXEngine |
| `game` | Game logic | CharacterManager, DialogueManager, KnowledgeBase, SceneManager |
| `assets` | File management | AssetManager, SaveManager |
| `scripting` | Rhai scripting | ScriptEngine |
| `authoring` | Headless human/agent project operations | ProjectConfigState, JSON catalog, Agent transaction planner/applier |
| `mcp-server` | Standard agent transport over stdio | Five schema-backed project tools, read-only default, write lease |
| `tauri-app` | Tauri commands | AppState, 25 command modules |

Script execution is treated as bounded authoring logic. Tauri script commands validate payload size and hidden control characters before execution or DSL parsing, and `ScriptEngine::execute` repeats that source validation for workflow and future plugin callers. Condition expressions use shared 2,000-character/control-character validation and run through a separate read-only Rhai engine that can inspect variables, flags, and temporary workflow context values without registering state mutation functions; workflow validation applies the same condition rules before imported graphs run. Workflow condition nodes expose relationship, evaluation score, and evaluation-count context as temporary scope variables so branches can react to chat state or author preview presets without writing story state. Desktop run-context previews snapshot script state and mirror variable, flag, relationship, emotion, and scene node effects in a per-run local state bag; browser-only Web/PWA workflow previews mirror the same state transitions plus weighted random-branch behavior so exported builds can exercise later condition branches, event gates, and trace diagnostics without touching persistent save data. Script variables and flags use shared portable state key validation before Rhai functions, workflow validation, workflow nodes, dialogue scripts, or save loading can write them, keeping persisted state names stable across desktop, Web/PWA, and exported project packages. The shared engine caps Rhai operations, recursive calls, expression depth, variable count, function definitions, and module imports so custom game logic cannot hang the workbench through runaway loops or recursion.

## Data Flow

1. **Developer starts a character test** from ChatView or GroupChatView
2. **Runtime builds context** from character personality, knowledge base, conversation history
3. **Backend planner reports candidates** from host detection and completed exact-model probes; detection alone never makes a backend selectable
4. **Target runtime generates output** through Transformers.js WebGPU, a generation-verified OpenAI-compatible local/managed service, or the linked DirectML executor for a compatible full-sequence ONNX graph
5. **Response streams** directly in the browser or through Tauri events (`chat-chunk`, `chat-complete`)
6. **Evaluation triggered** every 5 messages - scores friendliness, engagement, creativity
7. **Events triggered** based on cumulative scores and relationship milestones

## Frontend Architecture

- **Router**: 22 routes with lazy-loaded views
- **State**: Pinia store for game state (saves, scenes, relationships)
- **Isolated Tests**: Vitest covers pure authoring and access contracts, renderer fallback selection, the browser workflow preview state machine, Pinia async state, and shared Vue component behavior in Happy DOM; production builds remain a separate package contract.
- **Browser Workflow Preview**: `workflowPreview.ts` owns graph validation, bounded execution, run-context normalization, per-run state mirrors, condition evaluation, event decisions, coverage, and deterministic random injection. `WorkflowEditor.vue` supplies project catalogs and renders traces; unsupported browser condition syntax stops the preview instead of silently selecting a story branch.
- **i18n**: Nested key resolution with localStorage persistence (zh-CN, ja-JP, ko-KR)
- **Tauri Bridge**: Browser-compatible `invokeCommand()` with fallback for non-Tauri environments
- **Web Distribution**: Production browser builds register a service worker, manifest, and offline fallback; Tauri runtime disables service worker registration.
- **Renderer Asset Pipeline**: Playtest resolves scene and character assets through a shared frontend resolver. Character staging prefers Live2D models, then GLB/GLTF 3D models, then 2D sprites or portraits, and falls back to a generated Three.js placeholder for assetless characters. The real animated Fox GLB fixture verifies arbitrary-unit normalization, responsive camera framing, texture color space, animation, and bounded framebuffer probes.
- **Runtime Log Hygiene**: Production frontend source avoids `console.log` and `console.debug` debug output; release verification scans `frontend/src` while preserving warning/error reporting for real failures.
- **Release Orchestration**: `verify-release.mjs` remains the single product gate, while importable source-invariant checks live under `scripts/lib` so security and integration contracts do not own the implementation layout of large Vue views.
- **DOM Injection Boundaries**: The frontend shell renders navigation symbols through normal text bindings, and release verification rejects `v-html` plus direct raw-HTML assignments in runtime source.
- **Desktop CSP**: Packaged Tauri WebViews use an explicit Content Security Policy that allows local app assets, Tauri asset URLs, blob/data media, embedded GLB texture blob fetches, HTTPS provider connectivity, and localhost dev tooling while blocking object, frame, form, wildcard default, and `unsafe-eval` surfaces.
- **Web/PWA CSP**: Static browser builds ship a matching `index.html` Content Security Policy meta tag, copied into the `404.html` SPA fallback. It blocks JavaScript `unsafe-eval`, object, frame, form, and wildcard defaults while explicitly allowing `wasm-unsafe-eval` for ONNX Runtime Web and `blob:` fetches for Three.js embedded GLB textures.
- **Static Hosting Headers**: Web/PWA dist preparation also emits a `_headers` file for Netlify/Cloudflare-style hosts with CSP, `X-Content-Type-Options`, `Referrer-Policy`, and `Permissions-Policy` headers; release verification checks both the generated file and the source script so response-header capable hosts get stronger browser enforcement without breaking GitHub Pages fallback.
- **Static Hosting Redirects**: Web/PWA dist preparation emits a `_redirects` file for Netlify/Cloudflare-style hosts with explicit project asset passthrough rewrites before the final `index.html` SPA fallback, keeping static renderer assets and locale files reachable on hosts that evaluate redirects before normal file serving.
- **Azure Static Web Apps Config**: Web/PWA dist preparation emits `staticwebapp.config.json` with SPA navigation fallback to `index.html`, explicit asset/service-worker exclusions, a `404.html` rewrite, and matching global security headers so Azure Static Web Apps deployments share the same browser enforcement and route behavior.
- **Vercel Static Config**: Web/PWA dist preparation emits `vercel.json` with a static SPA rewrite to `index.html` and matching security headers, and release verification rejects missing headers, external rewrite targets, or missing fallback routing.

## Agent Authoring Boundary

The repository-level `.agents/skills/author-visual-novel` Skill gives agents a discoverable authoring workflow over the same versioned project data consumed by desktop and Web/PWA runtimes. It requires dependency-ordered content generation, portable IDs and paths, Quality Suite evidence, mirrored built-in data roots, and the real release gate; it does not define a second story schema.

`llm-authoring` is the transport-neutral application boundary. It owns project settings inspection and scrubbed atomic persistence, strict portable project paths, bounded JSON catalog inspection, exact byte and semantic content fingerprints, rollback-capable content mutations, and a headless core-runtime loader; transports call these services instead of owning duplicate filesystem or loader rules. The loader creates the real character, dialogue, and knowledge managers, rejects duplicate IDs, and validates character relationships, pinned knowledge, dialogue speakers, and dialogue relationship targets. Tauri initialization consumes the loaded managers directly. Its `monogatari-agent-project-transaction/v1` protocol adds bounded multi-file JSON plans, exact missing/SHA-256 preconditions, duplicate and case-collision rejection, candidate validation, reverse-order rollback, structured results, and stable error codes.

`monogatari-mcp` is the standard stdio adapter over that core. Startup fixes one canonical project root, stdout carries MCP frames only, and tools cannot select another filesystem root. `inspect_project`, `list_project_json`, `read_project_json`, `plan_transaction`, `validate_project`, and `validate_delivery` are read-only. Inspection reports document acceptance; core validation returns the complete shared runtime report; delivery validation adds declared renderer/audio existence and placeholder evidence without writing. `apply_transaction` is unavailable unless startup includes `--allow-write`, requires the exact fingerprint returned by a freshly reviewed plan, and serializes in-process reads and writes. Read-only server processes share a project-level operating-system lease; a write-enabled process requires it exclusively, so another process cannot race or inspect a staged multi-file candidate. Candidate application and read-only validation share real character/dialogue/knowledge loading; strict scene/ending/Story Event/Workflow/Quality Suite loading; Workflow graph and Quality expectation checks; inferred background-scene discovery; and cross-catalog reference checks. Package archive inspection, Quality scenario execution, and rendered-experience validation remain explicit higher gates. The independently runnable `rust-mcp` gate uses the official Rust MCP SDK client against a real child process to prove handshake, schemas, reads, core/delivery validation, planning, default refusal, successful writes, reference rejection, and rollback.

Project-package commands use a thin Tauri adapter under `project_archive/commands.rs`. That adapter owns application-state reads, blocking ZIP task dispatch, staged runtime reload validation, and the final non-overwriting directory commit. Pure portable-path policy lives in `project_archive/path_validation.rs`, while `project_archive/manifest.rs` owns package models, schema/version checks, inventory bounds, deterministic fingerprints, sorting, and path-topology semantics. The parent archive module owns streaming ZIP I/O, checksums against real entries, extraction containment, and atomic package replacement. Each pure policy module has direct tests in addition to archive round-trip and tamper coverage.

## Installed Desktop Verification

The Windows release audit treats installer output as a separate trust boundary. It reads MSI properties through the Windows Installer API, reads NSIS PE version metadata, records SHA-256 hashes and Authenticode status for both formats, and administratively extracts the MSI into a uniquely owned temporary directory. The extracted `data/` tree must match the checked-in source tree exactly by portable relative path, byte length, and SHA-256 hash before the application executable is trusted.

Before Tauri initializes a window, the production binary recognizes `--verify-installation <absolute-report.json>`. This path resolves bundled resources beside the executable, rejects secret-bearing settings and non-regular filesystem entries, loads characters, dialogues, knowledge, and events through the real runtime managers, validates scenes, endings, workflows, locales, and Quality Suites, and rebuilds the complete project export inventory fingerprint. Bundled project configuration must verify without warnings; inference readiness is a separate exact-model generation gate. The atomic JSON report includes engine version and build Git commit, allowing a clean-worktree installer audit to reject stale binaries that were produced from another revision. Stable and beta channels require valid Authenticode signatures; internal, alpha, and nightly channels may explicitly audit unsigned candidates without marking them release-ready.

## i18n Locale Boundaries

i18n backend commands treat locale values as portable locale IDs rather than filenames or paths. Locale IDs resolve to direct JSON files under the active project `locales/` directory, listed locales are filtered through the same validator, and slashes, dots, URI/drive-style prefixes, empty hyphen segments, control characters, and non-portable characters are rejected before filesystem access.

## Live2D Model Boundaries

Live2D backend commands treat model paths as project-relative model file references. `.model3.json` and `.json` files resolve under the active project data root, sidecar expressions and motions are discovered next to that model file, and absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, unsupported extensions, and non-portable segments are rejected before filesystem access. Playtest uses the same shared renderer asset validator before passing character art paths to Live2D, GLB/GLTF, or sprite renderers.

## AI Pipeline

Inference follows the package target:
1. **WebGPU Runtime**: Web/PWA builds lazy-load the pinned Transformers.js 4.2.0 text-generation pipeline and `onnx-community/Qwen3.5-0.8B-Text-ONNX` Q4 contract from `inference-runtime.json`, require a secure WebGPU context, and stream output in Character and Ensemble Test. The package keeps ORT single-threaded for static-host compatibility and bundles the Asyncify WASM binary that matches the WebGPU module factory embedded by `onnxruntime-web/webgpu`; model weights remain in the browser cache rather than the application bundle.
2. **Backend Planner**: The versioned `monogatari-inference-backend-plan/v1` report combines read-only host detection with explicit probe signals. Only an exact-model generation result can create `ready`; detected runtimes remain `probe_required`, `setup_required`, `blocked`, or `unavailable`.
3. **Local and Managed Services**: llama.cpp, MLX-LM adapters, vLLM, and SGLang remain separately installed profiles. Once their health, generation, and streaming probes pass, the existing `APIEngine` consumes their OpenAI-compatible endpoint without a new wire protocol.
4. **DirectML Runtime**: Windows builds can load a project-relative ONNX model and standard `tokenizer.json`, require DirectML without CPU fallback, validate full-sequence causal-LM inputs plus float32 logits, and run bounded autoregressive generation off the async runtime thread. This executor does not implement Qwen3.5 hybrid convolution/recurrent state handling.
5. **OpenAI-Compatible API**: Remote or loopback endpoints remain available for authoring and deployment. Valid configuration is not equivalent to generation readiness.

The verified backend matrix, reproduced blockers, automatic preference order, and staged CUDA, Vulkan, Metal, ROCm, Intel, and MUSA rollout process live in [INFERENCE_BACKEND_MATRIX.md](INFERENCE_BACKEND_MATRIX.md).

Character responses use a structured prompt system:
- System prompt with character personality, background, and emotion
- Knowledge base context injected per-query
- Conversation history (last 10 messages)
- Evaluation prompt every 5 messages for scoring

Prompt and response guardrails are shared by single-character chat, group chat, workflow LLM nodes, quality suites, fallback scoring, and the reusable Rust AI prompt builder. Player-authored text is wrapped as untrusted dialogue data, reusable prompt history/context sections sanitize embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings, explicit XML/fence/comment role-control block bodies are omitted with their markers, creator-authored character mind and safety contracts stay in the system channel, and XML/header/JSON-shaped role-control blocks plus English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control phrases are detected before they can influence memory writes, relationship deltas, scoring, or hidden prompt boundaries. Group chat command boundaries normalize participant IDs, reject empty or duplicate participant sets, reject inactive sessions and blank messages, and represent per-character generation failures as stable runtime system messages for the author while filtering them out of later prompt transcripts so failed provider calls do not become character canon.

Workflow LLM nodes and Quality Suite workflow-output checks finalize guarded model output through the shared prompt guard. If generation leaves no safe story text after sanitization, such as blank output, a lone role marker, or only a stripped prompt-control block marker, the node returns stable failure text instead of advancing an empty or guard-only story result. Quality reports export that finalized guarded workflow text as audit evidence, so QA can inspect the exact safe output consumed by downstream story nodes without preserving unsafe raw model text.

When live evaluator output is unavailable, deterministic fallback scoring uses only trusted, normalized player messages. The fallback recognizes English, Chinese, Japanese, and Korean friendly sentiment, questions, and creative-story intent so international builds keep stable relationship and story-event previews without live model calls.

API backend configuration treats provider credentials as runtime-only secrets. Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields before writing `settings.json` or returning project config state to the frontend. The Rust API engine redacts API keys, bearer tokens, sensitive custom headers, and echoed secret assignments from debug output and API error surfaces before they can reach logs or frontend error reports. Settings-configured backends register through async-safe Tauri command paths, and OpenAI-compatible API engines validate runtime base URLs, API keys, and model IDs before activation; the planner still records them as unverified until a model-level generation probe succeeds. API base URLs must be HTTPS except localhost/loopback development endpoints, cannot carry embedded credentials, query strings, or fragments, and are normalized before request construction. OpenAI-compatible streaming responses are parsed through a buffered SSE delta parser so provider chunks can split JSON lines or UTF-8 content without dropping streamed character text, while provider error frames and malformed SSE data frames abort inference instead of being ignored. Streaming chat failures replace any partial assistant bubble with a stable failure message before surfacing the provider/runtime error to the author. Standard and streaming API completions must include non-blank generated text before they are reported as successful, so malformed 200 responses cannot become empty character dialogue or workflow output.

The shared inference pipeline treats unsuccessful `InferenceResult` envelopes as inference failures instead of successful empty generations. Active-engine calls retry these provider failure envelopes, while direct engine and streaming calls reject them before chat, workflow LLM nodes, or stream completion handlers can consume empty or stale generated text.

Windows ONNX configuration treats `modelPath` and `tokenizerPath` as project-relative file references under the active project data root. Model references must be `.onnx`, tokenizer references must be `.json`, and path-shaped or non-portable input is rejected before initialization. A DirectML session disables parallel execution and memory patterns as required by the provider, rejects unsupported KV-cache graph inputs, and registers as active only after model and tokenizer initialization succeed. The current Qwen3.5 profile remains blocked because its hybrid cache/state contract is not supported by this loop; tested WinML GenAI DirectML profiles are also blocked by operator availability or graph partition/capture failures.

The legacy C# AI path mirrors the same boundary-sanitization intent for bracket, fullwidth, XML/header, attributed XML-like, Markdown role-code-fence, comment-wrapped, punctuation-free heading, and JSON-shaped role spoofing, and redacts token-shaped values plus JSON/header/query secret assignments from provider error bodies and request exceptions while the legacy solution remains in the release gate.

## Workflow System

The visual workflow editor supports 21 node types across 5 categories:
- **Flow**: Start, End, Condition, Wait, Random Branch, Sub Workflow
- **Content**: Dialogue, Choice, Narration, Scene Change
- **AI**: LLM Generate, Evaluation
- **Character**: Relationship, Emotion Change
- **Media**: BGM, SFX, Camera, Shake

Workflows are validated for: node IDs, start/end structure, missing config, broken links, duplicate connections, and unreachable nodes. Backend save/load commands resolve workflow JSON paths against the active project `workflows/` directory, accepting simple filenames or `workflows/...` references while rejecting absolute paths, URI/drive prefixes, empty segments, `.`/`..` traversal, and non-JSON files before disk access.

## Story Event Catalog

`StoryEventCatalog` is project-scoped runtime state loaded from versioned `events/*.json` assets. Engine initialization stages and validates the catalog beside fresh character, dialogue, and knowledge managers, including configured path containment and character-scope references, before replacing active state. Failed loads or hot reloads leave the prior catalog untouched. Missing event directories retain compatibility defaults for legacy projects, while an existing empty directory represents an intentional zero-event catalog.

Live chat, manual scoring, workflow trigger nodes, workflow validation, Quality Suites, and browser workflow previews consume the same event definitions. Rules combine optional relationship, normalized score, and evaluation-count thresholds; can target explicit character IDs; and can opt into repeatable triggering. Typed actions unlock scenes, dialogues, endings, or set validated script flags. Legacy unlock fields under `data` normalize into the same actions. Default rule behavior retains v1 fingerprints pinned by Quality Suites, while scoped or repeatable behavior uses v2 fingerprints. A catalog fingerprint binds event descriptions, payload data, typed actions, and rule fingerprints for project/release audits.

`StoryContentAccess` derives gates directly from catalog actions: referenced content requires a matching progress unlock, while unreferenced content stays open. Dialogue start, Playtest scene entry, real workflow scene changes, and ending launch call the shared guard. Author previews may inspect decisions without mutating or blocking their local simulation. `StoryContentAccessSnapshot` exposes gate sources and progress/catalog fingerprints for frontend diagnostics.

The Story Event workbench converts normalized runtime definitions back into the authored v1 document shape. Saves require the fingerprint observed at load time, validate the complete candidate before filesystem mutation, reject ambiguous multi-document flattening, stage a temporary file, retain a backup during replacement, reload the project catalog, and restore the prior document if post-write validation fails.

Story endings use bounded, versioned `endings/*.json` assets that bind an ending ID to a scene and dialogue. The authoring catalog fingerprints normalized definitions and source paths. Mutations serialize through the project content authoring lock, reject stale fingerprints, validate scene/dialogue references before disk writes, stage temporary and backup files, reload the complete catalog, and restore prior files after post-write failure. Deletion is blocked while any project Story Event still unlocks the ending. Player launch verifies all three access decisions; author preview bypasses only those progress gates and still requires valid project assets.

Scene, dialogue, and ending authoring share one project content mutation lock plus a rollback-capable JSON transaction layer. Scene snapshots merge strict metadata documents with background-inferred virtual entries; promotion writes metadata without taking ownership of or deleting the source image. Scene deletion scans Story Event actions, ending associations, and workflow `scene_change` nodes. Dialogue snapshots preserve complete graphs and validate strict fields, node-map identity, reachability, transition targets, character/relationship references, source bounds, terminal metadata, and LLM prompt requirements. Successful dialogue writes replace the runtime script catalog only after post-write reload succeeds; active cursors reset while callbacks and dialogue-local state remain attached to the manager. Dialogue deletion scans Story Events and endings.

`StoryProgressState` is a separate project-scoped runtime ledger. The shared executor records event applications per character scope, applies unlock sets idempotently, increments repeatable event counts, updates validated script flags, and returns versioned action evidence plus a progress fingerprint. Non-streaming chat, streaming chat, and real workflow nodes use this executor. Quality Suite runs and workflow run-context previews only calculate decisions and actions, so author diagnostics cannot modify player progress.

Web/PWA builds copy `data/events` into the deployable `events/` tree, list those files in `project-assets.json`, cache them through the service worker, and fetch them relative to `VITE_BASE_PATH`. The checked-in catalog is compiled only as a final browser fallback when static event content cannot be reached.

## TTS Architecture

Three TTS provider types:
1. **System** (SAPI on Windows): Direct system voice synthesis
2. **Azure**: Cognitive Services REST API
3. **ElevenLabs**: Text-to-speech REST API

Character voice assignments persist in the AppState and can be configured per-character.

All generated TTS files, including system SAPI, Azure, and ElevenLabs outputs, are written under the active project `assets/tts/` directory. Character/provider filename components are sanitized before path construction so generated audio cannot escape the project data root or collide through fixed global temp filenames. Azure and ElevenLabs request failures, response bodies, token-shaped values, API-key assignments, and sensitive provider headers are redacted before errors reach frontend status surfaces. Runtime synthesis logs record text length metadata instead of raw spoken dialogue, prompt text, or token-shaped content.

## Asset Data Boundaries

Project asset files are scoped to the active project data root. The Rust assets `AssetManager` and the retained legacy C# `AssetManager` normalize asset references, reject absolute paths, drive/URI-style prefixes, empty path segments, `.`/`..` traversal, and control characters, then verify the resolved path still lives under the configured asset root before reading text, JSON, binary assets, or directory listings.

## Engine Project Root Boundaries

Engine initialization resolves an empty project path to the active/default project data root, accepts local filesystem project directories, and rejects URI-shaped or control-character input. The resolved root must exist and be a directory. Character, dialogue, knowledge, and story-event content is loaded into fresh temporary managers first; only a complete successful load replaces the active managers. Every reload clears mutable chat sessions, scene history, script state, and event audit state, including same-root reloads. Saving `settings.json` for another directory does not activate that project without loading its content managers. Settings saves are limited to 1 MiB and use the shared staged replacement transaction; project roots, settings targets, and export directories must be regular filesystem entries rather than symlinks.

## Project Package Boundaries

Desktop project handoff uses a `.monogatari` ZIP with a root `monogatari-project.json` manifest. The manifest inventories the exact sanitized settings and content bytes written into the package, with sorted portable paths, per-file SHA-256 and compatibility MD5 checksums, declared empty directories, category fingerprints, and one deterministic whole-package fingerprint. Inventory hashing and ZIP output use fixed 64 KiB buffers so large renderer/audio assets do not become proportional memory spikes. Export writes a sibling temporary file, verifies source bytes against the prepared inventory, syncs it, and atomically replaces the selected regular package while retaining the previous package on failure.

Inspection and import share the same streaming verifier. It bounds compressed and expanded size, entry/file/directory counts, JSON size, path depth and length; rejects traversal, platform-reserved paths, case-folded collisions, symbolic/special entries, undeclared content, runtime secrets, and all checksum/fingerprint mismatches. Import extracts only into a new same-volume staging directory, revalidates project configuration plus runtime character/dialogue/knowledge/event, scene, and ending catalogs, and then renames the staging directory to a fresh non-overwriting project name. Rejected imports remove only the owned staging directory and never modify the active project.

## Character Authoring Boundaries

Character create/delete commands resolve through the active or discovered default project data root and treat character IDs as portable slugs rather than filenames. IDs are validated before path construction, character JSON files are written or removed only as direct children of the project `characters/` directory, and deletion also removes the character from the in-memory runtime manager.

## Plugin Authoring Boundaries

Plugin listing, registration, and removal commands resolve through the active or discovered default project data root and treat plugin IDs as portable slugs rather than filenames. Plugin manifests are normalized before writing, optional `script_path` values must be plugin-root-relative `.rhai` references with no URI, drive, absolute, empty, current, or parent segments, manifest files are written, listed, or removed only as direct children of the project `plugins/` directory, and the Plugin workbench sends the backend `{ manifest }` and `{ pluginId }` command contracts directly.

## Marketplace Template Boundaries

Marketplace import/export commands treat template paths as project template references rather than raw filesystem paths. Template references resolve under the active project `templates/` directory, reject absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, and non-portable segments, and built-in catalog entries import by their safe catalog IDs.

## Content Loader Boundaries

Character, dialogue, and knowledge reload commands accept project content references rather than raw filesystem paths. `characters`, `dialogue`, and `knowledge` resolve to their canonical folders under the active project data root, while nested references stay under the same canonical folder. Absolute paths, drive/URI-style prefixes, empty segments, and `.`/`..` traversal are rejected before directory loading begins.

Web/PWA packaging copies project scenes, dialogues, endings, events, renderer assets, and an optional `data/models/webgpu` model directory into the static distribution, inventories project content in `project-assets.json`, and emits `inference-runtime.json` with the WebGPU model contract. The service worker pre-caches both manifests and project content; Transformers.js maintains its browser model cache. The browser Story Library uses the same access snapshot and delegates cursor, dialogue-local variable/flag, condition, bounded script, graph-error, and relationship-effect behavior to the pure Story Playtest state machine. Workflow preview and Story Playtest share one side-effect-free local condition evaluator. Condition-hidden choices retain their authored indices; false nodes follow a required linear fallback with cycle detection; unsupported browser syntax stops rather than misrouting. Tauri evaluates the same authored state through the bounded shared Rhai engine, preflights choice relationship targets, commits only against the inspected source node, then applies bounded deltas through CharacterManager. Browser authoring stores complete scene, dialogue, and ending catalog drafts in local storage, and Playtest reads those same drafts without changing packaged source files.

## Save Data Boundaries

Save files are scoped to the active project `saves/` directory. The Rust assets `SaveManager` and the retained legacy C# `SaveManager` both validate save IDs before constructing paths, allow only portable filename characters, reject traversal-shaped IDs, and filter listed save files whose embedded save ID does not match the filename. Tauri load/delete commands should consume save IDs returned by `save_game` or `list_saves`, not arbitrary filesystem paths.

Rust runtime snapshots use `monogatari-game-save/v3` while schema-less v1 and explicit v2 payloads remain readable. V3 snapshots include scene history, the validated dialogue cursor and dialogue-local variables, typed Rhai variables and flags, character emotion/relationships/full bounded memory, serialized chat sessions, and validated story progress with applied event scopes and unlock sets. V1/v2 sessions migrate known triggered events through the active catalog before restore. Restore validates schemas, state keys, dialogue references, chat identities, message bounds, score ranges, progress bounds, duplicate scopes, and fingerprints before mutating runtime state. Quick-save and auto-save provide stable IDs so repeated snapshots replace bounded slots. Stable-slot writes stage a temporary snapshot and retain a recoverable prior file until replacement succeeds; reads and writes reject payloads above 32 MiB.

## Cloud Sync Architecture

Cloud sync is project-scoped and manifest-driven. Save manifests live under the active project `saves/.sync_manifest.json`, not the process working directory, so installed builds keep user save state portable with the selected project data root. The backend status contract reports local save file counts, pending upload/download work, cross-device conflicts, provider mode, endpoint readiness, and token readiness. Sync tokens are accepted only as runtime command input and reduced to readiness booleans; token values are not written to disk or echoed into status payloads.

The Settings panel consumes the backend `CloudSyncStatus` shape directly and exposes local manifest mode plus remote preflight mode. Until a real remote storage adapter is wired in, Push updates the local manifest evidence and Pull reports manifest entries rather than claiming completed remote file transfer.
