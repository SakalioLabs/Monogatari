import { spawn } from 'node:child_process'
import { readdir, readFile, stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { createSourceInvariantVerifier } from './lib/source-invariant-verifier.mjs'
import { frontendRouteCoverageEvidence } from './lib/frontend-route-verifier.mjs'
import { createLocaleCoveragePolicy, localeMessages } from './lib/locale-coverage-policy.mjs'
import { createProjectDialoguePolicy } from './lib/project-content/dialogue-policy.mjs'
import { createProjectKnowledgeReferencePolicy } from './lib/project-content/knowledge-reference-policy.mjs'
import { createProjectQualitySuitePolicy } from './lib/project-content/quality-suite-policy.mjs'
import { createProjectRendererAssetPolicy } from './lib/project-content/renderer-asset-policy.mjs'
import { createProjectStoryEventPolicy } from './lib/project-content/story-event-policy.mjs'
import { createProjectWorkflowPolicy } from './lib/project-content/workflow-policy.mjs'
import { releaseChannelPolicyIssues } from './lib/release-channel-policy-verifier.mjs'
import { createRepositoryFileWalker } from './lib/repository-file-walker.mjs'
import { createRepositoryJsonPolicy } from './lib/repository-json-policy.mjs'
import { createRepositoryTextPolicy } from './lib/repository-text-policy.mjs'
import { createTauriPackagingVerifier } from './lib/tauri-packaging-verifier.mjs'
import {
  createWebDistributionVerifier,
  releaseSubpathBase,
  requiredLocaleFiles as requiredLocales,
} from './lib/web-distribution-verifier.mjs'
import {
  createWebPreviewVerifier,
} from './lib/web-preview-verifier.mjs'
import {
  extractHtmlCsp,
  requiredWebCspFragments,
  verifyCspPolicy,
} from './lib/web-hosting-verifier.mjs'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const frontendDir = path.join(root, 'frontend')
const rustDir = path.join(root, 'rust-engine')
const tauriAppDir = path.join(rustDir, 'crates', 'tauri-app')
const releasePolicyPath = path.join(root, 'scripts', 'release-channel-policy.json')

const rendererDataRoots = [
  { label: 'data', dir: path.join(root, 'data') },
  { label: 'rust-engine/data', dir: path.join(rustDir, 'data') },
]
const releaseCriticalRustFiles = [
  'crates/core/src/state_key.rs',
  'crates/ai/src/api_engine.rs',
  'crates/ai/src/prompt_builder.rs',
  'crates/assets/src/asset_manager.rs',
  'crates/assets/src/save_manager.rs',
  'crates/game/src/dialogue/dialogue_manager.rs',
  'crates/game/src/dialogue/dialogue_node.rs',
  'crates/game/src/dialogue/dialogue_script.rs',
  'crates/scripting/src/lib.rs',
  'crates/authoring/src/filesystem.rs',
  'crates/authoring/src/paths.rs',
  'crates/authoring/src/project.rs',
  'crates/authoring/src/project_package.rs',
  'crates/authoring/src/project_package/archive_reader.rs',
  'crates/authoring/src/project_package/archive_reader/tests.rs',
  'crates/authoring/src/project_package/archive_writer.rs',
  'crates/authoring/src/project_package/archive_writer/tests.rs',
  'crates/authoring/src/project_package/export.rs',
  'crates/authoring/src/project_package/manifest.rs',
  'crates/authoring/src/project_package/portable_path.rs',
  'crates/authoring/src/agent_transaction.rs',
  'crates/authoring/src/agent_transaction/plan.rs',
  'crates/authoring/src/agent_transaction/protocol.rs',
  'crates/authoring/src/agent_transaction/tests.rs',
  'crates/authoring/src/json_catalog.rs',
  'crates/authoring/src/json_catalog/inspect.rs',
  'crates/authoring/src/json_catalog/protocol.rs',
  'crates/authoring/src/json_catalog/read.rs',
  'crates/authoring/src/json_catalog/tests.rs',
  'crates/mcp-server/src/cli.rs',
  'crates/mcp-server/src/lib.rs',
  'crates/mcp-server/src/main.rs',
  'crates/mcp-server/src/package_transport.rs',
  'crates/mcp-server/src/package_transport/reimport.rs',
  'crates/mcp-server/src/protocol.rs',
  'crates/mcp-server/src/provenance.rs',
  'crates/mcp-server/src/server.rs',
  'crates/mcp-server/src/validation.rs',
  'crates/mcp-server/tests/stdio_e2e.rs',
  'crates/tauri-app/src/main.rs',
  'crates/tauri-app/src/installation_verifier.rs',
  'crates/tauri-app/src/state.rs',
  'crates/tauri-app/src/story_access.rs',
  'crates/tauri-app/src/story_events.rs',
  'crates/tauri-app/src/story_progress.rs',
  'crates/tauri-app/src/content_references.rs',
  'crates/tauri-app/src/commands/ai.rs',
  'crates/tauri-app/src/commands/engine.rs',
  'crates/tauri-app/src/commands/endings.rs',
  'crates/tauri-app/src/commands/project.rs',
  'crates/tauri-app/src/commands/project_archive.rs',
  'crates/tauri-app/src/commands/project_archive/commands.rs',
  'crates/tauri-app/src/commands/project_archive/tests.rs',
  'crates/tauri-app/src/commands/scenes.rs',
  'crates/tauri-app/src/commands/analytics.rs',
  'crates/tauri-app/src/commands/cloud_sync.rs',
  'crates/tauri-app/src/commands/tts.rs',
  'crates/game/src/characters/character.rs',
  'crates/game/src/knowledge/knowledge_base.rs',
  'crates/game/src/knowledge/knowledge_entry.rs',
  'crates/tauri-app/src/commands/character_manager.rs',
  'crates/tauri-app/src/commands/characters.rs',
  'crates/tauri-app/src/commands/chat.rs',
  'crates/tauri-app/src/commands/multi_chat.rs',
  'crates/tauri-app/src/commands/content_paths.rs',
  'crates/tauri-app/src/commands/dialogue.rs',
  'crates/tauri-app/src/commands/i18n.rs',
  'crates/tauri-app/src/commands/knowledge.rs',
  'crates/tauri-app/src/commands/live2d.rs',
  'crates/tauri-app/src/commands/marketplace.rs',
  'crates/tauri-app/src/commands/plugin.rs',
  'crates/authoring/src/conversation_quality.rs',
  'crates/authoring/src/prompt_guard.rs',
  'crates/authoring/src/quality_suite_execution.rs',
  'crates/authoring/src/quality_suite_execution/tests.rs',
  'crates/authoring/src/workflow_preview.rs',
  'crates/authoring/src/workflow_preview/tests.rs',
  'crates/tauri-app/src/commands/prompt_guard.rs',
  'crates/tauri-app/src/commands/quality_suite.rs',
  'crates/tauri-app/src/commands/script.rs',
  'crates/tauri-app/src/commands/story_events.rs',
  'crates/tauri-app/src/commands/workflow.rs',
]

const walkFiles = createRepositoryFileWalker()
const {
  frontendSourceExtensions,
  verifySensitivePatterns,
  verifyUiTextArtifacts,
} = createRepositoryTextPolicy({
  repositoryRoot: root,
  frontendDirectory: frontendDir,
  walkFiles,
  relativePath: relative,
})
const { verifyLocaleCoverage } = createLocaleCoveragePolicy({
  repositoryRoot: root,
  frontendDirectory: frontendDir,
  requiredLocaleFiles: requiredLocales,
})
const { verifyRepositoryJsonFiles } = createRepositoryJsonPolicy({
  repositoryRoot: root,
  walkFiles,
})

const {
  verifyFrontendSourceInvariants,
  verifyLegacyPromptBuilderInvariants,
  verifyLegacyRendererInvariants,
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
} = createSourceInvariantVerifier({
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
})

const verifyWebDist = createWebDistributionVerifier({
  repositoryRoot: root,
  frontendDirectory: frontendDir,
})
const verifyWebPreview = createWebPreviewVerifier({ frontendDirectory: frontendDir })
const verifyTauriPackagingConfig = createTauriPackagingVerifier({
  repositoryRoot: root,
  frontendDirectory: frontendDir,
  rustDirectory: rustDir,
  tauriAppDirectory: tauriAppDir,
})
const projectStoryEventPolicy = createProjectStoryEventPolicy({
  repositoryRoot: root,
  rustDirectory: rustDir,
  dataRoots: rendererDataRoots,
})
const { verifyStoryEventCatalogs } = projectStoryEventPolicy
const { verifyDialogueCatalogs } = createProjectDialoguePolicy({
  repositoryRoot: root,
  rustDirectory: rustDir,
  dataRoots: rendererDataRoots,
})
const { verifyKnowledgeReferences } = createProjectKnowledgeReferencePolicy({
  repositoryRoot: root,
  rustDirectory: rustDir,
  dataRoots: rendererDataRoots,
})
const { verifyRendererAssets } = createProjectRendererAssetPolicy({
  repositoryRoot: root,
  rustDirectory: rustDir,
  dataRoots: rendererDataRoots,
})
const { verifyWorkflowFiles } = createProjectWorkflowPolicy({
  repositoryRoot: root,
  rustDirectory: rustDir,
  dataRoots: rendererDataRoots,
  storyEventPolicy: projectStoryEventPolicy,
})
const { verifyQualitySuites } = createProjectQualitySuitePolicy({
  repositoryRoot: root,
  rustDirectory: rustDir,
  dataRoots: rendererDataRoots,
  storyEventPolicy: projectStoryEventPolicy,
})

async function main() {
  const started = Date.now()
  console.log('[release] Starting Monogatari release verification')

  await verifyRepositoryJsonFiles()
  await verifyStoryEventCatalogs()
  await verifyDialogueCatalogs()
  await verifyWorkflowFiles()
  await verifyRendererAssets()
  await verifyKnowledgeReferences()
  await verifyQualitySuites()
  await verifySensitivePatterns()
  await verifyUiTextArtifacts()
  await verifyLocaleCoverage()
  await verifyI18nLocalePathInvariants()
  await verifyFrontendSourceInvariants()
  await verifyLegacyPromptBuilderInvariants()
  await verifyLegacyRendererInvariants()
  await verifyAiBackendConfigInvariants()
  await verifyEngineProjectRootInvariants()
  await verifyAssetManagerInvariants()
  await verifySaveManagerInvariants()
  await verifyScriptCommandInvariants()
  await verifyWorkflowCommandInvariants()
  await verifyContentLoaderPathInvariants()
  await verifyAgentTransactionInvariants()
  await verifyMcpServerInvariants()
  await verifyCharacterManagerPathInvariants()
  await verifyPluginManagerPathInvariants()
  await verifyMarketplacePathInvariants()
  await verifyLive2dPathInvariants()
  await verifyTtsOutputInvariants()
  await verifyFrontendRouteCoverage()
  await verifyTauriPackagingConfig()
  await verifyReleaseChannelPolicy()

  await run('git diff whitespace check', 'git', ['diff', '--check'], root)
  await run('Automation contract tests', 'node', ['--test', 'scripts/tests/automation-contracts.test.mjs'], root)
  await run('Built-in project mirror parity', 'node', ['scripts/sync-project-mirror.mjs', '--check'], root)
  await run('Frontend i18n coverage', 'npm', ['run', 'verify:i18n'], frontendDir)
  await run('Frontend renderer asset selector contract', 'npm', ['run', 'verify:renderer-assets'], frontendDir)
  await run('Frontend mobile shell readiness', 'npm', ['run', 'verify:mobile-readiness'], frontendDir)
  await run('Frontend authoring browser workflows', 'npm', ['run', 'test:e2e'], frontendDir)
  await run('Tauri mobile deployment preflight', 'node', ['scripts/verify-tauri-mobile-preflight.mjs'], root)
  await run('Release-critical Rust format check', 'rustfmt', ['--edition', '2021', '--check', ...releaseCriticalRustFiles], rustDir)
  await run('Rust core tests', 'cargo', ['test', '--locked', '-p', 'llm-core'], rustDir)
  await run('Rust headless authoring tests', 'cargo', ['test', '--locked', '-p', 'llm-authoring'], rustDir)
  await run('Rust MCP stdio and authoring tests', 'cargo', ['test', '--locked', '-p', 'monogatari-mcp'], rustDir)
  await run('Rust MCP release binary build', 'cargo', ['build', '--locked', '--release', '-p', 'monogatari-mcp'], rustDir)
  await run('Rust AI prompt and pipeline tests', 'cargo', ['test', '--locked', '-p', 'llm-ai'], rustDir)
  await run('Rust asset management tests', 'cargo', ['test', '--locked', '-p', 'llm-assets'], rustDir)
  await run('Rust scripting tests', 'cargo', ['test', '--locked', '-p', 'llm-scripting'], rustDir)
  await run('Rust game tests', 'cargo', ['test', '--locked', '-p', 'llm-game'], rustDir)
  await run('Rust Tauri command tests', 'cargo', ['test', '--locked', '-p', 'llm-galgame-app'], rustDir, {
    env: { CARGO_INCREMENTAL: '0' },
  })
  await run('Rust Tauri app check', 'cargo', ['check', '--locked', '-p', 'llm-galgame-app'], rustDir)
  await run('Rust workspace Clippy', 'cargo', ['clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings'], rustDir)
  await run(
    'Frontend audit',
    'npm',
    ['audit', '--audit-level=moderate', '--fetch-retries=5', '--fetch-retry-mintimeout=5000', '--fetch-retry-maxtimeout=60000'],
    frontendDir,
    {
      retries: 5,
      retryDelayMs: 5000,
    },
  )
  await run('Frontend Web/PWA subpath build', 'npm', ['run', 'build:web'], frontendDir, {
    env: { VITE_BASE_PATH: releaseSubpathBase },
  })
  await verifyWebDist({ basePath: releaseSubpathBase })
  await verifyWebPreview({ basePath: releaseSubpathBase, env: { VITE_BASE_PATH: releaseSubpathBase } })
  await run('Frontend Web/PWA build', 'npm', ['run', 'build:web'], frontendDir)
  await verifyWebDist({ basePath: '/' })
  await verifyWebPreview({ basePath: '/' })
  await verifyWindowsInstallersIfPresent()
  await run('Release artifact manifest check', 'node', ['scripts/create-release-manifest.mjs', '--check', '--allow-missing-installers'], root)
  if (process.platform === 'win32') {
    await run(
      'Legacy SDL2 runtime preparation',
      'powershell',
      ['-NoLogo', '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'scripts/prepare-legacy-sdl.ps1'],
      root,
    )
    await run('Legacy C# warnings-as-errors build', 'dotnet', ['build', 'LLMAssistant.sln', '--no-restore', '-warnaserror'], root)
  }
  await run('Legacy C# tests', 'dotnet', ['test', 'LLMAssistant.sln', '--no-restore'], root)

  const elapsed = ((Date.now() - started) / 1000).toFixed(1)
  console.log(`[release] Verification passed in ${elapsed}s`)
}

async function run(label, command, args, cwd, options = {}) {
  const attempts = (options.retries ?? 0) + 1
  let lastError

  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    console.log(`\n[release] ${label}${attempts > 1 ? ` (attempt ${attempt}/${attempts})` : ''}`)
    try {
      await runOnce(label, command, args, cwd, options)
      return
    } catch (error) {
      lastError = error
      if (attempt >= attempts) break
      console.warn(`[release] ${label} attempt ${attempt} failed: ${error.message}`)
      await delay(options.retryDelayMs ?? 1000)
    }
  }

  throw lastError
}

async function runOnce(label, command, args, cwd, options = {}) {
  await new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd,
      shell: process.platform === 'win32',
      stdio: 'inherit',
      env: { ...process.env, ...(options.env ?? {}) },
    })
    child.on('error', reject)
    child.on('exit', (code) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`${label} failed with exit code ${code}`))
      }
    })
  })
}

