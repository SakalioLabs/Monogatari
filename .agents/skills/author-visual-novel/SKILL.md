---
name: author-visual-novel
description: Create, revise, validate, and package Monogatari visual novel projects from story briefs or existing project data. Use for visual novel, galgame, VN, character, dialogue, scene, event, ending, workflow, quality-suite, or automated story-authoring tasks, including requests to generate a complete playable project for people or agents.
---

# Author Visual Novels

Build project data that the real Monogatari runtime can load. Treat a project as a connected content graph, not a collection of unrelated JSON examples.

## Establish The Contract

1. Resolve the repository root with `git rev-parse --show-toplevel`.
2. Resolve the target project root. It must contain `settings.json`; the checked-in example root is `data/`.
3. Read [references/project-contract.md](references/project-contract.md). Read `docs/DATA_FORMAT.md` only for the content types being changed.
4. Inspect nearby checked-in examples before choosing field names or schema versions.
5. Keep runtime credentials outside project files. Never place API keys, tokens, passwords, or authorization headers in content or reports.

## Author In Dependency Order

1. Define a stable ID map and story bible before writing files.
2. Create knowledge and character records.
3. Add renderer assets and scenes, using project-relative portable paths.
4. Add dialogue graphs and verify every node, speaker, choice, and terminal path.
5. Add event unlock rules and endings only after their referenced content exists.
6. Add workflows after scene, dialogue, character, and event IDs are stable.
7. Add or update a Quality Suite that proves critical branches, character identity, knowledge boundaries, and prompt-safety behavior.
8. Keep mirrored `data/` and `rust-engine/data/` roots byte-equivalent when editing the built-in project.

Synchronize and verify the built-in project after an accepted transaction:

```powershell
node scripts/sync-project-mirror.mjs --write
node scripts/sync-project-mirror.mjs --check
```

Use structured JSON editing and preserve unrelated author changes. Do not invent a parallel schema or bypass runtime validation with a custom parser.

Knowledge entries use lowercase portable IDs and normalized lowercase category labels. Trim titles, content, tags, and `related_entries`; deduplicate tags case-insensitively and relations exactly; keep importance in `0..=1`; and create every related target in the same transaction or beforehand. `relatedEntries` remains a read-compatibility alias for legacy projects, but new Agent output must write canonical `related_entries`. Core-runtime acceptance rejects non-canonical or dangling Knowledge candidates and rolls the transaction back.

## Apply Agent Transactions Safely

When an Agent transport offers transaction planning or application, read [references/agent-transaction.md](references/agent-transaction.md). Use `missing` only for new files and an exact current SHA-256 for updates or deletions. Plan first, review every resolved path and resulting hash, then apply with the authoritative candidate-project validator. Do not use the transaction API as a substitute for graph, runtime, package, or experience validation.

MCP stdio frames are UTF-8. On Windows PowerShell 5, do not pipe non-ASCII JSON directly to the native MCP process: the native-process pipe can replace authored text even when the input file was read with `-Encoding UTF8`. Use an MCP client that writes UTF-8 bytes and verify non-ASCII content through `read_project_json` after application.

## Use Standard MCP When Available

The repository ships `monogatari-mcp`, documented in `docs/MCP_SERVER.md`. Call `inspect_project`, `validate_project`, and `validate_delivery` first, use `list_project_json` and `read_project_json` to obtain exact preconditions, then call `plan_transaction`. Call `apply_transaction` only when the server explicitly reports write mode and pass the unchanged transaction plus the reviewed plan's `precondition_fingerprint`. Call both validators again after application. For every changed Workflow, call `preview_workflow` with the intended run context, environment, choices, and deterministic random inputs; review its executed nodes, stop reason, coverage, unvisited nodes, and source SHA. Then list the `quality_suites` catalog and call `run_quality_suite` for every intended suite path. Treat read-only `document` acceptance as JSON safety evidence, `core_runtime` as real catalog/runtime acceptance, delivery validation as declared asset readiness, `monogatari-mcp-workflow-preview/v1` as provider-free graph evidence, and `monogatari-mcp-quality-suite-run/v1` as deterministic scenario evidence bound to the reported source SHA. A Quality failure is a successful tool response with `passed: false` and actionable scenario issues.

For archive delivery, call `preview_project_package` only after project, delivery, and Quality acceptance. Review its complete credential-free manifest, inventory, and `content_sha256`. Call `export_project_package` only when the server reports both write mode and configured package output, pass the exact reviewed content fingerprint, and supply one portable `.monogatari` file name rather than a path. Leave `replace_existing` false unless replacement is intentional and independently reviewed. After export, call `inspect_project_package` and `validate_project_package` with the same file name; require the expected content fingerprint, `verified: true`, and `passed: true`. Inspection proves bounded archive integrity without extraction. Package validation proves private temporary extraction plus shared core-runtime and delivery acceptance, then removes staging; it does not prove persistent installation or rendered visual quality.

## Validate The Result

Run narrow gates while iterating:

```powershell
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs --module rust-authoring --module rust-mcp --module rust-game --module rust-tauri
```

Run the complete release gate before declaring a project deliverable:

```powershell
node scripts/verify-release.mjs
```

For UI or renderer changes, also exercise the relevant route at desktop and mobile sizes. A JSON parse, successful build, or generated image alone does not prove a playable story flow.

## Report Evidence

Summarize changed content IDs, reachable story paths, Quality Suite coverage, commands run, and any hardware-only or provider-only checks that remain unverified. Keep blockers explicit instead of silently substituting mock behavior.
