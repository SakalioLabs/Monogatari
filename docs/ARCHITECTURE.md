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

## AI Pipeline

The `InferencePipeline` supports two backends:
1. **API Engine**: OpenAI-compatible endpoints (GPT-4, Claude, etc.)
2. **ONNX Engine**: Local models via ONNX Runtime with DirectML acceleration

Character responses use a structured prompt system:
- System prompt with character personality, background, and emotion
- Knowledge base context injected per-query
- Conversation history (last 10 messages)
- Evaluation prompt every 5 messages for scoring

Prompt and response guardrails are shared by single-character chat, group chat, workflow LLM nodes, quality suites, fallback scoring, and the reusable Rust AI prompt builder. Player-authored text is wrapped as untrusted dialogue data, reusable prompt history/context sections sanitize embedded role-boundary markers, creator-authored character mind and safety contracts stay in the system channel, and XML/header/JSON-shaped role-control blocks plus English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control phrases are detected before they can influence memory writes, relationship deltas, scoring, or hidden prompt boundaries.

When live evaluator output is unavailable, deterministic fallback scoring uses only trusted, normalized player messages. The fallback recognizes English, Chinese, Japanese, and Korean friendly sentiment, questions, and creative-story intent so international builds keep stable relationship and story-event previews without live model calls.

API backend configuration treats provider credentials as runtime-only secrets. The Rust API engine redacts API keys, bearer tokens, sensitive custom headers, and echoed secret assignments from debug output and API error surfaces before they can reach logs or frontend error reports.

The legacy C# prompt builder mirrors the same boundary-sanitization intent for bracket, fullwidth, XML/header, and JSON-shaped role spoofing while the legacy solution remains in the release gate.

## Workflow System

The visual workflow editor supports 21 node types across 5 categories:
- **Flow**: Start, End, Condition, Wait, Random Branch, Sub Workflow
- **Content**: Dialogue, Choice, Narration, Scene Change
- **AI**: LLM Generate, Evaluation
- **Character**: Relationship, Emotion Change
- **Media**: BGM, SFX, Camera, Shake

Workflows are validated for: node IDs, start/end structure, missing config, broken links, duplicate connections, and unreachable nodes.

## TTS Architecture

Three TTS provider types:
1. **System** (SAPI on Windows): Direct system voice synthesis
2. **Azure**: Cognitive Services REST API
3. **ElevenLabs**: Text-to-speech REST API

Character voice assignments persist in the AppState and can be configured per-character.
