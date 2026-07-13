# Monogatari Module Verification

## Purpose

The integrated release gate is broad, but broad verification is not the same as independently addressable module evidence. `scripts/module-test-matrix.json` is the machine-readable ownership and execution map. `scripts/verify-modules.mjs` validates that map, selects gates by module or group, executes each command without a shell, and can emit a JSON report for agents and CI.

```powershell
node scripts/verify-modules.mjs --list
node scripts/verify-modules.mjs --module rust-ai
node scripts/verify-modules.mjs --group rust --report module-verification-rust.json
node scripts/verify-modules.mjs
```

The no-selector form runs every default gate. `release-gate` is intentionally non-default because it rebuilds and smoke-tests the complete distribution; run it explicitly or use `node scripts/verify-release.mjs` before a release claim.

Platform-specific gates declare their supported host IDs in the matrix. The Windows x64 SDL2 preparation and legacy application build are reported as skipped on other hosts instead of failing an otherwise valid frontend or portable-library audit.

## Current Verification Map

| Surface | Gate | Evidence type | Current boundary |
|---|---|---|---|
| Agent and test orchestration | `automation-contracts` | Node unit contracts | Matrix schema, ownership, selection, CLI parsing, platform command adaptation |
| Vue pure libraries, workflow/Story Playtest, Pinia, and shared components | `frontend-unit` | Vitest unit and Happy DOM component tests | Authoring validation, renderer fallback selection, story access, shared local condition evaluation, bounded browser workflow execution, dialogue graph transitions/scripts/conditions/relationship effects, Store async state, and shared interaction/accessibility behavior |
| Vue/TypeScript/Web/PWA distribution | `frontend-contracts` | Type check, production builds, static contract verifiers | Root/subpath package and responsive shell contracts |
| Rust core | `rust-core` | Unit and doc tests | Infrastructure crate |
| Headless authoring core | `rust-authoring` | Unit, integration, and doc tests | Atomic content mutation, portable paths, project settings, Agent transactions, and real core-runtime manager/reference validation without Tauri |
| Standard MCP adapter | `rust-mcp` | Unit, protocol, and real stdio child-process tests | Fixed project root, seven schema-backed tools, core/delivery validation, reviewed fingerprints, write exclusion, candidate validation, and rollback |
| Rust AI | `rust-ai` | Unit, integration, and doc tests | Inference contracts and backend planning |
| Rust assets | `rust-assets` | Unit tests | Asset and save boundaries |
| Rust scripting | `rust-scripting` | Unit tests | Rhai execution and condition boundaries |
| Rust game | `rust-game` | Unit and integration tests | Characters, dialogue, knowledge, and script parsing |
| Tauri application | `rust-tauri`, `rust-tauri-check` | Command/project tests and compile check | Project settings delegate to `llm-authoring`; remaining catalogs and packaging still live in the command crate |
| Rust workspace | `rust-clippy` | All-target warnings-as-errors lint | Runs after crate tests so diagnostics retain module context |
| Legacy .NET | `legacy-dotnet-native`, `legacy-dotnet-build`, `legacy-dotnet-tests` | Pinned SDL2 preparation, warnings-as-errors build, shared tests, native ABI/license probe | A headless render-loop probe is still pending |
| Complete product | `release-gate` | Build, package, preview, content, security, and runtime checks | Integrated and intentionally slower |

GitHub Actions runs the automation, frontend, Rust, and .NET groups as separate jobs and uploads their machine-readable reports. A green CI run therefore proves more than the previous frontend-build/Tauri-check pair, while the full release gate remains the final local or release-workflow requirement.

## Open Audit Work

1. Continue the frontend state-machine extraction from the now-isolated workflow preview into the remaining large editor views and end-to-end browser Story Playtest workflows.
2. Continue moving schema-specific catalog validation, cross-reference discovery, and packaging behind `llm-authoring`; project settings, portable paths, JSON document inspection, and transactions are now shared by Tauri or MCP without transport duplication.
3. Extend the new shared `core_runtime` candidate acceptance beyond characters/dialogue/knowledge to scenes, Story Events, endings, workflows, Quality Suites, and package-level references without moving transport concerns into the headless core.
4. Extend the retained .NET renderer ABI/load coverage with a headless SDL initialization and render-loop probe, or formally remove those projects from the supported product boundary.
5. Continue decomposing `scripts/verify-release.mjs`; frontend, AI, and path source-invariant checks now live in an importable module, while content, packaging, and browser gates remain in the entry point.

These are explicit gaps, not implied failures. They remain part of the project-wide convergence goal until each has authoritative tests or is removed from the supported architecture.
