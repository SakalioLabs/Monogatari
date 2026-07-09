# Monogatari Release Checklist

## Pre-Release Verification

### Automated Gate
- [ ] `node scripts/verify-release.mjs` passes from the repository root, covering JSON assets, workflow files, score-gate workflow execution regressions, renderer asset contracts, pinned knowledge-ref contracts, all quality suites, workflow branch coverage snapshots, locale coverage, sensitive token scans, frontend UI text artifact scans, frontend source invariants, legacy C# AI prompt/API invariants, AI backend config, engine project root, asset/save-manager, script command and state-key invariants, i18n locale, workflow command, content loader, character manager, plugin manager, marketplace, Live2D model, and TTS output/error/log-privacy invariants, frontend route/sidebar coverage, Tauri desktop packaging configuration, Rust core/AI/scripting/game/assets/Tauri checks/tests, frontend audit, root and subpath Web/PWA builds, Web/PWA dist assets, release artifact manifest checks, preview route smoke checks, and legacy C# tests

### Frontend
- [ ] `cd frontend && npm run build` passes with zero errors
- [ ] `cd frontend && npm run build:web` emits manifest, service worker, offline fallback, `404.html`, `.nojekyll`, `_headers`, `_redirects`, `staticwebapp.config.json`, `vercel.json`, and `project-assets.json` assets
- [ ] Web/PWA dist includes copied `data/assets` project sample backgrounds and character sprites under `dist/assets`
- [ ] Web/PWA manifest includes dedicated install and maskable icons, and `sw.js` precaches those icon assets plus generated project sample assets for offline install surfaces
- [ ] `cd frontend && npm run verify:mobile-readiness` passes, proving safe-area viewport metadata, iOS/PWA install metadata, bottom navigation safe-area padding, and compact Tauri shell limits
- [ ] `cd frontend && npm run verify:responsive-shell` passes after `npm run build:web`, proving built 375px mobile and 768px tablet Web/PWA shell layout signals
- [ ] `cd frontend && npm run verify:web-budget` passes with entry JS/CSS and lazy renderer chunks inside budget
- [ ] `node scripts/verify-tauri-mobile-preflight.mjs` passes, proving Android/iOS command readiness, Vite mobile host binding, compact Tauri shell config, and mobile deployment documentation
- [ ] `npm audit` shows zero vulnerabilities
- [ ] Frontend runtime source contains no `console.log` or `console.debug` debug output before Web/PWA or Tauri packaging
- [ ] Frontend runtime source contains no `v-html` or direct `innerHTML`/`outerHTML` assignment before Web/PWA or Tauri packaging
- [ ] Web/PWA `index.html` and `404.html` include a Content Security Policy meta tag that blocks object/frame/form/`unsafe-eval` surfaces while allowing required app assets, blob/data media, HTTPS providers, and localhost preview tooling
- [ ] Web/PWA dist includes a static-hosting `_headers` file with CSP, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and a restrictive browser `Permissions-Policy`
- [ ] Web/PWA dist includes a static-hosting `_redirects` file with project asset passthrough rules and a final `/* /index.html 200` SPA fallback
- [ ] Web/PWA dist includes an Azure Static Web Apps `staticwebapp.config.json` with SPA navigation fallback, static asset exclusions, 404 rewrite, and matching global security headers
- [ ] Web/PWA dist includes a Vercel `vercel.json` with SPA rewrite to `index.html`, no external rewrite destinations, and matching global security headers
- [ ] All 21 views render correctly (Dashboard, Title, Story Mode, AI Chat, Workflow, Character Editor, Scene Assets, Settings, Characters, Group Chat, Analytics, Quality, Marketplace, Plugins, Audio, Knowledge, Dialogue Editor, Scene Editor, CG Gallery, Backlog, Achievements)
- [ ] Sidebar navigation works for all 20 items
- [ ] Responsive layout verified on mobile viewport (375px) and tablet (768px), with the build-time responsive shell verifier attached as release evidence

