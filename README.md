# Monogatari v0.9.5

An LLM-powered visual novel / galgame engine. Build interactive story experiences where AI-driven characters respond dynamically to player conversations, with automatic conversation scoring that triggers special plot events.

## What It Is

Monogatari is a development engine for creating LLM-driven text adventure games. Creators provide story presets, scene images, and character artwork (2D sprites, Live2D, 3D). Through a visual drag-and-drop workflow editor (similar to Dify/Blueprint), they pre-arrange special event nodes. Players converse with AI characters powered by large language models. The LLM evaluates conversations and scores them, triggering special storylines or guiding the narrative direction.

## Key Features

- **AI Chat Mode** - Players talk freely with LLM-driven characters. The AI stays in character using personality, background, world knowledge, streaming response events, and streamed evaluation/event notifications.
- **Conversation Scoring** - The LLM evaluates every conversation on friendliness, engagement, and creativity. Cumulative scores unlock special events.
- **Event Trigger System** - Relationship milestones, dialogue achievements, and cumulative progress trigger plot events, scene changes, and special dialogues.
- **Quality Suites** - Offline regression scenarios validate character stability, prompt-injection resistance across structured role blocks, English, Chinese, Japanese, Korean, and Unicode-obfuscated player text, group chat runtime trace evidence, relationship and fallback scoring side-channel containment, memory-poisoning resistance, memory prompt replay safety, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, knowledge-boundary stability against player-supplied retcons, evaluation-summary safety, workflow output safety, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, story-event trigger thresholds/idempotence, and event-rule snapshots without requiring live model calls.
- **Web/PWA Distribution** - Browser builds include a web app manifest, dedicated install/maskable icons, offline fallback page, and service worker cache for installable cross-platform previews outside Tauri.
- **Web Bundle Budgets** - Production builds verify small entry assets while allowing bounded lazy chunks for Three.js, GLTF loading, OrbitControls, and Live2D.
- **Dialogue Editor** - Visual branching dialogue editor with node tree, inline choice editing, speaker assignment, validation, and JSON import/export.
- **Visual Workflow Editor** - Drag-and-drop node-based editor for designing dialogue flows, branching conditions, LLM generation nodes, evaluation triggers, and scene transitions.
- **Workflow Validation** - Import/export and project-scoped save/load paths validate node ids, start/end structure, missing config fields, broken links, duplicate links, and unreachable nodes.
- **Scene Asset Library** - Project scene metadata and background files are scanned, validated, listed, and selectable as the active runtime scene.
- **Renderer Fallback Pipeline** - Story Mode resolves project assets across Tauri and Web builds, preferring Live2D, then GLB/GLTF 3D models, then 2D sprites or portraits, with runtime load-failure fallback and a generated 3D stage placeholder when no art is available.
- **Project Control Panel** - Project settings, path readiness, AI backend selection, and runtime initialization are managed from one production-oriented console.
- **Character System** - Full personality model (Big Five traits), memory system, emotion tracking, and relationship scores per character.
- **Knowledge Base** - Keyword-indexed world lore, pinned character references, and release-verified context anchors that feed into AI prompts for consistent storytelling.
- **Branching Dialogue** - Pre-scripted dialogue trees with choices, relationship changes, and flag-based conditional branching.
- **Live2D Support** - Animated character models via PixiJS + pixi-live2d-display.
- **Save/Load System** - Full game state persistence including character states, flags, variables, and chat history.
- **Rhai Scripting** - Embedded scripting engine for custom game logic, conditions, and triggers.
- **Knowledge Base Manager** - Full CRUD interface for world lore, character backgrounds, and AI context entries with category filtering, tag cloud, and keyword search.
- **Professional Character Editor** - 5-tab editor with Big Five personality sliders, radar chart visualization, emotion configuration, relationship management, knowledge entries, renderer asset diagnostics, Story Mode-style preview, emotion sprite mapping, and JSON export.
- **Audio Manager** - Manage background music, ambient sounds, and sound effects with per-track volume control and master mixer.
- **Plugin System** - Register and manage custom workflow node types, event triggers, and action handlers through a dedicated management UI.
- **Cloud Save Sync** - Project-scoped save manifests track local changes, pending uploads/downloads, cross-device conflicts, and remote preflight readiness without persisting sync tokens.
- **Multi-Language Support** - i18n scaffold with zh-CN, ja-JP, and ko-KR locale files for international deployment.
- **Template Marketplace** - Browse, import, and export community-created templates, characters, and story modules.

