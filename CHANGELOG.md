# Changelog

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
