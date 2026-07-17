# Monogatari Project Contract

## Authoritative Sources

- `docs/DATA_FORMAT.md`: public content and package formats.
- `data/`: canonical checked-in examples used by the workbench and Web/PWA build.
- `rust-engine/data/`: packaged desktop mirror; keep it byte-equivalent to `data/`.
- `rust-engine/crates/tauri-app/src/commands/`: runtime authoring, validation, and reference rules.
- `rust-engine/crates/authoring/`: transport-neutral project, path, and Agent transaction contracts.
- `rust-engine/crates/mcp-server/`: standard stdio transport over the authoring contracts.
- `docs/MCP_SERVER.md`: MCP startup, tool, write-mode, and acceptance boundaries.
- `agent-transaction.md`: versioned optimistic multi-file JSON operation format.
- `data/quality_suites/character_stability.json`: executable story and safety acceptance examples.
- `scripts/module-test-matrix.json`: independently runnable engineering gates.
- `scripts/verify-release.mjs`: final integrated release gate.

## Content Graph

| Area | Directory | Stable references |
|---|---|---|
| Project configuration | `settings.json` | Configured content paths and render metadata |
| Character identity | `characters/` | Character IDs, knowledge refs, renderer assets |
| World knowledge | `knowledge/` | Entry IDs, tags, related entry IDs |
| Story space | `scenes/` and `assets/` | Scene IDs and project-relative asset paths |
| Branching prose | `dialogue/` | Dialogue IDs, node IDs, speaker IDs, next-node IDs |
| Progression | `events/` | Event IDs, character scopes, unlock targets, flags |
| Conclusions | `endings/` | Ending IDs plus scene/dialogue targets |
| Orchestration | `workflows/` | Node IDs and references to all upstream content |
| Acceptance | `quality_suites/` | Character, event, workflow, score, and safety evidence |
| Localization | `locales/` | Direct locale JSON files with portable locale IDs |

## Non-Negotiable Invariants

- Use portable relative paths. Reject absolute paths, URIs, traversal, control characters, and case-insensitive path collisions.
- Keep IDs stable after downstream references exist. Prefer lowercase ASCII IDs with underscores or hyphens unless the active runtime contract is stricter.
- Make every dialogue and workflow node reachable from its start node, with intentional terminal paths.
- Do not reference a character, scene, dialogue, ending, event, knowledge entry, or asset that does not exist.
- Write Knowledge with canonical `related_entries`, lowercase portable IDs/categories, trimmed unique tags and relations, bounded metadata, and importance in `0..=1`; relations cannot target the source entry itself.
- Keep generated story text separate from system prompts, tool calls, hidden reasoning, and runtime control data.
- Prove important branches with Quality Suite scenarios, including negative cases and boundary thresholds.
- Do not ship secrets in `settings.json`, project packages, logs, fixtures, or verification reports.
- For Agent transactions, require `missing` on creates and an exact current SHA-256 on updates/deletes; never use a blind overwrite operation.
- Treat MCP stdio as UTF-8 end to end. Avoid Windows PowerShell 5 native-process pipelines for non-ASCII frames, and read the applied documents back through MCP before accepting authored text.

For the checked-in built-in project, synchronize the canonical root into the desktop mirror and then prove exact parity:

```powershell
node scripts/sync-project-mirror.mjs --write
node scripts/sync-project-mirror.mjs --check
```

## Acceptance Levels

1. **Document-valid**: every JSON document parses and follows its local schema.
2. **Graph-valid**: IDs, paths, and cross-content references resolve; dialogue/workflow graphs are reachable.
3. **Runtime-valid**: Rust managers load the project and Quality Suites pass.
4. **Package-valid**: Web/PWA and desktop inventories contain the intended assets with stable fingerprints.
5. **Experience-valid**: representative desktop/mobile story flows render and advance without console errors.

Do not describe a project as complete before all applicable levels have evidence.
