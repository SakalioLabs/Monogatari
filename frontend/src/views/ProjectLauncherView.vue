<template>
  <div class="project-launcher">
    <header class="launcher-header">
      <div class="brand">
        <img :src="appIconUrl" alt="">
        <span>
          <strong>Monogatari</strong>
          <small>{{ t('projects.engine-label', 'Visual novel engine') }}</small>
        </span>
      </div>
      <label class="language-control">
        <Languages :size="16" aria-hidden="true" />
        <select :value="locale" :aria-label="t('app.language', 'Language')" @change="changeLocale">
          <option v-for="item in supportedLocales" :key="item.code" :value="item.code">{{ item.label }}</option>
        </select>
      </label>
    </header>

    <main class="launcher-main">
      <section class="launcher-intro">
        <span>{{ t('projects.eyebrow', 'Project launcher') }}</span>
        <h1>{{ t('projects.title', 'Choose a project') }}</h1>
        <p>{{ t('projects.copy', 'Open recent work, create an empty project, or import a verified project package.') }}</p>
      </section>

      <div v-if="errorMessage" class="launcher-notice error" role="alert">
        <TriangleAlert :size="17" aria-hidden="true" />
        <span>{{ errorMessage }}</span>
        <button class="icon-button" :title="t('common.close', 'Close')" @click="errorMessage = ''">
          <X :size="16" aria-hidden="true" />
        </button>
      </div>

      <section v-if="workspace?.active_project" class="active-project-band">
        <div>
          <span>{{ t('projects.active-label', 'Open project') }}</span>
          <strong>{{ workspace.active_project.project_title }}</strong>
          <code>{{ workspace.active_project.project_path }}</code>
        </div>
        <div class="active-actions">
          <button v-if="desktopAvailable" class="btn btn-secondary" :disabled="busy" @click="closeCurrentProject">
            <LogOut :size="15" aria-hidden="true" />
            {{ t('projects.close-project', 'Close') }}
          </button>
          <button class="btn btn-primary" :disabled="busy" @click="continueToWorkspace">
            {{ t('projects.continue', 'Continue') }}
            <ArrowRight :size="15" aria-hidden="true" />
          </button>
        </div>
      </section>

      <div class="launcher-grid">
        <section class="recent-projects">
          <div class="section-heading">
            <div>
              <span>{{ t('projects.recent-label', 'History') }}</span>
              <h2>{{ t('projects.recent-title', 'Recent projects') }}</h2>
            </div>
            <button class="icon-button" :title="t('common.refresh', 'Refresh')" :disabled="busy" @click="loadWorkspace">
              <RefreshCw :size="17" :class="{ spinning: loading }" aria-hidden="true" />
            </button>
          </div>

          <div v-if="workspace?.recent_projects.length" class="project-list">
            <article v-for="project in workspace.recent_projects" :key="project.project_path" class="project-row">
              <button class="project-open" :disabled="busy || !project.available" @click="openRecent(project)">
                <span class="project-icon"><FolderOpen :size="18" aria-hidden="true" /></span>
                <span class="project-copy">
                  <strong>{{ project.project_title }}</strong>
                  <code>{{ project.project_path }}</code>
                  <small v-if="project.available">{{ formatOpenedAt(project.last_opened_at) }}</small>
                  <small v-else class="missing">{{ t('projects.missing', 'Project folder is unavailable') }}</small>
                </span>
                <ArrowRight :size="16" aria-hidden="true" />
              </button>
              <button
                class="icon-button forget-button"
                :title="t('projects.forget', 'Remove from history')"
                :disabled="busy"
                @click="forgetRecent(project)"
              >
                <Trash2 :size="16" aria-hidden="true" />
              </button>
            </article>
          </div>

          <div v-else class="empty-state">
            <FolderClock :size="28" aria-hidden="true" />
            <strong>{{ t('projects.empty-title', 'No project history yet') }}</strong>
            <p>{{ t('projects.empty-copy', 'Projects you open or create will remain listed on this device.') }}</p>
          </div>
        </section>

        <aside class="project-actions">
          <section class="action-section">
            <span>{{ t('projects.new-label', 'Add project') }}</span>
            <h2>{{ t('projects.new-title', 'Start or bring in work') }}</h2>
            <div class="action-list">
              <button :disabled="busy || !desktopAvailable" @click="showCreateDialog">
                <span><FolderPlus :size="18" aria-hidden="true" /></span>
                <strong>{{ t('projects.create', 'Create empty project') }}</strong>
                <small>{{ t('projects.create-copy', 'Create a clean project with no story or character content.') }}</small>
              </button>
              <button :disabled="busy || !desktopAvailable" @click="browseExistingProject">
                <span><FolderOpen :size="18" aria-hidden="true" /></span>
                <strong>{{ t('projects.open-folder', 'Open project folder') }}</strong>
                <small>{{ t('projects.open-folder-copy', 'Select an existing folder containing settings.json.') }}</small>
              </button>
              <button :disabled="busy || !desktopAvailable" @click="importPackage">
                <span><PackageOpen :size="18" aria-hidden="true" /></span>
                <strong>{{ t('projects.import', 'Import project package') }}</strong>
                <small>{{ t('projects.import-copy', 'Verify and install a portable .monogatari package.') }}</small>
              </button>
            </div>
          </section>
        </aside>
      </div>

      <section v-if="workspace?.sample_projects.length" class="sample-section">
        <div class="section-heading">
          <div>
            <span>{{ t('projects.samples-label', 'Examples') }}</span>
            <h2>{{ t('projects.samples-title', 'Sample projects') }}</h2>
          </div>
          <p>{{ t('projects.samples-copy', 'Examples are distributed as independent project packages and are never embedded in the engine.') }}</p>
        </div>
        <div class="sample-list">
          <article v-for="sample in workspace.sample_projects" :key="sample.id">
            <div>
              <strong>{{ sample.title }}</strong>
              <p>{{ sample.description }}</p>
              <code>{{ sample.package_file_name }}</code>
            </div>
            <button class="btn btn-secondary" :disabled="busy" @click="importPackage">
              <PackageOpen :size="15" aria-hidden="true" />
              {{ t('projects.import-sample', 'Import package') }}
            </button>
          </article>
        </div>
      </section>
    </main>

    <dialog ref="createDialog" class="create-dialog" @close="resetCreateForm">
      <form method="dialog" @submit.prevent>
        <header>
          <div>
            <span>{{ t('projects.create-label', 'New project') }}</span>
            <h2>{{ t('projects.create-title', 'Create an empty project') }}</h2>
          </div>
          <button class="icon-button" :title="t('common.close', 'Close')" @click="createDialog?.close()">
            <X :size="17" aria-hidden="true" />
          </button>
        </header>

        <label>
          <span>{{ t('projects.project-name', 'Project name') }}</span>
          <input v-model.trim="createForm.projectTitle" maxlength="120" autocomplete="off">
        </label>
        <label>
          <span>{{ t('projects.directory-name', 'Folder name') }}</span>
          <input v-model.trim="createForm.directoryName" maxlength="80" autocomplete="off">
        </label>
        <label>
          <span>{{ t('projects.parent-folder', 'Parent folder') }}</span>
          <div class="path-picker">
            <input v-model="createForm.parentDirectory" readonly>
            <button class="btn btn-secondary" type="button" @click="chooseParentDirectory">
              <FolderSearch :size="15" aria-hidden="true" />
              {{ t('common.browse', 'Browse') }}
            </button>
          </div>
        </label>

        <footer>
          <button class="btn btn-secondary" type="button" @click="createDialog?.close()">{{ t('common.cancel', 'Cancel') }}</button>
          <button class="btn btn-primary" type="button" :disabled="!canCreate || busy" @click="createEmptyProject">
            <LoaderCircle v-if="busy" :size="15" class="spinning" aria-hidden="true" />
            <FolderPlus v-else :size="15" aria-hidden="true" />
            {{ t('projects.create-action', 'Create project') }}
          </button>
        </footer>
      </form>
    </dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ArrowRight,
  FolderClock,
  FolderOpen,
  FolderPlus,
  FolderSearch,
  Languages,
  LoaderCircle,
  LogOut,
  PackageOpen,
  RefreshCw,
  Trash2,
  TriangleAlert,
  X,
} from '@lucide/vue'
import { importProjectPackage } from '../lib/projectArchive'
import {
  closeWorkspaceProject,
  createWorkspaceProject,
  forgetWorkspaceProject,
  openWorkspaceProject,
  projectWorkspace,
  refreshProjectWorkspace,
  type ProjectLauncherEntry,
} from '../lib/projectWorkspace'
import { hasTauriRuntime } from '../lib/tauri'
import { useI18n } from '../lib/i18n'

