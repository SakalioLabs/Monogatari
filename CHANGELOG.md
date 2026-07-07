## [0.9.3] - 2026-07-08

### Added
- **GlobalSearch component**: Ctrl+K quick-search across characters, knowledge entries, and dialogues from any view. Features expandable search panel, real-time filtering, keyboard shortcut support, and integrated into App.vue sidebar.
- **LoadingSpinner component**: Reusable loading indicator with customizable size, thickness, text, and inline mode. Integrated into HomeView dashboard for async status loading.
- **GameView SVG background loading**: Scene backgrounds now display actual SVG image files instead of generated gradients.

### Fixed
- **ChatView.vue encoding corruption**: Fixed a corrupted template expression at line 37 that caused "Element is missing end tag" build error.

## [0.9.2] - 2026-07-07

### Added
- **Kai character**: Wandering musician with 12-node cafe_encounter dialogue (5 endings), cross-character connections to Mio, Sakura, Luna, and Yuki. Traveler songs knowledge entry.
- **Hana character**: Tea shop owner with 13-node whispering_leaf dialogue (8 endings), tea blends knowledge. Richest dialogue in the collection with deep emotional arcs.
- **Auto-save in GameView**: Automatic save every 2 minutes during active dialogue with auto-save indicator badge.

### Changed
- Content inventory expanded to 12 characters, 12 dialogues, 14 knowledge entries.
- All new characters include personality Big Five traits, emotion states, relationship networks, and knowledge references.

## [0.9.1] - 2026-07-07

### Added
- **AchievementsView**: Gamification system with 15 unlockable achievements across Social, Relationships, Creation, and Gameplay categories. Features progress bars, category filtering, stats strip (unlocked/total/complete/playtime), and localStorage persistence. Achievements track first chat, message milestones, relationship scores, evaluation scores, workflow creation, knowledge entries, and more.
- **Batch i18n integration**: All remaining views now have `useI18n` imports and key `t()` string replacements: WorkflowEditor, AudioView, SceneEditorView, GroupChatView, AnalyticsView, MarketplaceView, PluginView, SceneAssetsView, CharacterEditorView, DialogueEditorView.
- **Achievements route** (`/achievements`) added to router and sidebar navigation (19 nav items total).

### Changed
- **Router** expanded to 20 routes with achievements entry.
- **Sidebar navigation** expanded to 19 items with Achievements entry.
- **Total frontend views**: 20 (up from 19).
- **i18n coverage**: All 20 views now import `useI18n` and use `t()` for at least header/title strings.

## [0.9.0] - 2026-07-07

### Added
- **TitleScreenView**: Cinematic title screen with animated particle effects, glowing logo, menu navigation, version badge, and MIT license footer.
- **CGGalleryView**: Scene and character art collection gallery with grid layout, locked/unlocked states, scene preview modal, tag pills, and color-coded thumbnails.
- **BacklogView**: Full conversation history viewer with character selector chips, role-based filtering, avatar color coding, emotion badges, timestamps, and jump-to-latest.
- **Comprehensive i18n locale system**: Expanded from 13 keys to 280+ keys covering all views.
- **Chinese locale (zh-CN)**: 280 translation keys for full Simplified Chinese support.
- **Japanese locale (ja-JP)**: 187 translation keys for Japanese market readiness.
- **Korean locale (ko-KR)**: 159 translation keys for Korean market support.
- **i18n integration in App.vue**: All 18 sidebar navigation labels use `t()` function.
- **i18n in core views**: HomeView, ChatView, GameView, SettingsView with full `t()` integration.
- **Mio character**: Festival organizer with Starlight Festival dialogue (15 nodes, 4 endings) and festival lore knowledge entry.
- **festival_night scene**: Summer night festival setting with weather/time metadata.

### Changed
- Router expanded to 19 routes.
- Sidebar navigation expanded to 18 items with CG Gallery and Backlog entries.
- App.vue now imports `useI18n` composable and uses computed nav items.
- Total frontend views: 19 (up from 16).
- Version badge updated from v0.8 to v0.9 in sidebar.
- Tauri config version bumped to 0.9.0, window title updated to "Monogatari v0.9.0".
- README updated with v0.9.0 content counts: 10 characters, 10 dialogues, 12 knowledge entries.

# Changelog

## [0.9.0] - 2026-07-07

