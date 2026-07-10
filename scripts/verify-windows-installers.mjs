import { execFileSync, spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { createReadStream } from 'node:fs'
import { lstat, mkdtemp, readFile, readdir, rm, stat, writeFile, mkdir } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const rustDir = path.join(root, 'rust-engine')
const tauriAppDir = path.join(rustDir, 'crates', 'tauri-app')
const bundleDir = path.join(rustDir, 'target', 'release', 'bundle')
const sourceDataDir = path.join(root, 'data')
const defaultOutPath = path.join(root, 'release', 'monogatari-windows-installer-audit.json')
const auditSchema = 'monogatari-windows-installer-audit/v1'
const runtimeSchema = 'monogatari-installation-verification/v1'
const minimumInstallerBytes = 1024 * 1024
const maximumInstallerBytes = 512 * 1024 * 1024
const maximumPayloadFiles = 20_000
const maximumPayloadEntries = 30_000
const maximumPayloadBytes = 16 * 1024 * 1024 * 1024
const expectedSignerFragment = 'SakalioLabs'
const expectedMsiUpgradeCode = 'c4c2d20f-f307-5c7b-91e6-5edeea14fdd0'

const args = process.argv.slice(2)
const argSet = new Set(args)
const checkOnly = argSet.has('--check')
const allowUnsigned = argSet.has('--allow-unsigned')
const allowDirtyWorktree = argSet.has('--allow-dirty-worktree')
const outPath = path.resolve(root, readArg('out') ?? defaultOutPath)

main().catch((error) => {
  console.error(`[windows-installer] ${error.message}`)
  process.exit(1)
})

async function main() {
  if (process.platform !== 'win32') {
    throw new Error('Windows installer verification requires a Windows host.')
  }
  const version = await readVersion()
  const sourceState = gitSourceState()
  if (!checkOnly && sourceState.worktree_dirty && !allowDirtyWorktree) {
    throw new Error('Writing installer audit evidence requires a clean worktree.')
  }

  const installers = await locateInstallers(version)
  const sourceData = await contentSet(sourceDataDir)
  const metadata = installerMetadata(installers.msi.path, installers.nsis.path)
  validateInstallerMetadata(metadata, version)

  for (const installer of Object.values(installers)) {
    installer.sha256 = await sha256(installer.path)
    installer.size_bytes = (await stat(installer.path)).size
    if (installer.size_bytes < minimumInstallerBytes || installer.size_bytes > maximumInstallerBytes) {
      throw new Error(`${relative(installer.path)} has implausible size ${installer.size_bytes} bytes.`)
    }
  }

  validateSignatureSet([
    { name: 'MSI installer', signature: metadata.msi.signature },
    { name: 'NSIS installer', signature: metadata.nsis.signature },
  ], allowUnsigned, 'Installers')

  const temporaryRoot = await mkdtemp(path.join(tmpdir(), 'monogatari-msi-audit-'))
  let extractionEvidence
  try {
    extractionEvidence = await verifyAdministrativeImage(
      installers.msi.path,
      temporaryRoot,
      sourceData,
      version,
      sourceState,
      allowUnsigned,
    )
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true })
    await rm(`${temporaryRoot}.log`, { force: true })
  }

  const signatureStatuses = [
    metadata.msi.signature.status,
    metadata.nsis.signature.status,
    extractionEvidence.application_signature.status,
  ]
  const audit = {
    schema: auditSchema,
    product: 'Monogatari',
    version,
    generated_at: new Date().toISOString(),
    release_ready: signatureStatuses.every((status) => status === 'Valid'),
    unsigned_allowed_for_audit: allowUnsigned,
    source_state: sourceState,
    source_data: {
      file_count: sourceData.file_count,
      total_bytes: sourceData.total_bytes,
      content_sha256: sourceData.content_sha256,
    },
    installers: [
      installerEntry(installers.msi, metadata.msi),
      installerEntry(installers.nsis, metadata.nsis),
    ],
    administrative_image: extractionEvidence,
  }

  if (!checkOnly) {
    if (!outPath.startsWith(root + path.sep)) {
      throw new Error('Installer audit output path must stay inside the repository.')
    }
    await mkdir(path.dirname(outPath), { recursive: true })
    await writeFile(outPath, `${JSON.stringify(audit, null, 2)}\n`, 'utf8')
    console.log(`[windows-installer] Wrote ${relative(outPath)}`)
  }
  console.log(
    `[windows-installer] OK (MSI ${formatBytes(installers.msi.size_bytes)}, NSIS ${formatBytes(installers.nsis.size_bytes)}, ${sourceData.file_count} bundled data files, signatures=${signatureStatuses.join('/')}, runtime=${extractionEvidence.runtime.status})`,
  )
}