const appIconUrl = `${import.meta.env.BASE_URL}icons/app-icon.svg`
const router = useRouter()
const route = useRoute()
const { locale, supportedLocales, setLocale, t } = useI18n()
const workspace = projectWorkspace
const desktopAvailable = hasTauriRuntime()
const loading = ref(true)
const busy = ref(false)
const errorMessage = ref('')
const createDialog = ref<HTMLDialogElement | null>(null)
const createForm = reactive({
  projectTitle: '',
  directoryName: '',
  parentDirectory: '',
})
const canCreate = computed(() => (
  createForm.projectTitle.length > 0
  && createForm.directoryName.length > 0
  && createForm.parentDirectory.length > 0
))

async function runAction(action: () => Promise<void>) {
  busy.value = true
  errorMessage.value = ''
  try {
    await action()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    busy.value = false
  }
}

async function loadWorkspace() {
  loading.value = true
  errorMessage.value = ''
  try {
    await refreshProjectWorkspace()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
}

function destinationAfterOpen() {
  const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/workspace'
  return redirect.startsWith('/') && redirect !== '/' ? redirect : '/workspace'
}

function continueToWorkspace() {
  void router.push(destinationAfterOpen())
}

function openRecent(project: ProjectLauncherEntry) {
  void runAction(async () => {
    await openWorkspaceProject(project.project_path)
    continueToWorkspace()
  })
}

function forgetRecent(project: ProjectLauncherEntry) {
  void runAction(async () => {
    await forgetWorkspaceProject(project.project_path)
  })
}

function closeCurrentProject() {
  void runAction(async () => {
    await closeWorkspaceProject()
  })
}

async function browseExistingProject() {
  await runAction(async () => {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      title: t('projects.open-folder', 'Open project folder'),
      directory: true,
      multiple: false,
      recursive: false,
    })
    if (!selected || Array.isArray(selected)) return
    await openWorkspaceProject(selected)
    continueToWorkspace()
  })
}

