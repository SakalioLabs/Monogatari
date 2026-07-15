<template>
  <div class="settings-page">
    <header class="page-header">
      <div class="header-copy">
        <span class="eyebrow"><SlidersHorizontal :size="13" />{{ t('settings.control-center', 'Engine control') }}</span>
        <h1>{{ t('settings.title', 'Settings') }}</h1>
        <p><FolderKanban :size="14" />{{ displayedProjectPath }}</p>
      </div>
      <div class="header-actions">
        <button
          class="icon-command"
          :class="{ spinning: refreshing }"
          :disabled="refreshing"
          :title="t('common.refresh', 'Refresh')"
          :aria-label="t('common.refresh', 'Refresh')"
          @click="refreshAll"
        >
          <RefreshCw :size="16" />
        </button>
        <button class="btn btn-primary btn-sm" :disabled="savingProject || !projectState" @click="saveProject">
          <Save :size="15" />
          {{ savingProject ? t('settings.saving', 'Saving') : t('settings.save-btn', 'Save project') }}
        </button>
      </div>
    </header>

    <section class="health-bar" :aria-label="t('settings.health-label', 'Project readiness')">
      <div class="health-item" :class="projectState?.settings_exists ? 'success' : 'muted-state'">
        <FileJson :size="15" />
        <span>{{ t('settings.config-file', 'Configuration') }}</span>
        <strong>{{ projectState?.settings_exists ? t('settings.saved', 'Saved') : t('settings.defaults', 'Defaults') }}</strong>
      </div>
      <div class="health-item" :class="projectState?.valid ? 'success' : 'danger'">
        <FolderCheck :size="15" />
        <span>{{ t('settings.project', 'Project') }}</span>
        <strong>{{ projectState?.valid ? t('settings.ready', 'Ready') : t('settings.review', 'Review') }}</strong>
      </div>
      <div class="health-item" :class="engineStatus?.initialized ? 'success' : 'muted-state'">
        <Activity :size="15" />
        <span>{{ t('settings.runtime', 'Runtime') }}</span>
        <strong>{{ engineStatus?.initialized ? t('settings.online', 'Online') : t('settings.idle', 'Idle') }}</strong>
      </div>
      <div class="health-item" :class="issueCount > 0 ? 'warning' : 'success'">
        <CircleAlert :size="15" />
        <span>{{ t('settings.issues', 'Issues') }}</span>
        <strong>{{ issueCount }}</strong>
      </div>
    </section>

    <div class="settings-shell">
      <nav class="settings-nav" :aria-label="t('settings.sections', 'Settings sections')">
        <button
          v-for="section in settingsSections"
          :key="section.id"
          class="nav-item"
          :class="{ active: activeSection === section.id }"
          :aria-current="activeSection === section.id ? 'page' : undefined"
          @click="selectSection(section.id)"
        >
          <component :is="section.icon" :size="17" />
          <span class="nav-copy">
            <strong>{{ section.label }}</strong>
            <small>{{ section.summary }}</small>
          </span>
          <span class="nav-signal" :class="section.tone"></span>
        </button>
      </nav>

      <main class="settings-workspace">
        <Transition name="section-swap" mode="out-in">
          <section :key="activeSection" class="settings-section">
            <template v-if="activeSection === 'project'">
              <header class="section-header">
                <div>
                  <span class="eyebrow"><FolderKanban :size="13" />{{ t('settings.project', 'Project') }}</span>
                  <h2>{{ t('settings.workspace', 'Workspace') }}</h2>
                  <p>{{ t('settings.project-summary', 'Project identity, package, and directory contract.') }}</p>
                </div>
                <button
                  class="btn btn-secondary btn-sm"
                  :disabled="initializing || !desktopRuntimeAvailable"
                  :title="desktopActionTitle"
                  @click="initEngine"
                >
                  <Play :size="14" />
                  {{ initializing ? t('settings.initializing', 'Initializing') : t('settings.initialize', 'Initialize') }}
                </button>
              </header>

              <div v-if="!desktopRuntimeAvailable" class="context-notice">
                <CircleAlert :size="15" />
                <span>{{ t('settings.browser-notice', 'Browser preview keeps project changes in memory. Desktop builds write settings.json and initialize the engine.') }}</span>
              </div>

              <div class="settings-group">
                <div class="group-heading">
                  <HardDrive :size="16" />
                  <div><h3>{{ t('settings.project-root', 'Project root') }}</h3><p>{{ displayedSettingsPath }}</p></div>
                </div>
                <div class="form-grid two">
                  <label class="form-field wide">
                    <span>{{ t('settings.project-path', 'Project path') }}</span>
                    <input
                      v-model="projectPath"
                      class="input mono-input"
                      :placeholder="t('settings.project-path-placeholder', './data')"
                      spellcheck="false"
                      @change="loadProjectConfig"
                    />
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.project-title', 'Project title') }}</span>
                    <input v-model="projectTitle" class="input" />
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.target-fps', 'Target FPS') }}</span>
                    <input v-model.number="targetFps" type="number" min="15" max="240" class="input" />
                  </label>
                </div>
              </div>

              <div class="settings-group">
                <div class="group-heading split-heading">
                  <div class="heading-copy">
                    <PackageCheck :size="16" />
                    <div><h3>{{ t('settings.project-package', 'Project package') }}</h3><p>{{ t('settings.package-format', 'Verified .monogatari archive and portable JSON manifest') }}</p></div>
                  </div>
                  <span class="state-chip" :class="{ ready: archiveSummary }">{{ packageStateLabel }}</span>
                </div>
                <div class="command-row">
                  <button
                    class="btn btn-primary btn-sm"
                    :disabled="!projectPackagesEnabled || packagingProject || importingProject || projectState?.valid === false"
                    :title="t('settings.export-package-title', 'Export a verified .monogatari project package')"
                    @click="exportProjectPackageFile"
                  >
                    <Archive :size="14" />
                    {{ packagingProject ? t('settings.packaging', 'Packaging') : t('settings.export-package', 'Export package') }}
                  </button>
                  <button
                    class="btn btn-secondary btn-sm"
                    :disabled="!projectPackagesEnabled || packagingProject || importingProject"
                    :title="t('settings.import-package-title', 'Import and verify a .monogatari project package')"
                    @click="importProjectPackageFile"
                  >
                    <Upload :size="14" />
                    {{ importingProject ? t('settings.importing', 'Importing') : t('settings.import-package', 'Import package') }}
                  </button>
                  <button class="btn btn-secondary btn-sm" :disabled="exportingProject" @click="exportProjectManifest">
                    <Download :size="14" />
                    {{ exportingProject ? t('settings.exporting', 'Exporting') : t('settings.export-manifest', 'Export manifest') }}
                  </button>
                </div>
                <dl v-if="archiveSummary" class="package-summary">
                  <div><dt>{{ t('settings.project', 'Project') }}</dt><dd>{{ archiveSummary.project_title }}</dd></div>
                  <div><dt>{{ t('settings.files', 'Files') }}</dt><dd>{{ archiveSummary.file_count }}</dd></div>
                  <div><dt>{{ t('settings.content-size', 'Content') }}</dt><dd>{{ formatBytes(archiveSummary.total_bytes) }}</dd></div>
                  <div><dt>{{ t('settings.package-size', 'Package') }}</dt><dd>{{ formatBytes(archiveSummary.archive_bytes) }}</dd></div>
                  <div class="hash-row"><dt>{{ t('settings.fingerprint', 'Fingerprint') }}</dt><dd><code :title="archiveSummary.content_sha256">{{ archiveSummary.content_sha256.slice(0, 16) }}</code></dd></div>
                </dl>
              </div>

              <div class="settings-group paths-group">
                <button class="group-toggle" :aria-expanded="pathsExpanded" @click="pathsExpanded = !pathsExpanded">
                  <span class="heading-copy">
                    <Database :size="16" />
                    <span><strong>{{ t('settings.project-directories', 'Project directories') }}</strong><small>{{ t('settings.directory-count', '{count} mapped paths', { count: editablePaths.length }) }}</small></span>
                  </span>
                  <ChevronDown :size="16" :class="{ rotated: pathsExpanded }" />
                </button>
                <div v-if="pathsExpanded" class="path-list">
                  <label v-for="path in editablePaths" :key="path.key" class="path-row" :class="{ missing: !path.exists && path.required }">
                    <span class="path-meta">
                      <b>{{ pathDisplayLabel(path) }}</b>
                      <small>{{ path.exists ? t('settings.item-count', '{count} items', { count: path.item_count }) : t('settings.missing', 'Missing') }}</small>
                    </span>
                    <input v-model="pathEdits[path.key]" class="input mono-input" spellcheck="false" />
                  </label>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'ai'">
              <header class="section-header">
                <div>
                  <span class="eyebrow"><Bot :size="13" />{{ t('settings.ai-backend', 'Inference runtime') }}</span>
                  <h2>{{ providerLabel }}</h2>
                  <p>{{ t('settings.ai-summary', 'Configure the model runtime shipped with each build target.') }}</p>
                </div>
                <button
                  class="btn btn-primary btn-sm"
                  :disabled="savingAI || !aiReadyToConnect || !aiActionAvailable"
                  :title="aiActionTitle"
                  @click="saveAI"
                >
                  <Sparkles :size="14" />
                  {{ savingAI ? t('settings.connecting', 'Connecting') : provider === 'webgpu' ? t('settings.initialize', 'Initialize') : t('settings.connect', 'Connect') }}
                </button>
              </header>

              <div class="runtime-line">
                <span><Activity :size="15" />{{ t('settings.active-engine', 'Active runtime') }}</span>
                <strong :class="{ online: activeRuntimeReady }">{{ activeEngineLabel }}</strong>
              </div>

              <div class="runtime-target-grid">
                <article class="runtime-target" :class="{ current: !desktopRuntimeAvailable }">
                  <span class="target-icon"><Globe2 :size="17" /></span>
                  <span><small>{{ t('settings.web-build', 'Web / PWA build') }}</small><strong>WebGPU</strong></span>
                  <b>{{ !desktopRuntimeAvailable ? t('settings.current-target', 'Current') : t('settings.packaged-target', 'Packaged') }}</b>
                </article>
                <article class="runtime-target" :class="{ current: desktopRuntimeAvailable }">
                  <span class="target-icon"><MonitorCog :size="17" /></span>
                  <span><small>{{ t('settings.windows-build', 'Windows build') }}</small><strong>DirectML</strong></span>
                  <b>{{ desktopRuntimeAvailable ? t('settings.current-target', 'Current') : t('settings.packaged-target', 'Packaged') }}</b>
                </article>
              </div>

              <div class="settings-group">
                <div class="group-heading">
                  <Cpu :size="16" />
                  <div><h3>{{ t('settings.provider', 'Connection') }}</h3><p>{{ t('settings.provider-contract', 'Choose a deployment runtime or a development API connection.') }}</p></div>
                </div>
                <div class="segmented-control" :aria-label="t('settings.provider', 'Provider')">
                  <button :class="{ active: provider === 'webgpu' }" :aria-pressed="provider === 'webgpu'" @click="provider = 'webgpu'">
                    <Globe2 :size="15" />WebGPU
                  </button>
                  <button :class="{ active: provider === 'onnx' }" :aria-pressed="provider === 'onnx'" @click="provider = 'onnx'">
                    <MonitorCog :size="15" />DirectML
                  </button>
                  <button :class="{ active: provider === 'api' }" :aria-pressed="provider === 'api'" @click="provider = 'api'">
                    <CloudCog :size="15" />{{ t('settings.api-provider-short', 'Development API') }}
                  </button>
                </div>
              </div>

              <div v-if="provider === 'api'" class="settings-group">
                <div class="group-heading">
                  <KeyRound :size="16" />
                  <div><h3>{{ t('settings.api-connection', 'API connection') }}</h3><p>{{ t('settings.secret-note', 'Credentials stay in runtime memory and are excluded from project files.') }}</p></div>
                </div>
                <div class="form-grid two">
                  <label class="form-field wide">
                    <span>{{ t('settings.base-url', 'Base URL') }}</span>
                    <input v-model="apiBaseUrl" class="input mono-input" :placeholder="t('settings.base-url-placeholder', 'https://api.openai.com/v1')" spellcheck="false" />
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.model', 'Model') }}</span>
                    <input v-model="apiModel" class="input mono-input" :placeholder="t('settings.model-placeholder', 'gpt-4o-mini')" spellcheck="false" />
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.api-key', 'API key') }}</span>
                    <input v-model="apiKey" type="password" class="input mono-input" :placeholder="t('settings.api-key-placeholder', 'Runtime credential')" autocomplete="off" spellcheck="false" />
                  </label>
                </div>
              </div>

              <div v-else-if="provider === 'webgpu'" class="settings-group">
                <div class="group-heading">
                  <Globe2 :size="16" />
                  <div><h3>{{ t('settings.webgpu-model', 'WebGPU model') }}</h3><p>{{ t('settings.webgpu-model-note', 'Transformers.js loads ONNX weights into the browser GPU runtime.') }}</p></div>
                </div>
                <div class="form-grid two">
                  <label class="form-field wide">
                    <span>{{ t('settings.model-id', 'Model ID or packaged path') }}</span>
                    <input v-model="webGpuModelId" class="input mono-input" placeholder="onnx-community/Qwen3.5-0.8B-Text-ONNX" spellcheck="false" />
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.dtype', 'Precision') }}</span>
                    <select v-model="webGpuDtype" class="input">
                      <option value="q4">Q4</option>
                      <option value="q4f16">Q4 F16</option>
                      <option value="q8">Q8</option>
                      <option value="fp16">FP16</option>
                      <option value="fp32">FP32</option>
                    </select>
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.max-new-tokens', 'Max new tokens') }}</span>
                    <input v-model.number="webGpuMaxNewTokens" class="input" type="number" min="1" max="2048" step="1" />
                  </label>
                </div>
                <div class="runtime-contract" :class="{ unavailable: !webGpuSupport.available }">
                  <PackageCheck v-if="webGpuSupport.available" :size="16" />
                  <CircleAlert v-else :size="16" />
                  <span><strong>{{ webGpuSupportLabel }}</strong><small>{{ t('settings.webgpu-cache-note', 'Model weights are cached by the browser after the first load.') }}</small></span>
                </div>
              </div>

              <div v-else class="settings-group">
                <div class="group-heading">
                  <HardDrive :size="16" />
                  <div><h3>{{ t('settings.local-model', 'Windows model') }}</h3><p>{{ t('settings.local-model-note', 'Paths remain relative to the active project root.') }}</p></div>
                </div>
                <div class="form-grid">
                  <label class="form-field">
                    <span>{{ t('settings.model-path', 'Model path') }}</span>
                    <input v-model="modelPath" class="input mono-input" :placeholder="t('settings.model-path-placeholder', 'models/model.onnx')" spellcheck="false" />
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.tokenizer-path', 'Tokenizer path') }}</span>
                    <input v-model="tokenizerPath" class="input mono-input" :placeholder="t('settings.tokenizer-path-placeholder', 'models/tokenizer.json')" spellcheck="false" />
                  </label>
                </div>
                <div class="runtime-contract">
                  <PackageCheck :size="16" />
                  <span><strong>{{ t('settings.directml', 'DirectML required') }}</strong><small>{{ t('settings.directml-note', 'Windows builds fail clearly when a DirectML adapter or compatible ONNX model is unavailable.') }}</small></span>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'voice'">
              <header class="section-header">
                <div>
                  <span class="eyebrow"><Volume2 :size="13" />{{ t('settings.tts', 'Text-to-speech') }}</span>
                  <h2>{{ ttsProviderLabel }}</h2>
                  <p>{{ t('settings.voice-summary', 'Default provider and delivery profile for generated character voice.') }}</p>
                </div>
                <button
                  class="btn btn-primary btn-sm"
                  :disabled="savingTts || !ttsReadyToSave || !desktopRuntimeAvailable"
                  :title="desktopActionTitle"
                  @click="saveTts"
                >
                  <Save :size="14" />
                  {{ savingTts ? t('settings.saving', 'Saving') : t('settings.save-voice', 'Save voice') }}
                </button>
              </header>

              <div class="settings-group">
                <div class="group-heading">
                  <Volume2 :size="16" />
                  <div><h3>{{ t('settings.voice-provider', 'Voice provider') }}</h3><p>{{ t('settings.voice-provider-note', 'System speech is local; API providers require runtime credentials.') }}</p></div>
                </div>
                <div class="form-grid two">
                  <label class="form-field">
                    <span>{{ t('settings.provider', 'Provider') }}</span>
                    <select v-model="ttsConfig.provider" class="input">
                      <option value="system">{{ t('settings.tts-system', 'Windows system voice') }}</option>
                      <option value="azure">{{ t('settings.tts-azure', 'Azure Speech') }}</option>
                      <option value="elevenlabs">{{ t('settings.tts-elevenlabs', 'ElevenLabs') }}</option>
                    </select>
                  </label>
                  <label class="form-field">
                    <span>{{ t('settings.language', 'Language') }}</span>
                    <select v-model="ttsConfig.language" class="input">
                      <option value="ja">{{ t('settings.language-ja', 'Japanese') }}</option>
                      <option value="en">{{ t('settings.language-en', 'English') }}</option>
                      <option value="zh">{{ t('settings.language-zh', 'Chinese') }}</option>
                      <option value="ko">{{ t('settings.language-ko', 'Korean') }}</option>
                    </select>
                  </label>
                  <label v-if="ttsConfig.provider === 'azure'" class="form-field">
                    <span>{{ t('settings.azure-region', 'Azure region') }}</span>
                    <input v-model="ttsConfig.api_region" class="input mono-input" :placeholder="t('settings.azure-region-placeholder', 'eastus')" spellcheck="false" />
                  </label>
                  <label v-if="ttsConfig.provider !== 'system'" class="form-field">
                    <span>{{ t('settings.voice-id', 'Voice ID') }}</span>
                    <input v-model="ttsConfig.api_voice_id" class="input mono-input" :placeholder="t('settings.voice-id-placeholder', 'Provider voice identifier')" spellcheck="false" />
                  </label>
                  <label v-if="ttsConfig.provider !== 'system'" class="form-field wide">
                    <span>{{ t('settings.provider-key', 'Provider API key') }}</span>
                    <input v-model="ttsConfig.api_key" type="password" class="input mono-input" :placeholder="t('settings.api-key-placeholder', 'Runtime credential')" autocomplete="off" spellcheck="false" />
                  </label>
                </div>
              </div>

              <div class="settings-group">
                <div class="group-heading">
                  <Gauge :size="16" />
                  <div><h3>{{ t('settings.delivery', 'Delivery') }}</h3><p>{{ t('settings.delivery-note', 'Defaults can be overridden by individual character voices.') }}</p></div>
                </div>
                <div class="range-grid">
                  <label class="range-field">
                    <span><strong>{{ t('settings.speed', 'Speed') }}</strong><output>{{ ttsConfig.speed.toFixed(1) }}x</output></span>
                    <input v-model.number="ttsConfig.speed" type="range" min="0.5" max="2.0" step="0.1" />
                  </label>
                  <label class="range-field">
                    <span><strong>{{ t('settings.pitch', 'Pitch') }}</strong><output>{{ ttsConfig.pitch.toFixed(1) }}</output></span>
                    <input v-model.number="ttsConfig.pitch" type="range" min="0.5" max="2.0" step="0.1" />
                  </label>
                </div>
              </div>
            </template>

            <template v-else-if="activeSection === 'sync'">
              <header class="section-header">
                <div>
                  <span class="eyebrow"><CloudCog :size="13" />{{ t('settings.cloud-sync', 'Save sync') }}</span>
                  <h2>{{ syncProviderLabel }}</h2>
                  <p>{{ t('settings.sync-summary', 'Track save inventory and validate remote connection settings.') }}</p>
                </div>
                <button class="icon-command" :class="{ spinning: syncLoading }" :disabled="syncLoading" :title="t('settings.check-status', 'Refresh status')" :aria-label="t('settings.check-status', 'Refresh status')" @click="checkSyncStatus">
                  <RefreshCw :size="16" />
                </button>
              </header>

              <div class="context-notice info">
                <CircleAlert :size="15" />
                <span>{{ t('settings.sync-preflight-note', 'Remote mode validates credentials and records the local save manifest; remote file transfer is not enabled in this build.') }}</span>
              </div>

              <div class="sync-overview">
                <div><span>{{ t('settings.status', 'Status') }}</span><strong :class="syncStatusTone">{{ syncStatusLabel }}</strong></div>
                <div><span>{{ t('settings.last-sync', 'Last update') }}</span><strong>{{ syncLastSyncLabel }}</strong></div>
                <div><span>{{ t('settings.files', 'Files') }}</span><strong>{{ syncStatus?.file_count ?? 0 }}</strong></div>
                <div><span>{{ t('settings.pending', 'Pending') }}</span><strong>{{ t('settings.pending-counts', '{uploads} up / {downloads} down', { uploads: syncStatus?.pending_uploads ?? 0, downloads: syncStatus?.pending_downloads ?? 0 }) }}</strong></div>
                <div><span>{{ t('settings.conflicts', 'Conflicts') }}</span><strong :class="{ danger: (syncStatus?.conflict_count ?? 0) > 0 }">{{ syncStatus?.conflict_count ?? 0 }}</strong></div>
              </div>

              <div class="settings-group">
                <div class="group-heading">
                  <Server :size="16" />
                  <div><h3>{{ t('settings.sync-provider', 'Sync provider') }}</h3><p>{{ syncProvider === 'local' ? t('settings.local-manifest-note', 'Manifest stays inside the active project.') : t('settings.remote-preflight-note', 'Endpoint and token are validated in runtime memory.') }}</p></div>
                </div>
                <div class="form-grid">
                  <label class="form-field">
                    <span>{{ t('settings.provider', 'Provider') }}</span>
                    <select v-model="syncProvider" class="input">
                      <option value="local">{{ t('settings.local-manifest', 'Local manifest') }}</option>
                      <option value="custom">{{ t('settings.remote-preflight', 'Remote preflight') }}</option>
                    </select>
                  </label>
                  <label v-if="syncProvider === 'custom'" class="form-field">
                    <span>{{ t('settings.remote-endpoint', 'Remote endpoint') }}</span>
                    <input v-model="syncEndpoint" class="input mono-input" :placeholder="t('settings.remote-endpoint-placeholder', 'https://sync.example.com')" spellcheck="false" />
                  </label>
                  <label v-if="syncProvider === 'custom'" class="form-field">
                    <span>{{ t('settings.sync-token', 'Sync token') }}</span>
                    <input v-model="syncToken" type="password" class="input mono-input" :placeholder="t('settings.api-key-placeholder', 'Runtime credential')" autocomplete="off" spellcheck="false" />
                  </label>
                </div>
                <div class="command-row">
                  <button class="btn btn-primary btn-sm" :disabled="syncLoading || !syncReady || !desktopRuntimeAvailable" :title="desktopActionTitle" @click="pushToCloud">
                    <ArrowUpToLine :size="14" />{{ t('settings.record-snapshot', 'Record snapshot') }}
                  </button>
                  <button class="btn btn-secondary btn-sm" :disabled="syncLoading || !desktopRuntimeAvailable" :title="desktopActionTitle" @click="pullFromCloud">
                    <ArrowDownToLine :size="14" />{{ t('settings.inspect-manifest', 'Inspect manifest') }}
                  </button>
                </div>
              </div>
            </template>

            <template v-else>
              <header class="section-header">
                <div>
                  <span class="eyebrow"><Activity :size="13" />{{ t('settings.diagnostics', 'Diagnostics') }}</span>
                  <h2>{{ projectState?.valid ? t('settings.clean', 'Project ready') : t('settings.attention', 'Attention required') }}</h2>
                  <p>{{ t('settings.diagnostics-summary', 'Runtime inventory and project configuration checks.') }}</p>
                </div>
                <button class="icon-command" :class="{ spinning: refreshing }" :disabled="refreshing" :title="t('common.refresh', 'Refresh')" :aria-label="t('common.refresh', 'Refresh')" @click="refreshAll">
                  <RefreshCw :size="16" />
                </button>
              </header>

              <div class="runtime-line">
                <span><Cpu :size="15" />{{ t('settings.active-engine', 'Active engine') }}</span>
                <strong :class="{ online: engineStatus?.active_ai_engine }">{{ activeEngineLabel }}</strong>
              </div>

              <div v-if="backendPlan" class="settings-group backend-plan-group">
                <div class="group-heading split-heading">
                  <div class="heading-copy">
                    <Bot :size="16" />
                    <div>
                      <h3>{{ t('settings.inference-selection', 'Inference selection') }}</h3>
                      <p>{{ backendPlan.schema }}</p>
                    </div>
                  </div>
                  <span class="state-chip" :class="{ ready: backendPlan.recommended_backend }">{{ backendRecommendationLabel }}</span>
                </div>
                <div class="backend-plan-list">
                  <div
                    v-for="item in backendPlanRows"
                    :key="item.backend"
                    class="backend-plan-row"
                    :class="item.readiness"
                  >
                    <span class="backend-plan-icon">
                      <CheckCircle2 v-if="item.readiness === 'ready'" :size="14" />
                      <Activity v-else-if="item.readiness === 'probe_required'" :size="14" />
                      <Download v-else-if="item.readiness === 'setup_required'" :size="14" />
                      <CircleAlert v-else :size="14" />
                    </span>
                    <span class="backend-plan-copy">
                      <strong>{{ backendName(item.backend) }}</strong>
                      <small>{{ item.reason_code }}</small>
                    </span>
                    <b>{{ backendReadinessLabel(item.readiness) }}</b>
                  </div>
                </div>
              </div>

              <div class="metric-grid">
                <div><span>{{ t('home.stats.characters', 'Characters') }}</span><strong>{{ engineStatus?.character_count ?? 0 }}</strong></div>
                <div><span>{{ t('home.stats.dialogues', 'Dialogues') }}</span><strong>{{ engineStatus?.dialogue_count ?? 0 }}</strong></div>
                <div><span>{{ t('home.stats.knowledge', 'Knowledge') }}</span><strong>{{ engineStatus?.knowledge_count ?? 0 }}</strong></div>
                <div><span>{{ t('settings.engines', 'Engines') }}</span><strong>{{ engineStatus?.ai_engines.length ?? 0 }}</strong></div>
              </div>

              <div class="settings-group">
                <div class="group-heading split-heading">
                  <div class="heading-copy">
                    <CircleAlert :size="16" />
                    <div><h3>{{ t('settings.project-checks', 'Project checks') }}</h3><p>{{ t('settings.issue-summary', '{errors} errors and {warnings} warnings', { errors: projectState?.error_count ?? 0, warnings: projectState?.warning_count ?? 0 }) }}</p></div>
                  </div>
                  <span class="state-chip" :class="{ ready: issueCount === 0, danger: (projectState?.error_count ?? 0) > 0 }">{{ issueCount }}</span>
                </div>
                <div v-if="projectState?.issues.length" class="issue-list">
                  <div v-for="(issue, index) in projectState.issues" :key="`${issue.code}-${index}`" class="issue-item" :class="issue.severity">
                    <span class="issue-marker"><CircleAlert :size="14" /></span>
                    <div>
                      <span>{{ issueSeverityLabel(issue.severity) }} · {{ issue.code }}</span>
                      <strong>{{ issue.path || t('settings.project', 'Project') }}</strong>
                      <p>{{ projectIssueMessage(issue) }}</p>
                    </div>
                  </div>
                </div>
                <div v-else class="empty-state"><CheckCircle2 :size="22" /><span>{{ t('settings.no-issues', 'No project issues detected.') }}</span></div>
              </div>

              <div class="settings-group save-target">
                <div class="group-heading">
                  <FileJson :size="16" />
                  <div><h3>{{ t('settings.save-target', 'Save target') }}</h3><p>{{ displayedSettingsPath }}</p></div>
                </div>
              </div>
            </template>
          </section>
        </Transition>
      </main>
    </div>

    <Transition name="toast">
      <div v-if="statusMessage" class="settings-toast" :class="statusOk ? 'success' : 'error'" role="status" aria-live="polite">
        <CheckCircle2 v-if="statusOk" :size="16" />
        <CircleAlert v-else :size="16" />
        <span>{{ statusMessage }}</span>
        <button :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="clearStatus"><X :size="14" /></button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import {
  Activity,
  Archive,
  ArrowDownToLine,
  ArrowUpToLine,
  Bot,
  CheckCircle2,
  ChevronDown,
  CircleAlert,
  CloudCog,
  Cpu,
  Database,
  Download,
  FileJson,
  FolderCheck,
  FolderKanban,
  Gauge,
  Globe2,
  HardDrive,
  KeyRound,
  MonitorCog,
  PackageCheck,
  Play,
  RefreshCw,
  Save,
  Server,
  SlidersHorizontal,
  Sparkles,
  Upload,
  Volume2,
  X,
} from '@lucide/vue'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { useI18n } from '../lib/i18n'
import {
  configureWebGpuRuntime,
  detectWebGpuSupport,
  getWebGpuRuntimeConfig,
  hasVerifiedWebGpuGeneration,
  initializeWebGpuRuntime,
  loadPackagedWebGpuConfig,
  webGpuSupportMessage,
  type WebGpuDType,
} from '../lib/webgpuInference'
import {
  exportProjectPackage,
  importProjectPackage,
  projectPackagesAvailable,
  type ProjectArchiveExportResult,
  type ProjectArchiveInspection,
} from '../lib/projectArchive'
import type {
  BackendReadiness,
  CloudSaveEntry,
  CloudSyncStatus,
  EngineStatus,
  InferenceBackendId,
  InferenceBackendPlan,
  ProjectConfigIssue,
  ProjectConfigState,
  ProjectPathStatus,
  SettingsDocument,
  SettingsSection,
  SettingsAiProvider,
  TtsSettings,
} from '../lib/settingsContract'
import {
  buildSettingsConfig,
  createBrowserProjectManifest,
  createBrowserProjectState,
  createBrowserSyncStatus,
  createEmptyEngineStatus,
  createPreviewProjectState,
  formatSettingsBytes as formatBytes,
  getSettingsConfigValue as getConfigValue,
  normalizeSettingsSection,
  safeSettingsFileName as safeFileName,
  scrubRuntimeSecretString,
} from '../lib/settingsDomain'

