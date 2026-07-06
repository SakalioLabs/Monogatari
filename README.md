# Monogatari

An LLM-powered visual novel / galgame engine. Build interactive story experiences where AI-driven characters respond dynamically to player conversations, with automatic conversation scoring that triggers special plot events.

## What It Is

Monogatari is a development engine for creating LLM-driven text adventure games. Creators provide story presets, scene images, and character artwork (2D sprites, Live2D, 3D). Through a visual drag-and-drop workflow editor (similar to Dify/Blueprint), they pre-arrange special event nodes. Players converse with AI characters powered by large language models. The LLM evaluates conversations and scores them, triggering special storylines or guiding the narrative direction.

## Key Features

- **AI Chat Mode** - Players talk freely with LLM-driven characters. The AI stays in character using personality, background, world knowledge, streaming response events, and streamed evaluation/event notifications.
- **Conversation Scoring** - The LLM evaluates every conversation on friendliness, engagement, and creativity. Cumulative scores unlock special events.
- **Event Trigger System** - Relationship milestones, dialogue achievements, and cumulative progress trigger plot events, scene changes, and special dialogues.
- **Visual Workflow Editor** - Drag-and-drop node-based editor for designing dialogue flows, branching conditions, LLM generation nodes, evaluation triggers, and scene transitions.
- **Workflow Validation** - Import/export and save paths validate node ids, start/end structure, missing config fields, broken links, duplicate links, and unreachable nodes.
- **Scene Asset Library** - Project scene metadata and background files are scanned, validated, listed, and selectable as the active runtime scene.
- **Project Control Panel** - Project settings, path readiness, AI backend selection, and runtime initialization are managed from one production-oriented console.
- **Character System** - Full personality model (Big Five traits), memory system, emotion tracking, and relationship scores per character.
- **Knowledge Base** - Keyword-indexed world lore and context system that feeds into AI prompts for consistent storytelling.
- **Branching Dialogue** - Pre-scripted dialogue trees with choices, relationship changes, and flag-based conditional branching.
- **Live2D Support** - Animated character models via PixiJS + pixi-live2d-display.
- **Save/Load System** - Full game state persistence including character states, flags, variables, and chat history.
- **Rhai Scripting** - Embedded scripting engine for custom game logic, conditions, and triggers.
- **Multiple AI Backends** - OpenAI-compatible API (GPT, Claude, etc.) and local ONNX models with DirectML.
- **Commercial Workbench UI** - Desktop-first dashboard, streaming chat desk, story runtime, workflow authoring surface, and settings panels designed for repeated production use.

## Current Development Status

Verified on 2026-07-06 (updated):

- Frontend production build passes with `npm run build`.
- Full frontend dependency audit passes with `npm audit`.
- Rust Tauri app crate passes `cargo check --locked -p llm-galgame-app`.
- C# legacy solution exits successfully with `dotnet test LLMAssistant.sln --no-restore`.
- Live2D remains on `pixi-live2d-display@0.4.0`; its transitive `gh-pages` dependency is pinned to the safe `6.3.0` line through npm overrides.
- Rust desktop dependencies are pinned through `rust-engine/Cargo.lock` for reproducible Tauri builds.
- Commercial release gates are tracked in `docs/RELEASE_CHECKLIST.md`.

## Architecture

```
monogatari/
+-- rust-engine/           # Rust backend (Tauri desktop app)
|   +-- crates/
|   |   +-- core/          # EventBus, ServiceLocator, GameClock, error handling
|   |   +-- ai/            # API engine, ONNX engine, inference pipeline
|   |   +-- game/          # Characters, Dialogue, Knowledge, Scenes, Script parser
|   |   +-- assets/        # Asset management, save/load
|   |   +-- scripting/     # Rhai scripting engine
|   |   +-- tauri-app/     # Tauri commands (AI, Chat, Dialogue, Workflow, etc.)
|   +-- data/              # Example characters, dialogues, knowledge, scenes, assets
+-- frontend/              # Vue 3 + Vite + Pinia
|   +-- src/
|   |   +-- views/         # HomeView, ChatView, GameView, WorkflowEditor, SceneAssetsView, SettingsView
|   |   +-- components/    # Live2DCanvas
|   |   +-- stores/        # Pinia game store
|   |   +-- styles/        # Design system (CSS variables, components)
+-- src/                   # C# implementation (legacy, SDL2-based)
+-- tests/                 # C# tests
```

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

```bash
cd rust-engine/crates/tauri-app
cargo tauri build
```

## Usage

### AI Chat Mode (Core Feature)

1. Configure your AI backend in Settings (API key, model, base URL)
2. Open AI Chat from the dashboard
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
4. Set a scene active before testing Story Mode or saving runtime state

### Project Control

1. Open Settings from the sidebar
2. Set the project data path, title, target FPS, and content directory mappings
3. Review readiness diagnostics for characters, dialogue, knowledge, scenes, assets, and saves
4. Save `settings.json`, configure the AI backend, then initialize the runtime

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
      "api_key": "your-key-here",
      "model": "gpt-4o-mini"
    }
  }
}
```

### ONNX Mode (Local)

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
- [x] AI inference pipeline (API + ONNX with DirectML)
- [x] Character system (personality, memory, emotions, relationships)
- [x] Dialogue system (branching, choices, flags, scripts)
- [x] Knowledge base (keyword search, category/tag indexing)
- [x] Scripting engine (Rhai-based)
- [x] Save/load system
- [x] Free-form AI chat mode
- [x] Conversation evaluation and scoring
- [x] Event trigger system (relationship milestones, achievements)
- [x] Visual workflow editor (drag-and-drop)
- [x] Frontend streaming chat integration via Tauri events
- [x] Streaming evaluation and event notifications (`chat-evaluation`, `chat-events`)
- [x] Chat session lock optimization for slower LLM requests
- [x] Commercial workbench UI refresh (dashboard/chat/story/workflow shell)
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

### In Progress

- [x] Multiple character simultaneous group chat (group chat backend ready, frontend pending)
- [ ] 3D character model support
- [ ] Installer signing and distribution channel configuration

### Planned

- [ ] Voice synthesis integration (TTS for character voices)
- [ ] Music/ambient sound management
- [ ] Multi-language support (i18n)
- [x] Plugin system for custom node types (scaffold with register/list/remove)
- [ ] Cloud save sync
- [ ] Analytics dashboard (player behavior tracking)
- [ ] Template marketplace (shareable workflows/characters)
- [ ] Mobile deployment (Tauri mobile)

## Tech Stack

- **Backend**: Rust, Tauri 2.x
- **Frontend**: Vue 3, TypeScript, Vite, Pinia
- **AI**: OpenAI-compatible API, ONNX Runtime (DirectML)
- **Scripting**: Rhai
- **Rendering**: PixiJS, Live2D Cubism SDK
- **Desktop**: Tauri (WebView2 on Windows, WebKit on macOS/Linux)

## License

MIT
