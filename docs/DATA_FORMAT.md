# Monogatari Data Format Reference

## Project Package Format

Complete desktop project handoffs use the `.monogatari` extension. The file is a ZIP archive with one root manifest, `monogatari-project.json`, followed by the project files declared by that manifest. `settings.json` is regenerated with runtime credentials removed rather than copied from disk.

```json
{
  "format": "monogatari-project",
  "schema": "monogatari-project-export@1",
  "version": "1.0",
  "settings": { "render": { "title": "My Story" } },
  "package": {
    "file_count": 3,
    "total_bytes": 2048,
    "fingerprint_algorithm": "sha256:path-size-file-sha256-v1",
    "content_sha256": "<64 lowercase hex characters>",
    "directories": ["assets", "characters", "dialogue"],
    "files": [
      {
        "category": "settings",
        "path": "settings.json",
        "size_bytes": 512,
        "checksum_md5": "<32 hex characters>",
        "checksum_sha256": "<64 lowercase hex characters>"
      }
    ]
  },
  "archive": {
    "format": "zip",
    "manifest_path": "monogatari-project.json",
    "extension": ".monogatari"
  }
}
```

File records are sorted by path. The package fingerprint hashes each record as `path`, NUL, decimal size, NUL, lowercase SHA-256, newline. Paths use forward-slash relative segments and cannot contain traversal, empty/current segments, backslashes, control characters, platform-reserved characters/names, trailing dots/spaces, case-insensitive duplicates, or the manifest path itself. Imports require exactly the declared files and directories, validate every JSON document and checksum while streaming, and reject settings containing runtime secrets. Saves, analytics, and cloud-sync manifests are intentionally excluded.

## Character Format

Characters are stored as JSON files in `rust-engine/data/characters/`.

```json
{
  "id": "character_id",
  "name": "Display Name",
  "description": "Brief description for UI",
  "background": "Detailed backstory for AI context",
  "personality": {
    "openness": 0.8,
    "conscientiousness": 0.6,
    "extraversion": 0.7,
    "agreeableness": 0.9,
    "neuroticism": 0.2,
    "speech_style": "Description of how the character speaks"
  },
  "portrait_path": "assets/characters/hero_portrait.svg",
  "sprite_path": "assets/characters/hero_sprite.svg",
  "sprite_paths": {
    "happy": "assets/characters/hero_happy.svg",
    "neutral": "assets/characters/hero_sprite.svg"
  },
  "live2d_model_path": null
}
```

Renderer paths must be project-relative, portable, stay inside the active data root, and use supported extensions. Absolute paths, URI-like prefixes, empty segments, `.`/`..` traversal, and non-portable path segments are rejected by renderer diagnostics. Story Mode and Character Editor resolve Live2D first, then GLB/GLTF 3D models, emotion-specific sprites, fallback sprites, portraits, and finally the generated placeholder. Live2D backend commands load only `.model3.json`/`.json` files from the active project data root.

### Personality Traits (0.0 - 1.0)
- **openness**: Curiosity, creativity, willingness to try new things
- **conscientiousness**: Organization, dependability, self-discipline
- **extraversion**: Sociability, assertiveness, energy level
- **agreeableness**: Cooperation, trust, empathy
- **neuroticism**: Emotional instability, anxiety (lower is calmer)

## Dialogue Format

Dialogues are stored in `rust-engine/data/dialogue/`.

```json
{
  "id": "dialogue_id",
  "title": "Dialogue Title",
  "description": "Optional author-facing synopsis",
  "start_node_id": "start",
  "nodes": {
    "start": {
      "speaker_id": "character_id",
      "text": "Dialogue text",
      "choices": [
        {
          "text": "Choice text",
          "next_node_id": "response_node",
          "relationship_changes": { "character_id": 0.2 }
        }
      ]
    },
    "response_node": {
      "speaker_id": "character_id",
      "text": "Response text",
      "choices": [],
      "is_ending": true,
      "ending_type": "good"
    }
  }
}
```

### Choice Properties
- `text`: Player-facing choice text
- `next_node_id`: Node to transition to
- `relationship_changes`: Map of character_id to delta (-1.0 to 1.0)
- `condition`: Optional bounded read-only condition expression

Dialogue node map keys are authoritative node IDs. An optional nested `id` is accepted only when it matches the map key and is omitted from canonical author saves. Every linear and choice target must exist, all nodes must be reachable from `start_node_id`, a node cannot combine `next_node_id` with choices, and an explicit ending cannot have outgoing transitions. Speakers and relationship targets must reference project characters; relationship deltas are finite values in `-1..1`. Node and choice conditions, entry scripts, text, variables, and LLM prompts are bounded. `use_llm: true` requires `llm_prompt`. Unknown fields are rejected.

The Dialogue Editor consumes `monogatari-dialogue-authoring-catalog/v1` snapshots. Desktop saves compare the observed whole-catalog fingerprint, stage a bounded JSON replacement, reload and validate every script, roll back failures, then hot-reload the runtime catalog while retaining dialogue-local flags and variables. Deletion is rejected while a Story Event unlocks the dialogue or an ending references it. Browser builds persist a complete local dialogue draft catalog that Story Mode can play directly.

## Knowledge Format

Knowledge entries are stored in `rust-engine/data/knowledge/`.

