# Monogatari Module Verification

## Independent Project Production Simulation

The `projects/konosuba` Chapter 1 project is a real-content acceptance fixture,
not part of the built-in sample catalogs. Its first verified milestone covers
the afterlife selection through formation of the initial four-person party.

Chapter 2 extends the same independent fixture through consent-bounded skill
training, relationship repair, party coordination, and the cemetery encounter.

Chapter 3 extends it through explosion accountability, curse repair, revocable
lake-risk consent, recovery, and an agency-preserving confrontation with
Mitsurugi.

Chapter 4 extends it through the broken ceasefire, an undead lure and cleared
Explosion window, observable boss-mechanic inference, injury-aware frontline
rotation, coordinated purification, revival, and accountable reconstruction.

The four chapters are now connected by `volume1_campaign`. Browser Playtest
starts Chapter 1 as a free-input NPC scene, shows `Chapter 1 / 4`, and advances
only after the active Roleplay reaches a routed ending. Relationship values
carry into the next chapter while chapter-local scores and evidence remain in
the completion record.

- Campaign/game core: 47/47 library tests pass, including transcript replay,
  forged score/cursor rejection, route-history replay, explicit completion,
  cycle rejection, and relationship-only chapter handoff.
- Frontend: 201/201 unit tests and the production Web/PWA build pass. The
  generated project manifest inventories Campaign files and the service worker
  caches them.
- Browser Campaign Playtest: desktop `1440x900` and mobile `390x844` render
  without horizontal overflow. A real free-input turn produces an in-character
  Aqua response; unavailable external inference uses the authored in-scene
  recovery label without exposing `OrtRun` or `std::bad_alloc`.
- Save v4: assets tests 12/12 and focused Tauri save tests 7/7 pass. Active
  Campaign/Roleplay sessions round-trip, while forged roleplay state is
  rejected before existing runtime state changes.

- MCP `validate_project`: valid, 63 JSON documents, 7 characters, 14 Knowledge
  entries, 15 scenes, 4 Scene Roleplays, 12 endings, and 4 Quality Suites.
- MCP `validate_delivery`: valid, 44/44 declared renderer assets exist, with no
  placeholder characters or delivery issues.
- Chapter 4 MCP `run_quality_suite`: 2/2 scenarios pass. The defended-gate route
  executes 16 turns, visits 8/8 nodes, records all 8 required evidence IDs,
  reaches `chapter4_axel_defended`, and ends with Aqua `0.46`, Beldia `0.06`,
  Megumin `0.14`, and Darkness `0.23`. The structural forged-state attack is
  detected and guarded, leaves all four scores and eight evidence IDs empty,
  preserves the four exact initial relationships, and cannot select an ending.
- Chapter 4 browser Playtest: the main stage accepts free-form input and applies
  fallback evaluation to the active score state. With the configured external
  model unavailable, the player sees a diegetic Beldia recovery plus the
  explicit degraded-turn label, with no raw ORT or allocation error. Desktop
  `1440x900` and mobile `390x844` layouts render without horizontal overflow.
- Chapter 3 MCP `run_quality_suite`: 2/2 scenarios pass. The agency-preserving
  route executes 18 turns, visits 9/9 nodes, records all 9 required evidence
  IDs, reaches `chapter3_choice_not_prize`, and ends with Aqua `0.74`, Beldia
  `-0.07`, Megumin `0.24`, and Mitsurugi `0.14`. The forged-state attack is
  guarded and preserves the four exact initial relationships, zero scores, and
  zero evidence.
- `llm-authoring`: 160/160 tests pass, including project-seeded relationship
  previews and bounded Quality relationship expectations.
- Chapter 3 generated sprite validation: all four final character PNGs are
  RGBA, have transparent corners, and retain nonempty subject coverage.

- Before Chapter 3, MCP `validate_project`: valid, 32 JSON documents, 5
  characters, 8 Knowledge entries, 8 scenes, 2 Scene Roleplays, 6 endings, and
  2 Quality Suites.
- MCP `validate_delivery`: valid, 29/29 declared renderer assets exist, with no
  placeholder characters or delivery issues.
