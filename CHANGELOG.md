## [0.9.5] - 2026-07-08

### Added
- Added a configurable offline quality suite for character stability, prompt-injection resistance, relationship and fallback scoring side-channel containment, memory-poisoning resistance, memory prompt replay safety, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, evaluation-summary safety, workflow output safety, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, story-event trigger/idempotence regression, and event-rule snapshot checks.
- Added content loader path isolation tests and release-gate invariants so character, dialogue, and knowledge reload commands resolve only under the active project content directories.
- Added character manager path isolation tests and release-gate invariants so character create/delete commands use the active or discovered default project data root, safe portable IDs, and stay inside the project characters directory.
- Added plugin manager path isolation tests, Plugin workbench command-contract checks, and release-gate invariants so plugin listing, registration, and removal use the active or discovered default project data root plus safe portable IDs and optional `.rhai` script references inside the project plugins directory.
- Added script command input limits, Rhai execution budgets, and release-gate invariants so author scripts reject hidden control characters and abort runaway loops or recursion.
- Added shared script state key validation and release-gate invariants so Rhai variables, flags, workflow state writes, dialogue scripts, and save loading use portable save-friendly keys.
- Added workflow validation for script state key fields so invalid variable and flag names are caught during authoring/import before workflow execution.
- Added a read-only Rhai condition engine so condition expressions can inspect variables and flags without mutating story state.
- Added shared condition expression validation so command inputs and workflow condition nodes reject non-string, oversized, or hidden-control-character payloads before execution.
- Added TTS provider error redaction so Azure and ElevenLabs request failures, response bodies, sensitive headers, and token-shaped values are cleaned before reaching frontend error surfaces.
- Added TTS synthesis log privacy so runtime logs record spoken-text length metadata instead of raw dialogue, prompt text, or provider-token-shaped content.
- Added frontend runtime log hygiene so production source ships without `console.log`/`console.debug` debug output and the release verifier catches regressions.
- Added frontend HTML-injection hardening so shell navigation renders icons as text instead of `v-html`, with release-gate scans for raw HTML sinks.
- Added project settings runtime-secret scrubbing so API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields are omitted before `settings.json` saves or project config state returns to the frontend.
- Added read-only workflow condition context variables for relationship, evaluation scores, and evaluation count, plus matching Web/PWA preview evaluation for common condition expressions.
- Added Web/PWA workflow preview state mirroring so local `set_variable`, `set_flag`, and evaluation outputs can drive later `getVariable` and `hasFlag` conditions.
- Added Web/PWA workflow preview mirrors for relationship and emotion nodes so browser previews expose the same per-run state transitions as desktop workflow execution.
- Fixed Web/PWA workflow preview signed numeric parity so negative relationship deltas and camera offsets behave like desktop workflow execution.
- Added normalized random branch weights for desktop and Web/PWA workflow previews so weighted story branches do not collapse to the first connection or invalid negative probabilities.
- Added desktop workflow run-context state isolation so author previews can exercise variable, flag, relationship, emotion, and scene changes without mutating persistent runtime state.
- Added marketplace template path isolation tests and release-gate invariants so template import/export uses project-scoped template references instead of raw filesystem paths.
- Added Live2D model path isolation tests, renderer asset validation hardening, and release-gate invariants so model loading stays inside the active project data root.
- Added i18n locale path isolation tests and release-gate invariants so locale loading, listing, and translation use safe locale IDs inside the active project locales directory.
- Added ONNX backend config path isolation tests and release-gate invariants so local model configuration uses project-scoped model/tokenizer references and activates the ONNX engine.
- Added engine project-root validation tests and release-gate invariants so initialization binds only existing local project directories.
- Added a Quality Suites workbench view and sidebar entry for running release-gate checks from the desktop UI.
- Added Web/PWA distribution baseline with manifest metadata, offline fallback page, service worker runtime caching, and `npm run build:web`.
- Added dedicated Web/PWA install and maskable icons and release-gate checks that keep them in the manifest, app shell cache, and static-hosting dist.
- Added static-hosting preparation for Web/PWA builds, including GitHub Pages fallback assets and `VITE_BASE_PATH` subpath deployment support.
- Added mobile shell readiness verification for viewport safe-area support, iOS/PWA metadata, compact Tauri shell dimensions, and mobile navigation padding.
- Added responsive Web/PWA shell verification for built 375px mobile and 768px tablet layout signals.
- Added Tauri mobile deployment preflight verification for Android/iOS command readiness, Vite mobile host binding, and mobile release documentation.
- Added runtime trace evidence for character mind contract application and creator-pinned knowledge context anchoring, including resolved pinned knowledge ref IDs for QA audit.
- Added runtime chat story-event trigger decision evidence so authors can inspect relationship values, score metrics, evaluation counts, and blocker reasons directly from live conversations.
- Added an atomic manual scoring report command that returns conversation evaluation, matching story-event trigger decisions, and triggerable events together.
- Updated manual Chat scoring to consume the atomic scoring report for immediate author score-gate debugging.
- Aligned Quality Suite story-event reports with the same trigger decision contract used by live chat runtime responses.
- Added an explicit Web bundle budget verifier that keeps entry assets small while allowing bounded lazy renderer chunks for Three.js and Live2D.
- Added a renderer asset contract for characters with Live2D, GLB/GLTF, sprite, portrait, and generated 3D fallback support in Story Mode.
- Added a one-command release verification script covering JSON validation, all quality suite files, locale coverage, sensitive token pattern scanning, frontend UI text artifact scanning, frontend source invariants, Rust checks/tests, Web/PWA build, Web/PWA dist asset checks, frontend audit, and legacy C# tests.
- Added explainable event-trigger decisions for author tooling and quality reports, including actual relationship values, score metrics, evaluation counts, idempotence state, and blocker reasons.
- Added executable Workflow `evaluation` and `trigger_event` nodes so visual story graphs can read LLM conversation scores and drive score-aware event unlocks.
- Added executable Workflow runtime behavior for core authoring nodes: start, end, dialogue, choice, scene change, emotion change, relationship updates, and sub-workflow delegation.
- Added a guarded Workflow graph runner with execution traces, choice stop points, branch routing for conditions/scores/events, and a Run panel in the workflow editor.
- Added interactive choice selection for Workflow Run traces so authors can continue through choice branches during debugging.
- Added release-gate validation for checked-in workflow files across root and Rust data directories.
- Added a checked-in score-gate workflow fixture plus backend execution regression tests proving conversation scores can branch into score-aware story-event unlocks.
- Added score and event diagnostics to Workflow Run traces so authors can inspect evaluation metrics, thresholds, score sources, trigger decisions, and blocker reasons.
- Added Workflow canvas run badges that mark executed nodes, score pass/fail, blocked events, completed nodes, and waiting choices directly on the visual graph.
- Added a Workflow Run preview context so authors can simulate character scores, relationship values, evaluation counts, and already-triggered events while debugging score-gated story branches.
- Added frontend and Rust-side clamping for Workflow Run preview context scores/relationships before score-gated story branches consume author-simulated values.
- Added one-click Workflow preview context presets for unlock, low-score block, and repeat-trigger block scenarios.
- Added Workflow Run graph coverage summaries with executed node counts and unvisited node chips for branch QA.
- Added a Workflow Run preset matrix that executes all score-gate preview presets and merges graph coverage for branch QA.
- Added workflow command path isolation tests and release-gate invariants so backend save/load reads and writes only JSON workflows inside the active project `workflows/` directory.
- Added Quality Suite workflow coverage snapshots so release checks can prove score-gated story fixtures still cover unlock, low-score, and repeat-trigger branches.
- Added Quality Suite audit summary UI and JSON export with a stable schema marker for release QA evidence handoff.
- Added Quality Suite schema validation for score-bound ranges and contradictory expected/forbidden markers before release QA reports run.
- Added tool-role/function-call injection detection and a checked-in quality scenario proving spoofed runtime instructions cannot unlock events or alter character identity.
- Added structured role-block prompt-injection detection for XML, header, and JSON-shaped role spoofing before fallback scoring, memory, relationship, or story-event logic consumes player text.
- Added attributed XML-like role tag detection for Tauri prompt guards plus Rust and legacy C# prompt builders so `<system ...>` and `<tool ...>` prompt-control variants are omitted before role parsing.
- Added Markdown role-code-fence detection for Tauri prompt guards plus Rust and legacy C# prompt builders so backtick-fenced `system` and tilde-fenced `tool` prompt-control blocks are omitted without blocking non-role language fences.
- Added comment-wrapped role marker detection for Tauri prompt guards plus Rust and legacy C# prompt builders so HTML, C-style, and line-comment role headers are omitted before prompt assembly.
- Added punctuation-free role heading detection for Tauri prompt guards plus Rust and legacy C# prompt builders so `System Prompt`, `Developer Instructions`, and `Tool Message` headings are omitted before prompt assembly.
- Added reusable Rust AI prompt-builder boundary sanitization and release-gate `llm-ai` tests so downstream integrations cannot reintroduce role-marker prompt injection through shared prompt history or context assembly.
- Added Rust API engine secret redaction for debug output, bearer tokens, sensitive custom headers, and API error surfaces before provider credentials can leak into logs or frontend reports.
- Added legacy C# prompt-builder boundary sanitization and release-gate invariants so the retained legacy AI path cannot reintroduce role-marker prompt injection.
- Added legacy C# APIEngine error redaction for token-shaped values and JSON/header/query secret assignments so retained legacy provider failures cannot echo credentials into test or frontend error surfaces.
- Added relationship sentiment side-channel containment so prompt-injection text with positive words cannot advance relationship milestone events.
- Added fallback scoring side-channel containment so prompt-injection text cannot inflate engagement or creativity when model evaluation is unavailable.
- Added workflow tool-output containment checks proving generated node text shaped like a tool/function call is withheld before downstream story nodes consume it.
- Added memory-poisoning detection and a quality scenario proving player-authored "official canon" memory writes cannot replace creator-authored Sakura knowledge anchors.
- Added guarded character memory writes and a memory prompt replay quality scenario so stored prompt-injection text cannot re-enter future character prompts through recent memories.
- Added overrange score clamping regression coverage for above-100%, above-scale, and negative evaluator outputs before event decisions consume them.
- Added release-gate validation for frontend route, sidebar navigation, view component, and navigation locale coverage.
- Added release-gate subpath Web/PWA builds to verify static-hosting assets under `/Monogatari/` before restoring the default root-path dist output.
- Added release-gate Web/PWA preview smoke checks that start Vite preview and verify every app route returns the production SPA shell on root and subpath builds.
- Added a knowledge-boundary quality scenario and report flag to catch player-induced retcons or invented canon before they erode character knowledge stability.
- Added release-gate renderer asset contract checks for checked-in scene backgrounds and character Live2D/3D/sprite/portrait paths.
- Added Character Editor controls for emotion-specific sprite paths so creators can author Galgame expression art without editing character JSON by hand.
- Added Character Editor renderer asset diagnostics for unsupported extensions, absolute paths, external URLs, and parent traversal before assets reach the release gate.
- Added an in-editor renderer preview that mirrors Story Mode priority across Live2D, GLB/GLTF, sprite/portrait, and generated 3D fallback states.
- Added a shared frontend renderer asset selector so Story Mode and Character Editor previews use one source of truth for Live2D, 3D, sprite, portrait, and generated fallback priority.
- Added a renderer asset selector contract test to the release gate, covering fallback priority, path validation, and expression sprite resolution.
- Added real Audio Manager playback controls for BGM, ambient loops, and SFX previews with persisted track lists, path resolution across Web/Tauri builds, per-track gain, and master/channel mixer state.
- Added release-gate frontend source invariants that keep the Audio Manager tied to real audio elements, persistent mixer state, and BGM/ambient/SFX transport controls.
- Added Tauri desktop packaging metadata for Windows MSI/NSIS targets, installer icons, publisher/category descriptions, WebView2 bootstrap behavior, and bundled sample `data/` resources.
- Added release-gate validation for Tauri packaging configuration so desktop installer metadata, icons, bundled sample data, and Windows downgrade/WebView2 policy cannot drift silently.
- Added a versioned project export manifest with file inventory, per-file checksums, exportable directory coverage, and settings secret redaction for commercial package handoff.
- Added runtime chat safety trace evidence for prompt-injection detection, guarded character responses, memory guards, stream replacements, and relationship side-channel containment.
- Added runtime group chat safety trace evidence so multi-character conversations reuse the same prompt-injection, response guard, and relationship side-channel audit contract as single-character chat.
- Added Quality Suite runtime safety trace evidence and a checked-in group chat scenario proving multi-character prompt-injection attempts produce auditable guard notes.
- Added multilingual prompt-injection detection and a checked-in quality scenario for Chinese, Japanese, and Korean prompt-control attempts against score, relationship, memory, and hidden-prompt boundaries.
- Added Unicode-obfuscated prompt-injection normalization and a checked-in quality scenario for fullwidth role markers and zero-width character splitting.
- Added multilingual local fallback scoring signals and a checked-in quality scenario for friendly creative Chinese, Japanese, and Korean player text.
- Added a release artifact manifest generator with SHA-256 checksums, channel metadata, installer expectations, and code-signing readiness evidence.
- Added a checked-in release channel policy and manifest enforcement for stable/beta installer requirements, preflight exceptions, and verified installer signing evidence sidecars.
- Added release-gate validation that checked-in character pinned knowledge refs resolve to project knowledge entries across both data roots.
- Added missing Springtown lore anchors for character pinned knowledge refs so creator-declared identity and world context remain stable.
- Added checked-in portrait and sprite SVG assets for Sakura, Luna, and Kenji across Web and bundled Tauri data roots, with release-gate enforcement for core sample character renderer assets.
- Added Web/PWA dist packaging for checked-in project assets so sample backgrounds and character sprites remain reachable in static browser builds.
- Added a generated Web/PWA project asset manifest and service worker precaching so sample renderer assets are available after offline install.
- Added a restorable Chat session audit report so the latest safety trace, evaluation, story-event decisions, and triggerable events survive character switching in the author workbench.
- Added short retry handling for the release-gate frontend audit step so transient registry TLS failures do not abort otherwise passing release checks.
- Added a typed Cloud Sync status contract with project-scoped manifest analysis, pending upload/download counts, cross-device conflict evidence, Settings UI wiring, and runtime-only sync token readiness.
- Added TTS output path isolation tests and release-gate invariants so system, Azure, and ElevenLabs speech files use sanitized project `assets/tts/` filenames instead of fixed process-temp outputs.
- Added asset-manager path isolation tests and release-gate invariants so Rust and legacy C# text/JSON/binary asset reads reject absolute, URI-like, empty, and traversal-shaped paths before touching disk.
- Added save-manager path isolation tests and release-gate invariants so Rust and legacy C# save/load/delete flows reject traversal-shaped save IDs and filter mismatched save files.

