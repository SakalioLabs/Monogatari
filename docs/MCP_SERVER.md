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

The project root is required at startup and must contain `settings.json`. It is canonicalized once and is never accepted from tool input. An optional archive exchange directory is fixed separately with `--package-output-dir <path>`; that directory must already exist, be a regular directory, and stay outside the authored project root. Inspection, validation, and export tools accept only one portable package file name inside that directory, never an input, extraction, or output path. Stdout is reserved for MCP frames; diagnostics go to stderr. Cross-process reader/writer leases live in a SHA-256-named system temporary directory, so starting a read-only server does not create files in the authored project.

MCP stdio frames are UTF-8. Windows PowerShell 5 can encode text sent to a native process with the active system code page even when `Get-Content -Encoding UTF8` decoded the source correctly. Do not pipe non-ASCII request JSON directly from Windows PowerShell 5; use an MCP client that writes UTF-8 bytes, and call `read_project_json` after applying a transaction to verify authored non-ASCII content.

This repository includes a bounded UTF-8 client for one-shot tool calls:

```powershell
node scripts/call-monogatari-mcp.mjs --project-root data --tool inspect_project
node scripts/call-monogatari-mcp.mjs --project-root data --tool plan_transaction --arguments-file transaction.json
node scripts/call-monogatari-mcp.mjs --project-root data --tool apply_transaction --transaction-file transaction.json --expected-precondition-fingerprint <sha256> --allow-write
```

Use `--allow-write` only with a reviewed `apply_transaction` request. The client initializes the MCP session, fixes the server project root through startup arguments, enforces response and timeout bounds, and prints structured content without routing authored text through the PowerShell native-process pipeline.

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

This is read-only. Add `--allow-write` to `args` only for a client that should be able to apply reviewed transactions. To permit fixed-root archive inspection, private runtime validation, and optional output, also add `--package-output-dir` and one user-reviewed external directory, for example `C:\\path\\to\\packages`. The directory flag enables read-only inspection and validation but does not enable durable writes. Read-only processes share an operating-system project lease. A write-enabled process requires the exclusive lease, so no other reader or writer can observe its staged multi-file candidate or race an MCP package snapshot.

## Tools

| Tool | Mode | Contract |
|---|---|---|
| `inspect_project` | Read | Returns scrubbed settings readiness and a complete JSON catalog report |
| `validate_project` | Read | Runs every shared headless runtime/catalog/reference gate and returns structured evidence without writing |
| `validate_delivery` | Read | Extends core validation with declared renderer/audio asset existence, extension, and placeholder evidence |
| `list_project_json` | Read | Lists exact byte SHA-256, semantic content fingerprint, size, kind, and portable path; accepts an optional catalog filter |
| `read_project_json` | Read | Reads one exact-case JSON path beneath an authorable catalog |
| `preview_workflow` | Read | Executes one project Workflow through the deterministic provider-free preview domain and returns versioned trace, stop, coverage, and exact source SHA evidence |
| `run_quality_suite` | Read | Executes one exact `quality_suites/...json` path through the shared headless domain, including bounded per-run Workflow choice maps, and returns versioned scenario/audit evidence bound to its byte SHA-256 |
| `preview_project_package` | Read | Builds the complete credential-free package manifest and deterministic content fingerprint without writing; reports whether an output directory is configured |
| `inspect_project_package` | Read | Verifies one portable `.monogatari` file name inside the startup-fixed external package directory through the shared bounded archive reader; does not extract or write |
| `validate_project_package` | Read | Extracts one fixed-directory package into private process-owned staging, runs shared core-runtime and delivery acceptance, returns structured pass/failure evidence, and removes staging |
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
8. List the `workflows` catalog and call `preview_workflow` for every changed graph with the intended environment, run context, choices, step bound, seed, or injected random values. Review the source SHA, executed nodes, stop reason, coverage, and unvisited nodes; no model provider or persistent project state is used.
9. List the `quality_suites` catalog and call `run_quality_suite` for every intended suite path. Use `workflow_choice_selections` when deterministic choice nodes must be covered without desktop state; each map is one run and reports its applied selections, while multiple runs aggregate union coverage. Accept the suite only when `passed` is `true`; `passed: false` is a successful protocol response whose report contains actionable failed-scenario evidence.
10. Call `preview_project_package` and review the full manifest, file inventory, scrubbed settings, and `content_sha256`.
11. If an archive is required and the package directory is configured, call `export_project_package` with that exact fingerprint and one file name. Keep `replace_existing` false unless replacing the existing artifact is intentional; any intervening project change invalidates the fingerprint.
12. Call `inspect_project_package` with the same file name and require `verified: true` plus the expected content fingerprint.
13. Call `validate_project_package` with the same file name and require `passed: true`; `passed: false` is a successful protocol response with actionable extracted-runtime or delivery evidence.
14. Run persistent installation and rendered visual gates appropriate to the deliverable. When editing the repository's built-in `data/` project, also run `node scripts/sync-project-mirror.mjs --write` followed by `node scripts/sync-project-mirror.mjs --check` so `rust-engine/data/` remains byte-equivalent.

