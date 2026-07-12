import { readFile } from 'node:fs/promises'
import path from 'node:path'

export function createSourceInvariantVerifier({
  root,
  frontendDir,
  rustDir,
  tauriAppDir,
  frontendSourceExtensions,
  requiredWebCspFragments,
  walkFiles,
  relative,
  extractHtmlCsp,
  verifyCspPolicy,
}) {
  async function verifyFrontendSourceInvariants() {
    const issues = []
    const frontendPackageSource = await readFile(path.join(frontendDir, 'package.json'), 'utf8')
    const indexSource = await readFile(path.join(frontendDir, 'index.html'), 'utf8')
    const appSource = await readFile(path.join(frontendDir, 'src', 'App.vue'), 'utf8')
    const mainSource = await readFile(path.join(frontendDir, 'src', 'main.ts'), 'utf8')
    const globalStyleSource = await readFile(path.join(frontendDir, 'src', 'styles', 'main.css'), 'utf8')
    const i18nSource = await readFile(path.join(frontendDir, 'src', 'lib', 'i18n.ts'), 'utf8')
    const pwaSource = await readFile(path.join(frontendDir, 'src', 'lib', 'pwa.ts'), 'utf8')
    const rendererAssetsSource = await readFile(path.join(frontendDir, 'src', 'lib', 'rendererAssets.ts'), 'utf8')
    const storyEventsSource = await readFile(path.join(frontendDir, 'src', 'lib', 'storyEvents.ts'), 'utf8')
    const storyProgressSource = await readFile(path.join(frontendDir, 'src', 'lib', 'storyProgress.ts'), 'utf8')
    const storyAccessSource = await readFile(path.join(frontendDir, 'src', 'lib', 'storyAccess.ts'), 'utf8')
    const storyContentSource = await readFile(path.join(frontendDir, 'src', 'lib', 'storyContent.ts'), 'utf8')
    const storyPlaytestSource = await readFile(path.join(frontendDir, 'src', 'lib', 'storyPlaytest.ts'), 'utf8')
    const localConditionSource = await readFile(path.join(frontendDir, 'src', 'lib', 'localCondition.ts'), 'utf8')
    const knowledgeContentSource = await readFile(path.join(frontendDir, 'src', 'lib', 'knowledgeContent.ts'), 'utf8')
    const storyEndingsSource = await readFile(path.join(frontendDir, 'src', 'lib', 'storyEndings.ts'), 'utf8')
    const sceneAuthoringSource = await readFile(path.join(frontendDir, 'src', 'lib', 'sceneAuthoring.ts'), 'utf8')
    const dialogueAuthoringSource = await readFile(path.join(frontendDir, 'src', 'lib', 'dialogueAuthoring.ts'), 'utf8')
    const live2dCanvasSource = await readFile(path.join(frontendDir, 'src', 'components', 'Live2DCanvas.vue'), 'utf8')
    const characterModelSource = await readFile(path.join(frontendDir, 'src', 'components', 'CharacterModelView.vue'), 'utf8')
    const offlineSource = await readFile(path.join(frontendDir, 'public', 'offline.html'), 'utf8')
    const offlineI18nSource = await readFile(path.join(frontendDir, 'public', 'offline-i18n.js'), 'utf8')
    const prepareWebDistSource = await readFile(path.join(frontendDir, 'scripts', 'prepare-web-dist.mjs'), 'utf8')
    const mobileReadinessSource = await readFile(path.join(frontendDir, 'scripts', 'verify-mobile-readiness.mjs'), 'utf8')
    const responsiveShellSource = await readFile(path.join(frontendDir, 'scripts', 'verify-responsive-shell.mjs'), 'utf8')
    const syncLocalesSource = await readFile(path.join(frontendDir, 'scripts', 'sync-locales.mjs'), 'utf8')
    const verifyI18nSource = await readFile(path.join(frontendDir, 'scripts', 'verify-i18n-coverage.mjs'), 'utf8')
    const gameViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'GameView.vue'), 'utf8')
    const chatViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'ChatView.vue'), 'utf8')
    const groupChatViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'GroupChatView.vue'), 'utf8')
    const characterEditorSource = await readFile(path.join(frontendDir, 'src', 'views', 'CharacterEditorView.vue'), 'utf8')
    const analyticsViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'AnalyticsView.vue'), 'utf8')
    const workflowEditorSource = await readFile(path.join(frontendDir, 'src', 'views', 'WorkflowEditor.vue'), 'utf8')
    const workflowPreviewSource = await readFile(path.join(frontendDir, 'src', 'lib', 'workflowPreview.ts'), 'utf8')
    const workflowContractSource = `${workflowEditorSource}\n${workflowPreviewSource}`
    const storyEventEditorSource = await readFile(path.join(frontendDir, 'src', 'views', 'StoryEventEditorView.vue'), 'utf8')
    const endingEditorSource = await readFile(path.join(frontendDir, 'src', 'views', 'EndingEditorView.vue'), 'utf8')
    const sceneEditorSource = await readFile(path.join(frontendDir, 'src', 'views', 'SceneEditorView.vue'), 'utf8')
    const dialogueEditorSource = await readFile(path.join(frontendDir, 'src', 'views', 'DialogueEditorView.vue'), 'utf8')
    const qualitySuiteSource = await readFile(path.join(frontendDir, 'src', 'views', 'QualitySuiteView.vue'), 'utf8')
    const audioViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'AudioView.vue'), 'utf8')
    const knowledgeBaseViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'KnowledgeBaseView.vue'), 'utf8')
    const settingsSource = await readFile(path.join(frontendDir, 'src', 'views', 'SettingsView.vue'), 'utf8')
    const projectArchiveSource = await readFile(path.join(frontendDir, 'src', 'lib', 'projectArchive.ts'), 'utf8')
    const serviceWorkerSource = await readFile(path.join(frontendDir, 'public', 'sw.js'), 'utf8')
    const frontendRuntimeFiles = (await walkFiles(path.join(frontendDir, 'src'))).filter((file) =>
      frontendSourceExtensions.has(path.extname(file)),
    )

    for (const file of frontendRuntimeFiles) {
      const content = await readFile(file, 'utf8')
      content.split(/\r?\n/).forEach((line, index) => {
        if (/console\.(log|debug)\s*\(/.test(line)) {
          issues.push(`${relative(file)}:${index + 1}: frontend runtime code must not ship console.log/debug output`)
        }
        if (/v-html\s*=/.test(line)) {
          issues.push(`${relative(file)}:${index + 1}: frontend runtime code must not use v-html HTML injection`)
        }
        if (/\b(?:innerHTML|outerHTML)\s*=/.test(line)) {
          issues.push(`${relative(file)}:${index + 1}: frontend runtime code must not assign raw HTML strings`)
        }
      })
    }

    const sourceWebCsp = extractHtmlCsp(indexSource)
    if (!sourceWebCsp) {
      issues.push('frontend/index.html must declare a Web/PWA Content Security Policy meta tag')
    } else {
      verifyCspPolicy(sourceWebCsp, requiredWebCspFragments, 'frontend/index.html Web/PWA CSP', issues, {
        forbiddenFragments: ["frame-ancestors 'none'"],
      })
    }

    if (!i18nSource.includes('import.meta.env.BASE_URL')) {
      issues.push('frontend/src/lib/i18n.ts must use import.meta.env.BASE_URL for browser locale fallbacks')
    }
    if (i18nSource.includes('fetch("/locales/') || i18nSource.includes("fetch('/locales/")) {
      issues.push('frontend/src/lib/i18n.ts must not fetch browser locale fallbacks from absolute /locales/ paths')
    }

    const frontendI18nRequirements = [
      [frontendPackageSource, 'verify:i18n', 'expose the i18n coverage verifier'],
      [frontendPackageSource, 'sync:locales', 'expose deterministic locale synchronization'],
      [frontendPackageSource, 'npm run verify:i18n && vue-tsc', 'run i18n verification before frontend compilation'],
      [mainSource, 'await loadI18n()', 'load the selected locale before mounting the application'],
      [i18nSource, "code: 'zh-CN'", 'expose Simplified Chinese in the locale selector'],
      [i18nSource, "code: 'ja-JP'", 'expose Japanese in the locale selector'],
      [i18nSource, "code: 'ko-KR'", 'expose Korean in the locale selector'],
      [i18nSource, 'Promise.all([', 'load English fallback and target catalogs together'],
      [i18nSource, 'requestId !== loadSequence', 'ignore stale asynchronous locale responses'],
      [i18nSource, 'document.documentElement.lang = locale', 'synchronize the document language'],
      [syncLocalesSource, "const writeMode = process.argv.includes('--write')", 'support explicit locale copy synchronization'],
      [syncLocalesSource, 'embedded catalog', 'verify embedded locale copies'],
      [verifyI18nSource, 'interpolation tokens differ from en', 'verify translated interpolation tokens'],
      [verifyI18nSource, 'contains replacement characters or encoding damage', 'reject damaged Unicode catalogs'],
      [verifyI18nSource, 'is referenced but missing from catalogs', 'verify translation keys referenced by source'],
      [verifyI18nSource, 'strictLocalizedSurfaces', 'scan strict UI surfaces for untranslated visible text'],
    ]
    for (const [source, needle, description] of frontendI18nRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Frontend i18n readiness must ${description}`)
      }
    }

    if (!pwaSource.includes('import.meta.env.BASE_URL')) {
      issues.push('frontend/src/lib/pwa.ts must use import.meta.env.BASE_URL for service worker scope')
    }
    if (!pwaSource.includes("new URL('sw.js', scopeUrl)") && !pwaSource.includes('new URL("sw.js", scopeUrl)')) {
      issues.push('frontend/src/lib/pwa.ts must register sw.js relative to the resolved service worker scope')
    }
    if (!pwaSource.includes('hasTauriRuntime()')) {
      issues.push('frontend/src/lib/pwa.ts must keep service worker registration disabled inside Tauri')
    }

    const mobileShellRequirements = [
      [frontendPackageSource, 'verify:mobile-readiness', 'expose a mobile readiness verifier npm script'],
      [indexSource, 'viewport-fit=cover', 'enable safe-area viewport layout for mobile shells'],
      [indexSource, 'apple-mobile-web-app-capable', 'include iOS standalone PWA metadata'],
      [indexSource, 'apple-touch-icon', 'include an Apple touch icon'],
      [globalStyleSource, '100svh', 'use small viewport height units for mobile WebViews'],
      [appSource, 'env(safe-area-inset-bottom', 'protect bottom UI from mobile safe areas'],
      [globalStyleSource, 'touch-action: manipulation', 'use mobile-friendly touch handling'],
      [mobileReadinessSource, 'viewport-fit=cover', 'verify safe-area viewport metadata'],
      [mobileReadinessSource, 'manifest.webmanifest display must be standalone', 'verify standalone PWA display mode'],
      [mobileReadinessSource, 'minWidth must be <= 390', 'verify compact Tauri shell width limits'],
      [mobileReadinessSource, 'minHeight must be <= 640', 'verify compact Tauri shell height limits'],
    ]
    for (const [source, needle, description] of mobileShellRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Mobile shell readiness must ${description}`)
      }
    }

    const responsiveShellRequirements = [
      [frontendPackageSource, 'verify:responsive-shell', 'expose a responsive shell verifier npm script'],
      [frontendPackageSource, 'verify-responsive-shell.mjs', 'run responsive shell verification from production builds'],
      [responsiveShellSource, '375', 'verify the 375px mobile shell profile'],
      [responsiveShellSource, '768', 'verify the 768px tablet shell profile'],
      [responsiveShellSource, 'dist/index.html', 'verify built root HTML shell metadata'],
      [responsiveShellSource, 'dist/404.html', 'verify built static-hosting fallback shell metadata'],
      [responsiveShellSource, '@media (width<=860px)', 'verify built compact-shell CSS media output'],
      [responsiveShellSource, '@media (max-width: 860px)', 'verify the compact App shell breakpoint'],
      [responsiveShellSource, 'min-height: 100svh', 'verify small viewport height shell rules'],
      [responsiveShellSource, 'grid-template-columns: var(--sidebar-width) minmax(0, 1fr)', 'verify stable desktop sidebar and shrinkable workspace tracks'],
      [responsiveShellSource, 'grid-template-columns: repeat(5, minmax(0, 1fr))', 'verify stable mobile navigation tracks'],
    ]
    for (const [source, needle, description] of responsiveShellRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Responsive shell readiness must ${description}`)
      }
    }

    const serviceWorkerRequirements = [
      ['__APP_VERSION__', 'reserve a production package version placeholder for build-time cache identity'],
      ['__BUILD_FINGERPRINT__', 'reserve a production content fingerprint placeholder for build-time cache identity'],
      ['self.registration.scope', 'derive the service worker base path from registration scope'],
      ['const BASE_PATH', 'declare BASE_PATH for subpath deployments'],
      ['APP_SHELL_PATHS.map(withBase)', 'apply withBase to app shell paths'],
      ['/icons/app-icon.svg', 'precache the dedicated PWA app icon'],
      ['/icons/maskable-icon.svg', 'precache the dedicated maskable PWA icon'],
      ['/offline-i18n.js', 'precache the CSP-compatible offline localization script'],
      ['PROJECT_ASSET_MANIFEST_PATH', 'declare the generated project asset manifest path'],
      ['/project-assets.json', 'precache the generated project asset manifest'],
      ['INFERENCE_RUNTIME_PATH', 'declare the packaged inference runtime path'],
      ['/inference-runtime.json', 'precache the packaged inference runtime contract'],
      ['cacheProjectAssets()', 'cache generated project assets during service worker install'],
      ['manifest.event_catalogs', 'cache project story event catalogs during service worker install'],
      ['manifest.scene_files', 'cache project scene catalogs during service worker install'],
      ['manifest.dialogue_files', 'cache project dialogue scripts during service worker install'],
      ['manifest.ending_files', 'cache project ending catalogs during service worker install'],
      ['manifest.character_files', 'cache project character definitions during service worker install'],
      ['manifest.knowledge_files', 'cache project knowledge entries during service worker install'],
      ['path.startsWith("/events/")', 'serve project story events through an offline-aware strategy'],
      ['path.startsWith("/characters/")', 'serve project character definitions through an offline-aware strategy'],
      ['path.startsWith("/knowledge/")', 'serve project knowledge entries through an offline-aware strategy'],
      ['monogatari-web-project-assets/v1', 'validate the project asset manifest schema before caching'],
      ['function withBase', 'define withBase helper'],
      ['function routePath', 'define routePath helper'],
      ['routePath(url.pathname)', 'normalize incoming requests through routePath'],
      ['caches.match(withBase("/index.html"))', 'use base-aware SPA fallback cache lookup'],
      ['caches.match(withBase("/offline.html"))', 'use base-aware offline fallback cache lookup'],
    ]
    for (const [needle, description] of serviceWorkerRequirements) {
      if (!serviceWorkerSource.includes(needle)) {
        issues.push(`frontend/public/sw.js must ${description}`)
      }
    }

    const webDistPackagingRequirements = [
      ["from 'node:crypto'", 'use a cryptographic digest for deterministic PWA cache identity'],
      ['injectServiceWorkerBuildId()', 'inject a content-derived service worker cache identity after packaging'],
      ['distServiceWorkerPath', 'target the built service worker without mutating the source template'],
      ["'offline-i18n.js'", 'package the CSP-compatible offline localization script'],
      ["'data', 'assets'", 'copy checked-in project assets from data/assets'],
      ["'data', 'events'", 'copy checked-in story event catalogs from data/events'],
      ["'data', 'scenes'", 'copy checked-in scene catalogs from data/scenes'],
      ["'data', 'dialogue'", 'copy checked-in dialogue scripts from data/dialogue'],
      ["'data', 'endings'", 'copy checked-in ending catalogs from data/endings'],
      ["'data', 'characters'", 'copy checked-in character definitions from data/characters'],
      ["'data', 'knowledge'", 'copy checked-in knowledge entries from data/knowledge'],
      ["'data', 'models', 'webgpu'", 'support optional packaged WebGPU model artifacts'],
      ['distProjectAssetsDir', 'target copied project assets into dist/assets'],
      ['projectAssetManifestPath', 'write a generated project asset manifest into dist'],
      ['inferenceRuntimePath', 'write a generated WebGPU inference contract into dist'],
      ['webInferenceRuntime()', 'derive the WebGPU contract from project settings'],
      ['staticHostingHeadersPath', 'write static-hosting security headers into dist'],
      ['staticHostingRedirectsPath', 'write static-hosting SPA redirect rules into dist'],
      ['azureStaticWebAppConfigPath', 'write Azure Static Web Apps configuration into dist'],
      ['vercelConfigPath', 'write Vercel deployment configuration into dist'],
      ['navigationFallback', 'emit Azure Static Web Apps SPA navigation fallback config'],
      ['globalHeaders', 'emit Azure Static Web Apps global security headers'],
      ['rewrites', 'emit Vercel SPA rewrite config'],
      ['securityHeaderEntries', 'reuse security headers for Vercel responses'],
      ['Content-Security-Policy', 'emit a static-hosting CSP header for platforms that support response headers'],
      ['X-Content-Type-Options: nosniff', 'emit a nosniff header for static-hosting responses'],
      ['Permissions-Policy', 'emit a browser permissions policy for static-hosting responses'],
      ['/* /index.html 200', 'emit a static-hosting SPA fallback rewrite'],
      ['monogatari-web-project-assets/v1', 'version the generated project asset manifest schema'],
      ['monogatari-inference-runtime/v1', 'version the generated inference runtime schema'],
      ['walkFiles(projectAssetsDir', 'inventory copied project assets for offline PWA caching'],
      ['cp(projectAssetsDir, distProjectAssetsDir', 'merge project assets into the Web/PWA dist asset tree'],
      ['cp(projectEventsDir, distProjectEventsDir', 'merge story event catalogs into the Web/PWA dist tree'],
      ['cp(projectScenesDir, distProjectScenesDir', 'merge scene catalogs into the Web/PWA dist tree'],
      ['cp(projectDialoguesDir, distProjectDialoguesDir', 'merge dialogue scripts into the Web/PWA dist tree'],
      ['cp(projectEndingsDir, distProjectEndingsDir', 'merge ending catalogs into the Web/PWA dist tree'],
      ['cp(projectCharactersDir, distProjectCharactersDir', 'merge character definitions into the Web/PWA dist tree'],
      ['cp(projectKnowledgeDir, distProjectKnowledgeDir', 'merge knowledge entries into the Web/PWA dist tree'],
      ['cp(projectWebModelDir, distWebModelDir', 'copy optional WebGPU model artifacts into the Web/PWA package'],
      ['event_catalogs', 'inventory story event catalogs in the Web/PWA project manifest'],
      ['WebGPU inference contract', 'report the generated inference runtime in the Web/PWA preparation output'],
    ]
    for (const [needle, description] of webDistPackagingRequirements) {
      if (!prepareWebDistSource.includes(needle)) {
        issues.push(`frontend/scripts/prepare-web-dist.mjs must ${description}`)
      }
    }

    const storyEventFrontendRequirements = [
      [storyEventsSource, "../../../data/events/story_events.json", 'derive browser fallback rules from the checked-in project catalog'],
      [storyEventsSource, 'monogatari-story-event-catalog/v1', 'validate the browser story event catalog schema'],
      [storyEventsSource, "invokeCommand<StoryEventCatalogSnapshot>('get_story_event_catalog')", 'load the active desktop story event catalog'],
      [storyEventsSource, "new URL('events/story_events.json'", 'load deployed Web/PWA story events relative to the configured base path'],
      [storyEventsSource, 'normalizeActions', 'normalize typed and legacy story event actions in browser builds'],
      [storyEventsSource, "type: 'set_flag'", 'type supported story event actions in browser builds'],
      [storyProgressSource, "invokeCommand<StoryProgressSnapshot>('get_story_progress')", 'query persistent story progress in desktop builds'],
      [storyAccessSource, "invokeCommand<StoryContentAccessSnapshot>('get_story_content_access')", 'query event-derived content access in desktop builds'],
      [storyAccessSource, 'deriveStoryContentAccess', 'derive matching content access decisions for browser builds'],
      [storyContentSource, "invokeCommand<StorySceneInfo[]>('list_story_scenes')", 'load gated scenes from desktop projects'],
      [storyContentSource, "invokeCommand<StoryDialogueInfo[]>('list_dialogues')", 'load gated dialogues from desktop projects'],
      [storyContentSource, "invokeCommand<StoryEndingInfo[]>('list_story_endings')", 'load gated endings from desktop projects'],
      [storyContentSource, 'dialogue_files', 'load packaged dialogue scripts from Web/PWA project manifests'],
      [storyContentSource, 'character_files', 'load packaged character definitions from Web/PWA project manifests'],
      [storyContentSource, 'loadBrowserSceneDrafts()', 'load browser-authored scene drafts into Story Mode'],
      [storyContentSource, 'loadBrowserDialogueDrafts()', 'load browser-authored dialogue drafts into Story Mode'],
      [storyContentSource, 'loadBrowserStoryEndingDrafts()', 'load browser-authored ending drafts into Story Mode'],
      [storyContentSource, 'BROWSER_CHARACTER_DRAFT_KEY', 'version browser-authored character catalogs'],
      [storyContentSource, 'loadBrowserCharacterDrafts()', 'load browser-authored character drafts into Story Mode'],
      [storyContentSource, 'saveBrowserCharacterDrafts', 'persist browser-authored character catalogs'],
      [storyContentSource, 'resetBrowserCharacterDrafts', 'restore packaged project characters after browser authoring'],
      [storyContentSource, 'documents.flatMap', 'flatten packaged single and grouped character documents'],
      [characterEditorSource, 'loadKnowledgeAuthoringCatalog', 'bind character knowledge references to the project catalog'],
      [characterEditorSource, 'saveBrowserCharacterDrafts', 'wire Web/PWA character saves to browser authoring drafts'],
      [characterEditorSource, 'pendingAction', 'keep character discard and restore confirmation inside the application'],
      [characterEditorSource, 'isDirty', 'guard dirty character drafts during navigation'],
      [sceneAuthoringSource, "invokeCommand<SceneAuthoringCatalogSnapshot>('get_scene_authoring_catalog')", 'load editable scene catalog snapshots'],
      [sceneAuthoringSource, 'expectedCatalogFingerprint', 'save and delete scenes with optimistic concurrency'],
      [sceneAuthoringSource, 'saveBrowserSceneDrafts', 'persist browser scene authoring drafts'],
      [sceneEditorSource, 'validateSceneDefinition', 'validate scene definitions before save'],
      [sceneEditorSource, 'confirmDiscard', 'guard dirty scene drafts during navigation'],
      [sceneEditorSource, 'resolveAssetUrl', 'preview real scene background assets'],
      [sceneEditorSource, "invokeCommand('set_scene'", 'launch desktop scene author previews'],
      [dialogueAuthoringSource, "invokeCommand<DialogueAuthoringCatalogSnapshot>('get_dialogue_authoring_catalog')", 'load editable dialogue catalog snapshots'],
      [dialogueAuthoringSource, 'expectedCatalogFingerprint', 'save and delete dialogues with optimistic concurrency'],
      [dialogueAuthoringSource, 'saveBrowserDialogueDrafts', 'persist browser dialogue authoring drafts'],
      [dialogueAuthoringSource, 'validateDialogueDefinition', 'validate complete dialogue graphs before save'],
      [dialogueEditorSource, 'renameNode', 'rename nodes while preserving graph references'],
      [dialogueEditorSource, 'relationship_changes', 'edit per-character choice relationship effects'],
      [dialogueEditorSource, 'confirmDiscard', 'guard dirty dialogue drafts during navigation'],
      [dialogueEditorSource, "invokeCommand('preview_dialogue'", 'launch desktop dialogue author previews'],
      [storyEndingsSource, "invokeCommand<StoryEndingCatalogSnapshot>('get_story_ending_catalog')", 'load editable ending catalog snapshots'],
      [storyEndingsSource, 'expectedCatalogFingerprint', 'save and delete endings with optimistic concurrency'],
      [storyEndingsSource, 'saveBrowserStoryEndingDrafts', 'persist browser ending authoring drafts'],
      [endingEditorSource, 'validateStoryEndingDefinition', 'validate ending definitions before save'],
      [endingEditorSource, 'loadStoryScenes()', 'bind endings to real project scenes'],
      [endingEditorSource, 'loadStoryDialogues()', 'bind endings to real project dialogues'],
      [endingEditorSource, 'confirmDiscard', 'guard dirty ending drafts during navigation'],
      [endingEditorSource, "invokeCommand('preview_story_ending'", 'launch author previews without player unlock mutation'],
      [storyEventsSource, 'expectedCatalogFingerprint', 'save event catalogs with optimistic concurrency'],
      [storyEventEditorSource, 'validateDocument()', 'validate edited event catalogs before save'],
      [storyEventEditorSource, 'changeActionType', 'edit typed event effects'],
      [gameViewSource, 'loadStoryScenes()', 'populate Story Mode from the project scene catalog'],
      [gameViewSource, 'start_story_ending', 'launch gated ending assets from Story Mode'],
      [gameViewSource, 'route.query.previewEnding', 'launch browser ending author previews from saved drafts'],
      [gameViewSource, 'route.query.previewScene', 'launch browser scene author previews from saved drafts'],
      [gameViewSource, 'route.query.previewDialogue', 'launch browser dialogue author previews from saved drafts'],
      [gameViewSource, 'webDialogueRuntime', 'retain browser dialogue cursor, variables, and flags across transitions'],
      [gameViewSource, "from '../lib/storyPlaytest'", 'delegate browser dialogue traversal to the Story Playtest domain module'],
      [workflowPreviewSource, "from './localCondition'", 'share local condition evaluation instead of owning a workflow-only parser'],
      [storyPlaytestSource, "from './localCondition'", 'reuse the shared bounded browser condition evaluator'],
      [localConditionSource, 'evaluateLocalCondition', 'expose one pure browser condition evaluation boundary'],
      [localConditionSource, 'getVariable', 'read local preview variables from shared browser conditions'],
      [localConditionSource, 'hasFlag', 'read local preview flags from shared browser conditions'],
      [storyPlaytestSource, 'StoryPlaytestError', 'surface stable browser dialogue graph error codes'],
      [storyPlaytestSource, 'choice_target_missing', 'reject choices that target missing browser dialogue nodes'],
      [storyPlaytestSource, 'relationship_target_missing', 'reject unknown browser relationship targets before mutation'],
      [storyPlaytestSource, 'next_target_missing', 'reject linear transitions that target missing browser dialogue nodes'],
      [storyPlaytestSource, 'relationship_changes', 'return browser choice relationship effects to Story Mode'],
      [storyPlaytestSource, 'choice_unavailable', 'reject submissions for condition-hidden choices'],
      [storyPlaytestSource, 'node_condition_blocked', 'reject false conditional nodes without a fallback'],
      [storyPlaytestSource, 'condition_unsupported', 'refuse unsupported browser conditions instead of misrouting'],
      [storyPlaytestSource, 'script_unsupported', 'refuse unsupported browser dialogue scripts instead of ignoring state'],
      [chatViewSource, "listen<StoryEventApplication[]>('chat-event-applications'", 'surface applied event effects from streaming chat'],
      [chatViewSource, 'loadStoryProgress()', 'refresh persistent unlock counts in the chat workbench'],
      [workflowEditorSource, 'loadStoryEventCatalog()', 'load project story events into the workflow editor'],
      [workflowEditorSource, 'updateStoryEvent', 'bind workflow event selection to catalog metadata'],
      [workflowEditorSource, 'v-for="event in storyEvents"', 'render catalog-backed story event options'],
      [workflowEditorSource, "from '../lib/workflowPreview'", 'delegate browser workflow validation and execution to the domain module'],
      [workflowContractSource, 'node_event_unknown', 'reject unknown story event references in browser validation'],
      [workflowContractSource, 'rule?.character_ids?.length', 'honor character-scoped story events in browser previews'],
      [workflowContractSource, '!rule?.repeatable', 'honor repeatable story events in browser previews'],
      [workflowContractSource, 'actions: definition?.actions || []', 'preview typed story event actions without applying side effects'],
      [qualitySuiteSource, 'loadStoryEventCatalog()', 'load project story events into the Web/PWA quality report preview'],
      [qualitySuiteSource, 'previewQualityReport(eventCatalog.events.map((event) => event.rule))', 'derive preview event rule evidence from the shared catalog'],
    ]
    for (const [source, needle, description] of storyEventFrontendRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Story event frontend integration must ${description}`)
      }
    }

    const knowledgeAuthoringFrontendRequirements = [
      [knowledgeContentSource, 'knowledge_files', 'load packaged knowledge documents from the Web/PWA project manifest'],
      [knowledgeContentSource, "invokeCommand<KnowledgeCatalogSnapshot>('get_knowledge_authoring_catalog')", 'load editable desktop knowledge catalog snapshots'],
      [knowledgeContentSource, "invokeCommand<KnowledgeCatalogSnapshot>('save_knowledge_entry_definition'", 'save desktop knowledge entries through the authoring command'],
      [knowledgeContentSource, "invokeCommand<KnowledgeCatalogSnapshot>('delete_knowledge_entry_definition'", 'delete desktop knowledge entries through the authoring command'],
      [knowledgeContentSource, 'expectedCatalogFingerprint', 'save and delete knowledge entries with optimistic concurrency'],
      [knowledgeContentSource, 'window.localStorage.setItem(browserDraftKey', 'persist Web/PWA knowledge authoring drafts'],
      [knowledgeContentSource, 'resetBrowserKnowledgeDrafts', 'restore packaged project knowledge after browser authoring'],
      [knowledgeContentSource, 'loadBrowserCharacterKnowledgeReferences', 'protect character-pinned knowledge from browser draft deletion'],
      [knowledgeContentSource, 'validateKnowledgeRelations', 'validate related knowledge ids in browser authoring'],
      [knowledgeBaseViewSource, 'saveKnowledgeEntryDefinition(entry', 'wire the knowledge editor save path to real catalog persistence'],
      [knowledgeBaseViewSource, 'deleteKnowledgeEntryDefinition(pending.entry.id', 'wire the knowledge editor delete path to real catalog persistence'],
      [knowledgeBaseViewSource, 'pendingConfirmation', 'keep destructive knowledge actions inside the application confirmation flow'],
    ]
    for (const [source, needle, description] of knowledgeAuthoringFrontendRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Knowledge authoring frontend integration must ${description}`)
      }
    }
    if (knowledgeBaseViewSource.includes('window.confirm')) {
      issues.push('frontend/src/views/KnowledgeBaseView.vue must not block author previews with native browser confirmation dialogs')
    }
    if (characterEditorSource.includes('window.confirm')) {
      issues.push('frontend/src/views/CharacterEditorView.vue must not block author workflows with native browser confirmation dialogs')
    }

    const analyticsFrontendRequirements = [
      [analyticsViewSource, 'hasTauriRuntime()', 'distinguish project analytics from the Web/PWA sample dataset'],
      [analyticsViewSource, "dataSource.value = 'project'", 'identify analytics loaded from the active desktop project'],
      [analyticsViewSource, "dataSource.value = 'sample'", 'identify sample analytics in browser previews'],
      [analyticsViewSource, "dataSource.value = 'unavailable'", 'surface unavailable project analytics without substituting sample metrics'],
      [analyticsViewSource, "source: 'sample'", 'label exported Web/PWA analytics as sample data'],
      [analyticsViewSource, 'summary: summary.value', 'export the analytics summary currently visible in Web/PWA previews'],
      [offlineSource, 'src="./offline-i18n.js"', 'load offline localization from a CSP-compatible same-origin script'],
      [offlineI18nSource, "localStorage.getItem('monogatari-locale')", 'reuse the selected app locale on the offline fallback page'],
      [offlineI18nSource, "'zh-CN':", 'provide Chinese offline fallback copy'],
      [offlineI18nSource, "'ja-JP':", 'provide Japanese offline fallback copy'],
      [offlineI18nSource, "'ko-KR':", 'provide Korean offline fallback copy'],
    ]
    for (const [source, needle, description] of analyticsFrontendRequirements) {
      if (!source.includes(needle)) issues.push(`Analytics and offline frontend integration must ${description}`)
    }
    if (analyticsViewSource.includes("get_analytics_summary', {}, previewSummary")) {
      issues.push('frontend/src/views/AnalyticsView.vue must not silently replace failed desktop analytics with sample data')
    }
    if (/<script(?![^>]*\bsrc=)[^>]*>/i.test(offlineSource)) {
      issues.push('frontend/public/offline.html must not use inline scripts blocked by the static-hosting CSP')
    }
    if (workflowContractSource.includes('const rules: Record<string, Record<string, any>>')) {
      issues.push('Frontend workflow preview surfaces must not keep a second hardcoded story event rule catalog')
    }
    if (qualitySuiteSource.includes("{ event_id: 'close_friend', event_type: 'relationship_milestone'")) {
      issues.push('frontend/src/views/QualitySuiteView.vue must not keep a second hardcoded story event rule catalog')
    }

    const rendererAssetRequirements = [
      ['selectCharacterRendererAsset', 'export the shared character renderer asset selector'],
      ['rendererAssetValidationMessage', 'export the renderer asset validation helper'],
      ["mode: 'placeholder'", 'include an explicit generated 3D placeholder selection'],
      ['live2d_model_path', 'rank Live2D fields in the renderer selector'],
      ['model_3d_path', 'rank GLB/GLTF fields in the renderer selector'],
      ['sprite_path', 'rank sprite fallback fields in the renderer selector'],
      ['portrait_path', 'rank portrait fallback fields in the renderer selector'],
      ['blockedPaths', 'skip runtime-failed renderer asset paths before choosing the next fallback'],
      ['rendererBlockedPathSet', 'normalize runtime-failed renderer paths for fallback selection'],
    ]
    for (const [needle, description] of rendererAssetRequirements) {
      if (!rendererAssetsSource.includes(needle)) {
        issues.push(`frontend/src/lib/rendererAssets.ts must ${description}`)
      }
    }

    if (!gameViewSource.includes("from '../lib/rendererAssets'")) {
      issues.push('frontend/src/views/GameView.vue must use the shared renderer asset selector')
    }
    if (!gameViewSource.includes('selectCharacterRendererAsset(currentCharacter.value')) {
      issues.push('frontend/src/views/GameView.vue must derive Story Mode renderer priority through selectCharacterRendererAsset')
    }
    if (!gameViewSource.includes('markRendererAssetFailed') || !gameViewSource.includes('blockedPaths: Object.keys(failedRendererAssets.value)')) {
      issues.push('frontend/src/views/GameView.vue must fall back to the next renderer asset after runtime load failures')
    }
    if (!characterEditorSource.includes("from '../lib/rendererAssets'")) {
      issues.push('frontend/src/views/CharacterEditorView.vue must use shared renderer asset helpers')
    }
    if (!characterEditorSource.includes('selectCharacterRendererAsset(') || !characterEditorSource.includes('validatePaths: true')) {
      issues.push('frontend/src/views/CharacterEditorView.vue must derive preview renderer priority through selectCharacterRendererAsset with validation')
    }
    if (!characterEditorSource.includes('markPreviewRendererAssetFailed') || !characterEditorSource.includes('blockedPaths: Object.keys(previewFailedRendererAssets.value)')) {
      issues.push('frontend/src/views/CharacterEditorView.vue must preview fallback renderer assets after runtime load failures')
    }
    if (!live2dCanvasSource.includes("defineEmits") || !live2dCanvasSource.includes("'load-error'") || !live2dCanvasSource.includes('loadError')) {
      issues.push('frontend/src/components/Live2DCanvas.vue must emit load-error and surface Live2D runtime load failures')
    }
    if (!characterModelSource.includes("defineEmits") || !characterModelSource.includes("'load-error'") || !characterModelSource.includes('Could not load 3D model')) {
      issues.push('frontend/src/components/CharacterModelView.vue must emit load-error for runtime GLB/GLTF load failures')
    }
    for (const [needle, description] of [
      ['data-model-state', 'expose deterministic 3D load state for visual probes'],
      ['data-model-animations', 'expose loaded animation clip count for visual probes'],
      ['data-canvas-signature', 'expose a bounded WebGL pixel signature for visual probes'],
      ['data-canvas-motion', 'prove animated model frames change rendered pixels'],
      ['gl.readPixels', 'sample the rendered WebGL framebuffer inside the renderer boundary'],
      ['__MONOGATARI_3D_PROBE__', 'publish an opt-in canvas snapshot for Playwright visual verification'],
      ['data-canvas-preview', 'expose the opt-in canvas snapshot through the read-only DOM probe'],
      ['rendererProbeEnabled()', 'keep WebGL framebuffer readback disabled outside explicit visual probes'],
      ['loadRequestSequence', 'prevent stale GLB requests from replacing the active character model'],
      ['normalizeAndFrameModel', 'normalize arbitrary model units before display'],
      ['frameModel', 'frame loaded models for the current viewport aspect ratio'],
      ['THREE.SRGBColorSpace', 'render embedded model textures in the expected output color space'],
    ]) {
      if (!characterModelSource.includes(needle)) {
        issues.push(`frontend/src/components/CharacterModelView.vue must ${description}`)
      }
    }

    const workflowRunDiagnosticsRequirements = [
      ['validateWorkflowStateKey', 'validate workflow state keys in the local browser validator'],
      ['WORKFLOW_STATE_KEY_PATTERN', 'keep frontend workflow state key rules portable'],
      ['node_state_key_invalid', 'surface invalid workflow state keys before execution'],
      ['validateWorkflowCondition', 'validate workflow condition expressions in the local browser validator'],
      ['WORKFLOW_CONDITION_MAX_CHARS', 'keep frontend workflow condition limits aligned with the backend'],
      ['node_condition_invalid', 'surface invalid workflow conditions before execution'],
      ['localConditionScope', 'provide workflow preview condition variables for Web/PWA fallback execution'],
      ['evaluateLocalCondition', 'evaluate common workflow condition expressions in browser previews'],
      ['condition_supported', 'report whether browser fallback condition evaluation was supported'],
      ['createLocalWorkflowState', 'maintain local workflow state during browser preview execution'],
      ['localState.variables', 'mirror workflow variable writes in browser previews'],
      ['localState.flags', 'mirror workflow flag writes in browser previews'],
      ['localState.relationships', 'mirror workflow relationship writes in browser previews'],
      ['localState.emotions', 'mirror workflow emotion changes in browser previews'],
      ['localRelationshipValue', 'reuse local relationship snapshots for browser preview conditions and events'],
      ['signedNumericConfig(node.config.delta)', 'allow negative workflow relationship deltas in browser previews'],
      ['signedNumericConfig(node.config.target_x)', 'allow signed camera X offsets in browser workflow previews'],
      ['signedNumericConfig(node.config.target_y)', 'allow signed camera Y offsets in browser workflow previews'],
      ['workflowBranchWeights(node.config, node.connections.length)', 'normalize random branch weights in browser workflow previews'],
      ['selectWeightedBranchIndex(weights, random)', 'select weighted random branches through an injectable browser preview random source'],
      ["case 'relationship'", 'execute relationship nodes in browser workflow previews'],
      ["case 'emotion_change'", 'execute emotion change nodes in browser workflow previews'],
      ['isEvaluationStep(step)', 'render evaluation score diagnostics in workflow run traces'],
      ['isTriggerEventStep(step)', 'render story-event trigger diagnostics in workflow run traces'],
      ['eventBlockers(step)', 'surface event trigger blocker reasons in workflow run traces'],
      ['scorePercent(step.output.score)', 'show evaluation score as a compact visual meter'],
      ['local_preview_no_chat_session', 'explain local workflow preview event blocks without chat state'],
      ['trace-diagnostics', 'keep a stable style hook for score/event run diagnostics'],
      ['executionStepsByNode', 'map workflow run steps back onto canvas nodes'],
      ['nodeRunClass(node)', 'render workflow run state classes on canvas nodes'],
      ['nodeRunBadge(node)', 'render compact workflow run badges on canvas nodes'],
      ['nodeRunDetail(node)', 'render compact workflow run details on canvas nodes'],
      ['run-executed', 'keep a stable style hook for executed canvas nodes'],
      ['node-run-badge', 'keep a stable style hook for canvas node run badges'],
      ['workflow.preview-context', 'expose localized author-controlled workflow preview context controls'],
      ['workflowRunContextPayload()', 'send author preview context to workflow execution'],
      ['function clampScore', 'clamp workflow preview context scores before sending them to the backend'],
      ['function clampRelationship', 'clamp workflow preview context relationship values before sending them to the backend'],
      ['runContext: runContextPayload', 'pass workflow preview context through the Tauri command payload'],
      ['localEventDecision(node, context, localState, storyEvents)', 'simulate score-gated event decisions against the injected project catalog'],
      ['run_context_evaluation', 'label simulated workflow score sources distinctly from chat sessions'],
      ['run-context-panel', 'keep a stable style hook for workflow preview context controls'],
      ['runContextPresets', 'provide one-click workflow preview context presets'],
      ['applyRunContextPreset(preset)', 'wire workflow preview context presets to the form state'],
      ['high_engagement', 'include a score-gated event preset for repeat-trigger blocking'],
      ['context-preset-btn', 'keep a stable style hook for workflow preview context preset controls'],
      ['coverage_percent', 'surface workflow run graph coverage from execution reports'],
      ['unvisited_node_ids', 'surface unvisited workflow nodes after a run'],
      ['workflowCoverage(currentWorkflow, steps)', 'compute workflow graph coverage for browser previews'],
      ['coverage-row', 'keep a stable style hook for workflow run coverage summaries'],
      ['unvisited-node-list', 'keep a stable style hook for unvisited workflow node chips'],
      ['runPresetMatrix()', 'provide one-click execution of all workflow preview presets'],
      ['aggregatePresetMatrixCoverage(currentWorkflow, matrixRuns)', 'merge workflow preview preset coverage'],
      ['presetMatrixReport', 'surface aggregate workflow preview matrix coverage'],
      ['matrix-coverage-panel', 'keep a stable style hook for workflow preset matrix coverage'],
    ]
    for (const [needle, description] of workflowRunDiagnosticsRequirements) {
      if (!workflowContractSource.includes(needle)) {
        issues.push(`Frontend workflow preview integration must ${description}`)
      }
    }

    const qualitySuiteRequirements = [
      ['workflow_coverage', 'surface workflow coverage reports in quality suites'],
      ['WorkflowCoverageReport', 'type workflow coverage reports'],
      ['workflow-coverage-row', 'keep a stable style hook for workflow coverage rows'],
      ['workflow-coverage-chip', 'keep a stable style hook for workflow coverage chips'],
      ['score-gate-workflow-coverage', 'include the score-gate workflow coverage preview scenario'],
      ['workflow-tool-output-sanitized', 'include the workflow tool-output containment preview scenario'],
      ['workflow-guard-only-output-fallback', 'include the workflow guard-only fallback preview scenario'],
      ['workflow_output', 'type finalized workflow output evidence in quality scenario reports'],
      ['workflow-output-row', 'keep a stable style hook for finalized workflow output evidence rows'],
      ['fallback-injection-score-contained', 'include the fallback scoring injection containment preview scenario'],
      ['structured-role-injection-contained', 'include the structured role-block injection containment preview scenario'],
      ['block-body-prompt-injection-contained', 'include the block-body prompt-control containment preview scenario'],
      ['relationship-injection-delta-contained', 'include the relationship injection side-channel preview scenario'],
      ['scenario_count: 29', 'keep the browser preview quality suite scenario count aligned with the default suite'],
      ["{ category: 'injection', total: 8", 'keep the browser preview injection category count aligned with the default suite'],
      ['relationship_delta', 'type relationship delta evidence in quality scenario reports'],
      ['memory_prompt_leak_detected', 'surface memory prompt replay safety in quality suites'],
      ['memory-leak', 'keep a stable style hook for memory prompt replay safety badges'],
      ['exportQualityReport()', 'provide JSON export for quality reports'],
      ['quality_report_schema', 'include a stable quality report export schema marker'],
      ['monogatari-quality-report', 'use stable quality report export filenames'],
      ['run_metadata', 'export quality suite run metadata for QA provenance'],
      ['QualitySuiteRunMetadata', 'type quality suite run metadata'],
      ['QualitySuiteSummary', 'type quality suite summaries with source provenance'],
      ['suite_path', 'surface the backend-confirmed quality suite source path'],
      ['suite_sha256', 'surface the backend-confirmed quality suite content fingerprint'],
      ['suite-fingerprint', 'show quality suite content fingerprints before running reports'],
      ['suite_source', 'export the backend-confirmed quality suite source separately from the UI selection'],
      ['git_short_commit', 'surface the quality report source commit in run metadata'],
      ['formatTimestamp', 'format quality report generation timestamps'],
      ['run-metadata-list', 'keep a stable style hook for quality run metadata'],
      ['audit_summary', 'include backend audit summaries in quality report exports'],
      ['failed_scenario_ids', 'export failed quality scenario ids for QA triage'],
      ['safety_signal_counts', 'export quality safety signal counts'],
      ['category_summary', 'export quality category summaries'],
      ['runtime_safety_trace', 'surface runtime safety trace evidence in quality scenarios'],
      ['mind_contract_applied', 'type character mind contract trace evidence'],
      ['knowledge_context_pinned', 'type pinned knowledge context trace evidence'],
      ['pinned_knowledge_ref_ids', 'type pinned knowledge ref id trace evidence'],
      ['runtimeTraceSummary', 'summarize quality runtime safety traces'],
      ['runtimeInterventionNotes', 'separate positive trace evidence from runtime interventions'],
      ['runtime_guard_interventions', 'count runtime guard interventions in quality audits'],
      ['runtimeGuardNoteCounts', 'compute runtime guard note counts for quality report exports'],
      ['runtime_guard_note_counts', 'export runtime guard note counts for QA evidence'],
      ['activeRuntimeGuardNotes', 'surface runtime guard note counts in the quality workbench'],
      ['runtime-trace-row', 'keep a stable style hook for quality runtime trace diagnostics'],
      ['runtime-guard-note-list', 'keep a stable style hook for runtime guard note summaries'],
      ['guard-note-chip', 'keep a stable style hook for runtime guard note chips'],
      ['rule_fingerprint', 'type event rule fingerprints in quality reports'],
      ['ruleChipLabel', 'show short event rule fingerprints in quality event-rule chips'],
      ['activeSafetySignals', 'surface active safety signal counts in the quality workbench'],
      ['audit-panel', 'keep a stable style hook for quality audit summaries'],
      ['category-audit-list', 'surface quality category audit summaries'],
      ['safety-signal-list', 'surface quality safety signal counts'],
      ['workflow-audit-list', 'surface workflow coverage audit summaries'],
    ]
    for (const [needle, description] of qualitySuiteRequirements) {
      if (!qualitySuiteSource.includes(needle)) {
        issues.push(`frontend/src/views/QualitySuiteView.vue must ${description}`)
      }
    }

    const audioManagerRequirements = [
      ['resolveAssetUrl', 'resolve audio file paths through the shared Tauri/Web asset resolver'],
      ['new Audio(resolvedUrl)', 'create real HTMLAudioElement instances for music and SFX previews'],
      ['audioStateKey', 'persist Audio Manager track and mixer state under a stable storage key'],
      ['window.localStorage.setItem(audioStateKey', 'write Audio Manager state to localStorage'],
      ['effectiveTrackVolume', 'compute per-track effective volume from track, master, and channel gains'],
      ['watch([masterVolume, bgmVolume, ambientVolume, sfxVolume, voiceVolume]', 'reactively apply master, BGM, ambient, SFX, and voice mixer changes'],
      ['playingMusicId', 'track active BGM/ambient playback state'],
      ['sfxPreviewIds', 'track active SFX preview state independently from looping music'],
      ['stopMusic()', 'provide a stable music stop path for transport controls and cleanup'],
      ['audio.onerror', 'surface audio load/playback failures in the UI'],
    ]
    for (const [needle, description] of audioManagerRequirements) {
      if (!audioViewSource.includes(needle)) {
        issues.push(`frontend/src/views/AudioView.vue must ${description}`)
      }
    }

    const projectExportRequirements = [
      ['export_project', 'call the backend project export command from Settings'],
      ['monogatari-project-export@1', 'preserve the project export manifest schema marker'],
      ['export_metadata', 'include project export build provenance metadata'],
      ['git_short_commit', 'include compact source commit evidence in browser preview export manifests'],
      ['content_summary', 'include project export content summaries in browser preview export manifests'],
      ['monogatari-project-content-summary/v1', 'version browser preview project export content summaries'],
      ['fingerprint_algorithm', 'include explicit project export package fingerprint algorithms in browser preview export manifests'],
      ['category_fingerprint_algorithm', 'include project export category fingerprint algorithms in browser preview export manifests'],
      ['category_fingerprints', 'include project export category fingerprints in browser preview export manifests'],
      ['content_sha256', 'include whole-package content fingerprints in browser preview export manifests'],
      ['downloadJson(', 'download project export manifests as JSON'],
      ['sanitizeManifestSettings', 'redact sensitive settings in browser preview export manifests'],
      ['@click="exportProjectManifest"', 'surface a project manifest export control'],
      ['runtimeSecretSettingKeys', 'centralize frontend runtime secret setting keys'],
      ['scrubRuntimeSecretSettings', 'scrub runtime secrets before saving project settings'],
      ['scrubRuntimeSecretString', 'scrub token-like and assignment-shaped secrets inside setting string values'],
      ['scrubTokenLikeValues', 'scrub token-shaped values from frontend settings payloads'],
      ['scrubSecretAssignments', 'scrub secret assignments from frontend settings payloads'],
      ["setConfigValue(config, ['ai', 'api', 'api_key'], '')", 'keep API keys runtime-only when saving project settings'],
    ]
    for (const [needle, description] of projectExportRequirements) {
      if (!settingsSource.includes(needle)) {
        issues.push(`frontend/src/views/SettingsView.vue must ${description}`)
      }
    }
    const projectPackageRequirements = [
      [projectArchiveSource, "import('@tauri-apps/plugin-dialog')", 'load native project package dialogs only when needed'],
      [projectArchiveSource, 'export_project_archive', 'invoke verified project package exports'],
      [projectArchiveSource, 'inspect_project_archive', 'verify packages before choosing an import destination'],
      [projectArchiveSource, 'import_project_archive', 'invoke transactional project package imports'],
      [projectArchiveSource, "extensions: ['monogatari']", 'filter native dialogs to .monogatari packages'],
      [projectArchiveSource, 'projectPackagesAvailable', 'gate native package workflows outside Tauri'],
      [settingsSource, '@click="exportProjectPackageFile"', 'surface project package exports in Settings'],
      [settingsSource, '@click="importProjectPackageFile"', 'surface project package imports in Settings'],
      [settingsSource, 'archiveSummary', 'surface verified package fingerprints and sizes'],
      [settingsSource, "invokeCommand<void>('initialize_engine'", 'activate validated imported projects'],
    ]
    for (const [source, needle, description] of projectPackageRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Project package frontend integration must ${description}`)
      }
    }
    if (settingsSource.includes("setConfigValue(config, ['ai', 'api', 'api_key'], apiKey.value)")) {
      issues.push('frontend/src/views/SettingsView.vue must not persist runtime API keys into settings.json payloads')
    }
    if (settingsSource.includes("apiKey.value = getConfigValue(config, ['ai', 'api', 'api_key'])")) {
      issues.push('frontend/src/views/SettingsView.vue must not hydrate runtime API keys from project settings')
    }

    const cloudSyncSettingsRequirements = [
      ['CloudSyncStatus', 'type backend cloud sync status payloads'],
      ['configure_cloud_sync', 'configure sync provider before status/push/pull actions'],
      ['syncStatusLabel', 'map backend sync status codes to author-visible labels'],
      ['pending_uploads', 'surface pending sync uploads in Settings'],
      ['pending_downloads', 'surface pending sync downloads in Settings'],
      ['syncProvider', 'let authors choose local manifest mode or remote preflight mode'],
    ]
    for (const [needle, description] of cloudSyncSettingsRequirements) {
      if (!settingsSource.includes(needle)) {
        issues.push(`frontend/src/views/SettingsView.vue must ${description}`)
      }
    }

    const chatSafetyTraceRequirements = [
      ['ChatSafetyTrace', 'type runtime chat safety trace payloads'],
      ['chat-safety-trace', 'listen for runtime chat safety trace events'],
      ['safetyTraceSummary', 'summarize runtime chat guard interventions'],
      ['runtimeSafetyFlags', 'surface runtime guard flags in the chat insight panel'],
      ['mind_contract_applied', 'surface character mind contract trace evidence'],
      ['knowledge_context_pinned', 'surface pinned knowledge context trace evidence'],
      ['pinned_knowledge_ref_ids', 'surface pinned knowledge ref id trace evidence'],
      ['response_guard_applied', 'surface guarded character response evidence'],
      ['relationship_delta_blocked', 'surface relationship side-channel containment evidence'],
      ['ChatSessionAuditReport', 'type restorable chat session audit reports'],
      ['get_chat_session_audit', 'restore latest chat safety and event audit state after character switching'],
      ['last_safety_trace', 'restore the latest runtime safety trace from chat sessions'],
      ['EventTriggerDecision', 'type runtime event trigger decisions'],
      ['rule_fingerprint', 'type runtime event rule fingerprints'],
      ['ConversationEvaluationReport', 'type atomic manual scoring reports'],
      ['evaluate_conversation_report', 'refresh story event decisions from the manual scoring report'],
      ['triggerable_events', 'carry triggerable story events in manual scoring reports'],
      ['chat-event-decisions', 'listen for runtime event trigger decisions'],
      ['eventDecisionSummary', 'surface story event trigger decision summaries'],
      ['shortRuleFingerprint', 'show short event rule fingerprints in the chat event audit'],
      ['rule-fingerprint', 'keep a stable style hook for chat event rule fingerprint diagnostics'],
      ['event-decision-panel', 'keep a stable style hook for story event trigger diagnostics'],
      ['safety-trace-panel', 'keep a stable style hook for chat safety trace diagnostics'],
      ['STREAM_FAILURE_BUBBLE', 'keep a stable frontend streaming failure bubble'],
      ['function streamFailureBubble(): string', 'avoid embedding provider/runtime errors in assistant failure bubbles'],
      ['assistantMessage.content = streamFailureBubble()', 'force streaming failures to clear partial streamed text with stable copy'],
    ]
    for (const [needle, description] of chatSafetyTraceRequirements) {
      if (!chatViewSource.includes(needle)) {
        issues.push(`frontend/src/views/ChatView.vue must ${description}`)
      }
    }

    const groupChatSafetyTraceRequirements = [
      ['ChatSafetyTrace', 'type runtime group chat safety trace payloads'],
      ['safety_trace', 'carry backend group chat safety traces on messages'],
      ['groupSafetyFlags', 'surface group chat guard flags per character response'],
      ['groupSafetySummary', 'summarize group chat guard interventions'],
      ['mind_contract_applied', 'surface group chat character mind contract trace evidence'],
      ['knowledge_context_pinned', 'surface group chat pinned knowledge context trace evidence'],
      ['pinned_knowledge_ref_ids', 'surface group chat pinned knowledge ref id trace evidence'],
      ['group-safety-trace', 'keep a stable style hook for group chat safety trace diagnostics'],
      ['errorMessage', 'surface group chat command failures to authors'],
      ['group-error', 'render group chat command errors in the workbench'],
      ['finally {', 'clear group chat loading state after send failures'],
      ['loading.value = false', 'reset group chat loading state after command completion'],
      ['relationship_delta_blocked', 'surface group chat relationship side-channel containment evidence'],
    ]
    for (const [needle, description] of groupChatSafetyTraceRequirements) {
      if (!groupChatViewSource.includes(needle)) {
        issues.push(`frontend/src/views/GroupChatView.vue must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Frontend source invariant verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Frontend source invariants OK')
  }

  async function verifyLegacyPromptBuilderInvariants() {
    const issues = []
    const promptBuilderSource = await readFile(path.join(root, 'src', 'LLMAssistant.AI', 'PromptBuilder.cs'), 'utf8')
    const promptBuilderTests = await readFile(path.join(root, 'tests', 'LLMAssistant.Tests', 'PromptBuilderTests.cs'), 'utf8')
    const apiEngineSource = await readFile(path.join(root, 'src', 'LLMAssistant.AI', 'API', 'APIEngine.cs'), 'utf8')
    const apiEngineTests = await readFile(path.join(root, 'tests', 'LLMAssistant.Tests', 'APIEngineTests.cs'), 'utf8')

    const sourceRequirements = [
      ['SanitizePromptContent', 'sanitize prompt content before legacy C# prompt assembly'],
      ['NormalizeSecurityText', 'normalize security-sensitive Unicode before legacy C# prompt checks'],
      ['IsStructuralRoleControlLine', 'detect XML/header/JSON-shaped role spoofing'],
      ['ContainsRoleTag(line, compact, role)', 'detect attributed XML role spoofing'],
      ['ContainsRoleTagWithBoundary', 'match attributed XML role tags without broad substring false positives'],
      ['IsRoleCodeFenceLine', 'detect Markdown role-code-fence spoofing'],
      ['PromptControlBlockStartForLine', 'omit explicit prompt-control block bodies after detecting their opening marker'],
      ['PromptControlBlockEnds', 'resume prompt sanitization only after explicit prompt-control block closers'],
      ['StartsWith("<!--", StringComparison.Ordinal)', 'strip HTML comment prompt-control prefixes before role-line checks'],
      ["'!', '/', '-', '*', '`', '#'", 'strip slash/star comment prompt-control prefixes before role-line checks'],
      ['RoleHeadingMatches', 'detect punctuation-free role heading spoofing'],
      ['SafeRoleHeader', 'prevent arbitrary AddMessage role labels from creating prompt sections'],
      ['Guarded prompt-control marker omitted.', 'omit structural prompt-control marker lines'],
      ['\\uFF01', 'normalize fullwidth ASCII ranges'],
      ['\\u200B', 'remove zero-width obfuscation ranges'],
    ]

    for (const [needle, description] of sourceRequirements) {
      if (!promptBuilderSource.includes(needle)) {
        issues.push(`Legacy C# PromptBuilder must ${description}`)
      }
    }

    const testRequirements = [
      ['Build_SanitizesRoleMarkersInsidePromptContent', 'test bracket/header/XML role marker sanitization'],
      ['Build_SanitizesFullwidthAndJsonRoleSpoofing', 'test fullwidth and JSON role spoofing sanitization'],
      ['Build_SanitizesAttributedRoleTags', 'test attributed XML role tag sanitization'],
      ['Build_AllowsNonRoleTagPrefixes', 'test attributed XML role matching keeps role-name boundaries'],
      ['Build_SanitizesRoleCodeFences', 'test Markdown role-code-fence sanitization'],
      ['Build_AllowsNonRoleCodeFences', 'test Markdown role-code-fence matching keeps role-name boundaries'],
      ['Build_OmitsPromptControlBlockBodies', 'test explicit prompt-control block body omission'],
      ['Build_SanitizesCommentedRoleMarkers', 'test comment-wrapped role marker sanitization'],
      ['Build_AllowsNonRoleCommentPrefixes', 'test comment-wrapped role matching keeps role-name boundaries'],
      ['Build_SanitizesRoleHeadingsWithoutPunctuation', 'test punctuation-free role heading sanitization'],
      ['Build_AllowsNonRoleHeadingPrefixes', 'test punctuation-free role heading matching keeps role-name boundaries'],
      ['Build_DefaultsUnexpectedMessageRolesToUser', 'test arbitrary message roles cannot create prompt sections'],
    ]

    for (const [needle, description] of testRequirements) {
      if (!promptBuilderTests.includes(needle)) {
        issues.push(`Legacy C# PromptBuilder tests must ${description}`)
      }
    }

    const apiSourceRequirements = [
      ['RedactSensitiveText', 'centralize legacy API error/log redaction'],
      ['TokenLikeValueRegex', 'redact token-shaped provider echoes'],
      ['SecretJsonAssignmentRegex', 'redact JSON secret assignment echoes'],
      ['SecretQueryAssignmentRegex', 'redact URL query secret echoes'],
      ['SecretHeaderAssignmentRegex', 'redact header-shaped secret echoes'],
      ['API error ({response.StatusCode}): {RedactSensitiveText(responseBody)}', 'redact non-success provider response bodies'],
      ['API request failed: {RedactSensitiveText(ex.Message)}', 'redact request exception messages'],
    ]

    for (const [needle, description] of apiSourceRequirements) {
      if (!apiEngineSource.includes(needle)) {
        issues.push(`Legacy C# APIEngine must ${description}`)
      }
    }

    const apiTestRequirements = [
      ['RedactSensitiveText_RemovesTokenLikeValuesAndSecretAssignments', 'test direct secret redaction helpers'],
      ['InferAsync_RedactsSensitiveProviderErrorBodies', 'test provider error body redaction'],
      ['InferAsync_RedactsSensitiveRequestExceptions', 'test request exception redaction'],
    ]

    for (const [needle, description] of apiTestRequirements) {
      if (!apiEngineTests.includes(needle)) {
        issues.push(`Legacy C# APIEngine tests must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Legacy C# AI verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Legacy C# AI invariants OK')
  }

  async function verifyAiBackendConfigInvariants() {
    const issues = []
    const aiCommandSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'ai.rs'), 'utf8')
    const rustApiEngineSource = await readFile(path.join(rustDir, 'crates', 'ai', 'src', 'api_engine.rs'), 'utf8')
    const backendSelectionSource = await readFile(path.join(rustDir, 'crates', 'ai', 'src', 'backend_selection.rs'), 'utf8')
    const backendMatrixSource = await readFile(path.join(root, 'docs', 'INFERENCE_BACKEND_MATRIX.md'), 'utf8')
    const rustOnnxEngineSource = await readFile(path.join(rustDir, 'crates', 'ai', 'src', 'onnx_engine.rs'), 'utf8')
    const rustPipelineSource = await readFile(path.join(rustDir, 'crates', 'ai', 'src', 'pipeline.rs'), 'utf8')
    const rustPipelineTests = await readFile(path.join(rustDir, 'crates', 'ai', 'tests', 'pipeline_tests.rs'), 'utf8')
    const settingsViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'SettingsView.vue'), 'utf8')
    const chatViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'ChatView.vue'), 'utf8')
    const webGpuSource = await readFile(path.join(frontendDir, 'src', 'lib', 'webgpuInference.ts'), 'utf8')
    const webDistSource = await readFile(path.join(frontendDir, 'scripts', 'prepare-web-dist.mjs'), 'utf8')
    const frontendPackage = JSON.parse(await readFile(path.join(frontendDir, 'package.json'), 'utf8'))
    const tauriMainSource = await readFile(path.join(tauriAppDir, 'src', 'main.rs'), 'utf8')

    const aiRequirements = [
      ['onnx_model_config_in_project', 'centralize project-scoped ONNX config construction'],
      ['onnx_file_path_in_project', 'resolve ONNX model and tokenizer paths under the project root'],
      ['normalize_onnx_file_ref', 'normalize and validate ONNX path references before path construction'],
      ['current_project_data_root', 'bind ONNX model references to the active project data root'],
      ['ONNX file paths cannot contain drive prefixes or URI schemes', 'reject URI-like and drive-prefixed ONNX paths'],
      ['ONNX file paths cannot contain empty, current, or parent directory segments', 'reject traversal-shaped ONNX paths'],
      ['&[".onnx"]', 'restrict model references to ONNX files'],
      ['&[".json"]', 'restrict tokenizer references to JSON files'],
      ['path.starts_with(project_root)', 'prove ONNX file references stay under the project root'],
      ['register_initialized_api_engine', 'centralize initialized API registration'],
      ['engine.initialize().await', 'initialize the API backend before marking it active'],
      ['initialize_onnx_engine', 'initialize the DirectML backend before registration'],
      ['register_engine_with_name', 'register configured backends without blocking inside async commands'],
      ['register_onnx_engine', 'reuse the guarded ONNX registration helper'],
      ['set_active_engine("ONNX")', 'activate the ONNX backend after configuration'],
      ['onnx_file_paths_resolve_under_project_root', 'test compatible ONNX file path resolution'],
      ['onnx_file_paths_reject_escape_attempts', 'test ONNX traversal and absolute path rejection'],
      ['configure_onnx_registers_active_engine', 'test ONNX configuration activates the backend'],
      ['configure_onnx_registration_is_async_safe', 'test ONNX registration is safe inside an async runtime'],
      ['configure_api_initializes_ready_engine', 'test API configuration reports a ready active engine'],
      ['configure_api_rejects_invalid_config_without_registering_engine', 'test invalid API configuration is not registered as an active engine'],
    ]
    for (const [needle, description] of aiRequirements) {
      if (!aiCommandSource.includes(needle)) {
        issues.push(`AI backend configuration must ${description}`)
      }
    }

    const apiStreamingRequirements = [
      ['SseDeltaParser', 'buffer OpenAI-compatible SSE stream lines across network chunks'],
      ['push_bytes(&chunk)', 'feed raw response bytes into the buffered SSE parser'],
      ['if sse_parser.done', 'stop reading after an SSE [DONE] marker'],
      ['finish()', 'flush a final SSE line if the server closes without a trailing newline'],
      ['stream_error_message', 'detect provider error payloads inside SSE data frames'],
      ['Failed to parse stream response', 'reject malformed SSE data frames instead of ignoring provider payload drift'],
      ['sse_delta_parser_buffers_split_json_and_unicode_lines', 'test split JSON and UTF-8 stream chunks'],
      ['sse_delta_parser_flushes_final_line_without_newline', 'test final unterminated SSE line handling'],
      ['sse_delta_parser_reports_stream_error_frames', 'test provider error frames abort streaming inference'],
      ['sse_delta_parser_rejects_error_frame_after_partial_content', 'test provider error frames abort even after partial text'],
      ['sse_delta_parser_rejects_malformed_data_frames', 'test malformed SSE data frames are rejected'],
    ]
    for (const [needle, description] of apiStreamingRequirements) {
      if (!rustApiEngineSource.includes(needle)) {
        issues.push(`Rust API streaming must ${description}`)
      }
    }

    const apiRuntimeConfigRequirements = [
      ['validate_api_config', 'validate API runtime configuration before initialization'],
      ['normalize_api_base_url', 'normalize API base URLs before they are used for chat requests'],
      ['embedded credentials', 'reject API base URLs with embedded credentials'],
      ['query strings or fragments', 'reject API base URLs with query strings or fragments'],
      ['localhost or a loopback address', 'allow plaintext HTTP only for local API providers'],
      ['api_initialize_rejects_invalid_runtime_config', 'test invalid API runtime configuration rejection'],
      ['api_initialize_normalizes_valid_runtime_config', 'test valid API runtime configuration normalization'],
    ]
    for (const [needle, description] of apiRuntimeConfigRequirements) {
      if (!rustApiEngineSource.includes(needle)) {
        issues.push(`Rust API runtime configuration must ${description}`)
      }
    }

    const apiResponseShapeRequirements = [
      ['extract_chat_response_text', 'extract OpenAI-compatible response content through a guarded helper'],
      ['ensure_generated_text', 'reject missing or blank generated text before reporting API success'],
      ['api_response_text_rejects_missing_or_blank_content', 'test invalid non-streaming success payloads'],
      ['api_streaming_text_rejects_empty_completion', 'test empty streaming completions are rejected'],
    ]
    for (const [needle, description] of apiResponseShapeRequirements) {
      if (!rustApiEngineSource.includes(needle)) {
        issues.push(`Rust API response shape handling must ${description}`)
      }
    }

    const directmlRuntimeRequirements = [
      ['ep::DirectML::default()', 'create an ONNX Runtime DirectML execution provider'],
      ['error_on_failure()', 'reject missing DirectML support instead of silently falling back to CPU'],
      ['with_parallel_execution(false)', 'use DirectML-compatible sequential execution'],
      ['with_memory_pattern(false)', 'disable memory patterns required by DirectML'],
      ['Tokenizer::from_file', 'load a standard tokenizer.json artifact'],
      ['validate_model_contract', 'validate supported causal-LM model inputs and logits output'],
      ['.run(inputs)', 'execute the ONNX graph for generated tokens'],
      ['try_extract_tensor::<f32>()', 'read model logits from ONNX Runtime'],
      ['sample_token', 'sample generated tokens with runtime inference options'],
      ['tokio::task::spawn_blocking', 'keep blocking DirectML work off async runtime threads'],
      ['onnx_engine_enforces_directml_contract', 'test that Windows local inference cannot disable DirectML'],
      ['onnx_initialize_rejects_invalid_model_without_ready_state', 'test invalid models never report a ready runtime'],
      ['onnx_infer_requires_initialized_runtime', 'test inference rejects an uninitialized runtime'],
    ]
    for (const [needle, description] of directmlRuntimeRequirements) {
      if (!rustOnnxEngineSource.includes(needle)) {
        issues.push(`Windows DirectML runtime must ${description}`)
      }
    }
    if (rustOnnxEngineSource.includes('[ONNX inference placeholder - model not loaded]')) {
      issues.push('Windows DirectML runtime must not return placeholder text as a successful inference result')
    }

    const webGpuRuntimeRequirements = [
      [webGpuSource, "pipeline('text-generation'", 'create a browser text-generation pipeline'],
      [webGpuSource, "device: 'webgpu'", 'force browser model execution through WebGPU'],
      [webGpuSource, 'TextStreamer', 'stream browser-generated text'],
      [webGpuSource, 'detectWebGpuSupport', 'detect secure-context and WebGPU availability'],
      [webGpuSource, "packagedAssetUrl('inference-runtime.json')", 'load the packaged WebGPU runtime contract relative to the deployment base'],
      [webGpuSource, 'wasm.wasmPaths', 'override the Transformers.js CDN default with packaged ONNX runtime assets'],
      [webGpuSource, 'wasm.numThreads = 1', 'avoid worker and blob bootstrap requirements in static web packages'],
      [webGpuSource, "onnxruntime-web/dist/ort-wasm-simd-threaded.asyncify.wasm", 'bundle the matching ONNX WebGPU WASM binary with Vite'],
      [chatViewSource, 'generateWebGpuChat', 'use WebGPU generation in browser character tests'],
      [webDistSource, "schema: 'monogatari-inference-runtime/v1'", 'emit a versioned web inference contract'],
      [webDistSource, "backend: 'webgpu'", 'mark web packages as WebGPU runtimes'],
      [webDistSource, "'wasm-unsafe-eval'", 'allow the packaged ONNX WebAssembly bootstrap under CSP'],
    ]
    for (const [source, needle, description] of webGpuRuntimeRequirements) {
      if (!source.includes(needle)) {
        issues.push(`WebGPU runtime must ${description}`)
      }
    }
    if (frontendPackage.dependencies?.['@huggingface/transformers'] !== '4.2.0') {
      issues.push('WebGPU runtime must pin the verified Transformers.js runtime version')
    }

    const aiStatusRequirements = [
      [rustPipelineSource, 'engine_statuses', 'expose actual inference engine readiness from the pipeline'],
      [aiCommandSource, 'engine_statuses()', 'report actual engine readiness in get_ai_status'],
      [rustPipelineTests, 'test_inference_pipeline_engine_statuses_reflect_readiness', 'test mixed ready/not-ready engine status reporting'],
    ]
    for (const [source, needle, description] of aiStatusRequirements) {
      if (!source.includes(needle)) {
        issues.push(`AI backend status must ${description}`)
      }
    }
    if (aiCommandSource.includes('ready: true')) {
      issues.push('AI backend status must not hard-code registered engines as ready')
    }

    const backendSelectionRequirements = [
      [backendSelectionSource, 'monogatari-inference-backend-plan/v1', 'emit a versioned backend selection report'],
      [backendSelectionSource, 'BackendReadiness::ProbeRequired', 'separate runtime detection from model readiness'],
      [backendSelectionSource, 'No backend is recommended until a model-level generation probe succeeds.', 'refuse to recommend unprobed backends'],
      [backendSelectionSource, 'qwen35_hybrid_contract_unsupported', 'block the current DirectML executor for Qwen3.5 hybrid graphs'],
      [backendSelectionSource, 'qwen35_dml_graph_capture_partition_failure', 'record the current WinML DirectML graph partition blocker'],
      [backendSelectionSource, 'server_prefers_ready_vllm_then_sglang_then_llama_cpp', 'test managed Linux service priority'],
      [backendSelectionSource, 'backend_ids_use_stable_snake_case_wire_names', 'test stable frontend backend identifiers'],
      [backendSelectionSource, 'configured_api_still_requires_a_generation_probe', 'keep configured APIs unready until generation succeeds'],
      [backendSelectionSource, 'api_service_ready', 'represent completed API generation separately from configuration'],
      [aiCommandSource, 'get_inference_backend_plan', 'expose host capability planning through Tauri'],
      [tauriMainSource, 'commands::ai::get_inference_backend_plan', 'register the backend planning command'],
      [settingsViewSource, "invokeCommand<InferenceBackendPlan>('get_inference_backend_plan'", 'load the backend plan in desktop diagnostics'],
      [settingsViewSource, 'backendPlanRows', 'render actionable backend readiness states'],
      [settingsViewSource, 'webgpu_model_ready: hasVerifiedWebGpuGeneration()', 'report WebGPU ready only after successful generation'],
      [backendMatrixSource, 'Qwen3.5 is a vision-language model, not a raw 3D mesh model.', 'document the model and 3D boundary'],
      [backendMatrixSource, 'qwen35_text08_b', 'document the stable default model profile identifier'],
    ]
    for (const [source, needle, description] of backendSelectionRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Inference backend selection must ${description}`)
      }
    }

    const pipelineRegistrationRequirements = [
      [rustPipelineSource, '.try_read()', 'avoid blocking Tokio runtime threads while deriving registered engine names'],
      [rustPipelineSource, 'register_engine_with_name', 'allow async command paths to register engines by explicit backend name'],
      [rustPipelineTests, 'test_inference_pipeline_register_engine_is_async_safe', 'test inference engine registration inside an async runtime'],
    ]
    for (const [source, needle, description] of pipelineRegistrationRequirements) {
      if (!source.includes(needle)) {
        issues.push(`AI pipeline registration must ${description}`)
      }
    }

    const pipelineFailureRequirements = [
      [rustPipelineSource, 'ensure_successful_result', 'normalize unsuccessful inference results before callers consume generated text'],
      [rustPipelineSource, 'Inference returned unsuccessful result', 'provide a stable fallback error for unsuccessful inference results without provider details'],
      [rustPipelineTests, 'test_inference_pipeline_retries_unsuccessful_results', 'test retry handling for unsuccessful inference result envelopes'],
      [rustPipelineTests, 'test_inference_pipeline_specific_engine_rejects_unsuccessful_result', 'test direct engine calls reject unsuccessful inference result envelopes'],
      [rustPipelineTests, 'test_inference_pipeline_stream_rejects_unsuccessful_result', 'test streaming calls reject unsuccessful inference result envelopes'],
    ]
    for (const [source, needle, description] of pipelineFailureRequirements) {
      if (!source.includes(needle)) {
        issues.push(`AI pipeline failure handling must ${description}`)
      }
    }

    if (
      aiCommandSource.includes('PathBuf::from(&model_path)')
      || aiCommandSource.includes('PathBuf::from(&tokenizer_path)')
      || aiCommandSource.includes('std::path::PathBuf::from(&model_path)')
      || aiCommandSource.includes('std::path::PathBuf::from(&tokenizer_path)')
    ) {
      issues.push('AI backend configuration must not turn frontend ONNX strings directly into filesystem paths')
    }

    const settingsRequirements = [
      ["invokeCommand<void>('configure_onnx'", 'invoke the backend ONNX command contract'],
      ['modelPath: modelPath.value', 'send the project-relative model path'],
      ['tokenizerPath: tokenizerPath.value', 'send the project-relative tokenizer path'],
      ['initializeWebGpuRuntime', 'initialize the browser WebGPU runtime'],
      ['webGpuModelId', 'configure the packaged WebGPU model ID'],
      ['webGpuDtype', 'configure WebGPU model precision'],
      ["setConfigValue(config, ['ai', 'onnx', 'use_directml'], true)", 'persist DirectML as a required Windows runtime'],
    ]
    for (const [needle, description] of settingsRequirements) {
      if (!settingsViewSource.includes(needle)) {
        issues.push(`Settings AI backend UI must ${description}`)
      }
    }

    if (!aiCommandSource.includes('apply_onnx_runtime_options') || !aiCommandSource.includes('config.use_directml = true')) {
      issues.push('AI backend configuration must enforce DirectML for Windows local inference')
    }
    if (!aiCommandSource.includes('configure_onnx_enforces_directml')) {
      issues.push('AI backend configuration must test the required DirectML contract')
    }

    if (issues.length > 0) {
      throw new Error(`AI backend config verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] AI backend config invariants OK')
  }

  async function verifyEngineProjectRootInvariants() {
    const issues = []
    const engineSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'engine.rs'), 'utf8')
    const settingsViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'SettingsView.vue'), 'utf8')

    const engineRequirements = [
      ['current_project_data_root', 'reuse the active/default project root when initialization receives an empty project path'],
      ['normalize_project_path_from', 'centralize testable engine project path normalization'],
      ['validate_engine_project_root', 'validate engine project roots before binding state'],
      ['Project path cannot contain control characters', 'reject control-character project path input'],
      ['Project path must be a local filesystem path, not a URI', 'reject URI-shaped project path input'],
      ['Engine project path does not exist', 'reject missing project roots before initialization'],
      ['Engine project path is not a directory', 'reject file paths before initialization'],
      ['state.set_project_data_root(path).await', 'bind only the validated project root into app state'],
      ['engine_project_paths_resolve_existing_relative_dirs', 'test relative project root resolution'],
      ['engine_project_paths_reject_uri_and_control_input', 'test URI and control-character rejection'],
      ['engine_project_root_validation_requires_existing_directory', 'test missing and file project root rejection'],
    ]
    for (const [needle, description] of engineRequirements) {
      if (!engineSource.includes(needle)) {
        issues.push(`Engine project root handling must ${description}`)
      }
    }

    if (!settingsViewSource.includes("invokeCommand<void>('initialize_engine', { projectPath: projectPath.value })")) {
      issues.push('Settings project initialization must pass projectPath through the backend initialize_engine contract')
    }

    if (issues.length > 0) {
      throw new Error(`Engine project root verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Engine project root invariants OK')
  }

  async function verifyAssetManagerInvariants() {
    const issues = []
    const rustAssetManagerSource = await readFile(path.join(rustDir, 'crates', 'assets', 'src', 'asset_manager.rs'), 'utf8')
    const csharpAssetManagerSource = await readFile(path.join(root, 'src', 'LLMAssistant.Assets', 'AssetManager.cs'), 'utf8')
    const csharpAssetManagerTests = await readFile(path.join(root, 'tests', 'LLMAssistant.Tests', 'AssetManagerTests.cs'), 'utf8')

    const rustRequirements = [
      ['safe_asset_path', 'resolve asset paths through a guarded path helper'],
      ['normalize_asset_relative_path', 'normalize and validate project-relative asset paths before file access'],
      ['Asset paths must be relative to the asset root', 'reject absolute asset paths'],
      ['Asset paths cannot contain empty, current, or parent directory segments', 'reject traversal-shaped asset paths'],
      ['path.starts_with(&root)', 'defensively prove asset paths stay under the asset root'],
      ['load_text_rejects_paths_that_escape_asset_root', 'test text asset traversal rejection'],
      ['load_bytes_rejects_absolute_asset_paths', 'test absolute binary asset rejection'],
      ['list_directory_rejects_parent_traversal', 'test directory listing traversal rejection'],
      ['loads_nested_project_asset_paths', 'test valid nested project asset loading'],
      ['exists_returns_false_for_invalid_asset_paths', 'test invalid paths do not resolve through exists checks'],
    ]
    for (const [needle, description] of rustRequirements) {
      if (!rustAssetManagerSource.includes(needle)) {
        issues.push(`Rust AssetManager must ${description}`)
      }
    }

    const csharpSourceRequirements = [
      ['SafeAssetPath', 'resolve asset paths through a guarded path helper'],
      ['NormalizeAssetRelativePath', 'normalize and validate project-relative asset paths before file access'],
      ['Path.GetFullPath', 'normalize asset paths before boundary checks'],
      ['Path.IsPathRooted', 'reject rooted asset paths'],
      ['Asset path must stay inside the asset root', 'defensively prove asset paths stay under the asset root'],
      ['TryResolvePath', 'return null instead of reading invalid asset paths from load helpers'],
    ]
    for (const [needle, description] of csharpSourceRequirements) {
      if (!csharpAssetManagerSource.includes(needle)) {
        issues.push(`Legacy C# AssetManager must ${description}`)
      }
    }

    const csharpTestRequirements = [
      ['ResolvePath_RejectsTraversalAssetPaths', 'test direct traversal path rejection'],
      ['LoadText_ReturnsNullForEscapingAssetPaths', 'test text asset traversal containment'],
      ['LoadBytes_ReturnsNullForAbsoluteAssetPaths', 'test absolute binary path containment'],
      ['LoadJsonAsync_ReturnsNullForUriLikeAssetPaths', 'test URI-like JSON path containment'],
      ['LoadText_AllowsNestedProjectAssetPaths', 'test valid nested project asset loading'],
    ]
    for (const [needle, description] of csharpTestRequirements) {
      if (!csharpAssetManagerTests.includes(needle)) {
        issues.push(`Legacy C# AssetManager tests must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Asset manager path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Asset manager path invariants OK')
  }

  async function verifySaveManagerInvariants() {
    const issues = []
    const rustSaveManagerSource = await readFile(path.join(rustDir, 'crates', 'assets', 'src', 'save_manager.rs'), 'utf8')
    const rustSaveCommandSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'save.rs'), 'utf8')
    const gameViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'GameView.vue'), 'utf8')
    const gameStoreSource = await readFile(path.join(frontendDir, 'src', 'stores', 'game.ts'), 'utf8')
    const csharpSaveManagerSource = await readFile(path.join(root, 'src', 'LLMAssistant.Assets', 'SaveManager.cs'), 'utf8')
    const csharpSaveManagerTests = await readFile(path.join(root, 'tests', 'LLMAssistant.Tests', 'SaveManagerTests.cs'), 'utf8')

    const rustRequirements = [
      ['safe_save_path', 'resolve save ids through a guarded path helper'],
      ['is_valid_save_id', 'validate save ids before path construction'],
      ['Save id cannot contain path separators', 'reject traversal-shaped save ids'],
      ['path.parent() != Some(root.as_path())', 'defensively prove save paths stay under the save root'],
      ['is_save_json_path', 'filter listed save files through the same id rules'],
      ['save.save_id == file_save_id', 'reject listed saves whose embedded id does not match the file name'],
      ['save_rejects_ids_that_escape_save_directory', 'test save rejection for escaping ids'],
      ['load_rejects_ids_that_escape_save_directory', 'test load rejection for escaping ids'],
      ['delete_rejects_ids_that_escape_save_directory', 'test delete rejection for escaping ids'],
      ['list_saves_ignores_invalid_or_mismatched_save_ids', 'test list filtering for invalid or mismatched ids'],
      ['GAME_SAVE_SCHEMA_V3', 'version complete runtime snapshots through the v3 save schema'],
      ['validate_schema', 'reject unsupported save schemas before restore'],
      ['create_save_with_id', 'support stable quick-save and auto-save slots'],
      ['legacy_save_payloads_deserialize_with_v1_defaults', 'test backward-compatible v1 save loading'],
      ['new_and_stable_slot_saves_use_v3_schema', 'test generated and stable slots use the v3 contract'],
      ['MAX_GAME_SAVE_BYTES', 'bound serialized save file reads and writes'],
      ['write_staged', 'stage save overwrites before replacing the active slot'],
      ['recover_backup_if_needed', 'recover interrupted stable-slot replacements'],
      ['stable_slot_overwrite_replaces_save_without_staged_files', 'test stable slot replacement and cleanup'],
    ]
    for (const [needle, description] of rustRequirements) {
      if (!rustSaveManagerSource.includes(needle)) {
        issues.push(`Rust SaveManager must ${description}`)
      }
    }

    const rustCommandRequirements = [
      ['save_id: Option<String>', 'accept optional stable save slots without breaking manual UUID saves'],
      ['capture_game_save', 'centralize complete runtime snapshot capture'],
      ['restore_game_save', 'centralize validated runtime restoration'],
      ['snapshot_character_states', 'persist character emotion, relationships, and full memory'],
      ['snapshot_chat_sessions', 'persist chat history, evaluation, audit, and triggered-event state'],
      ['story_progress', 'persist applied story events and unlocked content'],
      ['let story_progress = state.story_progress.read().await', 'snapshot story progress before action-backed script flags using the executor lock order'],
      ['deserialize_story_progress', 'validate story progress before runtime restore'],
      ['migrate_legacy_story_progress', 'reconstruct unlock state from v1/v2 triggered event sessions'],
      ['dialogue_state', 'persist the active dialogue cursor and dialogue-local state'],
      ['script_variables_to_json', 'serialize Rhai variables without stringifying primitive types'],
      ['json_variables_to_script', 'restore persisted Rhai variable types'],
      ['game_save_round_trip_restores_character_chat_scene_and_script_state', 'test complete runtime save restoration'],
      ['v2_save_migrates_triggered_events_into_story_progress', 'test backward-compatible story progress migration'],
      ['invalid_story_progress_is_rejected_before_runtime_mutation', 'test atomic rejection of invalid progress snapshots'],
    ]
    for (const [needle, description] of rustCommandRequirements) {
      if (!rustSaveCommandSource.includes(needle)) {
        issues.push(`Rust save commands must ${description}`)
      }
    }

    const frontendRequirements = [
      [gameViewSource, 'saveId: QUICK_SAVE_ID', 'write quick saves to the stable quick-save slot'],
      [gameViewSource, 'saveId: AUTO_SAVE_ID', 'overwrite a bounded auto-save slot instead of creating unbounded files'],
      [gameStoreSource, "invokeCommand('save_game', { saveName, saveId })", 'send the backend save command contract'],
      [gameStoreSource, "invokeCommand('load_game', { saveId })", 'send the backend load command contract'],
      [gameStoreSource, 's.save_id !== saveId', 'consume backend save_id fields when deleting local rows'],
    ]
    for (const [source, needle, description] of frontendRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Frontend save flow must ${description}`)
      }
    }

    const csharpSourceRequirements = [
      ['SafeSavePath', 'resolve save ids through a guarded path helper'],
      ['IsValidSaveId', 'validate save ids before path construction'],
      ['Path.GetFullPath', 'normalize save paths before boundary checks'],
      ['Save id cannot contain path separators', 'reject traversal-shaped save ids'],
      ['StartsWith(rootPrefix', 'defensively prove save paths stay under the save root'],
      ['save.SaveId == fileSaveId', 'reject listed saves whose embedded id does not match the file name'],
    ]
    for (const [needle, description] of csharpSourceRequirements) {
      if (!csharpSaveManagerSource.includes(needle)) {
        issues.push(`Legacy C# SaveManager must ${description}`)
      }
    }

    const csharpTestRequirements = [
      ['Save_RejectsTraversalSaveIds', 'test save rejection for escaping ids'],
      ['Load_ReturnsNullForTraversalSaveIds', 'test load rejection for escaping ids'],
      ['DeleteSave_IgnoresTraversalSaveIds', 'test delete containment for escaping ids'],
      ['GetAllSaves_IgnoresInvalidOrMismatchedSaveIds', 'test list filtering for invalid or mismatched ids'],
    ]
    for (const [needle, description] of csharpTestRequirements) {
      if (!csharpSaveManagerTests.includes(needle)) {
        issues.push(`Legacy C# SaveManager tests must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Save manager path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Save manager path invariants OK')
  }

  async function verifyScriptCommandInvariants() {
    const issues = []
    const coreStateKeySource = await readFile(path.join(rustDir, 'crates', 'core', 'src', 'state_key.rs'), 'utf8')
    const scriptCommandSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'script.rs'), 'utf8')
    const scriptingSource = await readFile(path.join(rustDir, 'crates', 'scripting', 'src', 'lib.rs'), 'utf8')
    const gameDialogueSource = await readFile(path.join(rustDir, 'crates', 'game', 'src', 'dialogue', 'dialogue_manager.rs'), 'utf8')
    const dialogueCommandSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'dialogue.rs'), 'utf8')
    const saveCommandSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'save.rs'), 'utf8')
    const workflowSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'workflow.rs'), 'utf8')

    const commandRequirements = [
      ['validate_script_text', 'centralize script command input validation'],
      ['SCRIPT_MAX_TEXT_CHARS', 'reuse the shared Rhai script source size cap for direct execution'],
      ['validate_condition_source', 'reuse shared condition expression validation'],
      ['condition_inputs_use_shared_limits', 'test condition command inputs use shared limits'],
      ['MAX_DSL_SCRIPT_TEXT_CHARS', 'cap DSL parser payload size'],
      ['cannot contain control characters', 'reject hidden control-character payloads'],
      ['script_inputs_reject_control_characters', 'test control-character rejection'],
      ['script_inputs_limit_large_payloads', 'test script payload size limits'],
      ['script_inputs_allow_multiline_text', 'continue allowing normal multiline authoring scripts'],
    ]
    for (const [needle, description] of commandRequirements) {
      if (!scriptCommandSource.includes(needle)) {
        issues.push(`Script commands must ${description}`)
      }
    }

    const stateKeyRequirements = [
      ['SCRIPT_STATE_KEY_MAX_CHARS', 'define a shared script state key size cap'],
      ['normalize_script_state_key', 'centralize script variable and flag key validation'],
      ['normalize_script_state_map', 'normalize persisted script state maps before loading'],
      ['Script state key cannot contain control characters', 'reject hidden control-character state keys'],
      ['Script state key can contain only ASCII letters, numbers, dots, underscores, or hyphens', 'restrict script state keys to portable save-friendly characters'],
      ['script_state_keys_reject_control_and_path_like_values', 'test rejection of path-shaped and hidden state keys'],
      ['script_state_maps_reject_duplicate_normalized_keys', 'test ambiguous normalized state keys'],
    ]
    for (const [needle, description] of stateKeyRequirements) {
      if (!coreStateKeySource.includes(needle)) {
        issues.push(`Script state key validation must ${description}`)
      }
    }

    const engineRequirements = [
      ['SCRIPT_MAX_TEXT_CHARS', 'define a shared script source size cap'],
      ['SCRIPT_MAX_CONDITION_CHARS', 'define a shared condition expression size cap'],
      ['SCRIPT_STATE_KEY_MAX_CHARS', 're-export the shared script state key size cap'],
      ['condition_engine: Engine', 'separate read-only condition evaluation from mutating script execution'],
      ['register_state_read_functions', 'share read-only state access functions across script engines'],
      ['register_state_write_functions', 'keep state mutation functions out of condition evaluation'],
      ['condition_engine_can_read_but_not_mutate_state', 'test condition expressions cannot mutate script state'],
      ['condition_engine_can_read_temporary_scope_variables', 'test read-only temporary condition scope variables'],
      ['direct_scripts_keep_state_mutation_functions', 'test direct author scripts can still mutate state intentionally'],
      ['Box<rhai::EvalAltResult>', 'return Rhai runtime errors for invalid script state keys'],
      ['normalize_script_state_key(name)', 'validate Rhai variable and flag names before state access'],
      ['normalize_script_state_map(variables)', 'validate loaded script variables before replacing runtime state'],
      ['validate_script_source', 'centralize Rhai script source validation in the shared engine crate'],
      ['validate_script_source(script)?', 'validate all direct ScriptEngine executions before evaluating Rhai'],
      ['validate_condition_source(condition)?', 'validate condition expressions through the shared condition limits'],
      ['evaluate_condition_with_scope_variables', 'evaluate workflow conditions with temporary read-only context variables'],
      ['cannot contain control characters', 'reject hidden control characters in every ScriptEngine caller'],
      ['condition_engine_rejects_oversized_conditions', 'test condition expression size rejection'],
      ['condition_engine_rejects_control_characters', 'test condition expression control-character rejection'],
      ['SCRIPT_MAX_OPERATIONS', 'define a script operation budget'],
      ['set_max_operations(SCRIPT_MAX_OPERATIONS)', 'bound Rhai execution operations'],
      ['set_max_call_levels(SCRIPT_MAX_CALL_LEVELS)', 'bound Rhai recursive call depth'],
      ['set_max_expr_depths(SCRIPT_MAX_EXPR_DEPTH, SCRIPT_MAX_FUNCTION_EXPR_DEPTH)', 'bound Rhai expression nesting'],
      ['set_max_variables(SCRIPT_MAX_VARIABLES)', 'bound Rhai variable growth'],
      ['set_max_functions(SCRIPT_MAX_FUNCTIONS)', 'bound Rhai function definitions'],
      ['set_max_modules(0)', 'disable module imports in release scripting'],
      ['script_engine_limits_runaway_loops', 'test runaway loop aborts'],
      ['script_engine_limits_recursive_calls', 'test recursive call aborts'],
      ['script_engine_rejects_control_characters_before_execution', 'test shared control-character rejection'],
      ['script_engine_rejects_oversized_source_before_execution', 'test shared source size rejection'],
      ['script_engine_rejects_invalid_variable_names', 'test invalid script variable name rejection'],
      ['script_engine_rejects_invalid_flag_names', 'test invalid script flag name rejection'],
      ['load_state_rejects_invalid_keys', 'test invalid save-state key rejection'],
    ]
    for (const [needle, description] of engineRequirements) {
      if (!scriptingSource.includes(needle)) {
        issues.push(`Script engine limits must ${description}`)
      }
    }

    const callerRequirements = [
      [workflowSource, 'se.set_variable(name', 'validate workflow set_variable state keys through ScriptEngine'],
      [workflowSource, 'se.set_flag(name', 'validate workflow set_flag state keys through ScriptEngine'],
      [workflowSource, 'map_err(|e| e.to_string())?', 'return workflow script state key errors to callers'],
      [workflowSource, 'workflow_state_nodes_reject_invalid_state_keys', 'test workflow state key rejection'],
      [saveCommandSource, '.load_state(script_variables, save.flags.clone())', 'validate typed save-restored variables and flags as one state load'],
      [gameDialogueSource, 'normalize_script_state_key', 'validate legacy dialogue script state keys'],
      [gameDialogueSource, 'normalize_script_state_map', 'validate legacy dialogue loaded state maps'],
      [gameDialogueSource, 'dialogue_state_keys_reject_invalid_names', 'test legacy dialogue state key rejection'],
      [gameDialogueSource, 'choice_effects', 'inspect dialogue choice effects before committing the cursor'],
      [gameDialogueSource, 'select_choice_from', 'guard choice commits against a moved dialogue cursor'],
      [gameDialogueSource, 'available_choices', 'filter authored dialogue choices while preserving original indices'],
      [gameDialogueSource, 'resolve_conditional_nodes', 'skip false linear conditional nodes with cycle detection'],
      [gameDialogueSource, 'normalize_legacy_dialogue_script', 'preserve checked-in single-quoted dialogue script compatibility'],
      [gameDialogueSource, 'dialogue_conditions_filter_stable_choice_indices_and_skip_linear_nodes', 'test runtime condition and script-state behavior'],
      [gameDialogueSource, 'condition_and_script_failures_roll_back_dialogue_runtime_state', 'test failed dialogue conditions and scripts restore the prior cursor and state'],
      [scriptingSource, 'load_json_state', 'adapt JSON dialogue variables into the shared Rhai runtime'],
      [scriptingSource, 'json_state_round_trips_condition_and_script_values', 'test JSON script-state condition and mutation parity'],
      [dialogueCommandSource, 'resolve_dialogue_choice_relationship_targets', 'resolve all relationship targets before advancing dialogue'],
      [dialogueCommandSource, 'update_relationship("player", delta)', 'apply authored relationship deltas through CharacterManager clamping'],
      [dialogueCommandSource, 'dialogue_choices_apply_and_clamp_relationship_effects', 'test relationship choice effects and clamping'],
    ]
    for (const [source, needle, description] of callerRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Script state callers must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Script command verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Script command invariants OK')
  }

  async function verifyWorkflowCommandInvariants() {
    const issues = []
    const workflowSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'workflow.rs'), 'utf8')
    const workflowValidationSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'workflow_validation.rs'), 'utf8')
    const mainSource = await readFile(path.join(tauriAppDir, 'src', 'main.rs'), 'utf8')

    const workflowRequirements = [
      ['state.current_project_data_root().await', 'resolve workflow commands against the active project root'],
      ['workflow_path_in_project', 'resolve workflow files through a project-scoped path helper'],
      ['normalize_workflow_relative_path', 'normalize and validate workflow paths before file access'],
      ['workflow_validation_rejects_invalid_state_keys', 'test workflow validation rejects invalid state keys'],
      ['validate_condition_source', 'reuse shared condition expression validation'],
      ['workflow_condition_scope_variables', 'expose score and relationship context to workflow condition expressions'],
      ['workflow_validation_rejects_invalid_conditions', 'test workflow validation rejects invalid conditions'],
      ['workflow_validation_uses_project_event_catalog_and_character_scope', 'test workflow event ids and character scope against project catalogs'],
      ['workflow_condition_nodes_reject_invalid_payloads', 'test workflow condition nodes reject invalid payloads'],
      ['workflow_condition_nodes_can_read_preview_context', 'test condition nodes can branch on preview context'],
      ['checked_in_sakura_meeting_uses_relationship_condition_context', 'test checked-in relationship condition workflows execute'],
      ['WorkflowPreviewState', 'isolate desktop run-context preview state from persistent runtime state'],
      ['workflow_preview_script_engine', 'snapshot script state into desktop workflow previews'],
      ['run_context_preview_isolates_workflow_state_nodes', 'test desktop workflow previews do not persist state node effects'],
      ['prompt_guard::guard_workflow_story_output', 'finalize workflow LLM node output through the shared prompt guard'],
      ['workflow_llm_output_falls_back_when_guard_has_no_story_text', 'test workflow LLM guard-only output does not become story text'],
      ['workflow_branch_weights', 'normalize random branch weights before selecting workflow branches'],
      ['random_branch_uses_normalized_weights', 'test random branch execution uses normalized weights'],
      ['project_root.join("workflows")', 'scope workflow files to the project workflows directory'],
      ['Workflow paths cannot contain empty, current, or parent directory segments', 'reject traversal-shaped workflow paths'],
      ['Workflow paths must end with .json', 'limit workflow command file access to JSON workflow files'],
      ['tokio::fs::create_dir_all(parent)', 'create only the validated workflow parent directory before saving'],
      ['workflow_paths_resolve_under_project_workflows', 'test compatible project workflow path resolution'],
      ['workflow_paths_reject_escape_attempts', 'test workflow path traversal and absolute path rejection'],
      ['save_and_load_workflow_stay_inside_project_workflows', 'test workflow save/load containment under project workflows'],
      ['list_workflow_summaries', 'list only project-scoped loadable workflow files'],
      ['WORKFLOW_LIST_MAX_FILES', 'bound workflow file discovery'],
      ['WORKFLOW_LIST_MAX_DEPTH', 'bound recursive workflow discovery'],
      ['workflow_listing_is_sorted_scoped_and_skips_invalid_files', 'test sorted scoped workflow discovery'],
    ]
    const workflowDomainRequirements = [
      ['normalize_script_state_key', 'validate workflow state keys during workflow validation'],
      ['validate_workflow_state_keys', 'centralize workflow state-key config validation'],
      ['node_state_key_invalid', 'report invalid workflow state keys before execution'],
      ['validate_condition_source', 'reuse shared condition expression validation in the pure domain'],
      ['validate_workflow_condition', 'centralize workflow condition config validation'],
      ['node_condition_invalid', 'report invalid workflow conditions before execution'],
      ['load_project_workflows', 'load bounded project Workflow catalogs for Agent validation'],
      ['validate_workflow_references', 'validate Workflow cross-catalog references'],
    ]
    for (const [needle, description] of workflowRequirements) {
      if (!workflowSource.includes(needle)) {
        issues.push(`Workflow commands must ${description}`)
      }
    }
    for (const [needle, description] of workflowDomainRequirements) {
      if (!workflowValidationSource.includes(needle)) {
        issues.push(`Workflow domain must ${description}`)
      }
    }

    if (!mainSource.includes('commands::workflow::list_workflows')) {
      issues.push('Workflow commands must register list_workflows with the Tauri command handler')
    }

    if (issues.length > 0) {
      throw new Error(`Workflow command path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Workflow command path invariants OK')
  }

  async function verifyContentLoaderPathInvariants() {
    const issues = []
    const contentPathAdapterSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'content_paths.rs'), 'utf8')
    const sharedContentPathsSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'src', 'paths.rs'), 'utf8')
    const characterCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'characters.rs'), 'utf8')
    const knowledgeCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'knowledge.rs'), 'utf8')
    const dialogueCommandsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'dialogue.rs'), 'utf8')

    const adapterRequirements = [
      ['resolve_project_content_dir', 'centralize content loader directory resolution'],
      ['state.current_project_data_root().await', 'resolve content loader commands against the active project root'],
      ['use llm_authoring::paths::project_content_dir', 'delegate path policy to the transport-neutral authoring crate'],
      ['project_content_dir(&project_root, requested_dir, canonical_dir)', 'pass only the active root and requested catalog path to the shared resolver'],
    ]
    for (const [needle, description] of adapterRequirements) {
      if (!contentPathAdapterSource.includes(needle)) {
        issues.push(`Content loader path adapter must ${description}`)
      }
    }

    const sharedPathRequirements = [
      ['pub fn project_content_dir', 'expose one transport-neutral content path resolver'],
      ['project_root.join(canonical_dir)', 'scope content loaders to their canonical project content directories'],
      ['Content paths cannot contain drive prefixes or URI schemes', 'reject URI-like and drive-prefixed content paths'],
      ['Content paths cannot contain empty, current, or parent directory segments', 'reject traversal-shaped content paths'],
      ['path.starts_with(&root)', 'defensively prove content paths stay under the canonical content root'],
      ['content_dirs_resolve_canonical_and_nested_project_paths', 'test compatible project content path resolution'],
      ['content_dirs_reject_escape_attempts', 'test content directory traversal and absolute path rejection'],
    ]
    for (const [needle, description] of sharedPathRequirements) {
      if (!sharedContentPathsSource.includes(needle)) {
        issues.push(`Shared content path handling must ${description}`)
      }
    }

    const commandRequirements = [
      [characterCommandsSource, 'resolve_project_content_dir(&state, &directory, "characters")', 'scope character loading to project characters'],
      [knowledgeCommandsSource, 'resolve_project_content_dir(&state, &directory, "knowledge")', 'scope knowledge loading to project knowledge'],
      [dialogueCommandsSource, 'resolve_project_content_dir(&state, &directory, "dialogue")', 'scope dialogue loading to project dialogue'],
    ]
    for (const [source, needle, description] of commandRequirements) {
      if (!source.includes(needle)) {
        issues.push(`Content loader commands must ${description}`)
      }
      if (source.includes('PathBuf::from(&directory)')) {
        issues.push('Content loader commands must not turn frontend directory strings directly into filesystem paths')
      }
    }

    if (issues.length > 0) {
      throw new Error(`Content loader path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Content loader path invariants OK')
  }

  async function verifyAgentTransactionInvariants() {
    const issues = []
    const transactionSources = await Promise.all([
      'agent_transaction.rs',
      'agent_transaction/plan.rs',
      'agent_transaction/protocol.rs',
      'agent_transaction/tests.rs',
    ].map((relativePath) => readFile(path.join(rustDir, 'crates', 'authoring', 'src', relativePath), 'utf8')))
    const transactionSource = transactionSources.join('\n')
    const authoringCargoSource = await readFile(path.join(rustDir, 'crates', 'authoring', 'Cargo.toml'), 'utf8')
    const requirements = [
      ['monogatari-agent-project-transaction/v1', 'version Agent transaction requests'],
      ['monogatari-agent-project-transaction-plan/v1', 'version deterministic dry-run plans'],
      ['monogatari-agent-project-transaction-result/v1', 'version successful application results'],
      ['monogatari-agent-project-transaction-error/v1', 'version structured transaction errors'],
      ['deny_unknown_fields', 'reject unknown request and operation fields'],
      ['AgentFilePrecondition', 'require explicit file preconditions'],
      ['Missing', 'support create-only missing preconditions'],
      ['Sha256 { value: String }', 'support exact update/delete SHA-256 preconditions'],
      ['MAX_TRANSACTION_OPERATIONS', 'bound operation counts'],
      ['MAX_TRANSACTION_FILE_BYTES', 'bound individual JSON documents'],
      ['MAX_TRANSACTION_TOTAL_BYTES', 'bound aggregate transaction writes'],
      ['ALLOWED_JSON_ROOTS', 'allowlist authorable JSON catalog roots'],
      ['portable_path.to_ascii_lowercase()', 'reject duplicate targets across ASCII case'],
      ['reject_case_collision', 'reject collisions with existing case-variant paths'],
      ['stage_json_replacement', 'stage JSON replacements through the shared filesystem transaction'],
      ['stage_json_deletion', 'stage JSON deletions through the shared filesystem transaction'],
      ['rollback_staged(&mut staged).await?', 'roll back every staged operation on failure'],
      ['FnOnce(PathBuf) -> ValidationFuture', 'require a caller-supplied authoritative candidate validator'],
      ['backup_cleanup_failed', 'distinguish post-commit cleanup warnings from application failures'],
      ['plan_is_deterministic_and_does_not_write', 'test side-effect-free deterministic plans'],
      ['apply_commits_multiple_writes_and_deletion_after_validation', 'test validated multi-file commits'],
      ['validation_failure_rolls_back_every_staged_operation', 'test complete rollback after candidate rejection'],
    ]
    for (const [needle, description] of requirements) {
      if (!transactionSource.includes(needle)) {
        issues.push(`Agent transaction handling must ${description}`)
      }
    }
    if (/\btauri\b/i.test(authoringCargoSource)) {
      issues.push('llm-authoring must remain transport-neutral and cannot depend on Tauri')
    }

    if (issues.length > 0) {
      throw new Error(`Agent transaction verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Agent transaction invariants OK')
  }

  async function verifyMcpServerInvariants() {
    const issues = []
    const mcpRoot = path.join(rustDir, 'crates', 'mcp-server')
    const [
      mcpCargoSource,
      mcpLibSource,
      mcpMainSource,
      mcpProtocolSource,
      mcpServerSource,
      mcpValidationSource,
      mcpE2eSource,
      jsonCatalogSource,
      runtimeValidationSource,
    ] = await Promise.all([
      readFile(path.join(mcpRoot, 'Cargo.toml'), 'utf8'),
      readFile(path.join(mcpRoot, 'src', 'lib.rs'), 'utf8'),
      readFile(path.join(mcpRoot, 'src', 'main.rs'), 'utf8'),
      readFile(path.join(mcpRoot, 'src', 'protocol.rs'), 'utf8'),
      readFile(path.join(mcpRoot, 'src', 'server.rs'), 'utf8'),
      readFile(path.join(mcpRoot, 'src', 'validation.rs'), 'utf8'),
      readFile(path.join(mcpRoot, 'tests', 'stdio_e2e.rs'), 'utf8'),
      Promise.all([
        'json_catalog.rs',
        'json_catalog/inspect.rs',
        'json_catalog/protocol.rs',
        'json_catalog/read.rs',
        'json_catalog/tests.rs',
      ].map((relativePath) => readFile(path.join(rustDir, 'crates', 'authoring', 'src', relativePath), 'utf8')))
        .then((sources) => sources.join('\n')),
      Promise.all([
        'runtime_validation.rs',
        'runtime_validation/tests.rs',
      ].map((relativePath) => readFile(path.join(rustDir, 'crates', 'authoring', 'src', relativePath), 'utf8')))
        .then((sources) => sources.join('\n')),
    ])
    const requirements = [
      [mcpCargoSource, 'rmcp = { version = "2.2.0"', 'use the pinned official Rust MCP SDK'],
      [mcpCargoSource, '"transport-io"', 'enable the SDK stdio transport'],
      [mcpCargoSource, '"transport-child-process"', 'exercise the server through a real child-process client'],
      [mcpLibSource, 'rmcp::transport::stdio()', 'serve MCP through the SDK stdio transport'],
      [mcpMainSource, '.with_writer(std::io::stderr)', 'reserve stdout exclusively for MCP frames'],
      [mcpServerSource, 'canonical_project_root(&project_root)', 'bind one canonical project root at startup'],
      [mcpServerSource, 'pub async fn inspect_project', 'expose project inspection'],
      [mcpServerSource, 'pub async fn list_project_json', 'expose bounded JSON catalog listing'],
      [mcpServerSource, 'pub async fn read_project_json', 'expose exact JSON document reads'],
      [mcpServerSource, 'pub async fn plan_transaction', 'expose side-effect-free transaction planning'],
      [mcpServerSource, 'pub async fn apply_transaction', 'expose validated transaction application'],
      [mcpServerSource, 'if !self.allow_write', 'keep writes disabled unless startup explicitly enables them'],
      [mcpProtocolSource, 'expected_precondition_fingerprint', 'require the caller to confirm the reviewed plan fingerprint'],
      [mcpServerSource, 'self.access.write().await', 'serialize reads against staged write candidates'],
      [mcpServerSource, 'std::fs::File::try_lock(&file)', 'exclude concurrent server processes while a writer owns the project'],
      [mcpServerSource, 'std::fs::File::try_lock_shared(&file)', 'share the project lease only between read-only server processes'],
      [mcpServerSource, 'apply_agent_project_transaction_with_validator', 'delegate writes to the shared rollback-capable authoring core'],
      [mcpValidationSource, 'validate_core_runtime_project', 'delegate candidate acceptance to the shared headless runtime validator'],
      [runtimeValidationSource, 'JsonAcceptanceLevel::CoreRuntime', 'label candidate acceptance as core-runtime rather than full-project validation'],
      [runtimeValidationSource, 'CharacterManager::new()', 'load candidate characters through the real runtime manager'],
      [runtimeValidationSource, 'DialogueManager::new()', 'load candidate dialogue graphs through the real runtime manager'],
      [runtimeValidationSource, 'KnowledgeBase::new()', 'load candidate knowledge through the real runtime manager'],
      [runtimeValidationSource, 'validate_character_references', 'validate character relationship and knowledge references'],
      [runtimeValidationSource, 'validate_dialogue_references', 'validate dialogue speaker and relationship references'],
      [runtimeValidationSource, 'rejects_duplicate_runtime_ids_instead_of_accepting_loader_overwrites', 'test duplicate runtime IDs are rejected'],
      [jsonCatalogSource, 'monogatari-json-catalog-report/v1', 'version shared JSON catalog reports'],
      [jsonCatalogSource, 'MAX_AUTHORABLE_JSON_BYTES', 'bound JSON reads and transaction writes consistently'],
      [jsonCatalogSource, 'verify_exact_path', 'require exact-case paths before reads'],
      [jsonCatalogSource, 'content_fingerprint', 'publish semantic fingerprints separately from exact file SHA-256'],
      [mcpE2eSource, 'real_stdio_handshake_lists_and_reads_schema_backed_tools', 'test real stdio initialization, discovery, and reads'],
      [mcpE2eSource, 'readonly_stdio_plans_but_structurally_rejects_apply', 'test default read-only refusal without filesystem changes'],
      [mcpE2eSource, 'writable_stdio_requires_reviewed_fingerprint_and_rolls_back_invalid_candidate', 'test fingerprint confirmation, application, and rollback'],
      [mcpE2eSource, 'runtime_reference_rollback', 'test real stdio rollback after core-runtime reference rejection'],
    ]
    for (const [source, needle, description] of requirements) {
      if (!source.includes(needle)) {
        issues.push(`MCP Agent authoring must ${description}`)
      }
    }
    if (/\btauri\b/i.test(mcpCargoSource)) {
      issues.push('monogatari-mcp must remain independent of the Tauri command crate')
    }
    if (/pub\s+project_root\s*:/.test(mcpProtocolSource)) {
      issues.push('MCP tool requests must not be able to replace the startup-bound project root')
    }

    if (issues.length > 0) {
      throw new Error(`MCP Agent authoring verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] MCP Agent authoring invariants OK')
  }

  async function verifyCharacterManagerPathInvariants() {
    const issues = []
    const characterManagerSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'character_manager.rs'), 'utf8')
    const gameCharacterSource = await readFile(path.join(rustDir, 'crates', 'game', 'src', 'characters', 'character.rs'), 'utf8')

    const characterManagerRequirements = [
      ['state.current_project_data_root().await', 'resolve character authoring against the active or discovered default project root'],
      ['character_file_path', 'centralize character JSON file path construction'],
      ['normalize_character_id', 'validate character ids before path construction'],
      ['project_root.join("characters")', 'scope character JSON files to the project characters directory'],
      ['Character ids can contain only ASCII letters, numbers, underscores, or hyphens', 'reject path-shaped and non-portable character ids'],
      ['path.parent() != Some(root.as_path())', 'prove character JSON files stay directly under project characters'],
      ['cm.remove_character(&id)', 'remove deleted characters from the in-memory manager'],
      ['character_file_paths_stay_inside_project_characters', 'test compatible character file path resolution'],
      ['character_file_paths_reject_escape_attempts', 'test traversal and absolute character id rejection'],
    ]
    for (const [needle, description] of characterManagerRequirements) {
      if (!characterManagerSource.includes(needle)) {
        issues.push(`Character manager path handling must ${description}`)
      }
    }

    if (!gameCharacterSource.includes('pub fn remove_character(&mut self, id: &str) -> bool')) {
      issues.push('Game CharacterManager must support removing deleted characters from runtime state')
    }

    if (characterManagerSource.includes('dir.join(format!("{id}.json"))') || characterManagerSource.includes('dir.join(format!("{character_id}.json"))')) {
      issues.push('Character manager commands must not build character JSON paths directly from raw command input')
    }
    if (characterManagerSource.includes('No project path configured.')) {
      issues.push('Character manager commands must not fail before trying the default project data root')
    }

    if (issues.length > 0) {
      throw new Error(`Character manager path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Character manager path invariants OK')
  }

  async function verifyPluginManagerPathInvariants() {
    const issues = []
    const pluginSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'plugin.rs'), 'utf8')
    const pluginViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'PluginView.vue'), 'utf8')

    const pluginRequirements = [
      ['state.current_project_data_root().await', 'resolve plugin management against the active or discovered default project root'],
      ['plugin_file_path', 'centralize plugin JSON file path construction'],
      ['normalize_plugin_id', 'validate plugin ids before path construction'],
      ['normalize_plugin_manifest', 'normalize plugin manifests before writing them'],
      ['normalize_plugin_script_path', 'normalize optional plugin script paths before writing or listing manifests'],
      ['project_root.join("plugins")', 'scope plugin JSON files to the project plugins directory'],
      ['Plugin ids can contain only ASCII letters, numbers, underscores, or hyphens', 'reject path-shaped and non-portable plugin ids'],
      ['Plugin script paths must be relative files under project plugins', 'reject absolute, URI, and drive-shaped plugin script paths'],
      ['Plugin script paths cannot contain empty, current, or parent directory segments', 'reject traversal-shaped plugin script paths'],
      ['Plugin script paths must end in .rhai', 'limit plugin script references to Rhai files'],
      ['path.parent() != Some(root.as_path())', 'prove plugin JSON files stay directly under project plugins'],
      ['manifest.id == file_id', 'skip listed plugin manifests that do not match their file name'],
      ['plugin_file_paths_stay_inside_project_plugins', 'test compatible plugin file path resolution'],
      ['plugin_file_paths_reject_escape_attempts', 'test traversal and absolute plugin id rejection'],
      ['plugin_manifest_normalization_fills_defaults_and_safe_ids', 'test plugin manifest normalization defaults'],
      ['plugin_script_paths_reject_escape_attempts', 'test traversal and absolute plugin script path rejection'],
    ]
    for (const [needle, description] of pluginRequirements) {
      if (!pluginSource.includes(needle)) {
        issues.push(`Plugin manager path handling must ${description}`)
      }
    }

    if (pluginSource.includes('dir.join(format!("{}.json", manifest.id))') || pluginSource.includes('dir.join(format!("{plugin_id}.json"))')) {
      issues.push('Plugin manager commands must not build plugin JSON paths directly from raw command input')
    }
    if (pluginSource.includes('No project path configured.')) {
      issues.push('Plugin manager commands must not fail before trying the default project data root')
    }

    const pluginViewRequirements = [
      ['interface PluginManifest', 'type plugin manifests with the backend contract'],
      ['pluginManifestPayload()', 'send a complete plugin manifest payload when registering'],
      ["invokeCommand<void>('register_plugin', { manifest: pluginManifestPayload() })", 'wrap plugin registration args with manifest'],
      ["invokeCommand<void>('remove_plugin', { pluginId: id })", 'use pluginId when removing plugins'],
      ['removePlugin(plugin.id, plugin.name)', 'remove plugins by id rather than display name'],
    ]
    for (const [needle, description] of pluginViewRequirements) {
      if (!pluginViewSource.includes(needle)) {
        issues.push(`Plugin workbench must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Plugin manager path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Plugin manager path invariants OK')
  }

  async function verifyMarketplacePathInvariants() {
    const issues = []
    const marketplaceSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'marketplace.rs'), 'utf8')
    const marketplaceViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'MarketplaceView.vue'), 'utf8')

    const marketplaceRequirements = [
      ['template_dir_in_project', 'centralize marketplace template directory resolution'],
      ['normalize_template_ref', 'normalize and validate marketplace template references'],
      ['project_root.join("templates")', 'scope marketplace templates to the project templates directory'],
      ['Marketplace template references cannot contain drive prefixes or URI schemes', 'reject URI-like and drive-prefixed template references'],
      ['Marketplace template references cannot contain empty, current, or parent directory segments', 'reject traversal-shaped template references'],
      ['path.starts_with(&root)', 'prove marketplace template paths stay under project templates'],
      ['export_template_to_project', 'reuse the guarded project template exporter'],
      ['import_template_from_project', 'reuse the guarded project template importer'],
      ['marketplace_catalog_manifest(&template_id)', 'allow built-in catalog entries to import by safe id'],
      ['marketplace_template_dirs_resolve_under_project_templates', 'test compatible marketplace template path resolution'],
      ['marketplace_template_dirs_reject_escape_attempts', 'test marketplace traversal and absolute path rejection'],
      ['export_template_writes_manifest_inside_project_templates', 'test template export containment'],
      ['import_template_reads_project_manifest_or_catalog_entry', 'test guarded project import and catalog fallback'],
    ]
    for (const [needle, description] of marketplaceRequirements) {
      if (!marketplaceSource.includes(needle)) {
        issues.push(`Marketplace template handling must ${description}`)
      }
    }

    if (marketplaceSource.includes('PathBuf::from(&output_path)') || marketplaceSource.includes('PathBuf::from(&template_path)')) {
      issues.push('Marketplace commands must not turn frontend template strings directly into filesystem paths')
    }

    const marketplaceViewRequirements = [
      ["invokeCommand('import_template', { templatePath: entry.id })", 'import marketplace entries by catalog id rather than raw filesystem path'],
    ]
    for (const [needle, description] of marketplaceViewRequirements) {
      if (!marketplaceViewSource.includes(needle)) {
        issues.push(`Marketplace workbench must ${description}`)
      }
    }

    if (issues.length > 0) {
      throw new Error(`Marketplace path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Marketplace path invariants OK')
  }

  async function verifyLive2dPathInvariants() {
    const issues = []
    const live2dSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'live2d.rs'), 'utf8')
    const rendererAssetsSource = await readFile(path.join(frontendDir, 'src', 'lib', 'rendererAssets.ts'), 'utf8')
    const gameViewSource = await readFile(path.join(frontendDir, 'src', 'views', 'GameView.vue'), 'utf8')

    const live2dRequirements = [
      ['live2d_model_path_in_project', 'centralize Live2D model path resolution'],
      ['normalize_live2d_model_ref', 'normalize and validate Live2D model references'],
      ['current_project_data_root', 'resolve Live2D models under the active project data root'],
      ['Live2D model paths cannot contain drive prefixes or URI schemes', 'reject URI-like and drive-prefixed model paths'],
      ['Live2D model paths cannot contain empty, current, or parent directory segments', 'reject traversal-shaped model paths'],
      ['Live2D model paths must point to a .model3.json or .json file', 'restrict Live2D command loading to model JSON files'],
      ['path.starts_with(project_root)', 'prove Live2D paths stay under the project root before filesystem access'],
      ['canonical_model.starts_with(&canonical_root)', 'prove canonical Live2D paths stay under the project root'],
      ['load_live2d_model_from_project', 'reuse the guarded project Live2D loader'],
      ['live2d_model_paths_resolve_under_project_root', 'test compatible Live2D model path resolution'],
      ['live2d_model_paths_reject_escape_attempts', 'test Live2D traversal and absolute path rejection'],
      ['load_live2d_model_reads_project_model_sidecars', 'test guarded model loading and sidecar discovery'],
    ]
    for (const [needle, description] of live2dRequirements) {
      if (!live2dSource.includes(needle)) {
        issues.push(`Live2D command path handling must ${description}`)
      }
    }

    if (live2dSource.includes('PathBuf::from(&model_path)') || live2dSource.includes('std::path::PathBuf::from(&model_path)')) {
      issues.push('Live2D commands must not turn frontend model strings directly into filesystem paths')
    }

    const rendererRequirements = [
      ['Path segments must be portable', 'reject empty, current, and non-portable renderer asset segments'],
      ['^[a-zA-Z][a-zA-Z0-9+.-]*:', 'reject URI-like renderer asset paths before resolution'],
    ]
    for (const [needle, description] of rendererRequirements) {
      if (!rendererAssetsSource.includes(needle)) {
        issues.push(`Renderer asset validation must ${description}`)
      }
    }

    if (!gameViewSource.includes('validatePaths: true')) {
      issues.push('Story Mode renderer selection must validate project-relative asset paths before rendering')
    }

    if (issues.length > 0) {
      throw new Error(`Live2D path verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] Live2D model path invariants OK')
  }

  async function verifyTtsOutputInvariants() {
    const issues = []
    const ttsSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'tts.rs'), 'utf8')

    const ttsRequirements = [
      ['tts_output_path', 'centralize generated TTS output path construction'],
      ['safe_tts_file_component', 'sanitize character/provider names before building filenames'],
      ['path.parent() != Some(output_dir.as_path())', 'prove generated TTS files stay directly under assets/tts'],
      ['write_tts_output_bytes', 'reuse the guarded output writer for API provider bytes'],
      ['tts_output_path(&project_root, "azure"', 'write Azure provider output under the active project root'],
      ['tts_output_path(&project_root, "elevenlabs"', 'write ElevenLabs provider output under the active project root'],
      ['tts_output_path(&project_root, "system"', 'write system provider output under the active project root'],
      ['redact_tts_error_text', 'redact TTS provider error surfaces'],
      ['tts_provider_error_message', 'redact non-success provider response bodies'],
      ['tts_text_log_summary', 'summarize spoken TTS text before logging'],
      ['tts_failure_redacts_error_surface', 'test final TTS error surface redaction'],
      ['redacts_tts_provider_error_text', 'test TTS provider secret redaction helpers'],
      ['tts_text_log_summary_omits_spoken_content', 'test TTS synthesis logs omit raw spoken content'],
      ['tts_output_path_sanitizes_character_ids_and_stays_in_project_assets', 'test sanitized character ids cannot escape assets/tts'],
      ['api_provider_tts_outputs_are_project_scoped', 'test API provider output paths are project-scoped'],
      ['tts_output_path_rejects_unsupported_extensions', 'test unsupported generated audio extensions are rejected'],
    ]
    for (const [needle, description] of ttsRequirements) {
      if (!ttsSource.includes(needle)) {
        issues.push(`TTS output handling must ${description}`)
      }
    }
    if (ttsSource.includes('std::env::temp_dir()')) {
      issues.push('TTS output handling must not write provider audio to the process temp directory')
    }
    if (ttsSource.includes('monogatari_tts_')) {
      issues.push('TTS output handling must avoid fixed global provider output filenames')
    }
    if (ttsSource.includes('TTS synthesis for {}: \\"{}\\"')) {
      issues.push('TTS synthesis logs must not include raw spoken text')
    }

    if (issues.length > 0) {
      throw new Error(`TTS output/error verification failed:\n${issues.join('\n')}`)
    }

    console.log('[release] TTS output/error invariants OK')
  }

  return {
    verifyFrontendSourceInvariants,
    verifyLegacyPromptBuilderInvariants,
    verifyAiBackendConfigInvariants,
    verifyEngineProjectRootInvariants,
    verifyAssetManagerInvariants,
    verifySaveManagerInvariants,
    verifyScriptCommandInvariants,
    verifyWorkflowCommandInvariants,
    verifyContentLoaderPathInvariants,
    verifyAgentTransactionInvariants,
    verifyMcpServerInvariants,
    verifyCharacterManagerPathInvariants,
    verifyPluginManagerPathInvariants,
    verifyMarketplacePathInvariants,
    verifyLive2dPathInvariants,
    verifyTtsOutputInvariants,
  }
}
