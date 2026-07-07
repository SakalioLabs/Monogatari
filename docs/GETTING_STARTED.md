# Getting Started with Monogatari

## Quick Setup

1. Clone the repository
2. Install frontend dependencies: `cd frontend && npm install`
3. Run the dev server: `cd rust-engine/crates/tauri-app && cargo tauri dev`

## First Steps After Launch

1. **Configure AI**: Go to Settings > AI Backend and enter your API key
2. **Load Characters**: Characters are auto-loaded from `rust-engine/data/characters/`
3. **Start Chatting**: Open AI Chat from the sidebar and select a character
4. **Try Story Mode**: Open Story Mode for branching dialogue playback

## Creating Your First Character

Create a JSON file in `rust-engine/data/characters/`:

```json
{
  "id": "my_character",
  "name": "My Character",
  "description": "A brief description",
  "background": "Character backstory",
  "personality": {
    "openness": 0.7,
    "extraversion": 0.5,
    "agreeableness": 0.6,
    "speech_style": "Friendly and casual"
  }
}
```

## Creating a Dialogue

Create a JSON file in `rust-engine/data/dialogue/`:

```json
{
  "id": "my_dialogue",
  "title": "My Dialogue",
  "start_node_id": "start",
  "nodes": {
    "start": {
      "speaker_id": "my_character",
      "text": "Hello there!",
      "choices": [
        { "text": "Hi!", "next_node_id": "response" }
      ]
    },
    "response": {
      "speaker_id": "my_character",
      "text": "Nice to meet you!",
      "choices": []
    }
  }
}
```

## Building for Production

```bash
cd frontend && npm run build
cd rust-engine/crates/tauri-app && cargo tauri build
```

## Available Views

| View | Path | Description |
|------|------|-------------|
| Dashboard | `/` | Engine status and navigation |
| AI Chat | `/chat` | Free-form conversation with AI characters |
| Story Mode | `/game` | Branching dialogue playback |
| Workflow | `/editor` | Visual node-based workflow editor |
| Characters | `/characters` | Character gallery |
| Group Chat | `/group-chat` | Multi-character conversations |
| Settings | `/settings` | Project and AI configuration |
| Analytics | `/analytics` | Engagement metrics |
| Marketplace | `/marketplace` | Template browsing |
| Plugins | `/plugins` | Plugin management |
| Audio | `/audio` | BGM/SFX management |
| Scene Assets | `/assets` | Background and scene management |
| Character Editor | `/character-editor` | Character customization |

## Troubleshooting

- **No characters loading**: Check that JSON files are in `rust-engine/data/characters/`
- **AI not responding**: Verify API key in Settings > AI Backend
- **Build fails**: Run `npm audit fix` and ensure Rust toolchain is up to date