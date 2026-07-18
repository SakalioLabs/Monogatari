---
name: author-visual-novel
description: Create, revise, validate, and package Monogatari AI visual novel and scene-roleplay projects from story briefs or existing project data. Use for real-time NPC roleplay, dynamic scene nodes, scoring and ending routes, visual novels, galgames, characters, dialogue, scenes, events, workflows, quality suites, or automated story authoring for people and agents.
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
4. Add endings before any dynamic route targets them.
5. For interactive AI stories, create `roleplays/` as the primary story graph: define scene-bound nodes, player and NPC goals, score dimensions, evidence rules, inference budgets, deterministic transitions, timeouts, ending targets, scene-specific `intrusion_response`, grounded `response_guard` recoveries, and conservative `fallback_evaluation` signals.
6. Add `dialogue/` only for intentionally scripted sequences, tutorials, or non-AI projects; verify every scripted node, speaker, choice, and terminal path.
7. Add event unlock rules and workflows after their referenced character, scene, roleplay, dialogue, and ending IDs are stable.
8. Add or update a Quality Suite that replays critical free-form turns and adversarial controls. Prove node coverage, score/evidence boundaries, every required ending, character identity, knowledge boundaries, exact intrusion/guard counts, zero unguarded intrusions, and forbidden response-marker absence.
9. Keep mirrored `data/` and `rust-engine/data/` roots byte-equivalent when editing the built-in project.

When the brief requires real-time AI roleplay, do not replace it with fixed Dialogue or an optional chat panel. The main playable loop must accept free-form player input, generate normal clean-turn NPC responses from the current node's scene, character, goals, bounded transcript, pinned Knowledge, and closed grounding vocabulary, obtain a separate structured score/evidence evaluation, and let only the deterministic roleplay state machine select transitions and endings. Fixed authored lines are permitted only as bounded intrusion or inference/output recovery. Detected control attempts must not enter model context literally: convert them into authored in-world uncertainty or redirection, skip model scoring, and commit zero score/evidence. Do not use real-world mental-health diagnoses as a fallback. A provider-free replay proves authored rules, route coverage, and deterministic containment; it does not prove live model generation. Verify the configured desktop or WebGPU clean-turn generation path separately, including forced NPC/evaluator failure, and report fallback turns as degraded behavior rather than successful model evidence.

Synchronize and verify the built-in project after an accepted transaction:

```powershell
node scripts/sync-project-mirror.mjs --write
node scripts/sync-project-mirror.mjs --check
```

Binary assets are outside Agent JSON transactions. In this repository, plan a bounded local import with `scripts/import-project-asset.mjs`, review its source hash, destination, precondition, byte count, media kind, and `plan_fingerprint`, then repeat the unchanged command with `--write --expected-plan-fingerprint <sha256>`. Use `missing` for a new destination or the exact current destination SHA-256 for replacement. The importer accepts only supported files beneath `assets/`, rejects symlinks, traversal, and case collisions, validates GLB 2.0 structure, stages atomically, and refuses a stale plan. Run delivery validation after importing; importing bytes does not prove that they render correctly or that redistribution rights exist.

Use structured JSON editing and preserve unrelated author changes. Do not invent a parallel schema or bypass runtime validation with a custom parser.

Knowledge entries use lowercase portable IDs and normalized lowercase category labels. Trim titles, content, tags, and `related_entries`; deduplicate tags case-insensitively and relations exactly; keep importance in `0..=1`; and create every related target in the same transaction or beforehand. `relatedEntries` remains a read-compatibility alias for legacy projects, but new Agent output must write canonical `related_entries`. Core-runtime acceptance rejects non-canonical or dangling Knowledge candidates and rolls the transaction back.

## Apply Agent Transactions Safely

When an Agent transport offers transaction planning or application, read [references/agent-transaction.md](references/agent-transaction.md). Use `missing` only for new files and an exact current SHA-256 for updates or deletions. Plan first, review every resolved path and resulting hash, then apply with the authoritative candidate-project validator. Do not use the transaction API as a substitute for graph, runtime, package, or experience validation.

