# Developer Test Handoff

This repository is ready for feature-development testing against the checked-in `master` baseline. This handoff describes evidence, not a commercial-release claim.

## Verified Baseline

The complete default module matrix and integrated release gate passed on Windows on 2026-07-18:

- 18 default modules passed, with 0 failed, planned, or skipped; the separate integrated release gate also passed.
- Frontend: 183 Vitest tests, production Web/PWA build contracts, and 22 Chromium authoring/Playtest workflows.
- Rust: core, authoring, MCP, AI, assets, scripting, game, Tauri tests/checks, and workspace Clippy.
- Scene Roleplay: provider-neutral game-state tests, source-bound headless previews, all three Blue Frame route/ending Quality replays, real MCP stdio execution, and desktop/mobile main-stage layout coverage.
- Legacy .NET: warnings-as-errors build and 55 tests.
- Repository security, project mirror, Web/PWA distribution, package, and release-manifest checks passed in the integrated gate.

Reproduce the independent module evidence:

```powershell
node scripts/verify-modules.mjs --report .cache/module-verification.json
```

Reproduce the integrated release evidence:

```powershell
node scripts/verify-release.mjs
```

## Human Development Flow

1. Install frontend dependencies with `cd frontend; npm ci`.
2. Install the browser test runtime once with `npx playwright install chromium`.
3. Run `npm run dev` for Web/PWA authoring or the documented Tauri development command for desktop integration.
4. Exercise the free-input Scene Roleplay main stage, generated NPC reply, evaluator evidence, score transition, and ending selection on the target model/hardware. Also cover character creation, scripted Dialogue compatibility, scene/event/ending references, Workflow validation, and Quality Suite execution for the feature being changed.
5. Run the owning module from `scripts/module-test-matrix.json` while iterating.
6. Run the complete release gate before handing the branch to another developer.

The module inventory and exact commands are documented in [MODULE_VERIFICATION.md](MODULE_VERIFICATION.md). Commercial packaging requirements remain in [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md).

## Agent Development Flow

Use the repository Skill at `.agents/skills/author-visual-novel/SKILL.md` for dependency-ordered visual-novel authoring. The standard MCP server is documented in [MCP_SERVER.md](MCP_SERVER.md).

1. Start `monogatari-mcp` with one fixed project root; omit `--allow-write` for inspection-only clients, and add a reviewed external `--package-output-dir` when the Agent should inspect, privately validate, or emit archives.
2. Call `inspect_project`, `validate_project`, and `validate_delivery` before editing.
3. Read exact documents and hashes with `list_project_json` and `read_project_json`.
4. Call `preview_scene_roleplay` for every dynamic story route and require source-bound node, ending, score, evidence, and coverage results. This deterministic replay does not claim live model quality.
5. Submit `monogatari-agent-project-transaction/v1` to `plan_transaction`.
6. Review every path and resulting hash, then call `apply_transaction` with the unchanged transaction and exact plan fingerprint.
7. Repeat both validators and execute every intended Quality Suite.
8. Call `preview_project_package`, review its credential-free manifest and content fingerprint, then call `export_project_package` with that exact fingerprint and one portable file name when archive output is required.
9. Call `inspect_project_package` with the same file name and require verified archive evidence with the expected content fingerprint.
10. Call `validate_project_package` with the same file name and require `passed: true` plus the expected package fingerprint.
11. Run persistent installation, rendered-experience, and live provider/hardware generation gates appropriate to the deliverable.

`document` acceptance proves bounded JSON safety. `core_runtime` acceptance additionally proves real catalog loading and cross-reference validation. Delivery acceptance proves declared asset readiness. Package export proves reviewed manifest generation and streamed archive output under the fixed destination policy; package inspection proves bounded archive integrity without extraction; package validation proves temporary extraction plus shared runtime/delivery acceptance and then removes staging. None of these levels alone claims persistent installation, rendered visual quality, or provider-backed generation.

## Test Boundaries

The following are intentionally environment-dependent and are not blockers for ordinary development testing:

- Stable/beta installer publication still requires real Authenticode signing evidence.
- The post-hardening main-stage WebGPU Scene Roleplay path still requires a physical target-GPU turn test. Confirm a generated NPC reply, independent evaluator result, and deterministic state update; specifically retest the reduced-context retry after any ORT `std::bad_alloc` failure.
- CUDA, ROCm, Metal, mobile SDK, and managed-provider generation require matching hardware, SDKs, models, or credentials.
- The cached Windows linker may emit non-failing `LNK4209` debug-information warnings; Clippy and all executable tests still pass.
- Visual quality remains a human/rendered-experience review even when `validate_delivery` reports every declared asset present.

Do not place provider credentials in project JSON, reports, commits, or MCP transactions. Project settings and package exports are designed to scrub runtime secrets, but source-control hygiene remains mandatory.
