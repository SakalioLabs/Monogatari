# Monogatari API Reference

All Tauri commands are invoked from the frontend via `invokeCommand(commandName, args)`.

## Engine

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `initialize_engine` | `{ projectPath: string }` | `void` | Initialize engine with data path |
| `get_engine_status` | - | `EngineStatus` | Get current engine state |

## Characters

Content loader `directory` values are project content references, not arbitrary filesystem paths. `characters` resolves to the active project `characters/` directory; nested references resolve under that same content root. Absolute paths, URI-like prefixes, empty path segments, and `.`/`..` traversal are rejected before load.

Character authoring IDs are portable slugs, not filenames or paths. `create_character.id` and `delete_character.characterId` may contain only ASCII letters, numbers, underscores, or hyphens; the backend writes or deletes only `<id>.json` directly under the active project `characters/` directory.

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

### Streaming Events
- `chat-chunk` - Token-by-token response
- `chat-complete` - Full response text
- `chat-emotion` - Detected emotion
- `chat-relationship` - Relationship delta
- `chat-evaluation` - Conversation scores
- `chat-event-decisions` - Explainable story-event trigger decisions
- `chat-events` - Triggered special events

## Dialogue

Dialogue loader `directory` values resolve under the active project `dialogue/` directory. `dialogue` reloads the canonical dialogue folder, and nested references remain inside that root.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `start_dialogue` | `{ dialogueId }` | `void` | Begin dialogue tree |
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

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `configure_api` | `{ baseUrl, apiKey, model }` | `void` | Set OpenAI-compatible API |
| `configure_onnx` | `{ modelPath, tokenizerPath }` | `void` | Set local ONNX model |
| `generate_response` | `{ prompt, options }` | `InferenceResult` | One-shot generation |
| `generate_stream` | `{ prompt, options }` | `void` | Streaming generation |
| `get_ai_status` | - | `AiStatus` | Current AI configuration |

## Workflow

Workflow command `path` values are project workflow references, not arbitrary filesystem paths. `workflow.json` resolves to the active project `workflows/workflow.json`; `workflows/foo.json` is also accepted. Absolute paths, URI-like prefixes, empty path segments, `.`/`..` traversal, and non-JSON files are rejected before save/load.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `get_workflow_nodes` | - | `NodeTypeInfo[]` | Available node types |
| `execute_workflow_node` | `{ nodeId, context }` | `NodeResult` | Execute single node |
| `validate_workflow` | `{ workflow }` | `ValidationResult` | Validate workflow graph |
| `save_workflow` | `{ workflow, path }` | `void` | Save workflow JSON under project `workflows/` |
| `load_workflow` | `{ path }` | `Workflow` | Load workflow JSON from project `workflows/` |

## Multi-Character Chat

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `start_group_chat` | `{ characterIds }` | `GroupSession` | Start group conversation |
| `send_group_message` | `{ session, message }` | `GroupSession` | Send to group |
| `get_group_chat_characters` | - | `[string, string][]` | Available characters |

## Save/Load

`saveId` values are opaque portable identifiers returned by `save_game` or `list_saves`; they are not file paths. Runtime save managers reject traversal-shaped IDs and filter mismatched save files before load/delete/list operations.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `save_game` | `{ saveName }` | `string` | Save game state |
| `load_game` | `{ saveId }` | `void` | Load game state by safe save ID |
| `list_saves` | - | `SaveInfo[]` | List all saves |
| `delete_save` | `{ saveId }` | `void` | Delete save by safe save ID |

## Scenes

Scene and renderer asset paths are project-relative asset references. Runtime asset managers reject absolute paths, URI-like prefixes, empty path segments, control characters, and `.`/`..` traversal before reading text, JSON, binary assets, or directory listings.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `list_scene_assets` | - | `SceneInfo[]` | List all scenes |
| `get_current_scene` | - | `ActiveScene` | Current active scene |
| `set_scene` | `{ sceneId }` | `void` | Set active scene |

## Scripting

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `execute_script` | `{ script }` | `ScriptResult` | Run Rhai script |
| `evaluate_condition` | `{ condition }` | `boolean` | Evaluate condition |
| `parse_script` | `{ script }` | `Ast` | Parse Rhai AST |

## TTS

Generated system, Azure, and ElevenLabs speech files are written under the active project `assets/tts/` directory with sanitized character/provider filename components.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `configure_tts` | `{ config }` | `void` | Set TTS provider config |
| `set_character_voice` | `{ characterId, voiceId }` | `void` | Assign voice to character |
| `synthesize_speech` | `{ text, voiceId }` | `string` | Generate audio file |
| `get_available_voices` | - | `VoiceInfo[]` | List available voices |

## Plugin System

Plugin manifest IDs are portable slugs, not filenames or paths. `register_plugin.manifest.id` and `remove_plugin.pluginId` may contain only ASCII letters, numbers, underscores, or hyphens; the backend writes or deletes only `<id>.json` directly under the active project `plugins/` directory.

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
| `save_project_config` | `{ projectPath, config }` | `ProjectConfig` | Save project settings |

## Live2D

Live2D `modelPath` values are project-relative model file references under the active project data root, not arbitrary filesystem paths. `.model3.json` and `.json` files are accepted; absolute paths, drive/URI-style prefixes, empty segments, `.`/`..` traversal, and non-portable segments are rejected before loading sidecar expressions or motions.

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `load_model` | `{ modelPath }` | `ModelInfo` | Load project-scoped Live2D model |
| `set_expression` | `{ expressionId }` | `void` | Set expression |
| `set_motion` | `{ motionGroup, index }` | `void` | Play motion |
| `get_model_info` | - | `ModelInfo` | Current model info |