MCP stdio frames are UTF-8. On Windows PowerShell 5, do not pipe non-ASCII JSON directly to the native MCP process: the native-process pipe can replace authored text even when the input file was read with `-Encoding UTF8`. Use an MCP client that writes UTF-8 bytes and verify non-ASCII content through `read_project_json` after application.

For this repository, call one tool with `node scripts/call-monogatari-mcp.mjs --project-root data --tool <name>`. Put non-empty arguments in a UTF-8 JSON object and pass `--arguments-file <path>`. For application, keep the reviewed raw transaction file unchanged and pass it with `--tool apply_transaction --transaction-file <path> --expected-precondition-fingerprint <sha256> --allow-write`; the client constructs the MCP apply envelope. The client fixes the project root at process startup, performs the MCP initialize handshake, writes UTF-8 bytes, enforces a bounded timeout, and returns structured tool evidence.

## Use Standard MCP When Available

The repository ships `monogatari-mcp`, documented in `docs/MCP_SERVER.md`. Call `inspect_project`, `validate_project`, and `validate_delivery` first, use `list_project_json` and `read_project_json` to obtain exact preconditions, then call `plan_transaction`. Call `apply_transaction` only when the server explicitly reports write mode and pass the unchanged transaction plus the reviewed plan's `precondition_fingerprint`. Call both validators again after application. For every changed roleplay, call `preview_scene_roleplay` with representative clean and adversarial player messages, NPC responses, and structured evaluations; review source SHA, per-turn safety categories, guard decisions, detected/guarded/unguarded totals, visited and unvisited nodes, final scores/evidence, selected ending, and trailing-turn errors. Require every adversarial turn to be guarded, every unguarded count to be zero, and all attacked-turn scores/evidence to remain unchanged. For every changed Workflow, call `preview_workflow` with the intended run context, environment, choices, and deterministic random inputs; review its executed nodes, stop reason, coverage, unvisited nodes, and source SHA. Then list the `quality_suites` catalog and call `run_quality_suite` for every intended suite path, including a multilingual/obfuscated security suite. Treat read-only `document` acceptance as JSON safety evidence, `core_runtime` as real catalog/runtime acceptance, delivery validation as declared asset readiness, `monogatari-mcp-scene-roleplay-preview/v1` as provider-free roleplay-rule evidence, `monogatari-mcp-workflow-preview/v1` as provider-free Workflow evidence, and `monogatari-mcp-quality-suite-run/v1` as deterministic scenario evidence bound to reported source hashes. A Quality failure is a successful tool response with `passed: false` and actionable scenario issues.

Scene documents may declare `model_3d_path` for a project-relative `.glb` or `.gltf` environment, with `background_path` retained as an optional loading fallback. Dialogue nodes may declare `scene_id`; Playtest activates that scene when the node is entered. Create every referenced scene before the dialogue transaction so core-runtime validation can reject dangling scene transitions.

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

For runtime AI roleplay, verify that the main story stage starts the intended roleplay and node, accepts free-form input, reveals clean NPC output only after the final guard, requires the authored number of distinct scene anchors, evaluates the completed clean exchange independently, applies only validated and clamped score/evidence changes, and displays the resulting node or ending. Force NPC and evaluator inference errors and verify rotating in-world recovery plus deterministic fallback scoring without technical UI text. Attempt multilingual, structural, state, tool, memory, Unicode, and encoded prompt attacks. Verify that attacks do not invoke the small model, do not appear literally in later model context, become scene-authored in-world responses, and cannot forge evidence, scores, or route conditions. Report actual provider or hardware generation separately from deterministic replay, mocked adapters, and layout evidence.

Browser authoring Playtest accepts `previewRoleplay=<id>&authoring=1` to open a dynamic roleplay on the main stage. It also accepts `previewDialogue=<id>&previewNode=<node-id>&authoring=1` for intentionally scripted Dialogue. Visual sampling does not replay or prove earlier route state, so Scene Roleplay Preview and Quality Suite execution remain the authoritative provider-free route and branch-coverage evidence.

## Report Evidence

Summarize changed content IDs, reachable roleplay nodes and endings, score/evidence dimensions, Quality Suite coverage, commands run, evaluator fallbacks, and any hardware-only or provider-only checks that remain unverified. Keep blockers explicit instead of silently substituting fixed Dialogue, mock behavior, or deterministic replay for live generation.