const { locale, t } = useI18n()

const projectPath = ref('./data')
const projectState = ref<ProjectConfigState | null>(null)
const engineStatus = ref<EngineStatus | null>(null)
const backendPlan = ref<InferenceBackendPlan | null>(null)
const activeSection = ref<SettingsSection>(normalizeSettingsSection(localStorage.getItem('monogatari-settings-section')))
const pathsExpanded = ref(false)
const refreshing = ref(false)
const savingProject = ref(false)
const savingAI = ref(false)
const savingTts = ref(false)
const exportingProject = ref(false)
const packagingProject = ref(false)
const importingProject = ref(false)
const initializing = ref(false)
const statusMessage = ref('')
const statusOk = ref(true)
const desktopRuntimeAvailable = hasTauriRuntime()
const projectPackagesEnabled = projectPackagesAvailable()
const archiveSummary = ref<(ProjectArchiveExportResult | ProjectArchiveInspection) | null>(null)
let statusTimer: number | undefined

const projectTitle = ref('Monogatari Engine')
const targetFps = ref(60)
const provider = ref<SettingsAiProvider>(desktopRuntimeAvailable ? 'onnx' : 'webgpu')
const apiBaseUrl = ref('https://api.openai.com/v1')
const apiKey = ref('')
const apiModel = ref('gpt-4o-mini')
const modelPath = ref('models/model.onnx')
const tokenizerPath = ref('models/tokenizer.json')
const useDirectML = ref(true)
const packagedWebGpuConfig = getWebGpuRuntimeConfig()
const webGpuModelId = ref(packagedWebGpuConfig.modelId)
const webGpuDtype = ref<WebGpuDType>(packagedWebGpuConfig.dtype)
const webGpuMaxNewTokens = ref(packagedWebGpuConfig.maxNewTokens)
const webGpuSupport = detectWebGpuSupport()
const webGpuReady = ref(false)