### Rust Backend
- [ ] `cargo check --locked -p llm-galgame-app` passes
- [ ] All 22 command modules register correctly in main.rs
- [ ] Chat streaming works with API backend
- [ ] Character personality/knowledge injection verified
- [ ] Shared Rust AI prompt builder sanitizes embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings in message history and context sections before OpenAI-compatible role parsing
- [ ] Legacy C# prompt builder sanitizes embedded role-boundary markers, attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, and punctuation-free role headings while the legacy solution remains release-gated
- [ ] Rust API engine debug output and API error surfaces redact API keys, bearer tokens, sensitive custom headers, and provider-echoed secret assignments before logs or frontend error reports expose them
- [ ] Project settings save/load paths scrub API keys, tokens, authorization headers, token-shaped values, query-secret assignments, and legacy persisted secret fields before writing `settings.json` or returning project config state to the frontend
- [ ] ONNX backend configuration accepts only project-relative `.onnx` model and `.json` tokenizer references and activates the ONNX engine after registration
- [ ] Engine initialization binds only existing local project directories as the active project root
- [ ] Legacy C# API engine redacts token-shaped values and JSON/header/query secret assignments from provider error bodies and request exceptions while the legacy solution remains release-gated
- [ ] Rust and legacy C# asset managers reject absolute, URI-like, empty, current-directory, and parent-traversal asset paths before reading project assets
- [ ] Rust and legacy C# save managers reject traversal-shaped save IDs before save/load/delete and filter listed saves whose embedded IDs do not match safe filenames
- [ ] Rhai script commands and direct `ScriptEngine` callers reject oversized or hidden-control-character payloads, condition expressions use shared limits and run through a read-only Rhai engine, workflow conditions receive read-only relationship/evaluation context variables for desktop and Web/PWA previews, desktop run-context previews and browser workflow previews mirror per-run variable, flag, signed relationship, emotion, scene, and weighted random-branch behavior for later branches without mutating persistent runtime state, workflow validation rejects invalid condition and state-key config before execution, script variables/flags use portable state keys before workflow, dialogue, or save data writes, and the shared script engine caps operations, recursion, expression depth, variables, functions, and module imports
- [ ] Workflow save/load commands read and write only JSON files under the active project `workflows/` directory
- [ ] Character, dialogue, and knowledge loader commands read only from the active project `characters/`, `dialogue/`, and `knowledge/` directories
- [ ] Character create/delete commands resolve through the active or discovered default project data root, validate portable character IDs, and touch only direct JSON files under `characters/`
- [ ] Plugin listing, registration, and removal commands resolve through the active or discovered default project data root, validate portable plugin IDs, normalize optional `.rhai` script references under `plugins/`, and touch only direct manifest JSON files under `plugins/`
- [ ] Marketplace import/export commands resolve template references only under the active project `templates/` directory or built-in catalog IDs
- [ ] Live2D model commands load only project-relative `.model3.json`/`.json` model files under the active project data root

### Content
- [ ] Example characters load correctly (Sakura, Luna, Kenji)
- [ ] Example dialogues play through with choices
- [ ] Knowledge base search returns relevant results
- [ ] Scene assets validate without missing file warnings
- [ ] Checked-in character renderer asset fields resolve to supported project-relative files or intentionally fall back to the generated 3D placeholder
- [ ] Core sample characters Sakura, Luna, and Kenji declare checked-in portrait/sprite renderer assets in both data roots
- [ ] Character Editor renderer asset diagnostics flag unsupported extensions, absolute paths, external URLs, and parent traversal before saving/exporting character JSON
- [ ] Character Editor renderer preview follows Story Mode priority for Live2D, GLB/GLTF, sprite/portrait, and generated 3D fallback states
- [ ] Story Mode and Character Editor preview both derive renderer priority from the shared frontend renderer asset selector
- [ ] `npm run verify:renderer-assets` passes, proving shared renderer selector priority, expression sprite resolution, validation skips, and generated fallback behavior
- [ ] Story Mode renderer fallback verified for Live2D, GLB/GLTF, sprite/portrait, and assetless character states
- [ ] Checked-in character `knowledge_refs`, legacy `knowledge`, and `knowledgeRefs` resolve to existing knowledge entries in both project data roots