```json
{
  "id": "knowledge_id",
  "title": "Entry Title",
  "content": "Detailed knowledge content for AI context",
  "category": "location|character|lore|event",
  "tags": ["tag1", "tag2"]
}
```

## Scene Format

Scenes are stored in `rust-engine/data/scenes/`.

```json
{
  "id": "scene_id",
  "name": "Scene Name",
  "background_path": "assets/backgrounds/scene.svg",
  "bgm_path": null,
  "weather": "spring|summer|autumn|winter|clear|rain|snow|enchanted",
  "time_of_day": "day|night|dawn|dusk|golden_hour|eternal_twilight",
  "tags": ["outdoor", "calm", "demo"]
}
```

Scene IDs and asset paths are portable. Backgrounds must use a supported image extension and resolve to an existing project file; BGM references use supported audio extensions. The `monogatari-scene-authoring-catalog/v1` snapshot includes both JSON-authored scenes and virtual scenes inferred from unclaimed background files. Saving an inferred entry promotes it into `scenes/<id>.json`. Deleting removes only that metadata document and never deletes the background asset; deletion is blocked by matching Story Event, ending, or workflow scene references. Desktop writes use an expected catalog fingerprint and rollback-capable replacement, while browser builds keep a complete local draft read by Story Mode.

## Story Event Catalog Format

Story events are stored in one or more JSON files under the configured project `events/` directory. IDs are unique across all catalog files. Every configured threshold is combined with AND semantics.

```json
{
  "schema": "monogatari-story-event-catalog/v1",
  "events": [
    {
      "event_id": "luna_secret",
      "event_type": "special_dialogue",
      "description": "Luna shares a secret.",
      "actions": [
        { "type": "unlock_dialogue", "dialogue_id": "luna_secret_dialogue" },
        { "type": "set_flag", "flag": "luna.secret", "value": true }
      ],
      "data": { "chapter": "luna_route" },
      "character_ids": ["luna"],
      "repeatable": false,
      "rule": {
        "min_relationship": 0.4,
        "score_metric": "overall",
        "min_score": 0.75,
        "min_evaluation_count": 2
      }
    }
  ]
}
```

- `score_metric` accepts `friendliness`, `engagement`, `creativity`, or `overall` and must be paired with `min_score` in the `0..1` range.
- `min_relationship` accepts `-1..1`; `min_evaluation_count` is a non-negative integer.
- Empty or omitted `character_ids` applies the event to every character. Scoped IDs must exist when the project activates.
- `repeatable` defaults to `false`. Non-repeatable events are recorded per character scope in persistent story progress and blocked after their first application.
- `actions` accepts at most 64 typed effects: `unlock_scene` with `scene_id`, `unlock_dialogue` with `dialogue_id`, `unlock_ending` with `ending_id`, or `set_flag` with a portable `flag` and boolean `value`.
- `data` remains bounded author-defined metadata. Legacy string fields `data.unlock_scene`, `data.dialogue_id`, and `data.unlock_ending` are migrated into equivalent typed actions at load time.
- Default unscoped rules preserve the `monogatari-event-trigger-rule/v1` fingerprint contract. Character-scoped or repeatable rules use v2 fingerprints, while the catalog fingerprint binds descriptions, metadata, actions, and rule fingerprints.
- An `unlock_scene`, `unlock_dialogue`, or `unlock_ending` target is gated until it appears in persistent story progress. Project content not referenced by any unlock action remains open, preserving legacy projects.
- The visual editor writes a single JSON catalog document with an expected catalog fingerprint. Multi-document catalogs remain runtime-supported but must be consolidated before visual save to avoid silently flattening author-owned files.

## Ending Format

Endings are individual JSON files under project `endings/`.

```json
{
  "schema": "monogatari-story-ending/v1",
  "id": "best_friend_ending",
  "title": "Under the Festival Stars",
  "description": "A quiet promise closes the night.",
  "scene_id": "festival_night",
  "dialogue_id": "observatory_night"
}
```

IDs must be portable and the referenced scene and dialogue must exist. Ending IDs become immutable after creation so source ownership remains stable; duplicate an ending to create a new ID. Titles are limited to 256 characters, descriptions to 2,048 characters, files to 64 KiB, and catalogs to 256 JSON files. Unknown fields and symlinked files/directories are rejected.

The Ending Route editor loads `monogatari-story-ending-catalog/v1` snapshots containing source paths plus per-content and whole-catalog fingerprints. Desktop saves require the observed catalog fingerprint, validate every resulting scene/dialogue reference, and roll back failed replacements. Deletion is rejected while a Story Event contains a matching `unlock_ending` action. Browser builds keep an equivalent complete catalog draft in local storage for non-destructive preview.

Release verification cross-checks all event unlock targets, ending references, strict dialogue graphs, character references, and matching dialogue catalogs in both checked-in data roots.

## Workflow Format

Workflows are stored as JSON files and loaded via the Workflow Editor.

```json
{
  "id": "workflow_id",
  "name": "Workflow Name",
  "start_node_id": "node_1",
  "nodes": [
    {
      "id": "node_1",
      "node_type": "start",
      "label": "Begin",
      "x": 100,
      "y": 200,
      "config": {},
      "connections": ["node_2"]
    }
  ]
}
```

### Node Types
See the Workflow Editor documentation for all 21 available node types and their configurable fields.
