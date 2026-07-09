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
|  OpenAI API | ONNX Runtime | TTS Providers        |
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
3. **AI pipeline generates response** via OpenAI-compatible API or ONNX model
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

## i18n Locale Boundaries

i18n backend commands treat locale values as portable locale IDs rather than filenames or paths. Locale IDs resolve to direct JSON files under the active project `locales/` directory, listed locales are filtered through the same validator, and slashes, dots, URI/drive-style prefixes, empty hyphen segments, control characters, and non-portable characters are rejected before filesystem access.

## Live2D Model Boundaries

Live2D backend commands treat model paths as project-relative model file references. `.model3.json` and `.json` files resolve under the active project data root, sidecar expressions and motions are discovered next to that model file, and absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, unsupported extensions, and non-portable segments are rejected before filesystem access. Story Mode uses the same shared renderer asset validator before passing character art paths to Live2D, GLB/GLTF, or sprite renderers.

## AI Pipeline

The `InferencePipeline` supports two backends:
1. **API Engine**: OpenAI-compatible endpoints (GPT-4, Claude, etc.)
2. **ONNX Engine**: Local models via ONNX Runtime with DirectML acceleration

Character responses use a structured prompt system:
- System prompt with character personality, background, and emotion
- Knowledge base context injected per-query
- Conversation history (last 10 messages)
- Evaluation prompt every 5 messages for scoring

Prompt and response guardrails are shared by single-character chat, group chat, workflow LLM nodes, quality suites, fallback scoring, and the reusable Rust AI prompt builder. Player-authored text is wrapped as untrusted dialogue data, reusable prompt history/context sections sanitize embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings, creator-authored character mind and safety contracts stay in the system channel, and XML/header/JSON-shaped role-control blocks plus English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control phrases are detected before they can influence memory writes, relationship deltas, scoring, or hidden prompt boundaries.

When live evaluator output is unavailable, deterministic fallback scoring uses only trusted, normalized player messages. The fallback recognizes English, Chinese, Japanese, and Korean friendly sentiment, questions, and creative-story intent so international builds keep stable relationship and story-event previews without live model calls.

API backend configuration treats provider credentials as runtime-only secrets. Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields before writing `settings.json` or returning project config state to the frontend. The Rust API engine redacts API keys, bearer tokens, sensitive custom headers, and echoed secret assignments from debug output and API error surfaces before they can reach logs or frontend error reports.

ONNX backend configuration treats `modelPath` and `tokenizerPath` as project-relative file references under the active project data root. Model references must be `.onnx`, tokenizer references must be `.json`, path-shaped or non-portable input is rejected before engine registration, and successful ONNX configuration activates the ONNX engine so Settings cannot silently leave an older backend selected.

The legacy C# AI path mirrors the same boundary-sanitization intent for bracket, fullwidth, XML/header, attributed XML-like, Markdown role-code-fence, comment-wrapped, punctuation-free heading, and JSON-shaped role spoofing, and redacts token-shaped values plus JSON/header/query secret assignments from provider error bodies and request exceptions while the legacy solution remains in the release gate.

## Workflow System

The visual workflow editor supports 21 node types across 5 categories:
- **Flow**: Start, End, Condition, Wait, Random Branch, Sub Workflow
- **Content**: Dialogue, Choice, Narration, Scene Change
- **AI**: LLM Generate, Evaluation
- **Character**: Relationship, Emotion Change
- **Media**: BGM, SFX, Camera, Shake

Workflows are validated for: node IDs, start/end structure, missing config, broken links, duplicate connections, and unreachable nodes. Backend save/load commands resolve workflow JSON paths against the active project `workflows/` directory, accepting simple filenames or `workflows/...` references while rejecting absolute paths, URI/drive prefixes, empty segments, `.`/`..` traversal, and non-JSON files before disk access.

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

Engine initialization resolves an empty project path to the active/default project data root, accepts local filesystem project directories, and rejects URI-shaped or control-character input. The resolved root must exist and be a directory before character, dialogue, knowledge, asset, and save managers are rebound to it.

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

## Cloud Sync Architecture

Cloud sync is project-scoped and manifest-driven. Save manifests live under the active project `saves/.sync_manifest.json`, not the process working directory, so installed builds keep user save state portable with the selected project data root. The backend status contract reports local save file counts, pending upload/download work, cross-device conflicts, provider mode, endpoint readiness, and token readiness. Sync tokens are accepted only as runtime command input and reduced to readiness booleans; token values are not written to disk or echoed into status payloads.

The Settings panel consumes the backend `CloudSyncStatus` shape directly and exposes local manifest mode plus remote preflight mode. Until a real remote storage adapter is wired in, Push updates the local manifest evidence and Pull reports manifest entries rather than claiming completed remote file transfer.