- Chapter 2 MCP `run_quality_suite`: 2/2 scenarios pass. The high-trust route
  executes 12 turns, visits 6/6 nodes, records all 6 required evidence IDs,
  reaches `chapter2_trust_in_practice`, and ends with Chris `0.28`, Darkness
  `0.18`, and Wiz `0.36`. The attack scenario guards one forged relationship,
  score, evidence, and ending request and leaves all story state at zero.

- MCP `validate_project`: valid, 18 JSON documents, 3 characters, 7 Knowledge
  entries, 5 scenes, 1 Scene Roleplay, 3 events, 3 endings, and 1 Quality Suite.
- MCP `validate_delivery`: valid, 19/19 declared renderer assets exist, no
  placeholder characters, and no delivery errors or warnings.
- MCP `run_quality_suite`: 2/2 scenarios pass. The working-party route executes
  14 turns, visits 7/7 nodes, records all 7 required evidence IDs, and reaches
  `chapter1_party_formed`; the intrusion case records one guarded intervention
  and leaves every score at zero.
- Generated sprite validation: all six final character PNGs are RGBA, have four
  transparent corners, and retain nonempty bounded subject coverage.
- Rust `llm-game` Scene Roleplay tests: 13/13 pass.
- Frontend targeted Scene Roleplay/inference tests: 12/12 pass, including
  project-API selection and ORT allocation-failure containment.
- Tauri Scene Roleplay command tests: 2/2 pass.

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
| Agent and test orchestration | `automation-contracts` | Node unit contracts | Matrix schema, ownership, selection, CLI parsing, target-platform command/path adaptation, deterministic repository walking and JSON parse/read evidence, bounded credential/UI text scan evidence, locale shape/key/value/public/embedded mirror evidence, structural frontend route/sidebar coverage, fail-closed release-channel/manifest evidence, multi-provider CSP/hosting policy validation, static Web distribution evidence including `roleplays/`, Web preview route/content response evidence, cross-root Story Event/Ending fingerprints, Dialogue graph/reference evidence, pure Workflow shape and catalog-aware file evidence, Renderer Asset path/binary/license evidence, Knowledge ID/alias/reference evidence, Quality Suite shape/default-baseline/Event-fingerprint evidence, real UTF-8 MCP stdio framing, fingerprinted project-asset import plans plus GLB/traversal/symlink/case/precondition rejection, independently injected Tauri package/mobile, installation, exact 115-command registration, dialog capabilities, 79-requirement conversation safety, headless Quality/Workflow/Scene Roleplay, build/toolchain, project runtime, project package, and Story Content policies |
| Vue pure libraries, workflow/Story Playtest, Pinia, and shared components | `frontend-unit` | Vitest unit and Happy DOM component tests | Provider-neutral browser Scene Roleplay sessions, local authoring API selection without browser credentials, attack isolation, grounded output guards, rotating recovery, authored fallback score/evidence, deterministic transitions, prompt/evaluator parsing, and component-level ORT `std::bad_alloc` containment; WebGPU context compaction and memory-error recovery classification; Workflow catalog parity, authoring and preview; grapheme-safe Story playback; bounded compatibility NPC prompting; exact Quality roleplay/Workflow report contracts and export shaping; Settings, Character, Knowledge, Dialogue, Story Event, Ending, Scene, renderer, Store, and shared interaction/accessibility behavior |
| Browser authoring and Playtest workflows | `frontend-e2e` | Playwright Chromium tests against an isolated in-process Vite server | Programmatic startup/cleanup; workspace, Workflow, Dialogue, Event, Ending, Scene, Scene Asset, Knowledge, Quality, and Settings authoring flows; Blue Frame 3D probes; main-stage dynamic Scene Roleplay node/goal/score/free-input layout without invoking a real model; and dedicated scripted Ending epilogues across desktop and 390px mobile layouts |
| Vue/TypeScript/Web/PWA distribution | `frontend-contracts` | Type check, production builds, static contract verifiers | Root/subpath package and responsive shell contracts |
| Rust core | `rust-core` | Unit and doc tests | Infrastructure crate |
| Headless authoring core | `rust-authoring` | Unit, integration, and doc tests | Bounded Scene Roleplay loading, cross-project references, source-bound provider-free turn replay, node/ending/score/evidence/intrusion/guard coverage, the checked-in 45-attack corpus, and complete Quality integration; atomic content mutation, portable paths, settings/packages, prompt guards, conversation safety/Event decisions, canonical Dialogue and Knowledge validation, Scene assets, Workflow persistence/execution previews, Agent transactions, and real core-runtime acceptance without Tauri |
| Standard MCP adapter | `rust-mcp` | Unit, protocol, and real stdio child-process tests | Fixed project root, fourteen schema-backed tools, source-bound provider-free Scene Roleplay and Workflow previews, shared-source Quality execution/evidence, project-external leases, core/delivery validation, reviewed transaction/package fingerprints, fixed external package inspection/private runtime validation/output, cleanup, write exclusion, candidate validation, and rollback |
| Rust AI | `rust-ai` | Unit, integration, and doc tests | Inference contracts and backend planning |
| Rust assets | `rust-assets` | Unit tests | Asset and save boundaries |
| Rust scripting | `rust-scripting` | Unit tests | Rhai execution and condition boundaries |
| Rust game | `rust-game` | Unit and integration tests | Provider-neutral Scene Roleplay definitions, multilingual/obfuscated input isolation, grounded/meta-leak output guards, rotating recoveries, authored fallback scoring/evidence, strict evaluator parsing, atomic score/evidence/session updates, deterministic transition priority/timeouts/exhaustion, bounded transcript/prompt construction, plus characters, dialogue, knowledge, and script parsing |
| Tauri application | `rust-tauri`, `rust-tauri-check` | Command/project tests and compile check | Desktop Scene Roleplay commands adapt active state, character/Knowledge context, guarded generation, NPC/evaluator inference recovery, authored fallback evaluation, optimistic atomic commit, and IPC around the shared game core; Quality execution delegates complete roleplay/Workflow evidence to authoring; existing Dialogue, Knowledge, Workflow, Scene, Ending, package, and runtime compatibility adapters remain covered |
| Rust workspace | `rust-clippy` | All-target warnings-as-errors lint | Runs after crate tests so diagnostics retain module context |
| Legacy .NET | `legacy-dotnet-hash`, `legacy-dotnet-native`, `legacy-dotnet-build`, `legacy-dotnet-tests` | Stream-owned SHA-256 unit, pinned SDL2 preparation, warnings-as-errors build, shared tests, native ABI/license probe, and Windows dummy-driver multi-frame render loop | Archive verification does not depend on runner cmdlets; interactive and headless modes execute through the same product `WindowManager` and `RenderContext` |
| Complete product | `release-gate` | Build, package, preview, content, security, and runtime checks | Integrated and intentionally slower |

