# Monogatari Architecture

## Overview

Monogatari is a desktop application built with Rust (Tauri 2.x) for the backend and Vue 3 + TypeScript for the frontend. The engine uses an AI inference pipeline to drive character conversations, with an event system that triggers plot developments based on conversation quality scores.

## System Architecture

```
+--------------------------------------------------+
|                  Frontend (Vue 3)                  |
|  Views (21) | PWA shell | Stores (Pinia)          |
+--------------------------------------------------+
|            Tauri IPC Bridge (invokeCommand)        |
+--------------------------------------------------+
|                  Rust Backend                      |
|  22 Command Modules | State (AppState)             |
+--------------------------------------------------+
|           Core Crates                             |
|  core/ | ai/ | game/ | assets/ | scripting/       |
+--------------------------------------------------+
|           External Services                        |
|  OpenAI API | ONNX Preflight | TTS Providers       |
+--------------------------------------------------+
```

## Crate Structure

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `core` | Shared infrastructure | EventBus, ServiceLocator, GameClock |
| `ai` | AI inference | InferencePipeline, APIEngine, ONNXEngine |
| `game` | Game logic | CharacterManager, DialogueManager, KnowledgeBase, SceneManager |
| `assets` | File management | AssetManager, SaveManager |
| `scripting` | Rhai scripting | ScriptEngine |
| `tauri-app` | Tauri commands | AppState, 22 command modules |

Script execution is treated as bounded authoring logic. Tauri script commands validate payload size and hidden control characters before execution or DSL parsing, and `ScriptEngine::execute` repeats that source validation for workflow and future plugin callers. Condition expressions use shared 2,000-character/control-character validation and run through a separate read-only Rhai engine that can inspect variables, flags, and temporary workflow context values without registering state mutation functions; workflow validation applies the same condition rules before imported graphs run. Workflow condition nodes expose relationship, evaluation score, and evaluation-count context as temporary scope variables so branches can react to chat state or author preview presets without writing story state. Desktop run-context previews snapshot script state and mirror variable, flag, relationship, emotion, and scene node effects in a per-run local state bag; browser-only Web/PWA workflow previews mirror the same state transitions plus weighted random-branch behavior so exported builds can exercise later condition branches, event gates, and trace diagnostics without touching persistent save data. Script variables and flags use shared portable state key validation before Rhai functions, workflow validation, workflow nodes, dialogue scripts, or save loading can write them, keeping persisted state names stable across desktop, Web/PWA, and exported project packages. The shared engine caps Rhai operations, recursive calls, expression depth, variable count, function definitions, and module imports so custom game logic cannot hang the workbench through runaway loops or recursion.

## Data Flow

1. **Player sends message** via ChatView -> invokeCommand("send_chat_message")
2. **Backend builds context** from character personality, knowledge base, conversation history
3. **AI pipeline generates response** via OpenAI-compatible API; ONNX mode is project-scoped configuration preflight until a local runtime executor is linked
4. **Response streamed** back via Tauri events (chat-chunk, chat-complete)
5. **Evaluation triggered** every 5 messages - scores friendliness, engagement, creativity
6. **Events triggered** based on cumulative scores and relationship milestones

## Frontend Architecture

- **Router**: 21 routes with lazy-loaded views
- **State**: Pinia store for game state (saves, scenes, relationships)
- **i18n**: Nested key resolution with localStorage persistence (zh-CN, ja-JP, ko-KR)
- **Tauri Bridge**: Browser-compatible `invokeCommand()` with fallback for non-Tauri environments
- **Web Distribution**: Production browser builds register a service worker, manifest, and offline fallback; Tauri runtime disables service worker registration.
- **Renderer Asset Pipeline**: Story Mode resolves scene and character assets through a shared frontend resolver. Character staging prefers Live2D models, then GLB/GLTF 3D models, then 2D sprites or portraits, and falls back to a generated Three.js placeholder for assetless characters.
- **Runtime Log Hygiene**: Production frontend source avoids `console.log` and `console.debug` debug output; release verification scans `frontend/src` while preserving warning/error reporting for real failures.
- **DOM Injection Boundaries**: The frontend shell renders navigation symbols through normal text bindings, and release verification rejects `v-html` plus direct raw-HTML assignments in runtime source.
- **Desktop CSP**: Packaged Tauri WebViews use an explicit Content Security Policy that allows local app assets, Tauri asset URLs, blob/data media, HTTPS provider connectivity, and localhost dev tooling while blocking object, frame, form, wildcard default, and `unsafe-eval` surfaces.
- **Web/PWA CSP**: Static browser builds ship a matching `index.html` Content Security Policy meta tag, copied into the `404.html` SPA fallback, so hosted previews block object, frame, form, wildcard default, and `unsafe-eval` surfaces while still allowing required project assets, blobs, media, HTTPS providers, and localhost preview tooling.
- **Static Hosting Headers**: Web/PWA dist preparation also emits a `_headers` file for Netlify/Cloudflare-style hosts with CSP, `X-Content-Type-Options`, `Referrer-Policy`, and `Permissions-Policy` headers; release verification checks both the generated file and the source script so response-header capable hosts get stronger browser enforcement without breaking GitHub Pages fallback.
- **Static Hosting Redirects**: Web/PWA dist preparation emits a `_redirects` file for Netlify/Cloudflare-style hosts with explicit project asset passthrough rewrites before the final `index.html` SPA fallback, keeping static renderer assets and locale files reachable on hosts that evaluate redirects before normal file serving.
- **Azure Static Web Apps Config**: Web/PWA dist preparation emits `staticwebapp.config.json` with SPA navigation fallback to `index.html`, explicit asset/service-worker exclusions, a `404.html` rewrite, and matching global security headers so Azure Static Web Apps deployments share the same browser enforcement and route behavior.
- **Vercel Static Config**: Web/PWA dist preparation emits `vercel.json` with a static SPA rewrite to `index.html` and matching security headers, and release verification rejects missing headers, external rewrite targets, or missing fallback routing.