async function readVersion() {
  const config = JSON.parse(await readFile(path.join(tauriAppDir, 'tauri.conf.json'), 'utf8'))
  const packageJson = JSON.parse(await readFile(path.join(root, 'frontend', 'package.json'), 'utf8'))
  if (config.version !== packageJson.version || !/^\d+\.\d+\.\d+$/.test(config.version ?? '')) {
    throw new Error(`Installer source versions are inconsistent: ${config.version} / ${packageJson.version}.`)
  }
  return config.version
}

async function locateInstallers(version) {
  const files = await walkFiles(bundleDir)
  const escapedVersion = version.replaceAll('.', '\\.')
  const msiPattern = new RegExp(`^Monogatari_${escapedVersion}_x64_en-US\\.msi$`)
  const nsisPattern = new RegExp(`^Monogatari_${escapedVersion}_x64-setup\\.exe$`)
  const msiFiles = files.filter((file) => msiPattern.test(path.basename(file)))
  const nsisFiles = files.filter((file) => nsisPattern.test(path.basename(file)))
  if (msiFiles.length !== 1 || nsisFiles.length !== 1) {
    throw new Error(`Expected one MSI and one NSIS installer for ${version}; found ${msiFiles.length}/${nsisFiles.length}.`)
  }
  return {
    msi: { kind: 'msi-installer', path: msiFiles[0] },
    nsis: { kind: 'nsis-installer', path: nsisFiles[0] },
  }
}