### Added
- **TitleScreenView**: Cinematic title screen with animated particle effects, glowing logo, menu navigation (Start Game, Continue, Workflow, Gallery, Settings), version badge, and MIT license footer. Hides sidebar for immersive first impression.
- **CGGalleryView**: Scene and character art collection gallery with grid layout, locked/unlocked states, scene preview modal with weather/time-of-day metadata, tag pills, and color-coded thumbnails derived from scene IDs.
- **BacklogView**: Full conversation history viewer with character selector chips, role-based filtering (All/Player/Character), avatar color coding, emotion badges, timestamps, and jump-to-latest functionality.
- **Comprehensive i18n locale system**: Expanded from 13 keys to 280+ keys across all views covering navigation, chat, game, settings, workflow, characters, knowledge, dialogue, scene, audio, analytics, marketplace, plugins, group chat, title screen, backlog, CG gallery, and common UI strings.
- **Chinese locale (zh-CN)**: 280 translation keys for full Simplified Chinese support.
- **Japanese locale (ja-JP)**: 187 translation keys for Japanese market readiness.
- **Korean locale (ko-KR)**: 159 translation keys for Korean market support.
- **i18n integration in App.vue**: All 18 sidebar navigation labels now use `t()` function with locale-aware rendering via `useI18n()` composable.

### Changed
- **Router expanded** to 19 routes with Title Screen, CG Gallery, and Backlog entries.
- **Sidebar navigation** expanded to 18 items with CG Gallery and Backlog entries.
- **App.vue** now imports `useI18n` composable and uses `computed` nav items with `t()` for all labels.
- **Title Screen and Story Mode** routes hide the sidebar for immersive gameplay experience.
- **Total frontend views**: 19 (up from 16).
- **Version badge**: Updated from v0.8 to v0.9 in sidebar.

## [0.8.2] - 2026-07-07

### Added
- **SceneEditorView**: Visual scene management with grid/list gallery view, scene detail panel with background preview, weather/time-of-day selectors, BGM path, and tag configuration. Create, edit, and delete scenes.
- **Sidebar navigation** expanded to 17 items with Scene Editor entry.
- **Total frontend views**: 16 (up from 15).


## [0.8.1] - 2026-07-07

### Added
- **DialogueEditorView**: Visual branching dialogue editor with node tree canvas, inline choice editing, speaker assignment, validation, and JSON import/export.
- **export_project command**: Export project as JSON manifest with content inventory (characters, dialogues, knowledge, scenes) for packaging and distribution.
- **Aoi character**: Gentle healer with herbal medicine knowledge, clinic visit dialogue (11 nodes, 3 branching paths, 2 endings), and herbal lore knowledge entry.
- **CharacterGalleryView overhaul**: Search, detail panel with radar chart visualization, personality traits, quick action buttons (Chat/Edit), responsive layout.

### Changed
- **Sidebar navigation** expanded to 16 items with Dialogue Editor entry.
- **Total Tauri commands**: 30 (up from 25).
- **Total frontend views**: 15 (up from 14).
- **Content inventory**: 7 characters, 8 dialogues, 9 knowledge entries, 5 scenes.


### Fixed
- **Locale files encoding**: Fixed mojibake in zh-CN.json and ja-JP.json locale files. All translations now use proper UTF-8 encoding.
- **SettingsView language picker**: Language selector now calls loadI18n() to apply locale changes immediately without restart.

### Added
- **Japanese locale**: Complete ja-JP.json with nav, chat, game, settings, and common translations.
- **Knowledge Base Rust commands**: list_knowledge_entries, get_knowledge_entry, list_knowledge_tags Tauri commands for full KB management.
- **KnowledgeBase backend methods**: all_entries, all_tags, all_categories for comprehensive knowledge base access.

## [0.8.0] - 2026-07-07

### Added
- **Knowledge Base View** (`KnowledgeBaseView.vue`): Full knowledge base management with category filtering, tag cloud, keyword search, entry creation/editing/detail views, and card grid display.
- **Character Editor overhaul** (`CharacterEditorView.vue`): Professional 5-tab character editor with Basic Info, Personality (Big Five sliders + radar chart SVG visualization), Emotions, Relationships, and Knowledge management tabs. Includes character list sidebar, JSON export, and responsive layout.
- **Frontend data sync**: All characters (Sakura, Yuki, Hiro, Mei), scenes, knowledge entries, dialogues, and SVG backgrounds now synchronized from `rust-engine/data` to `data/` for frontend access.
- **Knowledge Base route** added to router and sidebar navigation with book icon.

### Changed
- **Sidebar navigation** expanded to 14 items with Knowledge Base entry.
- **Engine version badge** bumped to v0.8 in sidebar.
- **CharacterEditorView** completely rewritten from minimal 70-line form to 880-line professional editor with tabbed interface, personality radar chart, emotion configuration, relationship management, and knowledge entries.


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