- **Project Export** - Export project as distributable JSON manifest with content inventory for packaging.
- **Multiple AI Backends** - OpenAI-compatible API (GPT, Claude, etc.) plus project-scoped ONNX configuration preflight with explicit runtime-unavailable guards until local ONNX execution is linked.
- **Title Screen** - Cinematic animated title screen with particle effects, glowing brand logo, and quick-access menu for game start, workflow editing, gallery, and settings.
- **CG Gallery** - Scene and character art collection viewer with grid layout, locked/unlocked states, preview modal with weather/time metadata, and color-coded thumbnails.
- **Backlog Viewer** - Full conversation history replay with character selector, role-based filtering (player/character/system), emotion badges, and jump-to-latest.
- **Full i18n Internationalization** - 280+ translation keys covering all views and UI strings. Complete Simplified Chinese (zh-CN), Japanese (ja-JP), and Korean (ko-KR) locale files for international deployment.
- **i18n-Integrated Sidebar** - All 20 navigation labels render through the `t()` translation function with automatic locale switching.
- **Achievement System** - 15 unlockable milestones tracking social, relationship, creation, and gameplay progress with progress bars and category filtering.
- **Commercial Workbench UI** - Desktop-first dashboard, streaming chat desk, story runtime, workflow authoring surface, and settings panels designed for repeated production use.

## Current Development Status

Verified on 2026-07-09:

- Frontend production build passes with `npm run build`.
- Web/PWA production build passes with `npm run build:web`, including static-hosting SPA fallback assets, dedicated install/maskable icons, copied project sample assets, and bundle-budget verification.
- Mobile shell readiness passes with `npm run verify:mobile-readiness`, covering viewport safe-area support, iOS/PWA metadata, compact Tauri shell limits, and bottom navigation safe-area padding.
- Responsive shell verification runs during `npm run build:web`, covering built 375px mobile and 768px tablet Web/PWA layout signals.
- Tauri mobile deployment preflight passes with `node scripts/verify-tauri-mobile-preflight.mjs`, covering Android/iOS command readiness, Vite `TAURI_DEV_HOST` binding, Tauri shell config, and mobile release documentation.
- Full frontend dependency audit passes with `npm audit`.
- Rust Tauri app crate passes `cargo check --locked -p llm-galgame-app`.
- Character quality suite regression tests pass inside `cargo test --locked -p llm-galgame-app`.
- Single-character and group chat prompts use the shared character mind contract and guarded response path for private reasoning leaks, identity drift, and tool-style response drift.
- The shared Rust AI prompt builder sanitizes embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings in message history and context sections so reusable integrations cannot accidentally reintroduce `[System]`/`[User]`/`[Assistant]` prompt-boundary injection.
- The legacy C# AI path mirrors role-boundary sanitization for bracket, fullwidth, XML/header, attributed XML-like, Markdown role-code-fence, comment-wrapped, punctuation-free heading, and JSON-shaped role spoofing, and redacts provider-error API secrets while the legacy solution remains part of the release gate.
- OpenAI-compatible API configuration debug output and API error surfaces redact API keys, bearer tokens, sensitive custom headers, and provider-echoed secret assignments before logs or frontend error reports can expose them.
- Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields so provider credentials remain runtime-only instead of landing in `settings.json`.
- Azure and ElevenLabs TTS provider errors redact token-shaped values, API-key assignments, authorization headers, and provider response bodies before Settings or speech-generation failures expose credentials.
- TTS synthesis runtime logs record spoken-text length metadata instead of raw dialogue, prompt text, or token-shaped content.
- Frontend runtime source ships without `console.log`/`console.debug` debug output, with release-gate coverage to keep production browser/Tauri consoles clean.
- Frontend shell navigation renders icons as escaped text rather than raw HTML, and release verification blocks `v-html` plus direct raw-HTML assignments in runtime source.
- Packaged Tauri desktop builds declare a production Content Security Policy covering local app assets, Tauri asset URLs, blob/data media, HTTPS connections, and localhost dev tooling while blocking object/frame/form surfaces and `unsafe-eval`.
- Web/PWA browser builds declare a matching app-shell Content Security Policy meta tag in both `index.html` and static-hosting fallback output, with release-gate coverage for required asset/connectivity sources and blocked object/frame/form/`unsafe-eval` surfaces.
- Web/PWA dist preparation emits a static-hosting `_headers` file with CSP, nosniff, referrer, and permissions policy headers for Netlify/Cloudflare-style hosts while keeping GitHub Pages fallback compatibility.
- Web/PWA dist preparation emits a static-hosting `_redirects` file with project asset passthrough rules and an SPA fallback rewrite for Netlify/Cloudflare-style hosts.
- Web/PWA dist preparation emits an Azure Static Web Apps `staticwebapp.config.json` with SPA navigation fallback, 404 rewrite, and matching global security headers, all tracked by release verification and artifact manifests.
- Web/PWA dist preparation emits a Vercel `vercel.json` with SPA rewrite and matching global security headers, so Vercel static deployments share the same route and browser-security baseline.
- Prompt-injection detection now covers structured role-control blocks, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, punctuation-free role headings, English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control phrases before scoring, memory writes, relationship deltas, and runtime safety traces consume player text; explicit XML, Markdown fence, and comment-wrapped role-control block bodies are omitted with their markers across Tauri, shared Rust AI, and legacy C# prompt builders.
- Chat, group chat, and quality-suite runtime traces now prove when the character mind contract and creator-pinned knowledge context were applied, including resolved pinned knowledge ref IDs for audit.
- OpenAI-compatible API streaming uses buffered SSE delta parsing so split JSON lines, split UTF-8 content, `[DONE]` markers, and final unterminated lines do not drop character response chunks.
- Prompt-injection detection covers player-authored memory writes such as "remember this as official canon" so long-term character knowledge cannot be casually poisoned by dialogue text.
- Player-authored meta-instructions are omitted from character memory writes, and legacy unsafe recent memories are replaced with guarded prompt placeholders before they can influence future replies.
- Prompt-injection text cannot advance local relationship sentiment deltas, so positive words inside meta-instructions cannot silently unlock relationship milestone events.
- Chat runtime responses emit safety trace evidence for player input wrapping, prompt-injection detection, memory guarding, response guarding, stream replacement, and relationship side-channel containment.
- Group chat character responses reuse the same runtime safety trace contract so multi-character scenes expose prompt-injection, response guard, memory guard, and relationship side-channel evidence per reply.
- ONNX backend configuration resolves model and tokenizer references under the active project data root, rejects raw filesystem paths, activates the ONNX engine after registration, and reports the backend as not ready instead of returning placeholder success while ONNX Runtime execution is not linked.
- Chat sessions now expose a restorable audit report with the latest safety trace, evaluation, story-event decisions, and triggerable events so author diagnostics survive character switching.
- Quality Suites now export runtime safety trace evidence and guard-note count summaries, and include group chat plus block-body prompt-injection scenarios that require concrete guard notes for input wrapping, response guarding, memory guarding, relationship side-channel containment, and score/event containment.
- Local fallback scoring ignores prompt-injection text for engagement and creativity boosts, so long meta-instructions cannot unlock score-gated story events when model evaluation is unavailable.
- Local fallback scoring now recognizes Chinese, Japanese, and Korean friendly, question, and creative-story signals so offline scoring remains useful when model evaluation is unavailable in international builds.
- Evaluation score parsing clamps overrange, above-scale, and negative model scores before quality reports or event triggers consume them.
- Workflow LLM nodes wrap runtime inputs as untrusted data and guard generated output before it can enter story node results.
- Workflow output safety now covers tool-role/function-call shaped text so generated node output cannot masquerade as a runtime event command.
- Workflow save/load commands resolve JSON files under the active project `workflows/` directory and reject absolute, URI-like, and traversal-shaped paths before touching disk.
- Rhai script commands and direct `ScriptEngine` callers validate payload size and hidden control characters before execution or parsing, condition expressions use shared 2,000-character/control-character validation and run through a read-only Rhai engine, workflow conditions can read relationship/evaluation context variables without mutating story state, desktop run-context previews and Web/PWA workflow previews mirror local variable, flag, signed relationship, emotion, scene, and weighted random-branch behavior for later branches without mutating persistent runtime state, workflow validation catches invalid condition and script state-key config during authoring/import, shared script state keys are normalized to portable save-friendly names, and the shared script engine caps operations, recursion, expression depth, variables, functions, and module imports.
- Character, dialogue, and knowledge loader commands resolve directory references under the active project `characters/`, `dialogue/`, and `knowledge/` roots and reject absolute, URI-like, and traversal-shaped paths before touching disk.
- Character authoring create/delete commands resolve through the active or discovered default project data root, validate portable character IDs before writing or removing JSON files under `characters/`, and deletion removes the runtime character from memory.
- Plugin listing, registration, and removal commands resolve through the active or discovered default project data root, validate portable plugin IDs before touching manifest JSON files under `plugins/`, normalize optional `.rhai` script references under that plugin root, and the Plugin workbench sends the backend manifest contract directly.
- Marketplace import/export commands resolve template references under the active project `templates/` directory, reject raw filesystem paths, and allow built-in catalog entries to import by safe catalog ID.
- Live2D model commands load only project-relative `.model3.json`/`.json` files under the active project data root, and Story Mode validates renderer asset paths before handing them to the runtime.
- Engine initialization validates that the selected project root is an existing local directory before loading managers or rebinding active project state.
- Checked-in score-gate workflow fixtures prove conversation evaluation can drive visual workflow branches and score-aware story-event unlocks.
- Chat runtime responses emit story-event trigger decisions with actual relationship values, score metrics, evaluation counts, and blocker reasons.
- Manual Chat scoring returns an atomic evaluation report with matching story-event trigger decisions and triggerable events so authors can debug score gates without waiting for periodic evaluation.
- Quality Suites reuse the same story-event decision contract as live chat so offline QA and runtime event audits stay aligned.
- Workflow Run traces expose evaluation metrics, thresholds, score sources, event trigger state, and blocker reasons for author debugging.
- Workflow canvas nodes show compact run badges for executed, pass/fail, blocked, completed, and waiting-choice states.
- Workflow Run preview context lets authors simulate scores, relationship values, evaluation counts, already-triggered events, and workflow state-node effects without live chat/model calls or persistent state mutation.
- Workflow Run preview context clamps author-provided scores and relationship values in both the editor payload and Rust execution path before score-gated branches consume them.
- Workflow Run preview presets cover unlock, low-score block, and repeat-trigger block branches for quick score-gate QA.
- Workflow Run reports graph coverage and unvisited nodes so authors can see which score/story branches still need testing.
- Workflow Run preset matrix executes all score-gate preview presets and merges graph coverage to confirm branch coverage quickly.
- Quality Suites can now pin workflow branch coverage snapshots, including the checked-in score-gate fixture's unlock, low-score, and repeat-trigger branches.
- Quality Suite reports show and export versioned audit evidence with failed-scenario ids, category summaries, safety-signal counts, and workflow coverage summaries for release QA, customer review, and branch-coverage audits.
- Quality suite schema validation rejects contradictory expectations such as out-of-range score bounds or events marked both expected and forbidden.
- Character prompts pin creator-declared knowledge references before keyword search results so core lore stays stable.
- Character content loading accepts single-object JSON files, legacy sprite field names, and optional renderer asset fields.
- Core sample characters Sakura, Luna, and Kenji ship with checked-in portrait and sprite SVG assets in both Web and bundled Tauri data roots.
- Audio Manager now controls real browser/Tauri audio elements for BGM, ambient loops, and SFX previews, with persisted track lists, per-track gain, and master/channel mixer state.
- C# legacy solution exits successfully with `dotnet test LLMAssistant.sln --no-restore`.
- Locale JSON files validate across project data and frontend fallback directories, including key coverage and Web/PWA fallback parity.
- i18n locale commands validate portable locale IDs before loading or listing JSON files under the active project `locales/` directory.
- Live2D remains on `pixi-live2d-display@0.4.0`; its transitive `gh-pages` dependency is pinned to the safe `6.3.0` line through npm overrides.
- Rust desktop dependencies are pinned through `rust-engine/Cargo.lock` for reproducible Tauri builds.
- Tauri desktop packaging configuration declares Windows MSI/NSIS targets, installer metadata, icons, WebView2 bootstrap behavior, and bundled sample project data, all checked by the release verifier.
- Installed Tauri builds discover bundled sample `data/` resources at startup and bind them as the default project root when no development data root is available.
- Analytics logs, cloud-sync manifests, and generated system/API TTS assets are written under the active project data root with sanitized filenames for portable installed desktop builds.
- Rust and legacy C# asset managers validate project-relative asset paths before file access, and save managers validate save IDs before save/load/delete/list operations, so local asset and save APIs cannot escape the active project data root through traversal-shaped input.
- Settings Cloud Sync status now consumes the backend manifest contract directly, showing local save file counts, pending upload/download work, cross-device conflicts, and remote preflight readiness while keeping sync tokens runtime-only.
- Project export emits a versioned manifest with file inventory, per-file MD5 checksums, generated asset coverage, and redacted sensitive settings for package handoff.
- Release artifact manifests can be generated with `node scripts/create-release-manifest.mjs` to capture Web/PWA and desktop installer artifact paths, SHA-256 checksums, checked-in release channel policy metadata, missing installer expectations, and verified installer signing evidence.
- One-command release verification passes with `node scripts/verify-release.mjs`, including all quality suite files, Rust core/state-key tests, Rust AI prompt/API/pipeline tests, Rust scripting and asset management tests, legacy C# AI prompt/API invariants, AI backend config, engine project root, asset/save-manager, script command, i18n locale, workflow command, content loader, character manager, plugin manager, marketplace, Live2D model, and TTS output/error/log-privacy invariants, structured role-block prompt-injection regressions, renderer asset contract checks, pinned knowledge-ref checks, locale coverage, frontend UI text artifact scanning, cloud-sync status contract checks, frontend source invariants, frontend route/sidebar coverage, Tauri packaging preflight, root and subpath Web/PWA builds, Web/PWA dist asset checks, release artifact manifest checks, and preview route smoke checks.
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
|   |   +-- views/         # 21 production views including chat, story runtime, editors, galleries, analytics, and quality gates
|   |   +-- components/    # Live2DCanvas, search, toasts, progress, dialogs, and shared UI
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