function installerMetadata(msiPath, nsisPath) {
  const source = String.raw`
$ErrorActionPreference = 'Stop'
function Signature-Info([string]$Path) {
  $signature = Get-AuthenticodeSignature -LiteralPath $Path
  return [ordered]@{
    status = [string]$signature.Status
    status_message = [string]$signature.StatusMessage
    subject = if ($signature.SignerCertificate) { [string]$signature.SignerCertificate.Subject } else { $null }
    thumbprint = if ($signature.SignerCertificate) { [string]$signature.SignerCertificate.Thumbprint } else { $null }
  }
}
$msiPath = $env:MONOGATARI_AUDIT_MSI
$nsisPath = $env:MONOGATARI_AUDIT_NSIS
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.GetType().InvokeMember('OpenDatabase', 'InvokeMethod', $null, $installer, @($msiPath, 0))
$propertyView = $database.GetType().InvokeMember('OpenView', 'InvokeMethod', $null, $database, @('SELECT Property, Value FROM Property'))
$propertyView.GetType().InvokeMember('Execute', 'InvokeMethod', $null, $propertyView, $null) | Out-Null
$properties = @{}
while ($record = $propertyView.GetType().InvokeMember('Fetch', 'InvokeMethod', $null, $propertyView, $null)) {
  $name = $record.GetType().InvokeMember('StringData', 'GetProperty', $null, $record, 1)
  $value = $record.GetType().InvokeMember('StringData', 'GetProperty', $null, $record, 2)
  $properties[$name] = $value
}
$fileView = $database.GetType().InvokeMember('OpenView', 'InvokeMethod', $null, $database, @('SELECT File FROM File'))
$fileView.GetType().InvokeMember('Execute', 'InvokeMethod', $null, $fileView, $null) | Out-Null
$fileCount = 0
while ($record = $fileView.GetType().InvokeMember('Fetch', 'InvokeMethod', $null, $fileView, $null)) { $fileCount++ }
$nsisVersion = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($nsisPath)
[ordered]@{
  msi = [ordered]@{
    product_name = [string]$properties['ProductName']
    product_version = [string]$properties['ProductVersion']
    manufacturer = [string]$properties['Manufacturer']
    product_code = [string]$properties['ProductCode']
    upgrade_code = [string]$properties['UpgradeCode']
    all_users = [string]$properties['ALLUSERS']
    arp_product_icon = [string]$properties['ARPPRODUCTICON']
    file_table_count = $fileCount
    signature = Signature-Info $msiPath
  }
  nsis = [ordered]@{
    product_name = [string]$nsisVersion.ProductName
    product_version = [string]$nsisVersion.ProductVersion
    file_version = [string]$nsisVersion.FileVersion
    file_description = [string]$nsisVersion.FileDescription
    legal_copyright = [string]$nsisVersion.LegalCopyright
    signature = Signature-Info $nsisPath
  }
} | ConvertTo-Json -Depth 6 -Compress
`
  const output = execFileSync('powershell.exe', [
    '-NoProfile',
    '-NonInteractive',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
    source,
  ], {
    cwd: root,
    encoding: 'utf8',
    windowsHide: true,
    env: {
      ...process.env,
      MONOGATARI_AUDIT_MSI: msiPath,
      MONOGATARI_AUDIT_NSIS: nsisPath,
    },
  }).trim()
  return JSON.parse(output)
}

function validateInstallerMetadata(metadata, version) {
  const guid = /^\{[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}\}$/i
  requireValue(metadata.msi.product_name, 'Monogatari', 'MSI ProductName')
  requireValue(metadata.msi.product_version, version, 'MSI ProductVersion')
  requireValue(metadata.msi.manufacturer, 'SakalioLabs', 'MSI Manufacturer')
  requireValue(metadata.msi.all_users, '1', 'MSI ALLUSERS')
  requireValue(metadata.msi.arp_product_icon, 'ProductIcon', 'MSI ARPPRODUCTICON')
  if (!guid.test(metadata.msi.product_code) || !guid.test(metadata.msi.upgrade_code)) {
    throw new Error('MSI ProductCode and UpgradeCode must be GUIDs.')
  }
  if (metadata.msi.upgrade_code.replace(/[{}]/g, '').toLowerCase() !== expectedMsiUpgradeCode) {
    throw new Error(`MSI UpgradeCode must stay ${expectedMsiUpgradeCode}; observed ${metadata.msi.upgrade_code}.`)
  }
  if (!Number.isInteger(metadata.msi.file_table_count) || metadata.msi.file_table_count < 2) {
    throw new Error('MSI File table is unexpectedly empty.')
  }
  requireValue(metadata.nsis.product_name, 'Monogatari', 'NSIS ProductName')
  requireValue(metadata.nsis.product_version, version, 'NSIS ProductVersion')
  requireValue(metadata.nsis.file_version, version, 'NSIS FileVersion')
  requireValue(metadata.nsis.file_description, 'Monogatari', 'NSIS FileDescription')
  if (!metadata.nsis.legal_copyright.includes('SakalioLabs')) {
    throw new Error('NSIS LegalCopyright must identify SakalioLabs.')
  }
}

