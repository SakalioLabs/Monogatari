# Monogatari API Reference

All Tauri commands are invoked from the frontend via `invokeCommand(commandName, args)`.

## Engine

Engine `projectPath` values must resolve to an existing local project directory before initialization binds runtime state. Empty input reuses the active/default project root; URI-like input, control characters, missing paths, and file paths are rejected before managers load content.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `initialize_engine` | `{ projectPath: string }` | `void` | Initialize engine with data path |
| `get_engine_status` | - | `EngineStatus` | Get current engine state |

## Characters

Content loader `directory` values are project content references, not arbitrary filesystem paths. `characters` resolves to the active project `characters/` directory; nested references resolve under that same content root. Absolute paths, URI-like prefixes, empty path segments, and `.`/`..` traversal are rejected before load.

Character authoring IDs are portable slugs, not filenames or paths. `create_character.id` and `delete_character.characterId` may contain only ASCII letters, numbers, underscores, or hyphens; the backend resolves the active or discovered default project data root and writes or deletes only `<id>.json` directly under `characters/`.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `get_characters` | - | `CharacterInfo[]` | List all loaded characters |
| `get_character` | `{ characterId: string }` | `CharacterInfo` | Get single character |
| `load_characters` | `{ directory }` | `usize` | Reload characters from project `characters/` |
| `create_character` | `{ character: object }` | `void` | Create new character |
| `delete_character` | `{ characterId: string }` | `void` | Delete character |
| `get_character_summaries` | - | `CharacterSummary[]` | Lightweight character list |

## Chat (Core Feature)

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `send_chat_message` | `{ characterId, message }` | `ChatResponse` | Send message, get AI response |
| `send_chat_message_stream` | `{ characterId, message }` | `void` | Streaming chat via Tauri events |
| `get_chat_history` | `{ characterId }` | `ChatMessage[]` | Get conversation history |
| `get_chat_session_audit` | `{ characterId }` | `ChatSessionAuditReport` | Restore latest safety, evaluation, and story-event audit state |
| `clear_chat_history` | `{ characterId }` | `void` | Clear conversation |
| `evaluate_conversation` | `{ characterId }` | `Evaluation` | Manually trigger scoring |
| `evaluate_conversation_report` | `{ characterId }` | `ConversationEvaluationReport` | Manually score and return matching story-event decisions plus triggerable events |
| `get_relationship_score` | `{ characterId }` | `float` | Get relationship value |
| `get_available_events` | `{ characterId }` | `TriggeredEvent[]` | Get unlockable events |
| `preview_event_triggers` | `{ characterId }` | `EventTriggerDecision[]` | Explain current story-event trigger state |

## Story Events

Story events load from the active project's configured `paths.events` directory, defaulting to `events/`. A missing directory in a legacy project uses the bundled compatibility catalog; an existing empty directory intentionally disables automatic story events. Catalog loading rejects invalid schemas, duplicate IDs, unsupported score metrics or actions, unsafe configured paths, oversized files, symlinks, and character scopes that reference unknown project characters. Real chat and workflow triggers atomically apply typed scene, dialogue, ending, and script-flag effects to persistent story progress; preview and Quality Suite paths do not mutate it.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `get_story_event_catalog` | - | `StoryEventCatalogSnapshot` | Return active definitions, trigger rules, source, count, and catalog fingerprint |
| `get_story_content_access` | - | `StoryContentAccessSnapshot` | Return event-derived scene/dialogue/ending gates, source events, and current unlock decisions |
| `get_story_progress` | - | `StoryProgressSnapshot` | Return applied event scopes, unlock sets, application counts, and progress fingerprint |
| `reload_story_event_catalog` | - | `StoryEventCatalogSnapshot` | Validate and atomically replace the active project catalog |
| `save_story_event_catalog` | `{ document, expectedCatalogFingerprint }` | `StoryEventCatalogSnapshot` | Validate and rollback-safely replace a single-document project catalog with optimistic concurrency |

### Streaming Events
- `chat-chunk` - Token-by-token response
- `chat-complete` - Full response text
- `chat-emotion` - Detected emotion
- `chat-relationship` - Relationship delta
- `chat-evaluation` - Conversation scores
- `chat-event-decisions` - Explainable story-event trigger decisions
- `chat-events` - Triggered special events
- `chat-event-applications` - Applied effects, idempotence state, and progress fingerprints

## Dialogue

