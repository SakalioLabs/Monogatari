# LLM Galgame Engine (Rust + Tauri)

A powerful visual novel / galgame engine powered by Large Language Models, built with Rust and Tauri.

## Features

- **LLM-Powered Characters**: AI-driven characters with personality, memory, and emotional states
- **Live2D Support**: Animated character models using Live2D Cubism SDK
- **Visual Workflow Editor**: Create stories visually without coding (Dify-style)
- **Multiple AI Backends**: Support for OpenAI-compatible APIs plus project-scoped ONNX configuration preflight with explicit runtime-unavailable guards until local ONNX execution is linked
- **Dialogue System**: Branching dialogue trees with choices and relationship tracking
- **Knowledge Base**: World lore and context system for consistent storytelling
- **Save/Load System**: Full game state persistence
- **Scripting Engine**: Rhai-based scripting for advanced logic
- **Cross-Platform**: Runs on Windows, macOS, and Linux via Tauri

## Architecture

```
rust-engine/
├── crates/
│   ├── core/          # Engine foundation (EventBus, ServiceLocator, GameClock)
│   ├── ai/            # LLM inference (API engine, ONNX engine, Pipeline)
│   ├── game/          # Game logic (Characters, Dialogue, Knowledge, Scenes)
│   ├── assets/        # Asset and save management
│   ├── scripting/     # Rhai scripting engine
│   └── tauri-app/     # Tauri desktop application
├── data/              # Game data (characters, dialogues, knowledge)
└── frontend/          # Vue.js frontend with Live2D support
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 18+
- npm or yarn

### Build & Run

```bash
# Install frontend dependencies
cd frontend
npm install

# Build and run the Tauri app
cd ../crates/tauri-app
cargo tauri dev
```

### Production Build

```bash
cd crates/tauri-app
cargo tauri build
```

## Usage

### No-Code Mode (Workflow Editor)

1. Open the Workflow Editor from the home screen
2. Drag and drop nodes from the palette
3. Connect nodes to create dialogue flows
4. Configure node properties in the side panel
5. Save and export your workflow

### Code Mode

```rust
use llm_game::characters::Character;
use llm_game::dialogue::DialogueManager;

// Create a character
let mut character = Character::new("sakura", "Sakura");
character.description = "A cheerful girl who loves cherry blossoms".to_string();

// Start a dialogue
let mut dm = DialogueManager::new();
dm.load_script(Path::new("data/dialogue/example_dialogue.json")).await?;
dm.start_dialogue("meeting_sakura").await?;
```

### AI Configuration

#### API Mode (OpenAI-compatible)
```json
{
  "ai": {
    "provider": "api",
    "api": {
      "base_url": "https://api.openai.com/v1",
      "api_key": "your-key-here",
      "model": "gpt-3.5-turbo"
    }
  }
}
```

#### ONNX Mode (Local)
ONNX configuration is project-scoped and validated, but this build does not link an ONNX Runtime executor yet. ONNX inference and streaming fail with an explicit runtime-unavailable error instead of returning placeholder character text; use the API backend for production dialogue until the local runtime integration is enabled.

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
  "description": "A cheerful girl",
  "personality": {
    "openness": 0.8,
    "extraversion": 0.7,
    "speech_style": "cheerful"
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
      "id": "start",
      "speaker_id": "sakura",
      "text": "Hello!",
      "choices": [
        { "text": "Hi!", "next_node_id": "response" }
      ]
    }
  }
}
```

## License

MIT