async function verifyAdministrativeImage(
  msiPath,
  temporaryRoot,
  sourceData,
  version,
  sourceState,
  allowUnsigned,
) {
  const extractionRoot = path.join(temporaryRoot, 'image')
  const logPath = `${temporaryRoot}.log`
  const extraction = spawnSync('msiexec.exe', [
    '/a',
    msiPath,
    '/qn',
    `TARGETDIR=${extractionRoot}`,
    '/l*v',
    logPath,
  ], {
    cwd: root,
    encoding: 'utf8',
    windowsHide: true,
    timeout: 180_000,
  })
  if (extraction.error || extraction.status !== 0) {
    const logTail = await tailFile(logPath, 40)
    throw new Error(`MSI administrative extraction failed (${extraction.status ?? extraction.error?.message}).\n${logTail}`)
  }

  const extractedFiles = await walkFiles(extractionRoot)
  const applicationExecutables = extractedFiles.filter((file) => path.basename(file).toLowerCase() === 'llm-galgame-app.exe')
  if (applicationExecutables.length !== 1) {
    throw new Error(`Administrative image must contain one application executable; found ${applicationExecutables.length}.`)
  }
  const executablePath = applicationExecutables[0]
  const installedDataDir = path.join(path.dirname(executablePath), 'data')
  const applicationSignature = authenticodeSignature(executablePath)
  validateSignatureSet(
    [{ name: 'extracted application', signature: applicationSignature }],
    allowUnsigned,
    'Extracted application',
  )
  const installedData = await contentSet(installedDataDir)
  compareContentSets(sourceData, installedData)

  const reportPath = path.join(temporaryRoot, 'runtime-verification.json')
  const runtime = spawnSync(executablePath, ['--verify-installation', reportPath], {
    cwd: path.dirname(executablePath),
    encoding: 'utf8',
    windowsHide: true,
    timeout: 120_000,
  })
  if (runtime.error || runtime.status !== 0) {
    throw new Error(`Extracted application verification failed (${runtime.status ?? runtime.error?.message}).`)
  }
  const envelope = JSON.parse(await readFile(reportPath, 'utf8'))
  if (envelope.schema !== runtimeSchema || envelope.status !== 'verified' || envelope.report?.status !== 'verified') {
    throw new Error(`Extracted application returned an invalid verification report: ${envelope.error ?? envelope.status}.`)
  }
  if (envelope.report.engine_version !== version) {
    throw new Error(`Extracted application reports engine version ${envelope.report.engine_version}, expected ${version}.`)
  }
  if (envelope.report.data_file_count !== sourceData.file_count) {
    throw new Error('Extracted runtime data file count does not match source data.')
  }
  if (!/^[a-f0-9]{64}$/.test(envelope.report.project_content_sha256 ?? '')) {
    throw new Error('Extracted runtime project fingerprint is invalid.')
  }
  if (comparableWindowsPath(envelope.report.executable_path) !== comparableWindowsPath(executablePath)
      || comparableWindowsPath(envelope.report.resource_root) !== comparableWindowsPath(installedDataDir)) {
    throw new Error('Extracted runtime report paths do not identify the audited administrative image.')
  }
  if (!/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/.test(envelope.report.git_commit ?? '')) {
    throw new Error('Extracted runtime build commit is invalid.')
  }
  if (!/^[a-f0-9]{12}$/.test(envelope.report.git_short_commit ?? '')
      || !envelope.report.git_commit.startsWith(envelope.report.git_short_commit)) {
    throw new Error('Extracted runtime short build commit is invalid.')
  }
  if (!sourceState.worktree_dirty && envelope.report.git_commit !== sourceState.git_commit) {
    throw new Error(
      `Extracted runtime was built from ${envelope.report.git_short_commit}, expected ${sourceState.git_short_commit}.`,
    )
  }

  const runtimeReport = {
    ...envelope.report,
    executable_path: path.relative(extractionRoot, executablePath).replaceAll('\\', '/'),
    resource_root: path.relative(extractionRoot, installedDataDir).replaceAll('\\', '/'),
  }

  return {
    extraction_exit_code: extraction.status,
    payload_file_count: extractedFiles.length,
    application_path: path.relative(extractionRoot, executablePath).replaceAll('\\', '/'),
    application_signature: applicationSignature,
    source_data_match: true,
    source_data_content_sha256: sourceData.content_sha256,
    runtime: runtimeReport,
  }
}