async function importPackage() {
  await runAction(async () => {
    const flow = await importProjectPackage()
    if (!flow?.imported) return
    await openWorkspaceProject(flow.imported.project_path)
    continueToWorkspace()
  })
}

function showCreateDialog() {
  createDialog.value?.showModal()
}

async function chooseParentDirectory() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({
    title: t('projects.parent-folder', 'Parent folder'),
    directory: true,
    multiple: false,
    recursive: false,
  })
  if (selected && !Array.isArray(selected)) createForm.parentDirectory = selected
}

function createEmptyProject() {
  void runAction(async () => {
    await createWorkspaceProject({ ...createForm })
    createDialog.value?.close()
    continueToWorkspace()
  })
}

function resetCreateForm() {
  createForm.projectTitle = ''
  createForm.directoryName = ''
  createForm.parentDirectory = ''
}

function formatOpenedAt(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return new Intl.DateTimeFormat(locale.value, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

async function changeLocale(event: Event) {
  await setLocale((event.target as HTMLSelectElement).value)
}

onMounted(() => {
  void loadWorkspace()
})
</script>

<style scoped>
.project-launcher {
  min-height: 100vh;
  min-height: 100svh;
  background: var(--surface-0);
  color: var(--text-primary);
}

.launcher-header {
  display: flex;
  min-height: 68px;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
  padding: 0 32px;
  background: var(--surface-1);
}

.brand { display: flex; align-items: center; gap: 10px; }
.brand img { width: 34px; height: 34px; }
.brand span { display: grid; }
.brand strong { font-size: 15px; }
.brand small, .launcher-intro p, .section-heading p, .empty-state p { color: var(--text-secondary); }
.language-control { display: flex; align-items: center; gap: 7px; color: var(--text-secondary); }
.language-control select { min-height: 34px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface-1); color: var(--text-primary); padding: 0 26px 0 9px; }

.launcher-main { width: min(1180px, 100%); margin: 0 auto; padding: 42px 32px 64px; }
.launcher-intro { margin-bottom: 24px; }
.launcher-intro > span, .section-heading span, .action-section > span, .active-project-band > div > span, .create-dialog header span {
  color: var(--text-tertiary);
  font-size: 10px;
  font-weight: 750;
  text-transform: uppercase;
}
.launcher-intro h1 { margin: 6px 0 7px; font-size: 31px; line-height: 1.15; }
.launcher-intro p { max-width: 680px; margin: 0; font-size: 13px; }

.launcher-notice { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; margin-bottom: 16px; border: 1px solid var(--danger); border-radius: var(--radius-sm); padding: 10px 12px; background: color-mix(in srgb, var(--danger) 7%, var(--surface-1)); color: var(--danger); font-size: 12px; }
.active-project-band { display: flex; align-items: center; justify-content: space-between; gap: 20px; margin-bottom: 22px; border-block: 1px solid var(--border); padding: 14px 0; }
.active-project-band > div:first-child { display: grid; min-width: 0; gap: 3px; }
.active-project-band strong { font-size: 16px; }
.active-project-band code, .project-copy code { overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 10px; text-overflow: ellipsis; white-space: nowrap; }
.active-actions { display: flex; flex: 0 0 auto; gap: 8px; }

.launcher-grid { display: grid; grid-template-columns: minmax(0, 1.65fr) minmax(300px, .8fr); gap: 34px; }
.section-heading { display: flex; align-items: flex-end; justify-content: space-between; gap: 18px; margin-bottom: 13px; }
.section-heading h2, .action-section h2 { margin: 4px 0 0; font-size: 17px; }
.section-heading p { max-width: 520px; margin: 0; font-size: 11px; text-align: right; }

.project-list { border-top: 1px solid var(--border); }
.project-row { display: grid; grid-template-columns: minmax(0, 1fr) 36px; align-items: center; border-bottom: 1px solid var(--border); }
.project-open { display: grid; min-width: 0; grid-template-columns: 38px minmax(0, 1fr) auto; align-items: center; gap: 11px; border: 0; padding: 12px 8px 12px 0; background: transparent; color: inherit; text-align: left; }
.project-open:not(:disabled):hover { background: var(--surface-1); }
.project-open:disabled { cursor: not-allowed; opacity: .55; }
.project-icon { display: grid; width: 34px; height: 34px; place-items: center; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-secondary); }
.project-copy { display: grid; min-width: 0; gap: 2px; }
.project-copy strong { font-size: 13px; }
.project-copy small { color: var(--text-secondary); font-size: 10px; }
.project-copy .missing { color: var(--danger); }
.forget-button { opacity: .65; }
.project-row:hover .forget-button { opacity: 1; }

