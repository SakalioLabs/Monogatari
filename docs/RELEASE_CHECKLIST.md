# Monogatari Release Checklist

## Pre-Release Verification

### Automated Gate
- [ ] `node scripts/verify-modules.mjs` passes all default module gates; CI artifacts contain separate automation, frontend, Rust, and .NET reports matching `scripts/module-test-matrix.json`
- [ ] Windows x64 compatibility verification runs `scripts/prepare-legacy-sdl.ps1`, verifies pinned official SDL2 archive hashes, and builds `LLMAssistant.sln` with warnings as errors before the legacy tests run
- [ ] `node scripts/verify-release.mjs` passes from the repository root, covering JSON assets, workflow files, score-gate workflow execution regressions, renderer asset contracts, pinned knowledge-ref contracts, all quality suites, workflow branch coverage snapshots, locale coverage, sensitive token scans, frontend UI text artifact scans, frontend source invariants, legacy C# AI prompt/API invariants, AI backend config, engine project root, asset/save-manager, script command and state-key invariants, i18n locale, workflow command, content loader, Agent transactions, MCP stdio and rollback contracts, character manager, plugin manager, marketplace, Live2D model, TTS output/error/log-privacy invariants, frontend route/sidebar coverage, Tauri desktop packaging and installed-runtime verification, available Windows installer audits, Rust core/authoring/MCP/AI/scripting/game/assets/Tauri checks/tests, frontend audit, root and subpath Web/PWA builds, Web/PWA dist assets, release artifact manifest checks, preview route smoke checks, and legacy C# tests
- [ ] `rust-engine/rust-toolchain.toml` remains pinned to `nightly-2026-07-03` with the minimal profile plus Clippy/rustfmt components; release tests do not override `CARGO_PROFILE_TEST_DEBUG`

### Frontend
- [ ] `cd frontend && npm run test:unit` passes pure authoring, renderer selection, story access, browser workflow/Story Playtest state machines, Pinia async-state, and shared component interaction/accessibility tests
- [ ] `cd frontend && npm run build` passes with zero errors
- [ ] `cd frontend && npm run build:web` emits manifest, service worker, offline fallback, `404.html`, `.nojekyll`, `_headers`, `_redirects`, `staticwebapp.config.json`, `vercel.json`, `project-assets.json`, and `inference-runtime.json` assets
- [ ] `cd frontend && npm run verify:inference-runtime` proves the Web/PWA package declares WebGPU, a supported precision, a bounded generation limit, CSP support, and service-worker caching without runtime secrets
- [ ] Web/PWA dist includes copied `data/assets` project sample backgrounds and character sprites under `dist/assets`
- [ ] Web/PWA manifest includes dedicated install and maskable icons, and `sw.js` precaches those icon assets plus generated project sample assets for offline install surfaces
- [ ] `cd frontend && npm run verify:mobile-readiness` passes, proving safe-area viewport metadata, iOS/PWA install metadata, bottom navigation safe-area padding, and compact Tauri shell limits
- [ ] `cd frontend && npm run verify:responsive-shell` passes after `npm run build:web`, proving built 375px mobile and 768px tablet Web/PWA shell layout signals
- [ ] `cd frontend && npm run verify:web-budget` passes with entry JS/CSS and lazy renderer chunks inside budget
- [ ] `node scripts/verify-tauri-mobile-preflight.mjs` passes, proving Android/iOS command readiness, Vite mobile host binding, compact Tauri shell config, and mobile deployment documentation
- [ ] `npm audit` shows zero vulnerabilities
- [ ] Frontend runtime source contains no `console.log` or `console.debug` debug output before Web/PWA or Tauri packaging
- [ ] Frontend runtime source contains no `v-html` or direct `innerHTML`/`outerHTML` assignment before Web/PWA or Tauri packaging
- [ ] Web/PWA `index.html` and `404.html` include a Content Security Policy meta tag that blocks object/frame/form and JavaScript `unsafe-eval` surfaces while allowing the explicit `wasm-unsafe-eval` needed by ONNX Runtime Web and `connect-src blob:` for embedded GLB textures
- [ ] Web/PWA dist includes a static-hosting `_headers` file with CSP, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and a restrictive browser `Permissions-Policy`
- [ ] Web/PWA dist includes a static-hosting `_redirects` file with project asset passthrough rules and a final `/* /index.html 200` SPA fallback
- [ ] Web/PWA dist includes an Azure Static Web Apps `staticwebapp.config.json` with SPA navigation fallback, static asset exclusions, 404 rewrite, and matching global security headers
- [ ] Web/PWA dist includes a Vercel `vercel.json` with SPA rewrite to `index.html`, no external rewrite destinations, and matching global security headers
- [ ] All 22 views render correctly (Dashboard, Title Preview, Playtest, Character Test, Story Flow, Character Editor, Asset Diagnostics, Settings, Cast Preview, Ensemble Test, Runtime Analytics, Quality Gates, Marketplace, Plugins, Audio, Knowledge, Dialogue Editor, Story Events, Endings, Scene Editor, Visual Review, Transcript)
- [ ] Sidebar navigation works for all 21 engine items, with no engine-level achievement or player-progression surface
- [ ] Responsive layout verified on mobile viewport (375px) and tablet (768px), with the build-time responsive shell verifier attached as release evidence