### Fixed
- Restored `cargo check --locked -p llm-galgame-app` by aligning Tauri command dependencies and current core APIs.
- Rebuilt corrupted zh-CN, ja-JP, and ko-KR locale JSON files with the full 280-key i18n surface.
- Fixed frontend i18n loading so Tauri `{ locale, strings }` payloads and browser `/locales/*.json` fallback files both resolve correctly.
- Fixed guarded chat streaming so private-reasoning leak replacements overwrite the visible reply instead of appending to partial streamed text.
- Fixed guarded character-response replacement text so the safety fallback no longer triggers the private-reasoning leak detector it is meant to satisfy.
- Fixed workflow LLM generation so guarded outputs replace prompt-control/internal text before node results enter the story flow.
- Fixed knowledge loading and chat context assembly so single-object knowledge files and creator-declared character knowledge references are pinned into prompts.
- Fixed event triggering so runtime checks and release-gate snapshots share the same serializable rule metadata.
- Fixed Quality Suite data-root discovery so release-gate runs can find project quality suites and knowledge anchors from nested desktop/dev working directories.
- Fixed Quality Suite runtime parsing so malformed suite metadata, duplicate scenario ids, and blank event-rule fields are rejected before execution.
- Fixed Quality Suites workbench error feedback so suite load and run failures show actionable validation messages instead of failing silently.
- Fixed visible separator artifacts in the Scene Assets and Quality Suites workbench metadata rows.
- Fixed browser locale fallback loading so Web/PWA deployments under `VITE_BASE_PATH` subpaths fetch locale JSON from the correct base URL.
- Fixed release verification coverage for Web/PWA subpath deployments by enforcing service worker base-path source invariants.
- Fixed installed desktop builds so Tauri-bundled `data/` resources are discovered at startup and rebound as the default project root when no development data root is available.
- Fixed project-scoped analytics, cloud-sync manifests, and generated TTS assets so installed desktop builds write them under the active project data root instead of the process working directory.
- Fixed evaluation score parsing so explanatory model strings such as `Score: 8/10`, `80% friendly`, and normalized decimal text still produce stable event-trigger scores.
- Fixed event availability previews so author tooling uses the same score-aware trigger decisions as runtime event firing instead of broad event-type approximations.
- Fixed the Sakura example workflow to demonstrate a score node feeding a story-event unlock node instead of ending immediately after scoring.
- Fixed workflow runtime and validation compatibility for legacy media fields such as `track`, `sound`, and second-based `duration`.
- Moved `synthesize_speech` onto the registered Tauri command path and connected saved TTS configuration to system, Azure, and ElevenLabs synthesis.
- Cleared stale example character sprite paths that pointed at missing files so browser Story Mode falls back cleanly to the generated 3D placeholder.