async function verifyWindowsInstallersIfPresent() {
  if (process.platform !== 'win32') return

  const config = JSON.parse(await readFile(path.join(tauriAppDir, 'tauri.conf.json'), 'utf8'))
  const bundleDir = path.join(rustDir, 'target', 'release', 'bundle')
  const expectedInstallers = [
    path.join(bundleDir, 'msi', `Monogatari_${config.version}_x64_en-US.msi`),
    path.join(bundleDir, 'nsis', `Monogatari_${config.version}_x64-setup.exe`),
  ]
  const presence = await Promise.all(expectedInstallers.map(fileExists))
  if (!presence.some(Boolean)) return

  const policy = JSON.parse(await readFile(releasePolicyPath, 'utf8'))
  const channel = (process.env.MONOGATARI_RELEASE_CHANNEL ?? 'stable').trim().toLowerCase()
  const channelPolicy = policy.channels?.[channel]
  if (!channelPolicy) {
    throw new Error(`Unknown MONOGATARI_RELEASE_CHANNEL: ${channel}`)
  }

  const installerArgs = ['scripts/verify-windows-installers.mjs', '--check']
  if (channelPolicy.preflight?.allow_unsigned_installers === true) {
    installerArgs.push('--allow-unsigned')
  }
  await run(`Windows installer audit (${channel})`, 'node', installerArgs, root)
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function verifyI18nLocalePathInvariants() {
  const issues = []
  const i18nCommandSource = await readFile(path.join(tauriAppDir, 'src', 'commands', 'i18n.rs'), 'utf8')

  const i18nRequirements = [
    ['locale_file_path', 'centralize locale JSON file path construction'],
    ['normalize_locale_id', 'validate locale ids before path construction'],
    ['project_root.join("locales")', 'scope locale JSON files to the project locales directory'],
    ['Locale ids can contain only ASCII letters, numbers, and hyphen separators', 'reject path-shaped and non-portable locale ids'],
    ['path.parent() != Some(root.as_path())', 'prove locale JSON files stay directly under project locales'],
    ['load_locale_from_project', 'reuse guarded project locale loading'],
    ['list_locale_ids', 'filter listed locale files through the same validator'],
    ['translate_from_project', 'reuse guarded project locale translation'],
    ['locale_file_paths_stay_inside_project_locales', 'test compatible locale file path resolution'],
    ['locale_file_paths_reject_escape_attempts', 'test locale traversal and absolute path rejection'],
    ['locale_loading_lists_and_translates_safe_locale_ids', 'test safe locale loading, listing, and translation'],
  ]
  for (const [needle, description] of i18nRequirements) {
    if (!i18nCommandSource.includes(needle)) {
      issues.push(`i18n locale path handling must ${description}`)
    }
  }

  if (i18nCommandSource.includes('format!("{locale}.json")') || i18nCommandSource.includes('format!("{loc}.json")')) {
    issues.push('i18n commands must not build locale JSON paths directly from raw command input')
  }

  if (i18nCommandSource.includes('ok_or("No project path")')) {
    issues.push('i18n commands must use the active/default project data root consistently')
  }

  if (issues.length > 0) {
    throw new Error(`i18n locale path verification failed:\n${issues.join('\n')}`)
  }

  console.log('[release] i18n locale path invariants OK')
}


async function verifyFrontendRouteCoverage() {
  const routerSource = await readFile(path.join(frontendDir, 'src', 'router', 'index.ts'), 'utf8')
  const appSource = await readFile(path.join(frontendDir, 'src', 'App.vue'), 'utf8')
  const enLocale = JSON.parse(await readFile(path.join(root, 'data', 'locales', 'en.json'), 'utf8'))
  const viewEntries = await readdir(path.join(frontendDir, 'src', 'views'), { withFileTypes: true })
  const { issues, routes, navItems } = frontendRouteCoverageEvidence({
    routerSource,
    appSource,
    localeMessages: localeMessages(enLocale),
    availableComponents: viewEntries.filter((entry) => entry.isFile()).map((entry) => entry.name),
  })

  if (issues.length > 0) {
    throw new Error(`Frontend route coverage verification failed:\n${issues.join('\n')}`)
  }

  console.log(`[release] Frontend route coverage OK (${routes.length} routes, ${navItems.length} sidebar nav item(s))`)
}


async function verifyReleaseChannelPolicy() {
  const policy = JSON.parse(await readFile(releasePolicyPath, 'utf8'))
  const manifestScript = await readFile(path.join(root, 'scripts', 'create-release-manifest.mjs'), 'utf8')
  const issues = releaseChannelPolicyIssues(policy, manifestScript)

  if (issues.length > 0) {
    throw new Error(`Release channel policy verification failed:\n${issues.join('\n')}`)
  }

  console.log('[release] Release channel policy OK')
}

async function fileExists(filePath) {
  try {
    const info = await stat(filePath)
    return info.isFile()
  } catch {
    return false
  }
}

async function directoryExists(filePath) {
  try {
    const info = await stat(filePath)
    return info.isDirectory()
  } catch {
    return false
  }
}

function relative(file) {
  return path.relative(root, file).replaceAll(path.sep, '/')
}

main().catch((error) => {
  console.error(`\n[release] Verification failed: ${error.message}`)
  process.exit(1)
})
