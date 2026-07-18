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
  "model_3d_path": "assets/models/hero.glb",
  "sprite_path": "assets/characters/hero_sprite.svg",
  "sprite_paths": {
    "happy": "assets/characters/hero_happy.svg",
    "neutral": "assets/characters/hero_sprite.svg"
  },
  "live2d_model_path": null
}
```

Renderer paths must be project-relative, portable, stay inside the active data root, and use supported extensions. Absolute paths, URI-like prefixes, empty segments, `.`/`..` traversal, and non-portable path segments are rejected by renderer diagnostics. Playtest and Character Editor resolve Live2D first, then GLB/GLTF 3D models, emotion-specific sprites, fallback sprites, portraits, and finally the generated placeholder. Loaded 3D models are normalized to a bounded stage size and reframed when the preview aspect changes. Live2D backend commands load only `.model3.json`/`.json` files from the active project data root.

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

The Dialogue Editor consumes `monogatari-dialogue-authoring-catalog/v1` snapshots. Desktop saves compare the observed whole-catalog fingerprint, stage a bounded JSON replacement, reload and validate every script, roll back failures, then hot-reload the runtime catalog while retaining dialogue-local flags and variables. Deletion is rejected while a Story Event unlocks the dialogue or an ending references it. Browser builds persist a complete local dialogue draft catalog that Playtest can play directly.

## Knowledge Format

Knowledge entries are stored as one object or a non-empty object array under the project `knowledge/` directory. The checked-in project is mirrored at `data/knowledge/` and `rust-engine/data/knowledge/`.

```json
{
  "id": "knowledge_id",
  "title": "Entry Title",
  "content": "Detailed knowledge content for AI context",
  "category": "world_lore",
  "tags": ["tag1", "tag2"],
  "importance": 0.8,
  "metadata": { "era": "spring" },
  "related_entries": ["another_knowledge_id"]
}
```

IDs and category labels use 1 to 128 and 1 to 64 lowercase portable ASCII characters respectively. Titles, content, tags, metadata, relation counts, entries, files, and aggregate catalog bytes are bounded; importance is finite and in `0..=1`. Author output trims fields, deduplicates tags case-insensitively and relations exactly, and must reference existing non-self entries. `relatedEntries` is accepted only as a legacy read alias; canonical writes use `related_entries`. Unknown fields, empty arrays, duplicates, invalid UTF-8/JSON, symlinks, and non-canonical Agent candidates are rejected before runtime construction. Custom normalized categories such as `world_lore` are preserved rather than collapsed into a built-in category.

## Scene Format

Scenes are stored in `rust-engine/data/scenes/`.

```json
{
  "id": "scene_id",
  "name": "Scene Name",
  "background_path": "assets/backgrounds/scene.svg",
  "model_3d_path": "assets/models/scene.glb",
  "bgm_path": null,
  "weather": "spring|summer|autumn|winter|clear|rain|snow|enchanted",
  "time_of_day": "day|night|dawn|dusk|golden_hour|eternal_twilight",
  "tags": ["outdoor", "calm", "demo"]
}
```

Scene IDs and asset paths are portable. Backgrounds must use a supported image extension, 3D scene models use `.glb` or `.gltf`, and both resolve to existing project files; BGM references use supported audio extensions. Playtest renders `model_3d_path` as the full scene presentation and retains `background_path` as a loading fallback. The `monogatari-scene-authoring-catalog/v1` snapshot includes both JSON-authored scenes and virtual scenes inferred from unclaimed background files. Saving an inferred entry promotes it into `scenes/<id>.json`. Deleting removes only that metadata document and never deletes the background asset; deletion is blocked by matching Story Event, ending, workflow, or dialogue-node scene references. Desktop writes use an expected catalog fingerprint and rollback-capable replacement, while browser builds keep a complete local draft read by Playtest.

Dialogue nodes may add `"scene_id": "scene_id"`. The runtime activates that scene when the node is entered, enabling one validated dialogue graph to drive visual scene changes. Core-runtime validation rejects unknown scene IDs.

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

## Scene Roleplay Format

AI-first stories are stored as versioned JSON files under `roleplays/`. A roleplay is the primary runtime story graph for free-form interaction; it does not contain fixed NPC lines. Each node binds a visual scene and character context to goals, score/evidence rules, deterministic transitions, and bounded inference:

```json
{
  "schema": "monogatari-scene-roleplay/v1",
  "id": "signal_inquiry",
  "title": "Signal Inquiry",
  "start_node_id": "first_contact",
  "exhaustion_ending_id": "signal_unresolved",
  "max_total_turns": 12,
  "score_dimensions": [
    {
      "id": "evidence_integrity",
      "label": "Evidence integrity",
      "description": "Separates facts from inference.",
      "min": -6,
      "max": 6,
      "initial": 0
    }
  ],
  "nodes": [
    {
      "id": "first_contact",
      "scene_id": "signal_room",
      "character_id": "echo",
      "supporting_character_ids": ["observer"],
      "opening_narration": "The recovered signal opens a live channel.",
      "situation": "The signal is real, but its claimed identity is unresolved.",
      "player_goal": "Establish a repeatable verification method through free conversation.",
      "character_goal": "Offer bounded clues without claiming an unverified identity.",
      "knowledge_refs": ["signal_protocol"],
      "min_turns": 2,
      "max_turns": 5,
      "score_rules": [
        {
          "dimension_id": "evidence_integrity",
          "guidance": "Reward repeatable checks; penalize unsupported certainty.",
          "max_delta_per_turn": 1
        }
      ],
      "evidence_rules": [
        {
          "id": "verification_plan",
          "description": "The player states a repeatable external check."
        }
      ],
      "transitions": [
        {
          "id": "verified_route",
          "priority": 20,
          "target": { "kind": "ending", "ending_id": "signal_verified" },
          "conditions": [
            { "kind": "node_turns_at_least", "value": 2 },
            { "kind": "score_at_least", "dimension_id": "evidence_integrity", "value": 2 },
            { "kind": "evidence_observed", "evidence_id": "verification_plan" }
          ]
        }
      ],
      "timeout_target": { "kind": "ending", "ending_id": "signal_unresolved" }
    }
  ],
  "inference": {
    "max_context_characters": 6000,
    "max_recent_turns": 8,
    "npc_max_tokens": 96,
    "evaluator_max_tokens": 160
  }
}
```

Targets use `kind: "node"` with `node_id` or `kind: "ending"` with `ending_id`. Conditions are `score_at_least`, `score_at_most`, `evidence_observed`, `node_turns_at_least`, or `total_turns_at_least`. Eligible transitions are ordered by descending `priority`, with authored order breaking ties. If no transition matches by `max_turns`, `timeout_target` applies; `max_total_turns` applies the roleplay's exhaustion ending.

The NPC generator receives the current situation, goals, character, pinned Knowledge, and bounded transcript, but returns only visible dialogue. A separate evaluator returns strict `score_deltas`, `evidence`, optional `npc_emotion`, and a summary. Every evidence observation must carry a non-empty `player_quote` that is an exact substring of the current player message. The runtime rejects unknown dimensions/evidence or fabricated quotes, clamps per-turn deltas and score ranges, deduplicates evidence, records the exchange, and then evaluates authored transitions atomically. Malformed evaluator output uses an explicit zero-score/no-evidence fallback and cannot choose a route.

Core-runtime validation requires unique portable IDs, reachable nodes, valid local targets, valid bounds, and resolved scene, character, Knowledge, and ending references. Catalogs are limited to 256 regular JSON files and each file to 512 KiB; symlinks and unknown schema fields are rejected. The `data/roleplays/blue_frame_roleplay.json` fixture is the canonical checked-in example.

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

## Quality Suite Format

Quality Suites are stored under `quality_suites/` and execute deterministic character, safety, Knowledge, Story Event, Scene Roleplay, and Workflow expectations without requiring a model provider. Each file contains a version, name, description, and bounded scenario array. A Workflow scenario references one project Workflow and may provide typed run contexts, explicit choice maps, or both:

```json
{
  "version": "0.1.0",
  "name": "Route Acceptance",
  "description": "Provider-free branch evidence.",
  "scenarios": [
    {
      "id": "all-routes",
      "category": "workflow_coverage",
      "description": "Three deterministic runs cover every terminal.",
      "messages": [{ "role": "player", "content": "Preview all routes." }],
      "workflow_path": "workflows/story_route.json",
      "workflow_max_steps": 64,
      "workflow_choice_selections": [
        { "opening_choice": 0, "ending_choice": 0 },
        { "opening_choice": 1, "ending_choice": 1 },
        { "opening_choice": 2, "ending_choice": 2 }
      ],
      "expect": {
        "min_workflow_coverage_percent": 100,
        "expected_workflow_unvisited_nodes": []
      }
    }
  ]
}
```

`workflow_choice_selections` is an array of run maps from portable Workflow choice-node IDs to zero-based option indices. A scenario can contain at most 64 run maps and each map at most 128 selections; indices are bounded to 128. When `workflow_run_contexts` is also present, both arrays must have the same run count and are paired by index. Complete Quality execution runs the greater of the context count, choice-map count, or one default run, records each applied selection map, and aggregates union coverage and unvisited nodes across those runs. Workflow expectations or selections require `workflow_path`. Files are limited to 2 MiB and catalogs to 256 regular JSON files.

A Scene Roleplay scenario supplies a project path plus provider-free turn records. Player messages, NPC responses, and strict evaluations are fixture input; session state, score clamping, evidence acceptance, transitions, and endings are always computed by the shared game core:

```json
{
  "id": "verified-ending",
  "category": "scene_roleplay",
  "description": "A verifiable exchange reaches the intended ending.",
  "messages": [{ "role": "player", "content": "Replay the verified route." }],
  "roleplay": {
    "path": "roleplays/signal_inquiry.json",
    "turns": [
      {
        "player_message": "Ask a second station to repeat the measurement.",
        "npc_response": "Record their method beside mine.",
        "evaluation": {
          "score_deltas": [
            { "dimension_id": "evidence_integrity", "delta": 1, "reason": "Repeatable check" }
          ],
          "evidence": [
            { "evidence_id": "verification_plan", "player_quote": "repeat the measurement" }
          ],
          "npc_emotion": "cautious",
          "summary": "An external verification step was established."
        }
      }
    ]
  },
  "expect": {
    "expected_roleplay_ending": "signal_verified",
    "min_roleplay_coverage_percent": 100,
    "required_roleplay_nodes": ["first_contact"],
    "required_roleplay_evidence": ["verification_plan"],
    "min_roleplay_scores": { "evidence_integrity": 1 }
  }
}
```

Roleplay expectations can assert an ending, minimum coverage, exact unvisited nodes, required/forbidden nodes, required evidence, and minimum/maximum final scores. Reports include exact roleplay source path/SHA-256, per-turn outcomes, final session, visited/unvisited nodes, ending, and audit summaries. This proves state-machine behavior and authored route coverage, not the quality or availability of live model generation.
