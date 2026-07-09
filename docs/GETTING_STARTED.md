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

Optional renderer fields can be added to the same character file:

```json
{
  "live2d_model_path": "assets/live2d/hero/hero.model3.json",
  "model_3d_path": "assets/models/hero.glb",
  "sprite_path": "assets/sprites/hero_neutral.png",
  "sprite_paths": {
    "happy": "assets/sprites/hero_happy.png",
    "neutral": "assets/sprites/hero_neutral.png"
  },
  "portrait_path": "assets/portraits/hero.png"
}
```

Story Mode uses the first available renderer in this order: Live2D, GLB/GLTF 3D, emotion sprite, portrait, generated 3D placeholder. The built-in Sakura, Luna, and Kenji samples include checked-in SVG portrait and sprite assets under `assets/characters/` in both Web and bundled Tauri data roots.

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
cd frontend && npm run build:web
cd rust-engine/crates/tauri-app && cargo tauri build
```

## Web Preview / PWA Build

Before cutting a release, run the automated gate from the repository root:

```bash
node scripts/verify-release.mjs
```

This validates JSON assets, checked-in workflow files, all quality suite files, workflow branch coverage snapshots, locale coverage, frontend UI text artifacts, frontend source invariants, legacy C# AI prompt/API invariants, asset/save-manager, workflow command, content loader, character manager, plugin manager, marketplace, and TTS output path invariants, mobile shell readiness, Tauri mobile deployment preflight, Rust AI/game/assets/Tauri checks and tests, frontend audit, the Web/PWA build, generated dist assets, responsive shell layout signals, release artifact manifest checks, and legacy C# tests.

```bash
node scripts/verify-tauri-mobile-preflight.mjs
cd frontend
npm run verify:mobile-readiness
npm run build:web
npm run verify:responsive-shell
npm run preview:web
```

The Tauri mobile preflight verifies Android/iOS command readiness, Vite `TAURI_DEV_HOST` binding, compact Tauri shell config, and `docs/MOBILE_DEPLOYMENT.md`. The mobile readiness check verifies safe-area viewport metadata, iOS/PWA install metadata, compact Tauri shell limits, and bottom navigation safe-area padding. The web build emits `manifest.webmanifest`, `sw.js`, an offline fallback page, `404.html`, `.nojekyll`, copied `data/assets` project sample assets, and `project-assets.json` for static hosting. It also runs the bundle budget verifier and responsive shell verifier so entry assets stay small while 375px mobile and 768px tablet layout signals stay present in the built Web/PWA shell. The service worker registers only in production browser builds, precaches the generated project asset manifest for offline sample renderer assets, and is disabled inside Tauri.

For GitHub Pages or any subpath deployment, set the base path before building:

```powershell
cd frontend
$env:VITE_BASE_PATH='/Monogatari/'
npm run build:web
Remove-Item Env:VITE_BASE_PATH
```

Use the generated `dist/` directory as the deploy root. The service worker scope, manifest link, and asset URLs follow `VITE_BASE_PATH`.

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
| Quality | `/quality` | Character stability, knowledge-reference anchoring, workflow output, scoring, event rules, and release-gate checks |
| Marketplace | `/marketplace` | Template browsing |
| Plugins | `/plugins` | Plugin management |
| Audio | `/audio` | BGM/SFX management |
| Scene Assets | `/assets` | Background and scene management |
| Character Editor | `/character-editor` | Character customization |

## Troubleshooting

- **No characters loading**: Check that JSON files are in `rust-engine/data/characters/`
- **AI not responding**: Verify API key in Settings > AI Backend
- **Build fails**: Run `npm audit fix` and ensure Rust toolchain is up to date