### AI Integration
- [ ] API mode: streaming chat with OpenAI-compatible endpoint
- [ ] ONNX mode: local model inference (if applicable)
- [ ] Evaluation triggers fire at correct intervals
- [ ] Relationship milestones unlock events correctly
- [ ] Workflow LLM nodes guard generated output before it is used by downstream story nodes
- [ ] Character prompts include creator-declared pinned knowledge references before keyword search results
- [ ] Chat runtime emits author-visible safety trace evidence for input wrapping, prompt-injection detection, guarded responses, memory guards, stream replacements, and relationship side-channel containment
- [ ] Prompt-injection detection covers attributed XML-like role tags, Markdown role-code-fence blocks, comment-wrapped role headers, punctuation-free role headings, English, Chinese, Japanese, Korean, fullwidth, and zero-width-obfuscated prompt-control attempts before scoring, memory writes, relationship deltas, and hidden prompt boundaries consume player text, and explicit XML/fence/comment role-control block bodies are omitted with their markers across active and legacy prompt builders
- [ ] Local fallback scoring recognizes English, Chinese, Japanese, and Korean friendly, question, and creative-story signals while continuing to ignore prompt-injection text
- [ ] Chat session audit restores the latest safety trace, evaluation, story-event trigger decisions, and triggerable events after character switching
- [ ] Chat runtime traces prove character mind contract application and creator-pinned knowledge context anchoring, including resolved pinned knowledge ref IDs
- [ ] Chat runtime emits story-event trigger decisions with actual relationship values, score metrics, evaluation counts, and blocker reasons
- [ ] Manual Chat scoring returns an atomic evaluation report with matching story-event trigger decisions and triggerable events for author score-gate debugging
- [ ] Quality Suite story-event reports reuse the same trigger decision contract as live chat runtime responses
- [ ] Quality Suite injection scenarios include XML, Markdown fence, and comment-wrapped role-control block bodies that must not boost scores, poison memory, or trigger story events
- [ ] Group chat runtime emits author-visible safety trace evidence per character response, reusing the single-character guard contract
- [ ] Quality Suites panel runs character stability, structured role-block prompt-injection, multilingual and Unicode-obfuscated prompt-injection, group chat runtime trace, relationship and fallback scoring side-channel containment, memory-poisoning containment, memory prompt replay containment, tool-role injection containment, identity drift, style drift, real knowledge-reference anchoring, knowledge-boundary stability, evaluation-summary safety, workflow output safety, workflow tool-call containment, workflow branch coverage, private reasoning leakage, fallback scoring, overrange score clamping, event-idempotence, and event-rule snapshot regression checks
- [ ] Quality suite files reject out-of-range score expectations and contradictory expected/forbidden events, markers, or workflow nodes before release reports run
- [ ] Quality Suites panel shows and exports versioned audit evidence with failed-scenario ids, category summaries, safety-signal counts, runtime safety trace guard notes, and workflow coverage summaries

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
- [ ] Tauri bundle config declares installer metadata, Windows MSI/NSIS targets, icon assets, WebView2 install mode, and bundled sample `data/` resources
- [ ] Tauri app security declares a production CSP instead of `csp: null`, keeps `script-src 'self'`, blocks `unsafe-eval`, and allows only required local asset, blob/data media, HTTPS, and localhost dev sources
- [ ] Web/PWA app-shell security declares a CSP meta tag in source and generated static fallback output, keeps `script-src 'self'`, blocks `unsafe-eval`, and allows only required app asset, blob/data media, HTTPS, and localhost preview sources
- [ ] Web/PWA static-hosting security headers are generated for response-header capable hosts and release-verified for CSP, MIME sniffing, referrer, and browser permission surfaces
- [ ] Web/PWA static-hosting redirects are generated and release-verified for asset passthrough, local rewrite targets, 200 rewrite status, and final SPA fallback ordering
- [ ] Web/PWA Azure Static Web Apps configuration is generated and release-verified for fallback routing, asset exclusions, 404 handling, and global security headers
- [ ] Web/PWA Vercel configuration is generated and release-verified for SPA fallback routing, local rewrite targets, and global security headers
- [ ] Installed Tauri build resolves bundled sample `data/` resources at startup when no development project data root is available
- [ ] Installed Tauri build writes analytics, sync manifests, saves, and generated system/API TTS assets under the active project data root with sanitized output filenames
- [ ] Azure and ElevenLabs TTS provider errors redact token-shaped values, API-key assignments, authorization headers, sensitive provider headers, and response bodies before reaching frontend status surfaces
- [ ] TTS synthesis logs record text length metadata instead of raw spoken dialogue, prompt text, or token-shaped content
- [ ] Project export manifest includes a versioned schema marker, file inventory, per-file checksums, generated assets, and redacted sensitive settings
- [ ] Release artifact manifest generated with SHA-256 checksums, checked-in release channel policy metadata, installer expectations, and signing evidence
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