const ttsConfig = ref<TtsSettings>({
  provider: 'system',
  api_url: null,
  api_region: '',
  api_voice_id: '',
  api_key: '',
  default_voice: null,
  language: 'ja',
  speed: 1.0,
  pitch: 1.0,
})

const syncProvider = ref('local')
const syncEndpoint = ref('')
const syncToken = ref('')
const syncLoading = ref(false)
const syncStatus = ref<CloudSyncStatus | null>(null)

watch([syncProvider, syncEndpoint, syncToken], () => {
  syncStatus.value = null
})

function setStatus(message: string, ok = true) {
  if (statusTimer !== undefined) window.clearTimeout(statusTimer)
  statusMessage.value = message
  statusOk.value = ok
  statusTimer = window.setTimeout(clearStatus, ok ? 5000 : 8000)
}

function clearStatus() {
  if (statusTimer !== undefined) window.clearTimeout(statusTimer)
  statusTimer = undefined
  statusMessage.value = ''
}

function formatError(error: unknown) {
  return scrubRuntimeSecretString(error instanceof Error ? error.message : String(error))
}

function reportFailure(action: string, error: unknown) {
  setStatus(t('settings.notice.failed', '{action} failed: {error}', { action, error: formatError(error) }), false)
}

async function checkSyncStatus() {
  syncLoading.value = true
  try {
    await configureSyncProvider()
    syncStatus.value = await invokeCommand<CloudSyncStatus>(
      'get_sync_status',
      undefined,
      () => createBrowserSyncStatus(syncProvider.value, syncEndpoint.value, syncToken.value),
    )
  } catch (error) {
    reportFailure(t('settings.action.sync-check', 'Sync status check'), error)
  } finally {
    syncLoading.value = false
  }
}

