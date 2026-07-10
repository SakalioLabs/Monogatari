# Monogatari Engine Runtime (Rust + Tauri)

The native runtime and command layer for the Monogatari low-code game development engine.

## Features

- **LLM-Powered Characters**: AI-driven characters with personality, memory, and emotional states
- **Live2D Support**: Animated character models using Live2D Cubism SDK
- **Visual Workflow Editor**: Create stories visually without coding (Dify-style)
- **Targeted AI Backends**: Windows ONNX inference through required DirectML, plus OpenAI-compatible development APIs
- **Dialogue System**: Branching dialogue trees with choices and relationship tracking
- **Knowledge Base**: World lore and context system for consistent storytelling
- **Save/Load System**: Full game state persistence
- **Scripting Engine**: Rhai-based scripting for advanced logic
- **Windows Runtime**: DirectML-backed local inference for packaged Windows projects; other native targets remain future work

## Architecture

```
rust-engine/
|-- crates/
|   |-- core/          # Engine foundation
|   |-- ai/            # API and DirectML inference
|   |-- game/          # Runtime characters, dialogue, knowledge, and scenes
|   |-- assets/        # Asset and save management
|   |-- scripting/     # Rhai scripting engine
|   `-- tauri-app/     # Tauri desktop application
`-- data/              # Bundled project data
```

## Quick Start

### Prerequisites

- The toolchain pinned in `rust-toolchain.toml`
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
      "api_key": "",
      "model": "gpt-3.5-turbo"
    }
  }
}
```

API credentials are supplied at runtime through Settings and are scrubbed from project files.

#### Windows DirectML Mode
Windows packages load a project-relative ONNX model and standard Hugging Face `tokenizer.json`. The runtime requires DirectML, rejects CPU fallback, accepts full-sequence causal-LM graphs with supported token inputs and float32 `logits`, and activates only after initialization succeeds.

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