async function contentSet(directory) {
  const files = await walkFiles(directory)
  if (files.length === 0 || files.length > maximumPayloadFiles) {
    throw new Error(`${relative(directory)} has invalid file count ${files.length}.`)
  }
  const records = []
  let totalBytes = 0
  for (const file of files) {
    const info = await stat(file)
    totalBytes += info.size
    if (totalBytes > maximumPayloadBytes) {
      throw new Error(`${relative(directory)} exceeds the payload size limit.`)
    }
    records.push({
      path: path.relative(directory, file).replaceAll('\\', '/'),
      size_bytes: info.size,
      sha256: await sha256(file),
    })
  }
  records.sort((a, b) => compareStrings(a.path, b.path))
  const folded = new Set()
  const fingerprint = createHash('sha256')
  for (const record of records) {
    const key = record.path.toLowerCase()
    if (folded.has(key)) throw new Error(`Payload has a case-colliding path: ${record.path}`)
    folded.add(key)
    fingerprint.update(record.path)
    fingerprint.update('\0')
    fingerprint.update(String(record.size_bytes))
    fingerprint.update('\0')
    fingerprint.update(record.sha256)
    fingerprint.update('\n')
  }
  return {
    file_count: records.length,
    total_bytes: totalBytes,
    content_sha256: fingerprint.digest('hex'),
    records,
  }
}

function compareContentSets(expected, actual) {
  const expectedMap = new Map(expected.records.map((record) => [record.path, record]))
  const actualMap = new Map(actual.records.map((record) => [record.path, record]))
  const missing = expected.records.filter((record) => !actualMap.has(record.path)).map((record) => record.path)
  const extra = actual.records.filter((record) => !expectedMap.has(record.path)).map((record) => record.path)
  const mismatched = expected.records
    .filter((record) => actualMap.has(record.path) && actualMap.get(record.path).sha256 !== record.sha256)
    .map((record) => record.path)
  if (missing.length || extra.length || mismatched.length) {
    throw new Error(`Installed data mismatch; missing=${missing.join(', ')}, extra=${extra.join(', ')}, changed=${mismatched.join(', ')}.`)
  }
  if (expected.content_sha256 !== actual.content_sha256) {
    throw new Error('Installed data aggregate fingerprint does not match source data.')
  }
}

function installerEntry(installer, metadata) {
  return {
    kind: installer.kind,
    path: relative(installer.path),
    size_bytes: installer.size_bytes,
    sha256: installer.sha256,
    signature: metadata.signature,
    metadata: Object.fromEntries(Object.entries(metadata).filter(([key]) => key !== 'signature')),
  }
}

async function walkFiles(directory) {
  const rootInfo = await lstat(directory).catch(() => null)
  if (!rootInfo?.isDirectory() || rootInfo.isSymbolicLink()) return []
  const files = []
  const pending = [{ directory, depth: 0 }]
  let entryCount = 0
  while (pending.length) {
    const current = pending.pop()
    if (current.depth > 32) throw new Error(`Directory tree is too deep: ${relative(current.directory)}`)
    for (const entry of await readdir(current.directory, { withFileTypes: true })) {
      entryCount += 1
      if (entryCount > maximumPayloadEntries) throw new Error('Release payload entry count exceeded its limit.')
      const file = path.join(current.directory, entry.name)
      const info = await lstat(file)
      if (info.isSymbolicLink()) throw new Error(`Symbolic links are not allowed in release payloads: ${relative(file)}`)
      if (info.isDirectory()) pending.push({ directory: file, depth: current.depth + 1 })
      else if (info.isFile()) files.push(file)
      else throw new Error(`Unsupported filesystem entry in release payload: ${relative(file)}`)
      if (files.length > maximumPayloadFiles) throw new Error('Release payload file count exceeded its limit.')
    }
  }
  return files.sort(compareStrings)
}

