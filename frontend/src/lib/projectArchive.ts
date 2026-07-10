import { hasTauriRuntime, invokeCommand } from './tauri'

export interface ProjectArchiveInspection {
  archive_path: string
  project_title: string
  engine_version: string
  file_count: number
  total_bytes: number
  archive_bytes: number
  content_sha256: string
  verified: boolean
}

export interface ProjectArchiveExportResult {
  archive_path: string
  project_path: string
  project_title: string
  file_count: number
  total_bytes: number
  archive_bytes: number
  content_sha256: string
}

export interface ProjectArchiveImportResult {
  archive_path: string
  project_path: string
  project_title: string
  directory_name: string
  file_count: number
  total_bytes: number
  content_sha256: string
}

export interface ProjectArchiveImportFlow {
  inspection: ProjectArchiveInspection
  imported: ProjectArchiveImportResult | null
}

const projectPackageFilter = [{
  name: 'Monogatari Project',
  extensions: ['monogatari'],
}]

export async function exportProjectPackage(
  projectPath: string,
  projectTitle: string,
): Promise<ProjectArchiveExportResult | null> {
  requireDesktopPackages()
  const { save } = await import('@tauri-apps/plugin-dialog')
  const selected = await save({
    title: 'Export Monogatari Project',
    defaultPath: `${safeFileName(projectTitle)}.monogatari`,
    filters: projectPackageFilter,
  })
  if (!selected) return null
  const destinationPath = selected.toLowerCase().endsWith('.monogatari')
    ? selected
    : `${selected}.monogatari`
  return invokeCommand<ProjectArchiveExportResult>('export_project_archive', {
    projectPath,
    destinationPath,
  })
}

export async function importProjectPackage(): Promise<ProjectArchiveImportFlow | null> {
  requireDesktopPackages()
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selectedArchive = await open({
    title: 'Open Monogatari Project',
    directory: false,
    multiple: false,
    filters: projectPackageFilter,
  })
  if (!selectedArchive || Array.isArray(selectedArchive)) return null

  const inspection = await invokeCommand<ProjectArchiveInspection>('inspect_project_archive', {
    archivePath: selectedArchive,
  })
  const destinationParent = await open({
    title: `Import ${inspection.project_title}`,
    directory: true,
    multiple: false,
    recursive: false,
  })
  if (!destinationParent || Array.isArray(destinationParent)) {
    return { inspection, imported: null }
  }
  const imported = await invokeCommand<ProjectArchiveImportResult>('import_project_archive', {
    archivePath: selectedArchive,
    destinationParent,
  })
  return { inspection, imported }
}

export function projectPackagesAvailable(): boolean {
  return hasTauriRuntime()
}

function requireDesktopPackages() {
  if (!projectPackagesAvailable()) {
    throw new Error('Project packages require the installed Monogatari app.')
  }
}

function safeFileName(value: string) {
  return value.trim().replace(/[^a-z0-9._-]+/gi, '-').replace(/^-+|-+$/g, '') || 'monogatari-project'
}
