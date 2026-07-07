# Changelog

## [0.7.2] - 2026-07-07

### Changed
- **README comprehensive update**: Version bumped to v0.7.2, architecture docs updated with all 13 views and 4 components, features section expanded with Audio Manager, GLTF 3D, and i18n.
- **CHANGELOG synchronized** with all changes since v0.6.0.


## [0.7.0] - 2026-07-07

### Added
- **Hiro character**: Young enthusiastic inventor with workshop dialogue (5 endings), knowledge entry, and workshop scene.
- **Yuki character**: Mysterious library guardian with branching dialogue (3 endings), knowledge entry, and Great Library scene.
- Engine now ships with **5 example characters** (Sakura, Luna, Kenji, Yuki, Hiro), **6 dialogue scripts**, **7 knowledge entries**, and **4 scenes**.


## [0.6.4] - 2026-07-07

### Changed
- **Tauri config version bumped** to 0.6.3 to match application version.
- **Release checklist** added at docs/RELEASE_CHECKLIST.md covering frontend, Rust backend, content, AI integration, workflow editor, audio, i18n, cloud sync, and distribution verification.


## [0.6.3] - 2026-07-07

### Changed
- **Enhanced AI prompt engineering**: Both streaming and non-streaming character AI prompts redesigned with stricter roleplay rules, emotional mirroring, varied speech patterns, and character growth awareness.


## [0.6.2] - 2026-07-07

### Added
- **GLTF 3D Model Loading**: CharacterModelView now loads .glb/.gltf models via Three.js GLTFLoader with OrbitControls, animation playback, ambient+directional lighting, and graceful fallback to a placeholder cube on error.
- **i18n nested key support**: Upgraded i18n composable with dot-notation nested keys, localStorage locale persistence, and local JSON file fallback.

### Changed
- CharacterModelView completely rewritten from static placeholder to full 3D pipeline with dynamic model loading and watch-based model path reactivity.


## [0.6.1] - 2026-07-07

### Added
- **Audio Manager** (AudioView.vue): Full BGM/SFX management with track listing, per-track volume control, play/pause, and master mixer panel with BGM/SFX/Voice channels.
- **Audio route and nav**: Added /audio route and sidebar navigation item for audio management.
- **i18n nested key support**: i18n.ts composable now supports dot-notation nested keys with localStorage locale persistence and local JSON fallback.
- **Enhanced prompt engineering**: Character AI system prompt redesigned with clearer roleplay instructions.


## [0.6.0] - 2026-07-07

### Added
- **Plugin Management UI** (`PluginView.vue`): Full frontend view for registering, listing, and removing custom plugins with modal registration form and status indicators.
- **Cloud Sync Settings** (SettingsView): Integrated cloud sync configuration with push/pull buttons, sync status display (last sync, file count, conflicts), and endpoint/token configuration.
- **i18n Locale Files**: Added zh-CN, ja-JP, and ko-KR locale files covering navigation, chat, game, settings, and common UI strings for multi-language support.
- **Sidebar Navigation**: Added Analytics and Marketplace nav items to main sidebar; added Plugins nav item.
- **Router Updates**: Added `/marketplace` and `/plugins` routes with lazy-loaded views.
- **Marketplace Dashboard Tile**: Added Marketplace tile to HomeView dashboard with community template browsing link.
- **Enhanced Group Chat**: Added streaming listener support, emotion display, relationship scores per participant, and animated spinner for typing indicators.

### Fixed
- **HomeView Dashboard**: Fixed Analytics tile route from `/settings` to `/analytics`.

### Changed
- Dashboard now shows 10 feature tiles covering all major modules.
- Sidebar navigation expanded to 12 items for complete feature coverage.
- Commercialization progress updated to reflect new capabilities.


## v0.5.0 - 2026-07-07 (Commercialization Push)

### Bug Fixes
- **Critical**: Fixed compile error in `chat.rs` where `.unwrap_or(0.0)` was called on an `f32` value in `check_event_triggers`. This blocked `cargo check` from passing.
- **Frontend**: Fixed SettingsView.vue broken HTML structure where the TTS section was misplaced inside the first panel's panel-head div.
- **Router**: Added missing `/characters` and `/group-chat` routes that were linked in sidebar but had no route definitions.

