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
      'monogatari-installation-verification/v1',
      'version the installed-runtime report schema',
    ],
    [
      installationVerifierSource,
      '--verify-installation',
      'expose an explicit installed-runtime verification flag',
    ],
    [
      installationVerifierSource,
      'discover_bundled_project_data_root',
      'resolve data from the installed executable resource directory',
    ],
    [
      installationVerifierSource,
      'scrub_runtime_secret_config(&settings)',
      'reject bundled runtime secrets through the shared project policy',
    ],
    [
      installationVerifierSource,
      'ALLOWED_PROJECT_WARNING_CODES',
      'reject every warning outside the allowed runtime-credential set',
    ],
    [
      installationVerifierSource,
      'engine::load_project_content',
      'load bundled content through real runtime managers',
    ],
    [
      installationVerifierSource,
      'validate_story_ending_references',
      'validate bundled ending references',
    ],
    [
      installationVerifierSource,
      'load_workflow_from_project',
      'validate bundled workflows through the runtime loader',
    ],
    [
      installationVerifierSource,
      'parse_quality_suite',
      'validate bundled Quality Suite schemas',
    ],
    [
      installationVerifierSource,
      'load_locale_from_project',
      'validate bundled locale schemas',
    ],
    [
      installationVerifierSource,
      'build_project_export_manifest',
      'fingerprint the complete bundled project inventory',
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
      'checked_in_data_passes_installed_runtime_verification',
      'test checked-in data through installed-runtime verification',
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
      'compareContentSets(sourceData, installedData)',
      'compare source and installed resource hashes',
    ],
    [
      windowsInstallerVerifierSource,
      "['--verify-installation', reportPath]",
      'run the extracted production executable verifier',
    ],
    [
      windowsInstallerVerifierSource,
      'JSON.stringify([])',
      'require the bundled DirectML project to install without configuration warnings',
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