async function pushToCloud() {
  if (!desktopRuntimeAvailable) return
  syncLoading.value = true
  try {
    await configureSyncProvider()
    await invokeCommand<string>('push_saves_to_cloud')
    setStatus(syncProvider.value === 'local'
      ? t('settings.notice.manifest-updated', 'Save manifest updated')
      : t('settings.notice.preflight-recorded', 'Remote preflight recorded'))
    await checkSyncStatus()
  } catch (error) {
    reportFailure(t('settings.action.record-snapshot', 'Record snapshot'), error)
  } finally {
    syncLoading.value = false
  }
}

async function pullFromCloud() {
  if (!desktopRuntimeAvailable) return
  syncLoading.value = true
  try {
    await configureSyncProvider()
    const entries = await invokeCommand<CloudSaveEntry[]>('pull_saves_from_cloud')
    setStatus(t('settings.notice.manifest-entries', 'Sync manifest contains {count} entries', { count: entries.length }))
    await checkSyncStatus()
  } catch (error) {
    reportFailure(t('settings.action.inspect-manifest', 'Inspect manifest'), error)
  } finally {
    syncLoading.value = false
  }
}

async function configureSyncProvider() {
  await invokeCommand<string>(
    'configure_cloud_sync',
    {
      provider: syncProvider.value,
      endpoint: syncEndpoint.value.trim() || null,
      apiKey: syncToken.value.trim() || null,
    },
    t('settings.notice.sync-provider-set', 'Sync provider set')
  )
}

async function saveTts() {
  if (!desktopRuntimeAvailable || !ttsReadyToSave.value) return
  savingTts.value = true
  try {
    const usesApi = ttsConfig.value.provider !== 'system'
    await invokeCommand('configure_tts', {
      config: {
        ...ttsConfig.value,
        api_url: null,
        api_region: ttsConfig.value.provider === 'azure' ? ttsConfig.value.api_region.trim() || null : null,
        api_voice_id: usesApi ? ttsConfig.value.api_voice_id.trim() || null : null,
        api_key: usesApi ? ttsConfig.value.api_key.trim() || null : null,
        default_voice: null,
      },
    })
    setStatus(t('settings.notice.voice-saved', 'Voice configuration saved'))
  } catch (error) {
    reportFailure(t('settings.action.save-voice', 'Save voice configuration'), error)
  } finally {
    savingTts.value = false
  }
}

const pathEdits = reactive<Record<string, string>>({
  characters: 'characters',
  dialogue: 'dialogue',
  knowledge: 'knowledge',
  scenes: 'scenes',
  assets: 'assets',
  events: 'events',
  endings: 'endings',
  saves: 'saves',
  quality_suites: 'quality_suites',
})

const previewState = createPreviewProjectState(packagedWebGpuConfig)

const browserProjectState = ref<ProjectConfigState>(previewState)

const providerLabel = computed(() => {
  if (provider.value === 'webgpu') return t('settings.webgpu-provider', 'WebGPU browser runtime')
  if (provider.value === 'onnx') return t('settings.onnx-provider', 'Windows DirectML runtime')
  return t('settings.api-provider', 'OpenAI-compatible development API')
})
const issueCount = computed(() => (projectState.value?.error_count || 0) + (projectState.value?.warning_count || 0))
const editablePaths = computed(() => projectState.value?.paths || previewState.paths)
const displayedProjectPath = computed(() => desktopRuntimeAvailable
  ? projectState.value?.project_path || projectPath.value
  : t('settings.browser-preview', 'Browser preview'))
const displayedSettingsPath = computed(() => desktopRuntimeAvailable
  ? projectState.value?.settings_path || t('settings.not-loaded', 'Not loaded')
  : t('settings.browser-settings-path', 'Browser preview / settings.json'))