GitHub Actions runs the automation, frontend, Rust, and .NET groups as separate jobs and uploads their machine-readable reports. A green CI run therefore proves more than the previous frontend-build/Tauri-check pair, while the full release gate remains the final local or release-workflow requirement.

## Open Audit Work

1. Continue frontend state-machine extraction after isolating Scene Roleplay, Workflow authoring/preview/presentation, Story playback, 3D framing, Quality Suite, Settings, Character, Dialogue, Story Event, Ending, Knowledge, Scene, and Scene Asset domains; remaining large runtime/workbench orchestration and authoring-view decomposition remain.
2. Continue moving remaining schema-specific catalog validation and cross-reference discovery behind `llm-authoring`; Scene Roleplay loading/references/preview, project settings, portable paths, Dialogue/Knowledge validation, Workflow persistence, Quality execution, package inventory/archive mechanics, JSON inspection, transactions, and runtime/delivery package re-import acceptance are now shared without transport duplication. MCP uses private removable staging for validation, while persistent imported-directory commit and active-state switching intentionally remain Tauri-owned.
3. Continue decomposing release verification; frontend, AI, path, route, release-channel, Web hosting/distribution/preview, repository JSON/text, locale coverage, Story Event/Ending, Dialogue, Workflow, Renderer Asset, Knowledge Reference, Quality Suite, and Tauri packaging evidence now live in importable modules. Project-content source-policy decomposition, the pure Tauri collector, shared security/UI scans, and locale checks are complete; remaining entry-point responsibilities are generic subprocess orchestration and report sequencing.

These are explicit gaps, not implied failures. They remain part of the project-wide convergence goal until each has authoritative tests or is removed from the supported architecture.
