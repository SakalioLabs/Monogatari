# Changelog

## Unreleased - 2026-07-06

### Commercialization Progress
- Redesigned the frontend into a more production-oriented workbench: dashboard, AI chat, story mode, and workflow editor now use cleaner dark surfaces, denser operational layouts, and consistent design tokens.
- Connected the AI Chat page to the existing Tauri streaming command (`send_chat_message_stream`) and event stream (`chat-chunk`, `chat-complete`, `chat-emotion`, `chat-relationship`, `chat-error`) for real-time response display.
- Added streamed evaluation and event notification parity through `chat-evaluation` and `chat-events`, so the streaming path now updates scoring and unlock toasts without manual refresh.
- Added a Tauri command wrapper with browser-preview fallbacks so design review in plain Vite no longer depends on desktop runtime APIs.
- Cleaned corrupted visible text in Story Mode and Workflow Editor and replaced placeholder-like icon glyphs with stable text markers.
- Improved Workflow Editor connection behavior by targeting the node under the mouse instead of linking to an arbitrary node.
- Added workflow validation through the new `validate_workflow` Tauri command, with save/load rejection for structural errors and a diagnostics panel in the editor.
- Added a Scene Assets workbench backed by `list_scene_assets`, `get_current_scene`, and `set_scene` Tauri commands, including metadata/background validation and sample scene assets.
- Added a Project Control settings console backed by `get_project_config` and `save_project_config`, with settings persistence, content path readiness, and runtime initialization controls.

### Security And Tooling
- Upgraded the frontend build toolchain to Vite 8, `@vitejs/plugin-vue` 6, `vue-tsc` 3, TypeScript 6, and explicit `esbuild` to remove dev-server audit findings.
- Kept Live2D on `pixi-live2d-display@0.4.0` while overriding its transitive `gh-pages` dependency to `6.3.0`, clearing the critical production audit issue without downgrading Live2D.
- Stopped ignoring `rust-engine/Cargo.lock` and added the generated lockfile so Tauri desktop builds can be verified with `cargo --locked`.
- Added `docs/RELEASE_CHECKLIST.md` with commercial release gates, packaging policy, signing reminders, and manual QA coverage.

### Performance
- Reduced chat-session write-lock duration in both non-streaming and streaming chat commands so slow LLM calls do not hold the global chat session lock for the full request.

### Verification
- `npm run build` passes for the Vue frontend.
- `npm audit` reports 0 vulnerabilities.
- `cargo check --locked -p llm-galgame-app` passes for the Tauri app crate.
- `dotnet test LLMAssistant.sln --no-restore` exits successfully for the legacy C# solution.

## v0.3.0 - 2026-07-06
### Frontend
- Added Group Chat View with character multi-select, shared conversation area, and participant tracking.
- Added Group Chat route and sidebar navigation entry.
- Added CSS classes for 8 new workflow editor node types with category-specific left-border colors.


### New Capabilities
- **Multi-Character Group Chat**: New multi_chat module enabling simultaneous conversations with multiple AI characters who react to each other and the player.
- **TTS Integration Scaffold**: New 	ts module with configuration, voice assignment, and speech synthesis command interfaces ready for backend provider integration.
- **8 New Workflow Node Types**: Narration, BGM, SFX, Wait, Random Branch, Sub Workflow, Camera, and Shake nodes expand the visual editor to 21 total node types across 6 categories (flow, content, logic, ai, character, media).

### Performance and Quality
- Fixed locking_read() call in async chat event trigger evaluation, preventing potential deadlocks during character relationship scoring.
- Added Cargo dev profile with opt-level = 0 for dev builds and opt-level = 2 for dependencies, significantly improving development iteration speed.
- Registered all new command modules in mod.rs and main.rs with proper Tauri command handler bindings.

### Architecture
- commands/multi_chat.rs: GroupChatSession, GroupChatMessage types with start_group_chat, send_group_message, get_group_chat_characters commands.
- commands/tts.rs: TtsConfig, CharacterVoice, TtsResult types with configure_tts, set_character_voice, synthesize_speech, get_available_voices commands.
- workflow.rs: Extended WorkflowNodeTypeInfo with 8 new nodes in media and extended flow categories.

## v0.2.0 - 2026-07-06

### New Features
- **Free-form AI Chat Mode**: Players can talk freely with LLM-driven characters. The AI stays in character using personality, background, and world knowledge from the knowledge base.
- **Conversation Evaluation System**: Every 5 player messages, the LLM evaluates conversation quality on friendliness, engagement, and creativity dimensions. Scores accumulate to trigger special events.
- **Event Trigger System**: 7 built-in event types including relationship milestones (friend/close friend/best friend), high engagement dialogues, creative talk achievements, and cumulative dedication rewards.
- **Streaming Chat Support**: Real-time streaming LLM responses via Tauri events (`chat-chunk`, `chat-complete`, `chat-emotion`, `chat-relationship`, `chat-error`).
- **Knowledge Base Integration**: Character responses incorporate relevant world knowledge automatically searched from the knowledge base.
- **Character Emotion Detection**: Automatic emotion detection from LLM response text (happy, sad, angry, surprised, love, embarrassed).
- **Relationship Tracking**: Sentiment analysis of player messages updates character relationship scores.

### UI Redesign
- **Professional Design System**: New CSS theme with Inter font stack, 5-level surface hierarchy, brand color variables, consistent spacing/radius/shadow system.
- **Collapsible Sidebar**: Gradient logo, active state indicators, smooth page transitions.
- **Dashboard Home View**: Stat cards, feature grid, engine status panel, responsive layout.
- **Component Primitives**: Badges, tags, tooltips, skeletons, spinners, buttons with variants.

### Architecture
- **6 Tauri Chat Commands**: `send_chat_message`, `send_chat_message_stream`, `get_chat_history`, `clear_chat_history`, `evaluate_conversation`, `get_relationship_score`, `get_available_events`.
- **Chat Session State**: Per-character chat history with cumulative scores and triggered event tracking.
- **Workflow Node Types**: Added evaluation, scene_change, trigger_event nodes to the visual editor.
- **Prompt Builder**: Structured prompt construction with system, character, knowledge, and world context sections.

### Documentation
- Comprehensive README with architecture diagram, feature list, node types table.
- Quick start guide, configuration examples (API and ONNX modes).
- Data format specifications for characters and dialogues.
- Development roadmap with completed/in-progress/planned items.

## v0.1.0 - 2026-07-06 (Initial Release)

### Core Engine
- Rust + Tauri 2.x desktop application with Vue 3 frontend
- 6 Rust crates: core, ai, game, assets, scripting, tauri-app
- EventBus, ServiceLocator, GameClock foundations
- AI inference pipeline with API (OpenAI-compatible) and ONNX (DirectML) backends
- Character system with Big Five personality, memory, emotions, relationships
- Branching dialogue system with choices, flags, scripts
- Knowledge base with keyword-based relevance search
- Rhai scripting engine for game logic
- Save/load system with full state persistence
- Live2D rendering via PixiJS
- Visual workflow editor (drag-and-drop node-based)
- Settings panel for AI configuration