.empty-state { display: grid; min-height: 210px; place-items: center; align-content: center; gap: 7px; border-block: 1px solid var(--border); color: var(--text-tertiary); text-align: center; }
.empty-state strong { color: var(--text-primary); font-size: 13px; }
.empty-state p { max-width: 340px; margin: 0; font-size: 11px; }

.project-actions { border-left: 1px solid var(--border); padding-left: 28px; }
.action-list { display: grid; gap: 8px; margin-top: 14px; }
.action-list > button { display: grid; grid-template-columns: 38px minmax(0, 1fr); gap: 2px 10px; border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 11px; background: var(--surface-1); color: inherit; text-align: left; }
.action-list > button:not(:disabled):hover { border-color: var(--border-strong); background: var(--surface-2); }
.action-list > button:disabled { cursor: not-allowed; opacity: .5; }
.action-list > button > span { display: grid; grid-row: 1 / 3; width: 34px; height: 34px; place-items: center; border-radius: var(--radius-sm); background: var(--surface-2); color: var(--text-secondary); }
.action-list strong { font-size: 12px; }
.action-list small { color: var(--text-secondary); font-size: 10px; line-height: 1.35; }

.sample-section { margin-top: 38px; padding-top: 22px; border-top: 1px solid var(--border); }
.sample-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
.sample-list article { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; background: var(--surface-1); }
.sample-list article > div { display: grid; min-width: 0; gap: 5px; }
.sample-list strong { font-size: 13px; }
.sample-list p { margin: 0; color: var(--text-secondary); font-size: 11px; line-height: 1.45; }
.sample-list code { overflow: hidden; color: var(--text-tertiary); font-family: var(--font-mono); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }

.create-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  width: min(520px, calc(100vw - 32px));
  max-height: calc(100svh - 32px);
  margin: 0;
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 0;
  background: var(--surface-1);
  color: var(--text-primary);
  box-shadow: var(--shadow-lg);
  transform: translate(-50%, -50%);
}
.create-dialog::backdrop { background: rgb(0 0 0 / 42%); }
.create-dialog form { display: grid; gap: 16px; padding: 20px; }
.create-dialog header, .create-dialog footer { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.create-dialog h2 { margin: 4px 0 0; font-size: 18px; }
.create-dialog label { display: grid; gap: 6px; font-size: 11px; font-weight: 650; }
.create-dialog input { width: 100%; min-height: 38px; border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 0 10px; background: var(--surface-0); color: var(--text-primary); }
.path-picker { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 7px; }
.create-dialog footer { margin-top: 4px; justify-content: flex-end; }
.spinning { animation: spin 900ms linear infinite; }

@media (max-width: 780px) {
  .launcher-header { padding: 0 18px; }
  .launcher-main { padding: 28px 18px 48px; }
  .launcher-grid { grid-template-columns: 1fr; }
  .project-actions { border-top: 1px solid var(--border); border-left: 0; padding-top: 24px; padding-left: 0; }
  .sample-list { grid-template-columns: 1fr; }
  .active-project-band, .sample-list article { align-items: stretch; flex-direction: column; }
  .active-actions { width: 100%; }
  .active-actions .btn { flex: 1; }
  .section-heading { align-items: flex-start; flex-direction: column; }
  .section-heading p { text-align: left; }
}
</style>