## i18n Locale Boundaries

i18n backend commands treat locale values as portable locale IDs rather than filenames or paths. Locale IDs resolve to direct JSON files under the active project `locales/` directory, listed locales are filtered through the same validator, and slashes, dots, URI/drive-style prefixes, empty hyphen segments, control characters, and non-portable characters are rejected before filesystem access.

## Live2D Model Boundaries

Live2D backend commands treat model paths as project-relative model file references. `.model3.json` and `.json` files resolve under the active project data root, sidecar expressions and motions are discovered next to that model file, and absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, unsupported extensions, and non-portable segments are rejected before filesystem access. Story Mode uses the same shared renderer asset validator before passing character art paths to Live2D, GLB/GLTF, or sprite renderers.

## AI Pipeline

The `InferencePipeline` supports two backends:
1. **API Engine**: OpenAI-compatible endpoints (GPT-4, Claude, etc.)
2. **ONNX Engine**: Project-scoped local model/tokenizer configuration preflight with explicit runtime-unavailable errors until ONNX Runtime execution is linked

Character responses use a structured prompt system:
- System prompt with character personality, background, and emotion
- Knowledge base context injected per-query
- Conversation history (last 10 messages)
- Evaluation prompt every 5 messages for scoring

Prompt and response guardrails are shared by single-character chat, group chat, workflow LLM nodes, quality suites, fallback scoring, and the reusable Rust AI prompt builder. Player-authored text is wrapped as untrusted dialogue data, reusable prompt history/context sections sanitize embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings, explicit XML/fence/comment role-control block bodies are omitted with their markers, creator-authored character mind and safety contracts stay in the system channel, and XML/header/JSON-shaped role-control blocks plus English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control phrases are detected before they can influence memory writes, relationship deltas, scoring, or hidden prompt boundaries. Group chat command boundaries normalize participant IDs, reject empty or duplicate participant sets, reject inactive sessions and blank messages, and represent per-character generation failures as stable runtime system messages for the author while filtering them out of later prompt transcripts so failed provider calls do not become character canon.

Workflow LLM nodes and Quality Suite workflow-output checks finalize guarded model output through the shared prompt guard. If generation leaves no safe story text after sanitization, such as blank output, a lone role marker, or only a stripped prompt-control block marker, the node returns stable failure text instead of advancing an empty or guard-only story result. Quality reports export that finalized guarded workflow text as audit evidence, so QA can inspect the exact safe output consumed by downstream story nodes without preserving unsafe raw model text.

When live evaluator output is unavailable, deterministic fallback scoring uses only trusted, normalized player messages. The fallback recognizes English, Chinese, Japanese, and Korean friendly sentiment, questions, and creative-story intent so international builds keep stable relationship and story-event previews without live model calls.

API backend configuration treats provider credentials as runtime-only secrets. Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields before writing `settings.json` or returning project config state to the frontend. The Rust API engine redacts API keys, bearer tokens, sensitive custom headers, and echoed secret assignments from debug output and API error surfaces before they can reach logs or frontend error reports. Settings-configured backends register through async-safe Tauri command paths, and OpenAI-compatible API engines validate runtime base URLs, API keys, and model IDs before activation so configured providers report ready and can serve chat or streaming immediately after configuration. API base URLs must be HTTPS except localhost/loopback development endpoints, cannot carry embedded credentials, query strings, or fragments, and are normalized before request construction. OpenAI-compatible streaming responses are parsed through a buffered SSE delta parser so provider chunks can split JSON lines or UTF-8 content without dropping streamed character text, while provider error frames and malformed SSE data frames abort inference instead of being ignored. Streaming chat failures replace any partial assistant bubble with a stable failure message before surfacing the provider/runtime error to the author. Standard and streaming API completions must include non-blank generated text before they are reported as successful, so malformed 200 responses cannot become empty character dialogue or workflow output.