const desktopActionTitle = computed(() => desktopRuntimeAvailable
  ? t('settings.desktop-runtime', 'Desktop runtime')
  : t('settings.desktop-required', 'Requires the desktop runtime'))
const packageStateLabel = computed(() => archiveSummary.value
  ? t('settings.verified', 'Verified')
  : projectPackagesEnabled
    ? t('settings.ready', 'Ready')
    : t('settings.web-manifest', 'Web manifest'))
const activeEngineLabel = computed(() => {
  if (provider.value === 'webgpu') {
    return webGpuReady.value ? 'WebGPU' : t('settings.not-initialized', 'Not initialized')
  }
  return engineStatus.value?.active_ai_engine || t('settings.no-ai-engine', 'No AI engine')
})
const activeRuntimeReady = computed(() => provider.value === 'webgpu'
  ? webGpuReady.value
  : Boolean(engineStatus.value?.active_ai_engine))
const backendPlanRows = computed(() => backendPlan.value?.backends.filter(item => item.readiness !== 'unavailable') || [])
const backendRecommendationLabel = computed(() => {
  if (backendPlan.value?.recommended_backend) return backendName(backendPlan.value.recommended_backend)
  if (backendPlan.value?.next_probe) {
    return t('settings.next-probe', 'Probe: {backend}', { backend: backendName(backendPlan.value.next_probe) })
  }
  return t('settings.no-verified-backend', 'No verified backend')
})
const aiReadyToConnect = computed(() => {
  if (provider.value === 'api') return Boolean(apiBaseUrl.value.trim() && apiModel.value.trim() && apiKey.value.trim())
  if (provider.value === 'webgpu') return Boolean(webGpuModelId.value.trim() && webGpuMaxNewTokens.value > 0)
  return Boolean(modelPath.value.trim() && tokenizerPath.value.trim())
})
const aiActionAvailable = computed(() => provider.value === 'webgpu'
  ? !desktopRuntimeAvailable && webGpuSupport.available
  : desktopRuntimeAvailable)
const aiActionTitle = computed(() => {
  if (provider.value === 'webgpu') {
    return webGpuSupport.available
      ? t('settings.initialize-webgpu', 'Initialize WebGPU model')
      : webGpuSupportMessage(webGpuSupport.reason)
  }
  return desktopActionTitle.value
})
const webGpuSupportLabel = computed(() => webGpuSupport.available
  ? t('settings.webgpu-available', 'WebGPU available')
  : t('settings.webgpu-unavailable', 'WebGPU unavailable'))
const ttsReadyToSave = computed(() => {
  if (ttsConfig.value.provider === 'system') return true
  if (!ttsConfig.value.api_key.trim() || !ttsConfig.value.api_voice_id.trim()) return false
  return ttsConfig.value.provider !== 'azure' || Boolean(ttsConfig.value.api_region.trim())
})
const ttsProviderLabel = computed(() => {
  if (ttsConfig.value.provider === 'azure') return t('settings.tts-azure', 'Azure Speech')
  if (ttsConfig.value.provider === 'elevenlabs') return t('settings.tts-elevenlabs', 'ElevenLabs')
  return t('settings.tts-system', 'Windows system voice')
})
const syncProviderLabel = computed(() => syncProvider.value === 'custom'
  ? t('settings.remote-preflight', 'Remote preflight')
  : t('settings.local-manifest', 'Local manifest'))
const syncReady = computed(() => syncProvider.value === 'local'
  || Boolean(syncEndpoint.value.trim() && syncToken.value.trim()))
const syncStatusLabel = computed(() => {
  const status = syncStatus.value?.status
  if (status === 'conflict') return t('settings.sync-status.conflict', 'Conflict')
  if (status === 'endpoint_missing') return t('settings.sync-status.endpoint-missing', 'Endpoint missing')
  if (status === 'local_changes') return t('settings.sync-status.local-changes', 'Local changes')
  if (status === 'local_clean') return t('settings.sync-status.local-clean', 'Local clean')
  if (status === 'remote_pending') return t('settings.sync-status.remote-pending', 'Remote pending')
  if (status === 'remote_ready') return t('settings.sync-status.remote-ready', 'Remote ready')
  if (status === 'token_missing') return t('settings.sync-status.token-missing', 'Token missing')
  return status || t('settings.not-checked', 'Not checked')
})
const syncStatusTone = computed(() => {
  if (syncStatus.value?.status === 'conflict') return 'danger'
  if (['endpoint_missing', 'token_missing', 'local_changes', 'remote_pending'].includes(syncStatus.value?.status || '')) return 'warning'
  if (['local_clean', 'remote_ready'].includes(syncStatus.value?.status || '')) return 'success'
  return 'muted-state'
})
const syncLastSyncLabel = computed(() => {
  if (!syncStatus.value?.last_sync) return t('settings.never', 'Never')
  const date = new Date(syncStatus.value.last_sync)
  if (Number.isNaN(date.getTime())) return syncStatus.value.last_sync
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium', timeStyle: 'short' }).format(date)
})

const settingsSections = computed(() => [
  {
    id: 'project' as SettingsSection,
    icon: FolderKanban,
    label: t('settings.project', 'Project'),
    summary: projectState.value?.valid ? t('settings.ready', 'Ready') : t('settings.review', 'Review'),
    tone: projectState.value?.valid ? 'success' : 'danger',
  },
  {
    id: 'ai' as SettingsSection,
    icon: Bot,
    label: t('settings.ai', 'AI'),
    summary: providerLabel.value,
    tone: engineStatus.value?.active_ai_engine ? 'success' : 'muted-state',
  },
  {
    id: 'voice' as SettingsSection,
    icon: Volume2,
    label: t('settings.voice', 'Voice'),
    summary: ttsProviderLabel.value,
    tone: 'muted-state',
  },
  {
    id: 'sync' as SettingsSection,
    icon: CloudCog,
    label: t('settings.sync', 'Sync'),
    summary: syncStatusLabel.value,
    tone: syncStatusTone.value,
  },
  {
    id: 'diagnostics' as SettingsSection,
    icon: Activity,
    label: t('settings.diagnostics', 'Diagnostics'),
    summary: issueCount.value === 0
      ? t('settings.clean', 'Project ready')
      : t('settings.issue-count', '{count} issues', { count: issueCount.value }),
    tone: issueCount.value === 0 ? 'success' : (projectState.value?.error_count ? 'danger' : 'warning'),
  },
])

function selectSection(section: SettingsSection) {
  activeSection.value = section
  localStorage.setItem('monogatari-settings-section', section)
}

function pathDisplayLabel(path: ProjectPathStatus) {
  if (path.key === 'characters') return t('settings.path.characters', 'Characters')
  if (path.key === 'dialogue') return t('settings.path.dialogue', 'Dialogue')
  if (path.key === 'knowledge') return t('settings.path.knowledge', 'Knowledge')
  if (path.key === 'scenes') return t('settings.path.scenes', 'Scenes')
  if (path.key === 'assets') return t('settings.path.assets', 'Assets')
  if (path.key === 'events') return t('settings.path.events', 'Story events')
  if (path.key === 'endings') return t('settings.path.endings', 'Story endings')
  if (path.key === 'saves') return t('settings.path.saves', 'Saves')
  if (path.key === 'quality_suites') return t('settings.path.quality-suites', 'Quality suites')
  return path.label
}

function issueSeverityLabel(severity: string) {
  return severity === 'error' ? t('settings.severity.error', 'Error') : t('settings.severity.warning', 'Warning')
}

function projectIssueMessage(issue: ProjectConfigIssue) {
  if (issue.code === 'settings_not_regular_file') return t('settings.issue.settings-not-file', 'settings.json must be a regular file inside the project root.')
  if (issue.code === 'settings_too_large') return t('settings.issue.settings-too-large', 'settings.json exceeds the supported size limit.')
  if (issue.code === 'settings_not_object') return t('settings.issue.settings-not-object', 'settings.json must contain a JSON object.')
  if (issue.code === 'settings_invalid_json') return t('settings.issue.settings-invalid-json', 'settings.json could not be parsed as valid JSON.')
  if (issue.code === 'settings_missing') return t('settings.issue.settings-missing', 'The project has no settings.json; defaults are active.')
  if (issue.code === 'project_path_missing') return t('settings.issue.path-missing', 'A required project directory is missing.')
  if (issue.code === 'project_path_duplicate') return t('settings.issue.path-duplicate', 'Multiple path entries resolve to the same directory.')
  if (issue.code === 'project_path_invalid') return t('settings.issue.path-invalid', 'The project path must remain relative to the project root.')
  if (issue.code === 'api_base_url_missing') return t('settings.issue.api-url-missing', 'The API base URL is empty.')
  if (issue.code === 'api_key_missing') return t('settings.issue.api-key-missing', 'The API key is not configured in runtime memory.')
  if (issue.code === 'onnx_model_missing') return t('settings.issue.onnx-model-missing', 'The ONNX model path is empty.')
  if (issue.code === 'webgpu_model_missing') return t('settings.issue.webgpu-model-missing', 'The WebGPU model ID is empty.')
  if (issue.code === 'ai_provider_invalid') return t('settings.issue.ai-provider-invalid', 'The AI provider must be WebGPU, DirectML, or API.')
  return issue.message
}

