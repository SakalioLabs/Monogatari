import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

export async function collectTauriInstallationPolicyEvidence(options = {}) {
  const {
    repositoryRoot,
    tauriAppDirectory,
  } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const issues = []
  const tauriMainSource = await readFile(
    path.join(tauriAppDirectory, 'src', 'main.rs'),
    'utf8',
  )
  const installationVerifierSource = await readFile(
    path.join(tauriAppDirectory, 'src', 'installation_verifier.rs'),
    'utf8',
  )
  const windowsInstallerVerifierSource = await readFile(
    path.join(repositoryRoot, 'scripts', 'verify-windows-installers.mjs'),
    'utf8',
  )

  const requirements = [
    [
      tauriMainSource,
      'installation_verifier::run_requested_verification()',
      'run headless installation verification before opening Tauri',
    ],
    [
      installationVerifierSource,
      'monogatari-installation-verification/v2',
      'version the installed-runtime report schema',
    ],
    [
      installationVerifierSource,
      '--verify-installation',
      'expose an explicit installed-runtime verification flag',
    ],
    [
      installationVerifierSource,
      'PROHIBITED_PROJECT_ENTRIES',
      'enumerate project content forbidden beside the installed executable',
    ],
    [
      installationVerifierSource,
      'project_content_embedded: false',
      'report that the installed engine contains no project content',
    ],
    [
      installationVerifierSource,
      'Installed engine must not contain project content',
      'reject an installation that contains adjacent project data',
    ],
    [
      installationVerifierSource,
      'installation_rejects_adjacent_project_content',
      'test that project content invalidates an installed shell',
    ],
    [
      installationVerifierSource,
      'MONOGATARI_GIT_COMMIT',
      'bind reports to the binary build commit',
    ],
    [installationVerifierSource, 'write_envelope', 'write a structured success or failure report'],
    [
      installationVerifierSource,
      'std::fs::rename(&stage_path, report_path)',
      'atomically replace the verification report',
    ],
    [
      installationVerifierSource,
      'project_free_installation_passes_verification',
      'test a project-free installed shell',
    ],
    [
      windowsInstallerVerifierSource,
      'monogatari-windows-installer-audit/v1',
      'version Windows installer audit evidence',
    ],
    [
      windowsInstallerVerifierSource,
      'WindowsInstaller.Installer',
      'query MSI package metadata through the Windows Installer API',
    ],
    [
      windowsInstallerVerifierSource,
      'Get-AuthenticodeSignature',
      'inspect real Authenticode status',
    ],
    [
      windowsInstallerVerifierSource,
      'application_signature: applicationSignature',
      'inspect the extracted application signature',
    ],
    [
      windowsInstallerVerifierSource,
      "signature.status === 'NotSigned'",
      'limit unsigned exceptions to genuinely unsigned files',
    ],
    [
      windowsInstallerVerifierSource,
      'expectedSignerFragment',
      'bind valid signatures to the expected publisher identity',
    ],
    [
      windowsInstallerVerifierSource,
      'expectedMsiUpgradeCode',
      'verify the stable MSI upgrade identity',
    ],
    [
      windowsInstallerVerifierSource,
      'createReadStream',
      'hash release artifacts with bounded streaming reads',
    ],
    [
      windowsInstallerVerifierSource,
      "spawnSync('msiexec.exe'",
      'administratively extract MSI payloads',
    ],
    [
      windowsInstallerVerifierSource,
      'Administrative image contains project content',
      'reject project content found in the extracted MSI image',
    ],
    [
      windowsInstallerVerifierSource,
      "['--verify-installation', reportPath]",
      'run the extracted production executable verifier',
    ],
    [
      windowsInstallerVerifierSource,
      'envelope.report.project_content_embedded !== false',
      'require the extracted application to prove project-free startup',
    ],
    [
      windowsInstallerVerifierSource,
      'envelope.report.git_commit !== sourceState.git_commit',
      'reject stale clean-worktree binaries',
    ],
    [
      windowsInstallerVerifierSource,
      "'--untracked-files=all'",
      'reject untracked source content from persisted audit evidence',
    ],
    [
      windowsInstallerVerifierSource,
      "argSet.has('--allow-unsigned')",
      'make unsigned internal audits explicit',
    ],
    [
      windowsInstallerVerifierSource,
      "status !== 'Valid'",
      'block public audits without valid signatures',
    ],
  ]

  for (const [source, needle, description] of requirements) {
    if (!source.includes(needle)) {
      issues.push(`Installed desktop verification must ${description}`)
    }
  }

  return {
    issues,
    requirementCount: requirements.length,
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    repositoryRoot: options.repositoryRoot,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri installation policy requires ${name}.`)
    }
  }
  return boundaries
}