The shared inference pipeline treats unsuccessful `InferenceResult` envelopes as inference failures instead of successful empty generations. Active-engine calls retry these provider failure envelopes, while direct engine and streaming calls reject them before chat, workflow LLM nodes, or stream completion handlers can consume empty or stale generated text.

ONNX backend configuration treats `modelPath` and `tokenizerPath` as project-relative file references under the active project data root. Model references must be `.onnx`, tokenizer references must be `.json`, path-shaped or non-portable input is rejected before engine registration, and successful ONNX configuration activates the ONNX engine so Settings cannot silently leave an older backend selected. Until a real ONNX Runtime executor is linked, ONNX initialization, inference, and streaming return an explicit runtime-unavailable error and AI status reports the backend as not ready, preventing placeholder text from entering character dialogue or scoring flows.

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

Live chat, manual scoring, workflow trigger nodes, workflow validation, Quality Suites, and browser workflow previews consume the same event definitions. Rules combine optional relationship, normalized score, and evaluation-count thresholds; can target explicit character IDs; and can opt into repeatable triggering. Default rule behavior retains v1 fingerprints pinned by Quality Suites, while scoped or repeatable behavior uses v2 fingerprints. A catalog fingerprint binds event descriptions, payload data, and rule fingerprints for project/release audits.

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

Engine initialization resolves an empty project path to the active/default project data root, accepts local filesystem project directories, and rejects URI-shaped or control-character input. The resolved root must exist and be a directory. Character, dialogue, knowledge, and story-event content is loaded into fresh temporary managers first; only a complete successful load replaces the active managers. Every reload clears mutable chat sessions, scene history, script state, and event audit state, including same-root reloads. Saving `settings.json` for another directory does not activate that project without loading its content managers.

## Character Authoring Boundaries

Character create/delete commands resolve through the active or discovered default project data root and treat character IDs as portable slugs rather than filenames. IDs are validated before path construction, character JSON files are written or removed only as direct children of the project `characters/` directory, and deletion also removes the character from the in-memory runtime manager.

## Plugin Authoring Boundaries

Plugin listing, registration, and removal commands resolve through the active or discovered default project data root and treat plugin IDs as portable slugs rather than filenames. Plugin manifests are normalized before writing, optional `script_path` values must be plugin-root-relative `.rhai` references with no URI, drive, absolute, empty, current, or parent segments, manifest files are written, listed, or removed only as direct children of the project `plugins/` directory, and the Plugin workbench sends the backend `{ manifest }` and `{ pluginId }` command contracts directly.

## Marketplace Template Boundaries

Marketplace import/export commands treat template paths as project template references rather than raw filesystem paths. Template references resolve under the active project `templates/` directory, reject absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, and non-portable segments, and built-in catalog entries import by their safe catalog IDs.

## Content Loader Boundaries

Character, dialogue, and knowledge reload commands accept project content references rather than raw filesystem paths. `characters`, `dialogue`, and `knowledge` resolve to their canonical folders under the active project data root, while nested references stay under the same canonical folder. Absolute paths, drive/URI-style prefixes, empty segments, and `.`/`..` traversal are rejected before directory loading begins.

## Save Data Boundaries

Save files are scoped to the active project `saves/` directory. The Rust assets `SaveManager` and the retained legacy C# `SaveManager` both validate save IDs before constructing paths, allow only portable filename characters, reject traversal-shaped IDs, and filter listed save files whose embedded save ID does not match the filename. Tauri load/delete commands should consume save IDs returned by `save_game` or `list_saves`, not arbitrary filesystem paths.

Rust runtime snapshots use `monogatari-game-save/v2` while schema-less legacy payloads deserialize as v1. V2 snapshots include scene history, the validated dialogue cursor and dialogue-local variables, typed Rhai variables and flags, character emotion/relationships/full bounded memory, and serialized chat sessions containing messages, evaluation state, safety traces, and triggered event IDs. Restore validates schemas, state keys, dialogue references, chat identities, message bounds, and score ranges before mutating runtime state. Quick-save and auto-save provide stable IDs so repeated snapshots replace bounded slots. Stable-slot writes stage a temporary snapshot and retain a recoverable prior file until replacement succeeds; reads and writes reject payloads above 32 MiB.

## Cloud Sync Architecture

Cloud sync is project-scoped and manifest-driven. Save manifests live under the active project `saves/.sync_manifest.json`, not the process working directory, so installed builds keep user save state portable with the selected project data root. The backend status contract reports local save file counts, pending upload/download work, cross-device conflicts, provider mode, endpoint readiness, and token readiness. Sync tokens are accepted only as runtime command input and reduced to readiness booleans; token values are not written to disk or echoed into status payloads.

The Settings panel consumes the backend `CloudSyncStatus` shape directly and exposes local manifest mode plus remote preflight mode. Until a real remote storage adapter is wired in, Push updates the local manifest evidence and Pull reports manifest entries rather than claiming completed remote file transfer.