async function sha256(file) {
  const hash = createHash('sha256')
  for await (const chunk of createReadStream(file, { highWaterMark: 64 * 1024 })) {
    hash.update(chunk)
  }
  return hash.digest('hex')
}

function gitSourceState() {
  const git = (...gitArgs) => execFileSync('git', gitArgs, { cwd: root, encoding: 'utf8', windowsHide: true }).trim()
  const trackedStatus = git('status', '--porcelain', '--untracked-files=no')
  const fullStatus = git('status', '--porcelain', '--untracked-files=all')
  return {
    git_commit: git('rev-parse', 'HEAD'),
    git_short_commit: git('rev-parse', '--short=12', 'HEAD'),
    tracked_worktree_dirty: trackedStatus.length > 0,
    worktree_dirty: fullStatus.length > 0,
  }
}

function authenticodeSignature(file) {
  const source = String.raw`
$ErrorActionPreference = 'Stop'
$signature = Get-AuthenticodeSignature -LiteralPath $env:MONOGATARI_AUDIT_FILE
[ordered]@{
  status = [string]$signature.Status
  status_message = [string]$signature.StatusMessage
  subject = if ($signature.SignerCertificate) { [string]$signature.SignerCertificate.Subject } else { $null }
  thumbprint = if ($signature.SignerCertificate) { [string]$signature.SignerCertificate.Thumbprint } else { $null }
} | ConvertTo-Json -Depth 3 -Compress
`
  const output = execFileSync('powershell.exe', [
    '-NoProfile',
    '-NonInteractive',
    '-ExecutionPolicy',
    'Bypass',
    '-Command',
    source,
  ], {
    cwd: root,
    encoding: 'utf8',
    windowsHide: true,
    env: { ...process.env, MONOGATARI_AUDIT_FILE: file },
  }).trim()
  return JSON.parse(output)
}

function validateSignatureSet(entries, unsignedAllowed, label) {
  const statuses = entries.map((entry) => entry.signature.status)
  const unacceptable = entries.filter(({ signature }) => (
    signature.status !== 'Valid' && !(unsignedAllowed && signature.status === 'NotSigned')
  ))
  if (unacceptable.length > 0) {
    const verb = label === 'Installers' ? 'require' : 'requires'
    throw new Error(`${label} ${verb} valid Authenticode signatures; observed ${statuses.join(', ')}.`)
  }
  for (const { name, signature } of entries) {
    if (signature.status === 'Valid'
        && !String(signature.subject ?? '').toLowerCase().includes(expectedSignerFragment.toLowerCase())) {
      throw new Error(`${name} signer subject must identify ${expectedSignerFragment}; observed ${JSON.stringify(signature.subject)}.`)
    }
  }
}

function comparableWindowsPath(value) {
  if (typeof value !== 'string' || value.length === 0) return ''
  return path.resolve(value.replace(/^\\\\\?\\/, '')).toLowerCase()
}

function compareStrings(left, right) {
  return left < right ? -1 : left > right ? 1 : 0
}

async function tailFile(file, lines) {
  try {
    return (await readFile(file, 'utf8')).split(/\r?\n/).slice(-lines).join('\n')
  } catch {
    return ''
  }
}

function requireValue(actual, expected, label) {
  if (actual !== expected) throw new Error(`${label} is ${JSON.stringify(actual)}, expected ${JSON.stringify(expected)}.`)
}

function relative(file) {
  const value = path.relative(root, file).replaceAll('\\', '/')
  return value || '.'
}

function formatBytes(bytes) {
  return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`
}

function readArg(name) {
  const prefix = `--${name}=`
  const value = args.find((arg) => arg.startsWith(prefix))
  return value ? value.slice(prefix.length) : null
}