Dialogue loader `directory` values resolve under the active project `dialogue/` directory. `dialogue` reloads the canonical dialogue folder, and nested references remain inside that root.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `start_dialogue` | `{ dialogueId }` | `DialogueState` | Begin an accessible dialogue tree, lazily loading project dialogue assets when needed |
| `preview_dialogue` | `{ dialogueId }` | `DialogueState` | Start a validated author preview without requiring player unlock progress |
| `list_dialogues` | - | `DialogueCatalogEntry[]` | List deterministic dialogue metadata with event-derived access decisions |
| `get_dialogue_authoring_catalog` | - | `DialogueAuthoringCatalogSnapshot` | Load complete normalized graphs, source paths, access decisions, and stable fingerprints |
| `save_dialogue_definition` | `{ dialogue, originalDialogueId?, expectedCatalogFingerprint }` | `DialogueAuthoringCatalogSnapshot` | Atomically create or replace a dialogue, validate the complete catalog, and hot-reload runtime scripts |
| `delete_dialogue_definition` | `{ dialogueId, expectedCatalogFingerprint }` | `DialogueAuthoringCatalogSnapshot` | Delete only after stale-write plus Story Event and ending reference checks |
| `advance_dialogue` | - | `void` | Next dialogue node |
| `select_choice` | `{ choiceIndex }` | `void` | Player picks choice |
| `get_dialogue_state` | - | `DialogueState` | Current dialogue state |
| `load_dialogues` | `{ directory }` | `usize` | Reload dialogues from project `dialogue/` |

## Knowledge

Knowledge loader `directory` values resolve under the active project `knowledge/` directory. `knowledge` reloads the canonical knowledge folder, and nested references remain inside that root.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `search_knowledge` | `{ query, limit }` | `KnowledgeEntry[]` | Search knowledge base |
| `load_knowledge` | `{ directory }` | `usize` | Reload knowledge from project `knowledge/` |

## AI Backend

ONNX `modelPath` and `tokenizerPath` values are project-relative references under the active project data root. Model references must end in `.onnx`, tokenizer references must end in `.json`, and absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, and non-portable path segments are rejected. `configure_onnx` registers and activates the ONNX backend.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `configure_api` | `{ baseUrl, apiKey, model }` | `void` | Set OpenAI-compatible API |
| `configure_onnx` | `{ modelPath, tokenizerPath }` | `void` | Set active project-scoped ONNX model |
| `generate_response` | `{ prompt, options }` | `InferenceResult` | One-shot generation |
| `generate_stream` | `{ prompt, options }` | `void` | Streaming generation |
| `get_ai_status` | - | `AiStatus` | Current AI configuration |

## Workflow

Workflow command `path` values are project workflow references, not arbitrary filesystem paths. `workflow.json` resolves to the active project `workflows/workflow.json`; `workflows/foo.json` is also accepted. Absolute paths, URI-like prefixes, empty path segments, `.`/`..` traversal, and non-JSON files are rejected before save/load.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `get_workflow_nodes` | - | `NodeTypeInfo[]` | Available node types |
| `execute_workflow_node` | `{ nodeId, context }` | `NodeResult` | Execute single node |
| `validate_workflow` | `{ workflow }` | `ValidationResult` | Validate graph and active-catalog event references |
| `save_workflow` | `{ workflow, path }` | `void` | Save workflow JSON under project `workflows/` |
| `load_workflow` | `{ path }` | `Workflow` | Load workflow JSON from project `workflows/` |

## Multi-Character Chat

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `start_group_chat` | `{ characterIds }` | `GroupSession` | Start group conversation |
| `send_group_message` | `{ session, message }` | `GroupSession` | Send to group |
| `get_group_chat_characters` | - | `[string, string][]` | Available characters |

## Save/Load

`saveId` values are opaque portable identifiers returned by `save_game` or `list_saves`; they are not file paths. Runtime save managers reject traversal-shaped IDs and filter mismatched save files before load/delete/list operations. Omitting `saveId` creates a UUID-backed manual save; passing a stable ID overwrites that quick-save or auto-save slot.

New saves use `monogatari-game-save/v3`. The snapshot restores scene history, the active dialogue cursor and local state, typed Rhai variables, character emotion/relationships/full memory, chat messages, evaluation state, safety traces, triggered event IDs, and validated story progress. Legacy schema-less v1 and v2 saves remain readable; known per-character triggered events migrate into the progress ledger and reconstruct their current typed unlock effects.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `save_game` | `{ saveName, saveId? }` | `string` | Save complete runtime state; optionally overwrite a stable slot |
| `load_game` | `{ saveId }` | `string` | Restore game state by safe save ID |
| `list_saves` | - | `SaveInfo[]` | List all saves |
| `delete_save` | `{ saveId }` | `void` | Delete save by safe save ID |

## Scenes