### Rust Backend
- [ ] `cargo check --locked -p llm-galgame-app` passes
- [ ] `cargo test --locked -p llm-authoring` passes independently of Tauri
- [ ] Agent transaction tests prove strict schemas, portable JSON-only paths, missing/exact-SHA preconditions, duplicate and case-collision rejection, deterministic dry-run plans, complete candidate validation, multi-file commit, and reverse-order rollback
- [ ] `cargo test --locked -p monogatari-mcp` passes, including a real stdio child-process handshake, seven schema-backed tools, scrubbed reads, structured core/delivery validation, deterministic plans, default write refusal, reviewed-plan fingerprint enforcement, one-writer exclusion, successful core-runtime application, and rollback after document or runtime-reference rejection
- [ ] `cargo build --locked --release -p monogatari-mcp` produces the distributable stdio server binary documented in `docs/MCP_SERVER.md`
- [ ] MCP inspection claims document acceptance; candidate application claims only `core_runtime` character/dialogue/knowledge manager and reference acceptance; scene, event, ending, workflow, package, Quality Suite, and experience gates remain separate evidence
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` passes
- [ ] All 25 command modules register correctly in main.rs
- [ ] Chat streaming works with API backend
- [ ] API streaming rejects provider error frames and malformed SSE data frames instead of finalizing partial text as a successful completion
- [ ] Streaming chat failures replace partial assistant bubbles with a stable failure message before surfacing the provider/runtime error
- [ ] Character personality/knowledge injection verified
- [ ] Shared Rust AI prompt builder sanitizes embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings in message history and context sections before OpenAI-compatible role parsing
- [ ] Legacy C# prompt builder sanitizes embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings while the legacy solution remains release-gated
- [ ] Rust API engine debug output and API error surfaces redact API keys, bearer tokens, sensitive custom headers, and provider-echoed secret assignments before logs or frontend error reports expose them
- [ ] Rust API engine rejects blank runtime API keys/models, non-local plaintext HTTP provider URLs, embedded URL credentials, query strings, and fragments before the backend can become active
- [ ] Rust API engine rejects standard and streaming 200 responses that omit non-blank generated text before reporting inference success
- [ ] Shared AI inference pipeline retries or rejects unsuccessful provider result envelopes before chat, streaming, or workflow LLM callers consume generated text
- [ ] Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields; enforce the 1 MiB limit; atomically replace regular `settings.json` files; and reject symlink/non-regular targets
- [ ] Windows ONNX configuration accepts only project-relative `.onnx` model and `.json` tokenizer references, requires DirectML without CPU fallback, validates compatible full-sequence causal-LM inputs and float32 logits, and activates only after initialization succeeds
- [ ] `get_inference_backend_plan` emits `monogatari-inference-backend-plan/v1`, keeps detected/unprobed profiles out of `recommended_backend`, and preserves the documented Qwen3.5 WinML/DirectML blockers
- [ ] Engine initialization stages fresh character/dialogue/knowledge managers before activation, replaces rather than merges previous project content, and clears mutable runtime state on same-root reloads and project switches
- [ ] Engine initialization stages the versioned story event catalog with character-reference validation, and failed loads or hot reloads leave the active catalog unchanged
- [ ] Saving project `settings.json` does not switch the active project root without loading the matching content managers
- [ ] Legacy C# API engine redacts token-shaped values and JSON/header/query secret assignments from provider error bodies and request exceptions while the legacy solution remains release-gated
- [ ] Rust and legacy C# asset managers reject absolute, URI-like, empty, current-directory, and parent-traversal asset paths before reading project assets
- [ ] Rust and legacy C# save managers reject traversal-shaped save IDs before save/load/delete and filter listed saves whose embedded IDs do not match safe filenames
- [ ] Rust `monogatari-game-save/v3` snapshots restore scene history, dialogue cursor/local state, typed Rhai variables, character emotion/relationships/full memory, chat history, evaluations, safety traces, triggered event IDs, and story progress while v1/v2 saves remain readable and migrate known events
- [ ] Quick-save and auto-save overwrite stable bounded slots, while manual saves continue receiving opaque UUID IDs
- [ ] Stable-slot overwrites stage and recover the previous save on replacement failure, clean temporary files after success, and reject save payloads larger than 32 MiB before unbounded reads or writes
- [ ] Rhai script commands and direct `ScriptEngine` callers reject oversized or hidden-control-character payloads, condition expressions use shared limits and run through a read-only Rhai engine, workflow conditions receive read-only relationship/evaluation context variables for desktop and Web/PWA previews, desktop run-context previews and the pure browser workflow preview module mirror per-run variable, flag, signed relationship, emotion, scene, and weighted random-branch behavior without mutating persistent runtime state, unsupported browser condition syntax stops rather than selecting a branch, workflow validation rejects invalid condition and state-key config before execution, script variables/flags use portable state keys before workflow, dialogue, or save data writes, and the shared script engine caps operations, recursion, expression depth, variables, functions, and module imports
- [ ] Workflow save/load commands read and write only JSON files under the active project `workflows/` directory
- [ ] Workflow validation, desktop execution, browser preview, chat scoring, and Quality Suites resolve trigger nodes from the same project story event catalog
- [ ] Story event catalogs reject invalid schemas, duplicate IDs, unsafe configured paths, symlinks, unsupported metrics, out-of-range thresholds, oversized content, and unknown scoped characters
- [ ] Story event catalogs reject unknown, malformed, duplicate, or excessive actions; real chat/workflow triggers share the atomic progress executor while previews remain side-effect free
- [ ] Story content access gates only IDs referenced by `unlock_*` actions; Playtest, dialogue starts, real workflow scene changes, and ending launches reject locked content and admit persisted unlocks
- [ ] Story Event editor validates trigger rules, character scopes, typed actions, target references, and metadata; saves reject stale fingerprints and multi-document flattening and roll back failed replacements
- [ ] Versioned ending assets reject unsafe or unknown fields, expose stable authoring fingerprints, atomically roll back rejected saves, protect Story Event references during deletion, and resolve existing scene/dialogue references before player launch or author preview
- [ ] Scene authoring catalogs merge metadata and inferred backgrounds, reject stale or invalid writes, roll back failed replacements, preserve background files during metadata deletion, and protect Story Event, ending, and workflow references
- [ ] Dialogue authoring catalogs reject unknown fields, broken or unreachable graphs, unknown speakers/relationship targets, invalid deltas, unsafe scripts/prompts, and stale writes; successful saves hot-reload runtime scripts and protected deletes scan Story Events and endings
- [ ] Character, dialogue, and knowledge loader commands read only from the active project `characters/`, `dialogue/`, and `knowledge/` directories
- [ ] Character create/delete commands resolve through the active or discovered default project data root, validate portable character IDs, and touch only direct JSON files under `characters/`
- [ ] Plugin listing, registration, and removal commands resolve through the active or discovered default project data root, validate portable plugin IDs, normalize optional `.rhai` script references under `plugins/`, and touch only direct manifest JSON files under `plugins/`
- [ ] Marketplace import/export commands resolve template references only under the active project `templates/` directory or built-in catalog IDs
- [ ] Live2D model commands load only project-relative `.model3.json`/`.json` model files under the active project data root

### Content
- [ ] Example characters load correctly (Sakura, Luna, Kenji)
- [ ] Example dialogues play through with choices
- [ ] Both checked-in project data roots load through the real character/dialogue/knowledge/event managers, including legacy relationship-object normalization and map-key dialogue node IDs
- [ ] Story Library lists scene/dialogue/ending lock state; Web/PWA and desktop dialogue playback hide false choices while preserving authored indices, follow required linear fallbacks for false nodes, carry script variables/flags into later conditions, reject unsupported browser syntax, reject broken targets, and apply bounded choice relationship effects; desktop playback preflights every relationship target before committing the inspected dialogue cursor
- [ ] Scene and Dialogue editors guard dirty drafts, persist browser catalogs, display real project diagnostics, and preview saved drafts through Playtest on desktop and Web/PWA
- [ ] Release dialogue validation passes for both checked-in data roots with matching catalogs, reachable nodes, valid targets, known characters, and bounded relationship changes
- [ ] Ending Route editor binds real scene/dialogue catalogs, reports event coverage, guards dirty drafts, persists browser drafts, and previews saved routes without requiring player unlock progress
- [ ] Knowledge base search returns relevant results
- [ ] Scene assets validate without missing file warnings
- [ ] Checked-in character renderer asset fields resolve to supported project-relative files or intentionally fall back to the generated 3D placeholder
- [ ] `renderer_fox` resolves `assets/models/fox.glb`; the GLB v2 header, declared length, SHA-256, and packaged attribution file pass release verification in both data roots
- [ ] Core sample characters Sakura, Luna, and Kenji declare checked-in portrait/sprite renderer assets in both data roots
- [ ] Character Editor renderer asset diagnostics flag unsupported extensions, absolute paths, external URLs, and parent traversal before saving/exporting character JSON
- [ ] Character Editor renderer preview follows Playtest priority for Live2D, GLB/GLTF, sprite/portrait, and generated 3D fallback states
- [ ] Playtest and Character Editor preview both derive renderer priority from the shared frontend renderer asset selector
- [ ] `npm run verify:renderer-assets` passes, proving shared renderer selector priority, expression sprite resolution, validation skips, and generated fallback behavior
- [ ] Playtest renderer fallback verified for Live2D, GLB/GLTF, sprite/portrait, assetless character states, and runtime Live2D/GLB/GLTF load failures that must skip to the next valid candidate
- [ ] Real GLB visual probe passes at 1440x900 and 375x812: `data-model-state=ready`, animation count is non-zero, canvas pixels are nonblank/nonuniform, model remains framed, and the animation changes rendered pixels over time
- [ ] Embedded GLB PNG textures decode with their authored colors and no current-build `GLTFLoader` console errors; Web/PWA and Tauri CSP include `connect-src blob:`
- [ ] Checked-in character `knowledge_refs`, legacy `knowledge`, and `knowledgeRefs` resolve to existing knowledge entries in both project data roots

### AI Integration
- [ ] API mode: exact configured model completes non-streaming and streaming chat through the OpenAI-compatible endpoint
- [ ] Web/PWA mode: character and ensemble tests generate through the packaged Transformers.js WebGPU runtime
- [ ] Windows DirectML generic profile: a compatible full-sequence causal-LM ONNX model initializes and generates through DirectML on device 0; this result must not be reported as Qwen3.5 readiness
- [ ] Platform backend evidence and blockers match `docs/INFERENCE_BACKEND_MATRIX.md`
- [ ] Evaluation triggers fire at correct intervals
- [ ] Relationship milestones unlock events correctly
- [ ] Workflow LLM nodes guard generated output and replace blank or guard-only results with stable failure text before it is used by downstream story nodes
- [ ] Character prompts include creator-declared pinned knowledge references before keyword search results
- [ ] Chat runtime emits author-visible safety trace evidence for input wrapping, prompt-injection detection, guarded responses, memory guards, stream replacements, and relationship side-channel containment
- [ ] Prompt-injection detection covers attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, punctuation-free role headings, English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control attempts before scoring, memory writes, relationship deltas, and hidden prompt boundaries consume player text, and explicit XML/fence/comment role-control block bodies are omitted with their markers across active and legacy prompt builders
- [ ] Local fallback scoring recognizes English, Chinese, Japanese, and Korean friendly, question, and creative-story signals while continuing to ignore prompt-injection text
- [ ] Chat session audit restores the latest safety trace, evaluation, story-event trigger decisions, event-rule fingerprints, and triggerable events after character switching
- [ ] Chat runtime traces prove character mind contract application and creator-pinned knowledge context anchoring, including resolved pinned knowledge ref IDs
- [ ] Chat runtime emits story-event trigger decisions with actual relationship values, score metrics, evaluation counts, stable SHA-256 event-rule fingerprints, and blocker reasons
- [ ] Manual Chat scoring returns an atomic evaluation report with matching story-event trigger decisions and triggerable events for author score-gate debugging
- [ ] Quality Suite story-event reports reuse the same trigger decision contract and event-rule fingerprints as live chat runtime responses
- [ ] Quality Suite injection scenarios include XML, Markdown fence, and comment-wrapped role-control block bodies that must not boost scores, poison memory, or trigger story events
- [ ] Group chat runtime emits author-visible safety trace evidence per character response, reusing the single-character guard contract
- [ ] Group chat per-character generation failures surface as stable system messages, stay out of future prompt transcripts, and do not log raw dialogue text
- [ ] Group chat command boundaries trim participant IDs, reject empty or duplicate participant sets, reject inactive sessions, and refuse blank messages before advancing multi-character scenes
- [ ] Quality Suites panel runs character stability, structured role-block prompt-injection, multilingual and Unicode-obfuscated prompt-injection, group chat runtime trace, relationship and fallback scoring side-channel containment, memory-poisoning containment, memory prompt replay containment, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, knowledge-boundary stability, evaluation-summary safety, workflow output safety, workflow guard-only fallback, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, event-idempotence, and event-rule fingerprint snapshot regression checks
- [ ] Quality suite files reject out-of-range score expectations and contradictory expected/forbidden events, markers, or workflow nodes before release reports run
- [ ] Quality Suites panel lists, shows, and exports versioned audit evidence with run metadata, suite source paths, SHA-256 suite fingerprints, build commit ids, failed-scenario ids, category summaries, safety-signal counts, event-rule fingerprints, runtime safety trace guard notes, finalized guarded workflow output text, and workflow coverage summaries

### Workflow Editor
- [ ] All 21 node types render in palette
- [ ] Drag-and-drop creates nodes on canvas
- [ ] Validation catches missing fields and broken links
- [ ] Run panel emits a trace for checked-in sample workflows and stops at unresolved player choices
- [ ] Checked-in score-gate workflow fixture completes both fallback and unlocked branches from seeded evaluation state
- [ ] Run panel shows evaluation metric, threshold, score source, event trigger state, and blocker reasons for score-gated story branches
- [ ] Canvas nodes show run badges for executed, pass/fail, blocked event, completed, and waiting-choice states
- [ ] Run preview context can simulate character scores, relationship values, evaluation counts, already-triggered events, and workflow state-node effects without mutating chat, script, character, or scene runtime state
- [ ] Run preview context clamps overrange author scores and relationship values before score-gated workflow branches consume them
- [ ] Run preview presets cover unlock, low-score block, and repeat-trigger block branches for score-gated workflows
- [ ] Run report shows graph coverage percentage, executed node count, and unvisited node ids for branch QA
- [ ] Run preset matrix executes all score-gate preview presets and reports merged graph coverage
- [ ] Quality Suite report shows the score-gate workflow coverage snapshot at 100% merged branch coverage
- [ ] Export produces valid JSON

### Audio
- [ ] BGM tracks list and volume controls work
- [ ] Ambient loop tracks play through the music transport and respond to the ambient mixer channel
- [ ] SFX preview plays correctly
- [ ] Master, BGM, ambient, SFX, and voice mixer channels respond to input and persist across reloads

### i18n
- [ ] Locale switching works (en, zh-CN, ja-JP, ko-KR)
- [ ] Nested key resolution works for all locale files
- [ ] Tauri i18n commands load, list, and translate only portable locale IDs under the active project `locales/` directory
- [ ] Browser fallback locale JSON loads correctly under root and `VITE_BASE_PATH` subpath deployments

### Cloud Sync
- [ ] Push updates the project-scoped save manifest without counting `.sync_manifest.json` as a save file
- [ ] Pull reports valid manifest entries without claiming remote file transfer when no remote adapter is configured
- [ ] Sync status in Settings shows backend-provided status, file count, pending uploads/downloads, conflict count, provider mode, and last sync time
- [ ] Remote preflight records endpoint/token readiness without writing sync token values to project files or status payloads

## Distribution
- [ ] Version bumped in tauri.conf.json
- [ ] Version bumped in package.json
- [ ] Version bumped in rust-engine/Cargo.toml
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md version and features updated
- [ ] Tauri bundle config declares installer metadata, Windows MSI/NSIS targets, the pinned `c4c2d20f-f307-5c7b-91e6-5edeea14fdd0` WiX upgrade code, icon assets, WebView2 install mode, and bundled sample `data/` resources
- [ ] Tauri app security declares a production CSP instead of `csp: null`, keeps `script-src 'self'`, blocks `unsafe-eval`, and allows only required local asset, blob/data media, HTTPS, and localhost dev sources
- [ ] Web/PWA app-shell security declares a CSP meta tag in source and generated static fallback output, keeps `script-src 'self'`, blocks `unsafe-eval`, and allows only required app asset, blob/data media, HTTPS, and localhost preview sources
- [ ] Web/PWA static-hosting security headers are generated for response-header capable hosts and release-verified for CSP, MIME sniffing, referrer, and browser permission surfaces
- [ ] Web/PWA static-hosting redirects are generated and release-verified for asset passthrough, local rewrite targets, 200 rewrite status, and final SPA fallback ordering
- [ ] Web/PWA Azure Static Web Apps configuration is generated and release-verified for fallback routing, asset exclusions, 404 handling, and global security headers
- [ ] Web/PWA Vercel configuration is generated and release-verified for SPA fallback routing, local rewrite targets, and global security headers
- [ ] Web/PWA dist includes `events/story_events.json`, inventories event catalogs in `project-assets.json`, and serves/caches them correctly at root and subpath bases
- [ ] Installed Tauri build resolves bundled sample `data/` resources at startup when no development project data root is available
- [ ] `node scripts/verify-windows-installers.mjs --check` passes for public Windows artifacts, proving MSI/NSIS identity, version, stable MSI upgrade code, hashes, size bounds, expected-publisher Authenticode signatures on both installers and the extracted application, exact MSI payload parity, and extracted runtime verification
- [ ] Internal/alpha unsigned candidates use the explicit `--allow-unsigned` audit only; their audit reports `release_ready: false` and is not reused as stable/beta evidence
- [ ] Extracted production executable writes a verified `monogatari-installation-verification/v1` report with current engine/build Git provenance, no bundled DirectML project warnings, 100-file project inventory, runtime content counts, and a valid project content fingerprint
- [ ] Installed Tauri build writes analytics, sync manifests, saves, and generated system/API TTS assets under the active project data root with sanitized output filenames
- [ ] Azure and ElevenLabs TTS provider errors redact token-shaped values, API-key assignments, authorization headers, sensitive provider headers, and response bodies before reaching frontend status surfaces
- [ ] TTS synthesis logs record text length metadata instead of raw spoken dialogue, prompt text, or token-shaped content
- [ ] Project export manifest includes a versioned schema marker, engine/build provenance, content category summaries/fingerprints, explicit whole-package SHA-256 fingerprint algorithm, file inventory, per-file SHA-256 and legacy MD5 checksums, generated assets, and redacted sensitive settings
- [ ] `.monogatari` export contains the root manifest plus the exact sanitized inventoried files, preserves an existing destination on failure, and round-trips checked-in project content through runtime loaders
- [ ] Package inspection/import rejects traversal, non-portable or case-colliding paths, symbolic/special ZIP entries, undeclared files, malformed JSON, runtime secrets, checksum/fingerprint mismatches, and declared file/count/expanded-size bombs
- [ ] Package import stages under the selected parent, validates project/runtime/scene/ending content, commits to a fresh directory without overwriting, and removes rejected staging directories
- [ ] Tauri capabilities grant only `dialog:allow-open` and `dialog:allow-save` for native project package selection; dedicated Rust commands retain filesystem validation
- [ ] Release artifact manifest generated from a clean tracked git worktree with SHA-256 checksums, checked-in Quality Suite, workflow, and project content source evidence plus aggregate and per-category source-set fingerprints, checked-in release channel policy metadata, source-state evidence, installer expectations, and signing evidence
- [ ] `scripts/release-channel-policy.json` confirms stable/beta releases require Windows MSI/NSIS installers and verified signing evidence, with missing-installer preflight exceptions explicitly policy-gated
- [ ] `docs/MOBILE_DEPLOYMENT.md` reviewed before Android/iOS project generation
- [ ] Git tag created: `git tag v0.9.5`
- [ ] Web/PWA preview verified with `npm run preview:web`
- [ ] Subpath web deployment verified with `VITE_BASE_PATH` when publishing to GitHub Pages or another non-root path
- [ ] Windows MSI installer built: `cargo tauri build`
- [ ] macOS DMG installer built (if applicable)
- [ ] Linux AppImage built (if applicable)
- [ ] Code signing applied to installers and recorded with `monogatari-signature-evidence/v1` sidecar evidence before final stable/beta manifest generation
- [ ] GitHub Release created with installers attached
