# Monogatari MCP Server

`monogatari-mcp` exposes the engine's transport-neutral authoring core through the standard Model Context Protocol over stdio. It is intended for local Agent clients that need inspectable, optimistic, rollback-capable project edits without access to arbitrary filesystem roots.

## Build

```powershell
cd rust-engine
cargo build --locked --release -p monogatari-mcp
```

The resulting binary is `rust-engine/target/release/monogatari-mcp.exe` on Windows. During development, the same server can be started through Cargo:

```powershell
cargo run --locked -p monogatari-mcp -- --project-root ..\data
```

The project root is required at startup and must contain `settings.json`. It is canonicalized once and is never accepted from tool input. Stdout is reserved for MCP frames; diagnostics go to stderr.

## Client Configuration

A generic local MCP client configuration looks like this:

```json
{
  "mcpServers": {
    "monogatari": {
      "command": "C:\\path\\to\\Monogatari\\rust-engine\\target\\release\\monogatari-mcp.exe",
      "args": ["--project-root", "C:\\path\\to\\visual-novel-project"]
    }
  }
}
```

This is read-only. Add `--allow-write` to `args` only for a client that should be able to apply reviewed transactions. Read-only processes share an operating-system project lease. A write-enabled process requires the exclusive lease, so no other reader or writer can observe its staged multi-file candidate.

## Tools

| Tool | Mode | Contract |
|---|---|---|
| `inspect_project` | Read | Returns scrubbed settings readiness and a complete JSON catalog report |
| `list_project_json` | Read | Lists exact byte SHA-256, semantic content fingerprint, size, kind, and portable path; accepts an optional catalog filter |
| `read_project_json` | Read | Reads one exact-case JSON path beneath an authorable catalog |
| `plan_transaction` | Read | Validates `monogatari-agent-project-transaction/v1` and returns a deterministic plan without writing |
| `apply_transaction` | Write | Requires `--allow-write` plus the exact reviewed `precondition_fingerprint`; stages, validates, commits, or rolls back |

The authorable JSON catalogs are `assets`, `characters`, `dialogue`, `endings`, `events`, `knowledge`, `locales`, `quality_suites`, `scenes`, and `workflows`. `settings.json`, saves, analytics, generated audio, binary assets, and arbitrary root files are outside the transaction protocol.

## Agent Flow

1. Call `inspect_project` and stop on project or catalog errors.
2. Call `list_project_json`, then `read_project_json` for every document that will be updated or deleted.
3. Use `missing` for creates and the returned exact `sha256` for updates or deletions.
4. Call `plan_transaction` and review every resolved path, operation, resulting hash, byte count, and the plan fingerprint.
5. Call `apply_transaction` with the unchanged transaction and reviewed `expected_precondition_fingerprint`.
6. Run graph, runtime, package, Quality Suite, and visual gates appropriate to the deliverable.

Planning and application both re-read current state. Any intervening file change invalidates the SHA precondition or plan fingerprint instead of overwriting newer work.

## Acceptance Boundary

Read-only inspection reports `acceptance_level: "document"`. Successful `apply_transaction` reports `acceptance_level: "core_runtime"`: in addition to document safety and settings readiness, the staged candidate must load through the real character, dialogue, and knowledge managers; load strict bounded scene, ending, Story Event, and Workflow catalogs; discover background-inferred scenes; reject duplicate IDs; validate Workflow graphs, conditions, state keys, Events, scenes, characters, and sub-workflows; and pass all prior runtime references. Rejection rolls back every staged operation. This level does not yet prove package, Quality Suite, or rendered desktop/mobile acceptance.

Use the full release gate before a release claim:

```powershell
node scripts/verify-release.mjs
```

Run the isolated MCP gate while iterating:

```powershell
node scripts/verify-modules.mjs --module rust-mcp
```