async function refreshAll() {
  if (refreshing.value) return
  refreshing.value = true
  try {
    await Promise.all([loadProjectConfig(), refreshStatus(), checkSyncStatus()])
    await refreshBackendPlan()
  } finally {
    refreshing.value = false
  }
}

async function loadProjectConfig() {
  try {
    projectState.value = await invokeCommand<ProjectConfigState>(
      'get_project_config',
      { projectPath: projectPath.value },
      () => browserProjectState.value
    )
    applyProjectState(projectState.value)
  } catch (error) {
    reportFailure(t('settings.action.load-project', 'Load project configuration'), error)
  }
}

function applyProjectState(state: ProjectConfigState) {
  const config = state.config
  projectTitle.value = String(getConfigValue(config, ['render', 'title'])
    || getConfigValue(config, ['engine', 'title'])
    || projectTitle.value)
  targetFps.value = Number(getConfigValue(config, ['engine', 'target_fps']) || getConfigValue(config, ['engine', 'targetFps']) || targetFps.value)
  const configuredProvider = getConfigValue(config, ['ai', 'provider'])
  if (configuredProvider === 'api' || configuredProvider === 'onnx' || configuredProvider === 'webgpu') {
    provider.value = configuredProvider
  }
  apiBaseUrl.value = String(getConfigValue(config, ['ai', 'api', 'base_url']) || getConfigValue(config, ['ai', 'api', 'baseUrl']) || apiBaseUrl.value)
  apiModel.value = String(getConfigValue(config, ['ai', 'api', 'model']) || apiModel.value)
  modelPath.value = String(getConfigValue(config, ['ai', 'onnx', 'model_path']) || getConfigValue(config, ['ai', 'onnx', 'modelPath']) || modelPath.value)
  tokenizerPath.value = String(getConfigValue(config, ['ai', 'onnx', 'tokenizer_path']) || getConfigValue(config, ['ai', 'onnx', 'tokenizerPath']) || tokenizerPath.value)
  useDirectML.value = true
  webGpuModelId.value = String(getConfigValue(config, ['ai', 'webgpu', 'model_id']) || getConfigValue(config, ['ai', 'webgpu', 'modelId']) || webGpuModelId.value)
  webGpuDtype.value = String(getConfigValue(config, ['ai', 'webgpu', 'dtype']) || webGpuDtype.value) as WebGpuDType
  webGpuMaxNewTokens.value = Number(getConfigValue(config, ['ai', 'webgpu', 'max_new_tokens']) || getConfigValue(config, ['ai', 'webgpu', 'maxNewTokens']) || webGpuMaxNewTokens.value)
  configureWebGpuRuntime({
    modelId: webGpuModelId.value,
    dtype: webGpuDtype.value,
    maxNewTokens: webGpuMaxNewTokens.value,
  })
  for (const path of state.paths) {
    pathEdits[path.key] = path.relative_path
  }
}

async function saveProject() {
  if (!projectState.value) return
  savingProject.value = true
  try {
    const config = buildConfigForSave(projectState.value.config)
    projectState.value = await invokeCommand<ProjectConfigState>(
      'save_project_config',
      { projectPath: projectPath.value, config },
      () => {
        browserProjectState.value = createBrowserProjectState(config, previewState)
        return browserProjectState.value
      }
    )
    applyProjectState(projectState.value)
    setStatus(desktopRuntimeAvailable
      ? t('settings.notice.project-saved', 'Project settings saved')
      : t('settings.notice.preview-updated', 'Browser preview settings updated for this session'))
  } catch (error) {
    reportFailure(t('settings.action.save-project', 'Save project settings'), error)
  } finally {
    savingProject.value = false
  }
}

async function saveAI() {
  if (!aiActionAvailable.value || !aiReadyToConnect.value) return
  savingAI.value = true
  try {
    if (provider.value === 'webgpu') {
      await initializeWebGpuRuntime({
        modelId: webGpuModelId.value,
        dtype: webGpuDtype.value,
        maxNewTokens: webGpuMaxNewTokens.value,
      })
      webGpuReady.value = true
    } else if (provider.value === 'api') {
      await invokeCommand<void>('configure_api', { baseUrl: apiBaseUrl.value, apiKey: apiKey.value, model: apiModel.value })
    } else {
      await invokeCommand<void>('configure_onnx', {
        modelPath: modelPath.value,
        tokenizerPath: tokenizerPath.value,
      })
    }
    setStatus(t('settings.notice.ai-connected', 'AI backend connected'))
    await refreshStatus()
    await refreshBackendPlan()
  } catch (error) {
    reportFailure(t('settings.action.connect-ai', 'Connect AI backend'), error)
  } finally {
    savingAI.value = false
  }
}

async function initEngine() {
  if (!desktopRuntimeAvailable) return
  initializing.value = true
  try {
    await invokeCommand<void>('initialize_engine', { projectPath: projectPath.value })
    setStatus(t('settings.notice.engine-initialized', 'Engine initialized'))
    await Promise.all([refreshStatus(), loadProjectConfig()])
  } catch (error) {
    reportFailure(t('settings.action.initialize-engine', 'Initialize engine'), error)
  } finally {
    initializing.value = false
  }
}

async function exportProjectPackageFile() {
  packagingProject.value = true
  try {
    const result = await exportProjectPackage(projectPath.value, projectTitle.value)
    if (!result) return
    archiveSummary.value = result
    setStatus(t('settings.notice.package-exported', 'Project package exported to {path}', { path: result.archive_path }))
  } catch (error) {
    reportFailure(t('settings.action.export-package', 'Export project package'), error)
  } finally {
    packagingProject.value = false
  }
}

async function importProjectPackageFile() {
  importingProject.value = true
  try {
    const flow = await importProjectPackage()
    if (!flow) return
    archiveSummary.value = flow.inspection
    if (!flow.imported) {
      setStatus(t('settings.notice.package-verified', 'Project package verified; no import destination was selected'))
      return
    }

    projectPath.value = flow.imported.project_path
    await loadProjectConfig()
    try {
      await invokeCommand<void>('initialize_engine', { projectPath: flow.imported.project_path })
      await refreshStatus()
      setStatus(t('settings.notice.package-imported', 'Imported and initialized {project}', { project: flow.imported.project_title }))
    } catch (error) {
      setStatus(t('settings.notice.import-init-failed', 'Project imported to {path}, but initialization failed: {error}', {
        path: flow.imported.project_path,
        error: formatError(error),
      }), false)
    }
  } catch (error) {
    reportFailure(t('settings.action.import-package', 'Import project package'), error)
  } finally {
    importingProject.value = false
  }
}

async function exportProjectManifest() {
  exportingProject.value = true
  try {
    const manifest = await invokeCommand<Record<string, unknown>>(
      'export_project',
      { projectPath: projectPath.value },
      () => createBrowserProjectManifest({
        exportedAt: new Date().toISOString(),
        projectPath: projectState.value?.project_path || projectPath.value,
        settings: buildConfigForSave(projectState.value?.config || previewState.config),
      }),
    )
    downloadJson(`${safeFileName(projectTitle.value || 'monogatari-project')}-manifest.json`, manifest)
    setStatus(t('settings.notice.manifest-exported', 'Project manifest exported'))
  } catch (error) {
    reportFailure(t('settings.action.export-manifest', 'Export project manifest'), error)
  } finally {
    exportingProject.value = false
  }
}

async function refreshStatus() {
  try {
    engineStatus.value = await invokeCommand<EngineStatus>('get_engine_status', undefined, createEmptyEngineStatus())
  } catch {}
}

async function refreshBackendPlan() {
  if (!desktopRuntimeAvailable) {
    backendPlan.value = null
    return
  }
  const activeEngine = engineStatus.value?.active_ai_engine || ''
  try {
    backendPlan.value = await invokeCommand<InferenceBackendPlan>('get_inference_backend_plan', {
      request: {
        target: 'desktop',
        signals: {
          webgpu_adapter_available: webGpuSupport.available,
          webgpu_model_ready: hasVerifiedWebGpuGeneration(),
          api_configured: activeEngine === 'API',
        },
      },
    })
  } catch {
    backendPlan.value = null
  }
}

function backendName(backend: InferenceBackendId): string {
  if (backend === 'web_gpu') return 'WebGPU'
  if (backend === 'llama_cpp') return 'llama.cpp'
  if (backend === 'win_ml_gen_ai') return 'WinML GenAI'
  if (backend === 'direct_ml_onnx') return 'DirectML ONNX'
  if (backend === 'mlx_lm') return 'MLX-LM'
  if (backend === 'vllm') return 'vLLM'
  if (backend === 'sglang') return 'SGLang'
  return t('settings.api-provider-short', 'Development API')
}

function backendReadinessLabel(readiness: BackendReadiness): string {
  if (readiness === 'ready') return t('settings.backend-ready', 'Ready')
  if (readiness === 'probe_required') return t('settings.backend-probe-required', 'Probe required')
  if (readiness === 'setup_required') return t('settings.backend-setup-required', 'Setup required')
  if (readiness === 'blocked') return t('settings.backend-blocked', 'Blocked')
  return t('settings.backend-unavailable', 'Unavailable')
}

function buildConfigForSave(source: SettingsDocument): SettingsDocument {
  return buildSettingsConfig(source, {
    projectTitle: projectTitle.value,
    targetFps: Number(targetFps.value),
    provider: provider.value,
    apiBaseUrl: apiBaseUrl.value,
    apiModel: apiModel.value,
    modelPath: modelPath.value,
    tokenizerPath: tokenizerPath.value,
    webGpuModelId: webGpuModelId.value,
    webGpuDtype: webGpuDtype.value,
    webGpuMaxNewTokens: Number(webGpuMaxNewTokens.value),
    paths: pathEdits,
  })
}