### Changed
- Project export manifests now scan project JSON content directories for characters, dialogues, knowledge, and scenes, making exports useful before runtime managers are initialized.
- Character loading now accepts one-character JSON files, legacy sprite field names, and partial personality definitions with stable defaults.
- Single-character and group chat prompts now share the character mind contract and guarded response path for stronger role stability, including AI/ChatGPT identity drift and customer-support/tool-style drift replacement.
- Version metadata synchronized to v0.9.5 across frontend, Rust workspace, Tauri config, and title screen UI.

## [0.9.4] - 2026-07-08

### Added
- **BackToTop component**: Scroll-to-top button with smooth scroll animation. Appears after 300px of scroll offset, integrated globally in App.vue.
- **Takeshi character**: Traveling photographer with 12-node through_the_lens dialogue (7 endings), cross-character connections to Sakura, Hana, Sora, Kai, Mio, and Nori. Springtown photographic archive knowledge entry.
- **ConfirmDialog component**: Polished confirmation dialog with backdrop blur for delete/destructive action confirmations. Supports custom title, message, and button labels via `v-model:visible` binding.
- **System info panel**: HomeView dashboard now shows engine version (v0.9.4), character/dialogue/knowledge/scene counts, AI engine status, and runtime state with color-coded Online/Idle indicator.

### Changed
- Content inventory expanded to 15 characters, 15 dialogues, 17 knowledge entries.
- HomeView ops-grid now includes a third panel for system information between the pipeline and getting-started sections.

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
