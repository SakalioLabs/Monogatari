# Agent Project Transaction v1

Use `monogatari-agent-project-transaction/v1` to plan or apply a bounded set of JSON changes through `llm-authoring`. The project root is transport configuration, never request data.

```json
{
  "schema": "monogatari-agent-project-transaction/v1",
  "transaction_id": "chapter_2_cast_and_intro",
  "operations": [
    {
      "op": "put_json",
      "path": "characters/aoi.json",
      "document": {
        "id": "aoi",
        "name": "Aoi"
      },
      "precondition": {
        "kind": "missing"
      }
    },
    {
      "op": "delete_json",
      "path": "dialogue/obsolete_intro.json",
      "precondition": {
        "kind": "sha256",
        "value": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      }
    }
  ]
}
```

## Preconditions

- `missing` is create-only and fails when any target already exists.
- `sha256` is required for updates and deletions and must match the exact current file bytes.
- Every operation requires a precondition. There is no blind overwrite or force flag.
- Plan and apply both re-read current state. A stale plan therefore fails instead of overwriting newer work.

## Path Boundary

v1 accepts lowercase `.json` files beneath `assets`, `characters`, `dialogue`, `endings`, `events`, `knowledge`, `locales`, `quality_suites`, `scenes`, or `workflows`. Paths use bounded portable ASCII segments. Absolute paths, URI/drive prefixes, traversal, hidden segments, missing parent directories, symlinks, duplicate targets, and ASCII case collisions are rejected.

`settings.json`, saves, analytics, generated audio, arbitrary root files, and binary assets are outside v1. For this repository's local Skill workflow, use the reviewed `scripts/import-project-asset.mjs` plan/apply operation for supported files beneath `assets/`; do not encode binary payloads as JSON transactions.

## Apply Semantics

1. The planner validates the schema, transaction ID, operation count, paths, payload bounds, existing file types, case collisions, and optimistic preconditions without writing.
2. Apply stages every replacement or deletion while preserving prior files.
3. The transport runs its authoritative candidate-project validator against the complete staged state.
4. A validator or staging failure rolls back staged operations in reverse order.
5. Successful validation commits the candidate state. Backup cleanup failures are reported as warnings because the requested content is already applied.

The plan schema is `monogatari-agent-project-transaction-plan/v1`; success uses `monogatari-agent-project-transaction-result/v1`; structured failures use `monogatari-agent-project-transaction-error/v1` with stable error codes and optional operation/path context.

Transaction success proves only that the candidate passed the validator supplied by its transport. A complete visual novel still requires the graph, runtime, package, and experience acceptance levels in `project-contract.md`.

## Standard MCP Transport

`monogatari-mcp` exposes this protocol through `plan_transaction` and `apply_transaction`. The server fixes the project root at startup; no tool accepts a replacement root. It starts read-only, and `apply_transaction` remains unavailable unless the process was launched with `--allow-write`.

Use `list_project_json` or `read_project_json` to obtain the exact file-byte `sha256`. The separate `content_fingerprint` normalizes parsed JSON and is useful for semantic comparison, but it is not a transaction precondition. After planning, pass the unchanged transaction and the returned `precondition_fingerprint` as `expected_precondition_fingerprint`; a changed plan is rejected and must be reviewed again.

MCP read-only inspection reports `acceptance_level: "document"`. After staging, successful candidate application reports `acceptance_level: "core_runtime"` only if settings/documents pass, the real character/dialogue managers and shared normalized Knowledge builder load, duplicate IDs are absent, and core character/knowledge/dialogue references resolve; otherwise every operation rolls back. Knowledge candidates additionally require known fields, canonical values, bounded content/metadata/catalogs, and complete non-self `related_entries`. Scene, Story Event, ending, workflow, Quality Suite, package, and visual validation remain separate gates.