Scene and renderer asset paths are project-relative asset references. Runtime asset managers reject absolute paths, URI-like prefixes, empty path segments, control characters, and `.`/`..` traversal before reading text, JSON, binary assets, or directory listings.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `list_scene_assets` | - | `SceneAssetCatalog` | List all scenes, backgrounds, and diagnostics |
| `list_story_scenes` | - | `StorySceneInfo[]` | List scenes with event-derived access decisions |
| `get_scene_authoring_catalog` | - | `SceneAuthoringCatalogSnapshot` | Load authored and background-inferred scenes with source, access, asset, and fingerprint evidence |
| `save_scene_definition` | `{ scene, originalSceneId?, expectedCatalogFingerprint }` | `SceneAuthoringCatalogSnapshot` | Atomically create, update, or promote inferred scene metadata |
| `delete_scene_definition` | `{ sceneId, expectedCatalogFingerprint }` | `SceneAuthoringCatalogSnapshot` | Delete metadata only after Story Event, ending, and workflow reference checks; background files remain untouched |
| `get_current_scene` | - | `ActiveScene` | Current active scene |
| `set_scene` | `{ sceneId }` | `SceneInfo` | Set an authoring/runtime scene without player gate enforcement |
| `enter_story_scene` | `{ sceneId }` | `SceneInfo` | Enter a scene through Playtest with unlock enforcement |

## Endings

Ending assets live under project `endings/`. Player launch checks the ending unlock and any event-derived gates on its referenced scene and dialogue. Author saves serialize through a project content lock, compare the catalog fingerprint observed at load time, validate both references, stage a replacement and backup, reload the complete catalog, and roll back rejected writes.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `list_story_endings` | - | `StoryEndingCatalogEntry[]` | List versioned ending assets with access decisions |
| `get_story_ending_catalog` | - | `StoryEndingCatalogSnapshot` | Load editable definitions, source paths, access decisions, and stable fingerprints |
| `save_story_ending` | `{ ending, originalEndingId?, expectedCatalogFingerprint }` | `StoryEndingCatalogSnapshot` | Atomically create or replace an ending after reference and concurrency validation |
| `delete_story_ending` | `{ endingId, expectedCatalogFingerprint }` | `StoryEndingCatalogSnapshot` | Delete an ending only after stale-write and Story Event reference checks |
| `start_story_ending` | `{ endingId }` | `StoryEndingLaunch` | Validate and launch the ending's scene and dialogue |
| `preview_story_ending` | `{ endingId }` | `StoryEndingLaunch` | Launch a valid author preview without requiring player unlock progress |

## Scripting

Scripting command text is author-controlled data, not an unbounded transport. Direct Rhai execution, condition evaluation, and DSL parsing reject hidden control characters and oversized payloads before invoking the runtime; every `ScriptEngine::execute` caller repeats the shared Rhai source validation and caps operations, call depth, expression depth, variables, functions, and module imports so runaway scripts fail instead of consuming the workbench. Condition evaluation uses shared 2,000-character/control-character validation and a read-only Rhai engine that can inspect variables and flags but does not register `setVariable` or `setFlag`.

Script variable and flag names are persisted state keys. They are trimmed, limited to 128 characters, and restricted to ASCII letters, numbers, dots, underscores, and hyphens before script execution, workflow nodes, dialogue scripts, or save loading can write them. Workflow validation applies the same state-key rule to `set_variable.variable_name`, `set_flag.flag_name`, and optional `evaluation.variable_name` fields, and applies shared condition-expression validation to `condition.condition`, before imported graphs are accepted as valid. Workflow condition nodes also receive read-only temporary variables for `relationship`, `relationship_score`, `evaluation_count`, `friendliness`, `engagement`, `creativity`, `overall`, and their `_score` aliases so score- and relationship-gated branches can run from chat state or preview context without writing script state. Desktop run-context previews and browser-only Web/PWA workflow previews keep a per-run local state mirror so `set_variable`, `set_flag`, evaluation `variable_name`, signed `relationship`, `emotion_change`, `scene_change`, and weighted `random_branch` outputs can drive later conditions, event gates, and trace diagnostics without persisting data.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `execute_script` | `{ script }` | `ScriptResult` | Run Rhai script |
| `evaluate_condition` | `{ condition }` | `boolean` | Evaluate condition |
| `parse_script` | `{ script }` | `Ast` | Parse Rhai AST |

## TTS

Generated system, Azure, and ElevenLabs speech files are written under the active project `assets/tts/` directory with sanitized character/provider filename components. Azure and ElevenLabs provider errors redact token-shaped values, API-key assignments, authorization headers, and response bodies before returning `TtsResult.error`. Synthesis logs record text length metadata instead of raw spoken text.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `configure_tts` | `{ config }` | `void` | Set TTS provider config |
| `set_character_voice` | `{ characterId, voiceId }` | `void` | Assign voice to character |
| `synthesize_speech` | `{ text, voiceId }` | `string` | Generate audio file |
| `get_available_voices` | - | `VoiceInfo[]` | List available voices |