function downloadJson(filename: string, value: unknown) {
  const blob = new Blob([JSON.stringify(value, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}

onMounted(async () => {
  await refreshAll()
  if (!desktopRuntimeAvailable) {
    const config = await loadPackagedWebGpuConfig()
    webGpuModelId.value = config.modelId
    webGpuDtype.value = config.dtype
    webGpuMaxNewTokens.value = config.maxNewTokens
  }
})
onUnmounted(clearStatus)
</script>

<style scoped>
.settings-page {
  width: 100%;
  max-width: 1240px;
  margin: 0 auto;
  padding: 28px 32px 42px;
}

.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 16px;
}

.header-copy {
  min-width: 0;
}

.eyebrow {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.eyebrow svg,
.group-heading > svg,
.heading-copy > svg {
  flex: 0 0 auto;
  color: var(--brand-light);
}

.page-header h1 {
  margin: 4px 0 0;
  color: var(--text-primary);
  font-size: 26px;
  line-height: 1.15;
}

.page-header p {
  display: flex;
  max-width: 720px;
  min-width: 0;
  align-items: center;
  gap: 7px;
  margin-top: 7px;
  overflow: hidden;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.page-header p svg {
  flex: 0 0 auto;
}

.header-actions {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 8px;
}

.btn {
  display: inline-flex;
  min-height: 34px;
  align-items: center;
  justify-content: center;
  gap: 7px;
}

.btn:disabled,
.icon-command:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.icon-command {
  display: inline-grid;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  place-items: center;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-command:hover:not(:disabled) {
  border-color: var(--border-strong);
  color: var(--text-primary);
}

.icon-command.spinning svg {
  animation: spin 0.8s linear infinite;
}

.health-bar {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-bottom: 14px;
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.health-item {
  display: grid;
  min-width: 0;
  min-height: 52px;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  padding: 10px 13px;
  border-right: 1px solid var(--border);
}

.health-item:last-child {
  border-right: 0;
}

.health-item svg {
  color: var(--text-tertiary);
}

.health-item span {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 750;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.health-item strong {
  overflow: hidden;
  color: var(--text-secondary);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.health-item.success svg,
.health-item.success strong,
.success {
  color: var(--success);
}

.health-item.warning svg,
.health-item.warning strong,
.warning {
  color: var(--warning);
}

.health-item.danger svg,
.health-item.danger strong,
.danger {
  color: var(--danger) !important;
}

.muted-state {
  color: var(--text-tertiary);
}

.settings-shell {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  align-items: start;
  gap: 14px;
}

.settings-nav {
  position: sticky;
  top: 14px;
  display: grid;
  gap: 3px;
  padding: 6px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  scrollbar-width: none;
}

.settings-nav::-webkit-scrollbar {
  display: none;
}

.nav-item {
  display: grid;
  width: 100%;
  min-height: 56px;
  grid-template-columns: 22px minmax(0, 1fr) 7px;
  align-items: center;
  gap: 8px;
  padding: 8px 9px;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  text-align: left;
}

.nav-item:hover {
  background: var(--surface-2);
  color: var(--text-secondary);
}

.nav-item.active {
  background: var(--surface-3);
  color: var(--brand-light);
  box-shadow: inset 2px 0 0 var(--brand);
}

.nav-item > svg {
  justify-self: center;
}

.nav-copy {
  display: grid;
  min-width: 0;
  gap: 2px;
}

.nav-copy strong,
.nav-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.nav-copy strong {
  color: var(--text-primary);
  font-size: 11px;
}

.nav-copy small {
  color: var(--text-tertiary);
  font-size: 8px;
}

.nav-signal {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-tertiary);
}

.nav-signal.success {
  background: var(--success);
}

.nav-signal.warning {
  background: var(--warning);
}

.nav-signal.danger {
  background: var(--danger);
}

.settings-workspace {
  min-width: 0;
  min-height: 620px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
}

.settings-section {
  min-width: 0;
  padding: 20px;
}

.section-header {
  display: flex;
  min-height: 72px;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border);
}

.section-header > div {
  min-width: 0;
}

.section-header h2 {
  margin: 4px 0 0;
  color: var(--text-primary);
  font-size: 18px;
  line-height: 1.2;
}

.section-header p {
  max-width: 620px;
  margin-top: 6px;
  color: var(--text-tertiary);
  font-size: 10px;
  line-height: 1.45;
}

.context-notice {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin: 14px 0 0;
  padding: 9px 11px;
  border-left: 2px solid var(--warning);
  background: color-mix(in srgb, var(--warning) 7%, transparent);
  color: var(--text-secondary);
  font-size: 10px;
  line-height: 1.45;
}

.context-notice.info {
  border-left-color: var(--info);
  background: color-mix(in srgb, var(--info) 7%, transparent);
}

.context-notice svg {
  flex: 0 0 auto;
  margin-top: 1px;
  color: var(--warning);
}

.context-notice.info svg {
  color: var(--info);
}

.settings-group {
  display: grid;
  min-width: 0;
  gap: 15px;
  padding: 20px 0;
  border-bottom: 1px solid var(--border);
}

.settings-group:last-child {
  border-bottom: 0;
  padding-bottom: 2px;
}

.group-heading,
.heading-copy {
  display: flex;
  min-width: 0;
  align-items: flex-start;
  gap: 9px;
}

.group-heading > div,
.heading-copy > span,
.heading-copy > div {
  min-width: 0;
}

.group-heading h3,
.heading-copy strong {
  margin: 0;
  color: var(--text-primary);
  font-size: 12px;
  line-height: 1.25;
}

.group-heading p,
.heading-copy small {
  display: block;
  max-width: 640px;
  margin-top: 3px;
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 9px;
  line-height: 1.4;
  text-overflow: ellipsis;
}

.split-heading {
  align-items: center;
  justify-content: space-between;
}

.form-grid {
  display: grid;
  max-width: 800px;
  gap: 12px;
}

.form-grid.two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.form-field {
  display: grid;
  min-width: 0;
  gap: 6px;
}

.form-field.wide {
  grid-column: 1 / -1;
}

.form-field > span {
  color: var(--text-secondary);
  font-size: 10px;
  font-weight: 750;
}

.input {
  min-height: 36px;
}

.mono-input {
  font-family: var(--font-mono);
  font-size: 11px;
}

select.input {
  cursor: pointer;
}

.command-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.state-chip {
  display: inline-flex;
  min-width: 56px;
  min-height: 24px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  padding: 3px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.state-chip.ready {
  border-color: color-mix(in srgb, var(--success) 45%, var(--border));
  color: var(--success);
}

.state-chip.danger {
  border-color: color-mix(in srgb, var(--danger) 45%, var(--border));
}

.package-summary {
  display: grid;
  grid-template-columns: minmax(0, 2fr) repeat(3, minmax(80px, 1fr));
  margin: 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

.package-summary > div {
  display: grid;
  min-width: 0;
  gap: 3px;
  padding: 10px;
  border-right: 1px solid var(--border);
}

.package-summary > div:nth-child(4) {
  border-right: 0;
}

.package-summary dt {
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.package-summary dd {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 10px;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.package-summary .hash-row {
  grid-column: 1 / -1;
  grid-template-columns: 90px minmax(0, 1fr);
  align-items: center;
  border-top: 1px solid var(--border);
  border-right: 0;
}

.package-summary code {
  color: var(--text-tertiary);
  font-size: 9px;
}

.paths-group {
  gap: 0;
}

.group-toggle {
  display: flex;
  width: 100%;
  min-height: 38px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  text-align: left;
}

.group-toggle:hover {
  color: var(--text-primary);
}

.group-toggle > svg {
  flex: 0 0 auto;
  transition: transform var(--transition-fast);
}

.group-toggle > svg.rotated {
  transform: rotate(180deg);
}

.path-list {
  display: grid;
  margin-top: 14px;
  border-top: 1px solid var(--border);
}

.path-row {
  display: grid;
  min-width: 0;
  grid-template-columns: 180px minmax(0, 1fr);
  align-items: center;
  gap: 14px;
  padding: 9px 0;
  border-bottom: 1px solid var(--border);
}

.path-row:last-child {
  border-bottom: 0;
}

.path-row.missing .path-meta small {
  color: var(--danger);
}

.path-meta {
  display: flex;
  min-width: 0;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}

.path-meta b,
.path-meta small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-meta b {
  color: var(--text-primary);
  font-size: 10px;
}

.path-meta small {
  color: var(--text-tertiary);
  font-size: 8px;
}

.runtime-line {
  display: flex;
  min-height: 42px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: 14px;
  padding: 0 11px;
  border-left: 2px solid var(--border-strong);
  background: var(--surface-2);
}

.runtime-line span {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-secondary);
  font-size: 10px;
}

.runtime-line span svg {
  color: var(--text-tertiary);
}

.runtime-line strong {
  color: var(--text-tertiary);
  font-size: 10px;
}

.runtime-line strong.online {
  color: var(--success);
}

.runtime-target-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  margin-top: 8px;
}

.runtime-target {
  display: grid;
  min-width: 0;
  grid-template-columns: 34px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  min-height: 62px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-1);
}

.runtime-target.current {
  border-color: var(--border-strong);
  box-shadow: inset 3px 0 0 var(--brand);
}

.target-icon {
  display: grid;
  width: 34px;
  height: 34px;
  place-items: center;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
}

.runtime-target > span:nth-child(2) {
  display: grid;
  min-width: 0;
  gap: 2px;
}

.runtime-target small,
.runtime-contract small {
  color: var(--text-tertiary);
  font-size: 8px;
}

.runtime-target strong,
.runtime-contract strong {
  color: var(--text-primary);
  font-size: 10px;
}

.runtime-target > b {
  color: var(--text-tertiary);
  font-size: 8px;
  text-transform: uppercase;
}

.runtime-contract {
  display: flex;
  max-width: 800px;
  min-height: 48px;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
  padding: 9px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--success);
}

.runtime-contract.unavailable {
  color: var(--danger);
}

.runtime-contract > span {
  display: grid;
  gap: 2px;
}

.segmented-control {
  display: inline-grid;
  width: min(560px, 100%);
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 3px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.segmented-control button {
  display: flex;
  min-width: 0;
  min-height: 34px;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 6px 9px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 10px;
  font-weight: 700;
}

.segmented-control button:hover {
  color: var(--text-primary);
}

.segmented-control button.active {
  background: var(--surface-4);
  color: var(--brand-light);
  box-shadow: var(--shadow-sm);
}

.switch-row {
  display: flex;
  max-width: 800px;
  min-height: 52px;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 8px 0;
  cursor: pointer;
}

.switch-row > span {
  display: grid;
  gap: 3px;
}

.switch-row strong {
  color: var(--text-primary);
  font-size: 10px;
}

.switch-row small {
  color: var(--text-tertiary);
  font-size: 9px;
}

.switch-row input {
  position: relative;
  width: 38px;
  height: 22px;
  flex: 0 0 auto;
  appearance: none;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  background: var(--surface-3);
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.switch-row input::before {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--text-tertiary);
  content: '';
  transition: transform var(--transition-fast), background var(--transition-fast);
}

.switch-row input:checked {
  border-color: var(--brand-dark);
  background: color-mix(in srgb, var(--brand) 25%, var(--surface-3));
}

.switch-row input:checked::before {
  background: var(--brand-light);
  transform: translateX(16px);
}

.range-grid {
  display: grid;
  max-width: 800px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 22px;
}

.range-field {
  display: grid;
  gap: 10px;
}

.range-field > span {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.range-field strong,
.range-field output {
  color: var(--text-secondary);
  font-size: 10px;
}

.range-field output {
  font-family: var(--font-mono);
  color: var(--brand-light);
}

.range-field input {
  width: 100%;
  accent-color: var(--brand);
}

.sync-overview {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  margin-top: 16px;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

.sync-overview > div {
  display: grid;
  min-width: 0;
  gap: 4px;
  padding: 11px;
  border-right: 1px solid var(--border);
}

.sync-overview > div:last-child {
  border-right: 0;
}

.sync-overview span {
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.sync-overview strong {
  overflow: hidden;
  color: var(--text-primary);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-top: 16px;
  border: 1px solid var(--border);
}

.backend-plan-group {
  padding-top: 16px;
}

.backend-plan-list {
  display: grid;
  border-top: 1px solid var(--border);
}

.backend-plan-row {
  display: grid;
  min-width: 0;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: 9px;
  min-height: 45px;
  border-bottom: 1px solid var(--border);
}

.backend-plan-row:last-child {
  border-bottom: 0;
}

.backend-plan-icon {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  color: var(--text-tertiary);
}

.backend-plan-row.ready .backend-plan-icon {
  color: var(--success);
}

.backend-plan-row.probe_required .backend-plan-icon {
  color: var(--info);
}

.backend-plan-row.setup_required .backend-plan-icon {
  color: var(--warning);
}

.backend-plan-row.blocked .backend-plan-icon {
  color: var(--danger);
}

.backend-plan-copy {
  display: grid;
  min-width: 0;
  gap: 3px;
}

.backend-plan-copy strong,
.backend-plan-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.backend-plan-copy strong {
  color: var(--text-primary);
  font-size: 10px;
}

.backend-plan-copy small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 8px;
}

.backend-plan-row > b {
  color: var(--text-secondary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.metric-grid > div {
  display: grid;
  min-width: 0;
  gap: 5px;
  padding: 13px;
  border-right: 1px solid var(--border);
}

.metric-grid > div:last-child {
  border-right: 0;
}

.metric-grid span {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.metric-grid strong {
  color: var(--brand-light);
  font-size: 20px;
  line-height: 1;
}

.issue-list {
  display: grid;
  border-top: 1px solid var(--border);
}

.issue-item {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: 9px;
  padding: 11px 0;
  border-bottom: 1px solid var(--border);
}

.issue-item:last-child {
  border-bottom: 0;
}

.issue-marker {
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  color: var(--warning);
}

.issue-item.error .issue-marker {
  color: var(--danger);
}

.issue-item > div {
  display: grid;
  min-width: 0;
  gap: 3px;
}

.issue-item > div > span {
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.issue-item strong {
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 9px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.issue-item p {
  color: var(--text-secondary);
  font-size: 10px;
  line-height: 1.45;
}

.empty-state {
  display: grid;
  min-height: 124px;
  place-items: center;
  align-content: center;
  gap: 8px;
  color: var(--text-tertiary);
  text-align: center;
}

.empty-state svg {
  color: var(--success);
}

.empty-state span {
  font-size: 10px;
}

.save-target .group-heading p {
  font-family: var(--font-mono);
}

.settings-toast {
  position: fixed;
  right: 20px;
  bottom: 20px;
  z-index: 80;
  display: grid;
  max-width: min(480px, calc(100vw - 32px));
  grid-template-columns: 18px minmax(0, 1fr) 26px;
  align-items: center;
  gap: 8px;
  padding: 9px 9px 9px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-primary);
  box-shadow: var(--shadow-lg);
  font-size: 10px;
}

.settings-toast.success > svg {
  color: var(--success);
}

.settings-toast.error > svg {
  color: var(--danger);
}

.settings-toast > span {
  min-width: 0;
  overflow-wrap: anywhere;
}

.settings-toast button {
  display: grid;
  width: 26px;
  height: 26px;
  place-items: center;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.settings-toast button:hover {
  background: var(--surface-4);
  color: var(--text-primary);
}

.section-swap-enter-active,
.section-swap-leave-active,
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.section-swap-enter-from,
.section-swap-leave-to {
  opacity: 0;
  transform: translateY(3px);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(5px);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@media (max-width: 980px) {
  .settings-page {
    padding-inline: 24px;
  }

  .settings-shell {
    grid-template-columns: 190px minmax(0, 1fr);
  }

  .health-item {
    grid-template-columns: 18px minmax(0, 1fr);
  }

  .health-item strong {
    grid-column: 2;
  }

  .sync-overview {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .sync-overview > div:nth-child(3) {
    border-right: 0;
  }

  .sync-overview > div:nth-child(n + 4) {
    border-top: 1px solid var(--border);
  }
}

@media (max-width: 720px) {
  .settings-page {
    padding: 18px 14px calc(86px + env(safe-area-inset-bottom));
  }

  .page-header {
    align-items: flex-end;
  }

  .page-header h1 {
    font-size: 22px;
  }

  .page-header p {
    max-width: 58vw;
  }

  .health-bar {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .health-item:nth-child(2) {
    border-right: 0;
  }

  .health-item:nth-child(n + 3) {
    border-top: 1px solid var(--border);
  }

  .settings-shell {
    grid-template-columns: 1fr;
  }

  .settings-nav {
    position: static;
    grid-auto-columns: minmax(132px, 1fr);
    grid-auto-flow: column;
    overflow-x: auto;
    overscroll-behavior-inline: contain;
  }

  .nav-item {
    min-width: 132px;
  }

  .settings-workspace {
    min-height: 560px;
  }

  .settings-section {
    padding: 16px;
  }

  .section-header {
    min-height: 66px;
  }

  .form-grid.two,
  .range-grid {
    grid-template-columns: 1fr;
  }

  .runtime-target-grid {
    grid-template-columns: 1fr;
  }

  .package-summary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .package-summary > div:nth-child(2),
  .package-summary > div:nth-child(4) {
    border-right: 0;
  }

  .package-summary > div:nth-child(n + 3) {
    border-top: 1px solid var(--border);
  }

  .path-row {
    grid-template-columns: 1fr;
    gap: 7px;
  }

  .sync-overview {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .sync-overview > div:nth-child(odd) {
    border-right: 1px solid var(--border);
  }

  .sync-overview > div:nth-child(even) {
    border-right: 0;
  }

  .sync-overview > div:nth-child(n + 3) {
    border-top: 1px solid var(--border);
  }

  .metric-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .metric-grid > div:nth-child(2) {
    border-right: 0;
  }

  .metric-grid > div:nth-child(n + 3) {
    border-top: 1px solid var(--border);
  }

  .settings-toast {
    right: 14px;
    bottom: calc(72px + env(safe-area-inset-bottom));
  }
}

@media (max-width: 430px) {
  .header-actions .btn {
    width: 34px;
    padding: 0;
    overflow: hidden;
    font-size: 0;
  }

  .header-actions .btn svg {
    margin: 0;
  }

  .health-item {
    padding-inline: 10px;
  }

  .settings-nav {
    grid-auto-columns: 108px;
  }

  .nav-item {
    min-width: 108px;
  }

  .nav-copy small {
    display: none;
  }

  .segmented-control button {
    min-height: 42px;
    flex-direction: column;
    font-size: 9px;
  }

  .command-row .btn {
    flex: 1 1 132px;
  }
}
</style>