Dialogue changes are accepted only when they pass the shared headless normalization and authoring rules used by the desktop editor, including bounded text and prompts, graph validity, LLM prompt requirements, character references, and relationship targets/deltas.

Knowledge changes are accepted only when they pass the shared desktop/headless document domain: regular bounded UTF-8 JSON files, known fields and shapes, canonical portable IDs/categories/tags/content, bounded metadata and catalog size, unique IDs, and complete non-self related-entry references. Runtime indexes are built from that normalized validated catalog, so successful Agent acceptance and desktop activation use identical categories, legacy relation aliases, ordering, and search inputs.

Planning and application both re-read current state. Any intervening file change invalidates the SHA precondition or plan fingerprint instead of overwriting newer work.

## Acceptance Boundary

Read-only inspection reports `acceptance_level: "document"`. Successful `apply_transaction` reports `acceptance_level: "core_runtime"`: in addition to document safety and settings readiness, the staged candidate must load through the real character and dialogue managers plus the shared validated normalized Knowledge document builder; load strict bounded scene, ending, Story Event, Workflow, and Quality Suite catalogs; validate Workflow graphs and Quality expectation/reference contracts; and pass all prior runtime references. Rejection rolls back every staged operation. `preview_workflow` is a separate read-only gate: it returns `monogatari-mcp-workflow-preview/v1` from the shared validated loader and provider-free executor, including exact source SHA-256, deterministic trace, stop reason, and coverage without provider calls or persistent state mutation. `run_quality_suite` accepts only a bounded path in the fixed project's `quality_suites` catalog and returns `monogatari-mcp-quality-suite-run/v1` with the shared complete report, exact source SHA-256, and MCP build/time provenance.

`preview_project_package` reports `monogatari-mcp-package-preview/v1`; it proves bounded inventory, portable paths, credential scrubbing, manifest self-validation, and the current package content fingerprint without writing. `export_project_package` reports `monogatari-mcp-package-export/v1`; it rebuilds and confirms that fingerprint, then streams and revalidates every file through the shared staged ZIP writer into the startup-fixed directory. `inspect_project_package` reports `monogatari-mcp-package-inspection/v1`; the shared reader verifies bounded regular unencrypted ZIP entries, strict portable paths, declared inventory and sizes, JSON syntax, SHA-256/MD5 checksums, manifest consistency, and credential-free settings without extraction.

`validate_project_package` reports `monogatari-mcp-package-validation/v1`. It extracts through the same bounded reader into a uniquely created system-temporary directory outside the authored project, runs the complete shared `core_runtime` and delivery validator over the restored files, returns the package fingerprint plus structured `passed`/delivery evidence, and removes staging on success or rejection. It does not accept an extraction path or persist an imported project. This proves ephemeral runtime re-import acceptance, not desktop installation, active-state switching, desktop/mobile rendering, or visual quality.

Use the full release gate before a release claim:

```powershell
node scripts/verify-release.mjs
```

Run the isolated MCP gate while iterating:

```powershell
node scripts/verify-modules.mjs --module rust-mcp
```
