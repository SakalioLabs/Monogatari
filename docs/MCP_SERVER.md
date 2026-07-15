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

The project root is required at startup and must contain `settings.json`. It is canonicalized once and is never accepted from tool input. Optional archive output is fixed separately with `--package-output-dir <path>`; that directory must already exist, be a regular directory, and stay outside the authored project root. Tools accept only one portable package file name, never an output path. Stdout is reserved for MCP frames; diagnostics go to stderr. Cross-process reader/writer leases live in a SHA-256-named system temporary directory, so starting a read-only server does not create files in the authored project.

MCP stdio frames are UTF-8. Windows PowerShell 5 can encode text sent to a native process with the active system code page even when `Get-Content -Encoding UTF8` decoded the source correctly. Do not pipe non-ASCII request JSON directly from Windows PowerShell 5; use an MCP client that writes UTF-8 bytes, and call `read_project_json` after applying a transaction to verify authored non-ASCII content.

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

This is read-only. Add `--allow-write` to `args` only for a client that should be able to apply reviewed transactions. To permit archive output, also add `--package-output-dir` and one user-reviewed external directory, for example `C:\\path\\to\\packages`. The output flag alone does not enable writes. Read-only processes share an operating-system project lease. A write-enabled process requires the exclusive lease, so no other reader or writer can observe its staged multi-file candidate or race an MCP package snapshot.

## Tools

| Tool | Mode | Contract |
|---|---|---|
| `inspect_project` | Read | Returns scrubbed settings readiness and a complete JSON catalog report |
| `validate_project` | Read | Runs every shared headless runtime/catalog/reference gate and returns structured evidence without writing |
| `validate_delivery` | Read | Extends core validation with declared renderer/audio asset existence, extension, and placeholder evidence |
| `list_project_json` | Read | Lists exact byte SHA-256, semantic content fingerprint, size, kind, and portable path; accepts an optional catalog filter |
| `read_project_json` | Read | Reads one exact-case JSON path beneath an authorable catalog |
| `run_quality_suite` | Read | Executes one exact `quality_suites/...json` path through the shared headless domain and returns versioned scenario/audit evidence bound to its byte SHA-256 |
| `preview_project_package` | Read | Builds the complete credential-free package manifest and deterministic content fingerprint without writing; reports whether an output directory is configured |
| `plan_transaction` | Read | Validates `monogatari-agent-project-transaction/v1` and returns a deterministic plan without writing |
| `apply_transaction` | Write | Requires `--allow-write` plus the exact reviewed `precondition_fingerprint`; stages, validates, commits, or rolls back |
| `export_project_package` | Write | Requires `--allow-write`, the startup-fixed external output directory, one portable `.monogatari` file name, and the exact current preview fingerprint; defaults to refusing existing files |

The authorable JSON catalogs are `assets`, `characters`, `dialogue`, `endings`, `events`, `knowledge`, `locales`, `quality_suites`, `scenes`, and `workflows`. `settings.json`, saves, analytics, generated audio, binary assets, and arbitrary root files are outside the transaction protocol.

## Agent Flow

1. Call `inspect_project` and stop on project or catalog errors.
2. Call `validate_project` to obtain current structured runtime/catalog/reference evidence.
3. Call `list_project_json`, then `read_project_json` for every document that will be updated or deleted.
4. Use `missing` for creates and the returned exact `sha256` for updates or deletions.
5. Call `plan_transaction` and review every resolved path, operation, resulting hash, byte count, and the plan fingerprint.
6. Call `apply_transaction` with the unchanged transaction and reviewed `expected_precondition_fingerprint`.
7. Call `validate_project` and `validate_delivery` again.
8. List the `quality_suites` catalog and call `run_quality_suite` for every intended suite path. Accept the run only when `passed` is `true`; `passed: false` is a successful protocol response whose report contains actionable failed-scenario evidence.
9. Call `preview_project_package` and review the full manifest, file inventory, scrubbed settings, and `content_sha256`.
10. If an archive is required and package output is configured, call `export_project_package` with that exact fingerprint and one file name. Keep `replace_existing` false unless replacing the existing artifact is intentional; any intervening project change invalidates the fingerprint.
11. Re-import or inspect the archive and run rendered visual gates appropriate to the deliverable. When editing the repository's built-in `data/` project, also run `node scripts/sync-project-mirror.mjs --write` followed by `node scripts/sync-project-mirror.mjs --check` so `rust-engine/data/` remains byte-equivalent.

Planning and application both re-read current state. Any intervening file change invalidates the SHA precondition or plan fingerprint instead of overwriting newer work.

## Acceptance Boundary

Read-only inspection reports `acceptance_level: "document"`. Successful `apply_transaction` reports `acceptance_level: "core_runtime"`: in addition to document safety and settings readiness, the staged candidate must load through the real character, dialogue, and knowledge managers; load strict bounded scene, ending, Story Event, Workflow, and Quality Suite catalogs; validate Workflow graphs and Quality expectation/reference contracts; and pass all prior runtime references. Rejection rolls back every staged operation. `run_quality_suite` is a separate read-only gate: it accepts only a bounded path in the fixed project's `quality_suites` catalog and returns `monogatari-mcp-quality-suite-run/v1` with the shared complete report, exact source SHA-256, and MCP build/time provenance.

`preview_project_package` reports `monogatari-mcp-package-preview/v1`; it proves bounded inventory, portable paths, credential scrubbing, manifest self-validation, and the current package content fingerprint without writing. `export_project_package` reports `monogatari-mcp-package-export/v1`; it rebuilds and confirms that fingerprint, then streams and revalidates every file through the shared staged ZIP writer into the startup-fixed directory. This is package-generation evidence, not archive re-import, installed-runtime, desktop/mobile rendering, or visual-quality evidence.

Use the full release gate before a release claim:

```powershell
node scripts/verify-release.mjs
```

Run the isolated MCP gate while iterating:

```powershell
node scripts/verify-modules.mjs --module rust-mcp
```