## Plugin System

Plugin manifest IDs are portable slugs, not filenames or paths. `register_plugin.manifest.id` and `remove_plugin.pluginId` may contain only ASCII letters, numbers, underscores, or hyphens; the backend resolves the active or discovered default project data root and writes or deletes only `<id>.json` directly under `plugins/`. Optional `manifest.script_path` values are plugin-root-relative `.rhai` references; absolute paths, URI/drive prefixes, backslashes, empty segments, `.`/`..` traversal, and non-portable segment characters are rejected before the manifest is stored or listed.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `list_plugins` | - | `PluginManifest[]` | List installed plugins |
| `register_plugin` | `{ manifest }` | `string` | Register plugin manifest under project `plugins/` |
| `remove_plugin` | `{ pluginId }` | `string` | Remove plugin manifest by safe ID |

## Cloud Sync

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `configure_cloud_sync` | `{ provider, endpoint?, apiKey? }` | `string` | Set local/remote preflight sync mode without persisting token values |
| `get_sync_status` | - | `CloudSyncStatus` | Get manifest-backed sync state, pending work, and conflict counts |
| `push_saves_to_cloud` | `{ saveIds? }` | `string` | Update project-scoped save manifest entries |
| `pull_saves_from_cloud` | - | `CloudSaveEntry[]` | Read valid manifest entries for sync inspection |
| `resolve_sync_conflict` | `{ saveId, useLocal }` | `string` | Resolve a manifest conflict |

## Analytics

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `record_analytics_event` | `{ event }` | `void` | Record event |
| `get_analytics_summary` | - | `AnalyticsSummary` | Get metrics |
| `export_analytics` | - | `string` | Export as JSON |

## i18n

i18n `locale` values are portable locale IDs, not filesystem paths. IDs such as `en` and `zh-CN` map to JSON files under the active project `locales/` directory; slashes, dots, URI-like prefixes, empty hyphen segments, and non-portable characters are rejected before loading or translating.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `load_locale` | `{ locale }` | `object` | Load locale strings |
| `list_locales` | - | `string[]` | Available locales |
| `translate` | `{ key, locale }` | `string` | Translate key |

## Marketplace

Marketplace `templatePath` and `outputPath` values are project template references, not arbitrary filesystem paths. `sakura_demo` resolves under the active project `templates/sakura_demo`; `templates/foo` is also accepted. Absolute paths, URI-like prefixes, empty path segments, `.`/`..` traversal, and non-portable reference segments are rejected before import/export.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `list_marketplace_entries` | - | `MarketplaceEntry[]` | Browse templates |
| `export_template` | `{ manifest, outputPath }` | `string` | Export template manifest under project `templates/` |
| `import_template` | `{ templatePath }` | `string` | Import project template or built-in catalog entry by safe reference |

## Project

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `get_project_config` | `{ projectPath }` | `ProjectConfig` | Get project settings |
| `save_project_config` | `{ projectPath, config }` | `ProjectConfig` | Save project settings with runtime secret fields and embedded token/query secrets scrubbed before writing `settings.json` |
| `export_project` | `{ projectPath? }` | `ProjectExportManifest` | Build the versioned sanitized inventory used by manifest downloads and project packages |
| `export_project_archive` | `{ projectPath?, destinationPath }` | `ProjectArchiveExportResult` | Atomically write a verified `.monogatari` ZIP package; an existing regular package is preserved if export fails |
| `inspect_project_archive` | `{ archivePath }` | `ProjectArchiveInspection` | Stream and verify a package without extracting it |
| `import_project_archive` | `{ archivePath, destinationParent }` | `ProjectArchiveImportResult` | Verify and stage a package, validate runtime catalogs, then commit it to a new non-overwriting project directory |

Archive paths must be local `.monogatari` files. Imports reject traversal, absolute/backslash paths, Windows-reserved names, case-insensitive collisions, symbolic/special ZIP entries, undeclared files, malformed JSON, checksum/fingerprint mismatches, runtime secrets, oversized manifests/files/packages, and nested destinations under the active project. The native dialog plugin grants only open/save dialog access; project bytes are read and written by the dedicated bounded commands.

## Live2D

Live2D `modelPath` values are project-relative model file references under the active project data root, not arbitrary filesystem paths. `.model3.json` and `.json` files are accepted; absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, and non-portable segments are rejected before loading sidecar expressions or motions.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `load_model` | `{ modelPath }` | `ModelInfo` | Load project-scoped Live2D model |
| `set_expression` | `{ expressionId }` | `void` | Set expression |
| `set_motion` | `{ motionGroup, index }` | `void` | Play motion |
| `get_model_info` | - | `ModelInfo` | Current model info |