### Backend Improvements
- **TTS**: Upgraded `tts.rs` from stub to real Windows SAPI integration. `synthesize_speech` now invokes PowerShell SAPI COM to generate actual WAV audio files with emotion-based speech rate adjustment. `get_available_voices` discovers installed system voices.
- **Analytics**: Upgraded `analytics.rs` from stub to real implementation with in-memory event store, file persistence to `data/analytics.json`, and aggregation logic that computes top characters, top choices, session counts, and conversation metrics from recorded events.
- **Cloud Sync**: Upgraded `cloud_sync.rs` from stub to real local file-based sync with MD5 checksum tracking, manifest persistence, device-aware conflict detection, and pending upload counting.

### Frontend Improvements
- **Analytics Dashboard**: New `AnalyticsView.vue` with metrics strip (events, sessions, conversations, relationship score), top character/choice rankings, engagement overview, and JSON export functionality.
- **Dashboard**: Added Characters, Group Chat, and Analytics feature tiles to the home dashboard.
- **Dashboard Readiness**: Updated commercialization progress to include analytics dashboard, i18n scaffold, plugin system, cloud sync, and bug fix milestones.
- **Version Badge**: Updated sidebar version from v0.2 to v0.5.

### Documentation
- Updated CHANGELOG with v0.5.0 release notes.

## v0.5.1 - 2026-07-07 (Commercialization Continued)

### Features
- Template marketplace scaffold with list, export, and import commands (Rust backend)
- MarketplaceView frontend with template browsing, filtering, and import functionality
- Three.js dependency added for 3D character model support
- CharacterModelView component with Three.js dynamic import and rotation animation
- Tauri app config rebranded to Monogatari v0.5.0 (product name, identifier, window title)
- Game store enhanced with saveGame, loadGame, listSaves, deleteSave, setActiveScene, getRelationshipScore

### Bug Fixes
- Fixed Tauri config to use proper Monogatari branding instead of generic LLM Galgame Engine

---

## v0.4.1 - 2026-07-06

### Features
- i18n scaffold with locale loading, listing, and translation commands (EN/JA/ZH/KO locale files).
- Character management CRUD with create, delete, and summary commands.
- Korean locale file for i18n support.
- Example characters and content documentation in README.

### Content
- Added Kenji character with dojo knowledge for group chat dynamics.
- Added Kenji dojo dialogue with martial arts and poetry themes.
- Added Chinese locale file for i18n support.
- Added English and Japanese locale files for i18n support.
- Added dynamic effects workflow demo with camera, shake, random branch nodes.

---

## v0.4.0 - 2026-07-06

### Features
- Cloud save sync scaffold with push/pull/conflict resolution commands.
- Analytics scaffold with event recording and summary commands.
- Plugin system scaffold for custom workflow node types with register/list/remove.
- Springtown world knowledge entry for shared universe context.
- Sakura nature diary knowledge entry for AI context.
- Sakura park walk dialogue with cherry blossom themes and branching paths.

### Dashboard
- Updated Dashboard with Group Chat, Characters tiles and new readiness items.

### Documentation
- Updated README with latest features, characters, examples, and roadmap.

---

## v0.3.0 - 2026-07-06

### Features
- Multi-character simultaneous group chat backend (`multi_chat.rs`).
- TTS integration scaffold with voice assignment (`tts.rs`).
- 21 workflow node types with execution handlers for all types (added narration, bgm, sfx, wait, random_branch, sub_workflow, camera, shake nodes).
- Workflow validation with comprehensive error checking.

### Fixes
- Async-safe chat evaluation (blocking_read fix).
- Cargo dev profile optimization for faster builds.

### Frontend
- GroupChatView for multi-character conversations.
- CharacterGalleryView with personality trait visualization.
- CharacterEditorView for character customization.
- TTS settings in Settings view.
- Workflow editor CSS improvements.

---

## v0.2.0 - 2026-07-05

### Features
- Core engine architecture (EventBus, ServiceLocator, GameClock).
- AI inference pipeline (API + ONNX with DirectML).
- Character system (personality, memory, emotions, relationships).
- Dialogue system (branching, choices, flags, scripts).
- Knowledge base (keyword search, category/tag indexing).
- Scripting engine (Rhai-based).
- Save/load system.
- Free-form AI chat mode with streaming.
- Conversation evaluation and scoring.
- Event trigger system (relationship milestones, achievements).
- Visual workflow editor (drag-and-drop).
- Scene asset management.
- Project configuration panel.
- Live2D rendering (PixiJS).
- Tauri desktop application.
- Professional dark theme UI design system.
- Browser preview fallback for non-Tauri UI review.