Run the automated pre-release gate first:

```bash
node scripts/verify-release.mjs
```

This verifies JSON assets, checked-in workflow files, renderer asset contracts for characters and scenes, pinned character knowledge refs, all quality suite files, workflow branch coverage snapshots, locale coverage, sensitive token patterns, frontend UI text artifacts, frontend source invariants, legacy C# AI prompt/API invariants, AI backend config, engine project root, asset/save-manager, script command, i18n locale, workflow command, content loader, character manager, plugin manager, marketplace, Live2D model, and TTS output/error/log-privacy invariants, frontend route/sidebar coverage, Tauri desktop packaging configuration, Tauri mobile deployment preflight, Rust core/state-key, AI, scripting, game, assets, and Tauri checks and tests, root and subpath Web/PWA builds with bundle budgets, Web/PWA dist assets, release artifact manifest checks, preview route smoke checks, frontend audit, and legacy C# tests.

```bash
cd frontend
npm run build:web

cd rust-engine/crates/tauri-app
cargo tauri build
```

After Web/PWA and installer builds are available, generate the distributable checksum manifest:

```bash
node scripts/create-release-manifest.mjs --channel=stable
```

The manifest is written under `release/` and records artifact SHA-256 hashes, expected Windows MSI/NSIS installer presence, release channel policy, and installer signing evidence for GitHub Release handoff. Stable and beta final manifests require Windows MSI/NSIS installers plus verified signing evidence; `--allow-missing-installers` is only a policy-gated release-preflight exception.

