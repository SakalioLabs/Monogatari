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
| Agent and test orchestration | `automation-contracts` | Node unit contracts | Matrix schema, ownership, selection, CLI parsing, platform command adaptation, structural frontend route/sidebar coverage, fail-closed release-channel/manifest evidence, and multi-provider CSP/hosting policy validation |
| Vue pure libraries, workflow/Story Playtest, Pinia, and shared components | `frontend-unit` | Vitest unit and Happy DOM component tests | Workflow catalog parity, node creation/layout/connections/document synchronization, execution-evidence parsing and node-state presentation, exact Quality and Settings contracts, generated preview isolation, Quality filtering/diagnostics/export shaping, Settings config/path/secret/manifest boundaries, character and Knowledge form conversion/validation/payload/isolation, browser Knowledge persistence and cross-catalog reference protection, shared proxy-safe JSON values, immutable Dialogue, Story Event, Ending, and Scene draft transformations, stable Scene filtering/tag/warning/diagnostic evidence, project-backed Scene Asset catalog shaping, active-state persistence/history/filtering/presentation, Metadata state, stable event/Ending reference and unlock evidence, portable ID handling, renderer fallback selection, story access, shared local condition evaluation, bounded browser workflow execution, dialogue runtime transitions/scripts/conditions/relationship effects, Store async state, and shared interaction/accessibility behavior |
| Browser authoring and Playtest workflows | `frontend-e2e` | Playwright Chromium tests against an isolated Vite server | Workspace navigation, deterministic Workflow trace/coverage/canvas evidence, validated browser character drafts and portable duplicate-ID rejection across reloads, two-node dialogue rename/connect/save-to-Playtest execution, Story Event Metadata-only dirty state plus reactive duplication persistence, real-reference Ending save/preview and case-folded collision rejection, real-background Scene save/Playtest and collision rejection, complete Scene Asset diagnostics plus persisted runtime selection, Knowledge save/reload plus browser-character reference deletion protection, the 29-scenario Quality workbench, and credential-free Settings manifest export across desktop and 390px mobile layouts |
| Vue/TypeScript/Web/PWA distribution | `frontend-contracts` | Type check, production builds, static contract verifiers | Root/subpath package and responsive shell contracts |
| Rust core | `rust-core` | Unit and doc tests | Infrastructure crate |
| Headless authoring core | `rust-authoring` | Unit, integration, and doc tests | Atomic content mutation with pre-write portable case-alias rejection, portable project/package paths, project settings, deterministic package inventory/manifest generation with shared bounds and self-validation, fixed-buffer ZIP output plus bounded inspection/empty-root extraction, explicit create/replace policy and rollback, package fingerprint/topology/secret policy, multilingual prompt guards, deterministic conversation scoring/safety/event decisions, authoritative Workflow node catalog, shared Quality/Workflow schemas, side-effect-free Workflow previews, complete Quality execution/evidence, Agent transactions, and real core-runtime manager/reference validation without Tauri |
| Standard MCP adapter | `rust-mcp` | Unit, protocol, and real stdio child-process tests | Fixed project root, ten schema-backed tools, project-external cross-process leases, core/delivery validation, bounded Quality execution/evidence, reviewed transaction and package fingerprints, fixed external package output, write exclusion, candidate validation, and rollback |
| Rust AI | `rust-ai` | Unit, integration, and doc tests | Inference contracts and backend planning |
| Rust assets | `rust-assets` | Unit tests | Asset and save boundaries |
| Rust scripting | `rust-scripting` | Unit tests | Rhai execution and condition boundaries |
| Rust game | `rust-game` | Unit and integration tests | Characters, dialogue, knowledge, and script parsing |
| Tauri application | `rust-tauri`, `rust-tauri-check` | Command/project tests and compile check | Desktop commands adapt state into shared authoring domains; Scene, Dialogue, and Ending saves prove portable case aliases cannot replace existing documents; provider calls, persistent runtime effects, remaining catalogs, plus package path selection, staged runtime reload, cleanup, and non-overwriting import commit stay in the command crate |
| Rust workspace | `rust-clippy` | All-target warnings-as-errors lint | Runs after crate tests so diagnostics retain module context |
| Legacy .NET | `legacy-dotnet-native`, `legacy-dotnet-build`, `legacy-dotnet-tests` | Pinned SDL2 preparation, warnings-as-errors build, shared tests, native ABI/license probe, and Windows dummy-driver multi-frame render loop | Interactive and headless modes execute through the same product `WindowManager` and `RenderContext` |
| Complete product | `release-gate` | Build, package, preview, content, security, and runtime checks | Integrated and intentionally slower |

GitHub Actions runs the automation, frontend, Rust, and .NET groups as separate jobs and uploads their machine-readable reports. A green CI run therefore proves more than the previous frontend-build/Tauri-check pair, while the full release gate remains the final local or release-workflow requirement.

## Open Audit Work

1. Continue frontend state-machine extraction after isolating Workflow authoring, preview, and execution presentation plus Quality Suite, Settings, Character, Dialogue, Story Event, Ending, Knowledge, Scene, and Scene Asset domains; remaining large runtime/workbench views plus end-to-end browser Story Playtest workflows remain.
2. Continue moving schema-specific catalog validation and cross-reference discovery behind `llm-authoring`; project settings, portable paths, package inventory/manifest generation, ZIP output/inspection/extraction, JSON document inspection, and transactions are now shared without transport duplication. MCP exposes fixed-root reviewed package output; schema-backed archive inspection/import still needs an explicit fixed input/output boundary before it can safely expose the shared reader.
3. Continue decomposing `scripts/verify-release.mjs`; frontend, AI, and path source-invariant checks plus frontend route, release-channel, and Web hosting policy validation now live in importable modules, while content, packaging, and live browser gates remain in the entry point.

These are explicit gaps, not implied failures. They remain part of the project-wide convergence goal until each has authoritative tests or is removed from the supported architecture.
