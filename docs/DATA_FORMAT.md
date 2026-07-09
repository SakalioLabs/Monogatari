# Monogatari Data Format Reference

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
      "choices": []
    }
  }
}
```

### Choice Properties
- `text`: Player-facing choice text
- `next_node_id`: Node to transition to
- `relationship_changes`: Map of character_id to delta (-1.0 to 1.0)

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