Installer signing evidence is stored next to each installer as `<installer>.sig.json`:

```json
{
  "schema": "monogatari-signature-evidence/v1",
  "artifact_sha256": "<installer sha256>",
  "status": "verified",
  "subject": "SakalioLabs",
  "verifier": "signtool verify /pa",
  "signed_at": "2026-07-08T00:00:00.000Z",
  "verified_at": "2026-07-08T00:00:00.000Z"
}
```

For static hosting under a subpath, set `VITE_BASE_PATH` before building:

```powershell
cd frontend
$env:VITE_BASE_PATH='/Monogatari/'
npm run build:web
Remove-Item Env:VITE_BASE_PATH
```

The web build emits `dist/404.html` for SPA fallback, `dist/.nojekyll` for GitHub Pages, `dist/_headers` and `dist/_redirects` for Netlify/Cloudflare-style hosts, `dist/staticwebapp.config.json` for Azure Static Web Apps, `dist/vercel.json` for Vercel, PWA assets, install/maskable icons, copied `data/assets` project sample assets, and `project-assets.json` so the service worker can precache sample renderer assets under the configured base path.

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
4. Save `settings.json`, configure the AI backend with the runtime API key, then initialize the runtime

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
      "api_key": "",
      "model": "gpt-4o-mini"
    }
  }
}
```

API keys are runtime-only. Set them through Settings when configuring the AI backend; `settings.json` saves retain only non-secret provider metadata.

### ONNX Mode (Local)

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

### Example Characters and Content

- **Sakura** (cheerful):: Nature-loving artist. Park walk dialogue with branching paths.
- **Luna** (thoughtful): Poetic stargazer. Stargazing dialogue with constellation lore.
- **Kenji** (honorable): Martial artist poet. Dojo visit dialogue with training themes.
- **Yuki** (mysterious): Ancient library guardian who speaks in riddles. Library encounter with branching paths.
- **Hiro** (enthusiastic): Young inventor beneath the observatory. Workshop dialogue with invention themes.
- **Aoi** (gentle): Village healer and herbal medicine expert. Clinic visit dialogue with 3 branching paths, herb lore knowledge.
- **Rin** (energetic): Chef with noodle_and_soul dialogue (12 nodes, 6 endings), Springtown cuisine knowledge.
- **Taro** (serene): Master woodcarver with woodcarver_workshop dialogue (12 nodes, 6 endings), woodcarving tradition knowledge.
- **Emi** (shy): Aspiring writer with writers_retreat dialogue (12 nodes, 7 endings), writing process knowledge.
- **Takeshi** (observant): Traveling photographer with through_the_lens dialogue (12 nodes, 7 endings), Springtown archive knowledge.
- **Nori** (serene): Postmaster with post_office_tales dialogue (12 nodes, 8 endings), Springtown history knowledge.
- **Sora** (focused): Astronomer with observatory_night dialogue (12 nodes, 8 endings), constellation knowledge.
- **Hana** (serene): Tea shop owner with Whispering Leaf dialogue (13 nodes, 8 endings), tea blends knowledge.
- **Kai** (contemplative): Wandering musician with cafe encounter dialogue (12 nodes, 5 endings), traveler lore knowledge.
- **Mio** (cheerful): Festival organizer with Starlight Festival dialogue (15 nodes, 4 endings), festival lore knowledge.
- **Springtown**: Shared world with cherry blossom park, observatory, dojo, the Great Library, and the Inventor's Workshop.

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
- [x] AI inference pipeline (API + project-scoped ONNX configuration preflight with runtime-unavailable guard)
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
- [x] Streaming evaluation and event notifications (`chat-evaluation`, `chat-event-decisions`, `chat-events`)
- [x] Chat session lock optimization for slower LLM requests
- [x] Commercial workbench UI refresh with 18-item sidebar navigation navigation (dashboard/chat/story/workflow/analytics/marketplace/plugins)
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
- [x] Title Screen with cinematic animated particle effects and quick-access menu
- [x] CG Gallery view with scene/character art collection and preview modal
- [x] Backlog viewer with conversation history replay and role-based filtering
- [x] Comprehensive i18n system with 280+ translation keys across all views
- [x] Full Simplified Chinese (zh-CN), Japanese (ja-JP), Korean (ko-KR) locale support
- [x] i18n-integrated sidebar with 20 navigation items using t() function
- [x] Quality Suites workbench for offline character stability and prompt-injection regression checks
- [x] Quality Suite workflow branch coverage snapshots for score-gated story QA
- [x] Quality Suite workbench and JSON export with stable audit summaries for QA evidence handoff
- [x] Runtime trace evidence for character mind contract and pinned knowledge context anchoring
- [x] Web/PWA distribution baseline with manifest, service worker, and offline fallback
- [x] Mobile Web/PWA shell readiness gate for safe-area viewport, install metadata, and compact Tauri shell limits
- [x] Responsive Web/PWA shell verification for built 375px mobile and 768px tablet layout signals
- [x] Tauri mobile deployment preflight gate for Android/iOS command readiness and Vite mobile dev host binding
- [x] Checked-in score-gate workflow fixture with backend execution regression for evaluation-driven story unlocks
- [x] Workflow Run score/event diagnostics for author-visible trigger debugging
- [x] Workflow canvas run badges for score-gated graph debugging
- [x] Workflow Run preview context for model-free score-gate branch testing
- [x] Workflow Run preview presets for unlock, low-score block, and repeat-trigger block QA
- [x] Workflow Run graph coverage summary with unvisited node reporting
- [x] Workflow Run preset matrix for merged score-gate branch coverage

### In Progress

- [x] Distribution channel policy and installer signing evidence gates
- [ ] Production installer code-signing credentials and final signed installer publication
- [x] Achievement system with 15 milestones, progress tracking, and localStorage persistence
- [ ] Mobile deployment (Tauri mobile)

### Planned

- [x] Voice synthesis integration (Windows SAPI TTS with emotion-based speech rate)
- [x] Music/ambient sound management
- [x] Multi-language support (i18n scaffold with locale loading and translation)
- [x] Plugin system for custom node types (scaffold with register/list/remove)
- [x] Cloud save sync with local manifest and checksum tracking
- [x] Analytics dashboard with engagement metrics and JSON export
- [x] Dialogue Editor view with visual branching node tree and inline editing
- [x] Project export command for distributable packaging
- [x] Knowledge Base Manager view with CRUD and filtering
- [x] Professional Character Editor with 5 tabs and radar chart
- [x] Frontend data sync with rust-engine content
- [x] Template marketplace scaffold (Rust backend + MarketplaceView frontend)
- [x] Plugin management frontend UI with register/list/remove
- [x] Cloud sync settings integration with project-scoped manifest status, push/pull preflight, and conflict counters
- [x] Multi-language locale files (zh-CN, ja-JP, ko-KR)
- [x] Enhanced group chat with streaming and emotion display
- [x] Release checklist document for packaging and QA gates
- [x] Mobile-responsive CSS with bottom tab navigation
- [x] Audio manager playback controls with BGM/ambient/SFX tracks, persisted lists, and master/channel mixer
- [x] Enhanced prompt engineering for character AI roleplay quality
- [x] GLTF/GLB 3D model loading with Three.js GLTFLoader, OrbitControls, animation playback
- [x] i18n composable with nested key support and locale persistence
- [x] i18n composable upgraded with nested key support and localStorage persistence
- [x] Sidebar navigation expanded to 20 items with Quality, Analytics, Marketplace, Plugins, galleries, backlog, and achievements
- [x] Title Screen, CG Gallery, Backlog viewer views
- [x] Full i18n with 280+ keys and 4 locale files (en, zh-CN, ja-JP, ko-KR)
- [x] Achievement system with 15 milestones, progress tracking, and localStorage persistence
- [ ] Mobile deployment (Tauri mobile)

## Tech Stack

- **Backend**: Rust, Tauri 2.x
- **Frontend**: Vue 3, TypeScript, Vite, Pinia
- **AI**: OpenAI-compatible API, guarded ONNX configuration preflight for future local runtime integration
- **Scripting**: Rhai
- **Rendering**: PixiJS, Live2D Cubism SDK
- **Desktop**: Tauri (WebView2 on Windows, WebKit on macOS/Linux)

## License

MIT
