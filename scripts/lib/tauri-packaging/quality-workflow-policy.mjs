import { readFile as readFileFromDisk } from 'node:fs/promises'
import path from 'node:path'

const qualityInputRequirements = [
  ['qualitySuite', 'pub struct QualitySuiteDocument', 'own the Quality Suite document model'],
  ['qualitySuite', 'pub struct QualityScenarioDocument', 'own the Quality scenario model'],
  ['qualitySuite', 'pub struct QualityMessage', 'own the Quality message model'],
  ['qualitySuite', 'pub struct QualityExpectation', 'own the Quality expectation model'],
  ['qualitySuite', 'pub workflow_run_contexts: Vec<WorkflowRunContext>', 'parse typed Workflow run contexts'],
  ['workflow', 'pub struct WorkflowRunContext', 'own the Workflow run-context model'],
  ['tauriQuality', 'QualitySuiteDocument as QualitySuite', 'reuse the shared Quality Suite model'],
  ['tauriQuality', 'QualityScenarioDocument as QualityScenario', 'reuse the shared Quality scenario model'],
  ['tauriQuality', 'QualityMessage', 'reuse the shared Quality message model'],
  ['tauriQuality', 'QualityExpectation', 'reuse the shared Quality expectation model'],
  ['tauriWorkflow', 'WorkflowRunContext', 'reuse the shared Workflow run-context model'],
]

const workflowPreviewRequirements = [
  ['workflowPreview', 'pub fn execute_workflow_preview', 'expose deterministic Workflow preview execution'],
  ['workflowPreview', 'pub struct WorkflowPreviewEnvironment', 'accept transport-neutral preview state'],
  ['workflowPreview', 'pub struct WorkflowPreviewOptions', 'accept bounded execution and deterministic branch options'],
  ['workflowPreview', 'struct DeterministicRandom', 'make random branches reproducible'],
  ['workflowPreview', '"simulated": true', 'simulate LLM nodes without requiring a provider'],
  ['workflowPreviewTests', 'executes_context_state_and_conditions_without_tauri', 'test stateful preview execution without Tauri'],
  ['workflowPreviewTests', 'random_branches_are_deterministic_and_injectable', 'test deterministic random branches'],
  ['workflowPreviewTests', 'event_decisions_use_typed_context_and_trigger_history', 'test Event decisions from typed preview context'],
  ['tauriWorkflow', 'execute_workflow_preview(', 'delegate run-context previews to the headless executor'],
  ['tauriWorkflow', 'workflow_preview_environment', 'adapt desktop state into the headless preview environment'],
  ['qualityExecution', 'execute_workflow_preview(', 'run Quality Workflow coverage without desktop state'],
]

const qualityExecutionRequirements = [
  ['qualityExecution', 'pub fn execute_quality_suite', 'own complete Quality Suite execution'],
  ['qualityExecution', 'fn run_quality_scenario', 'own scenario evidence aggregation'],
  ['qualityExecution', 'fn scenario_knowledge_evidence', 'own project knowledge evidence'],
  ['qualityExecution', 'pub struct QualitySuiteReport', 'own the structured Quality report contract'],
  ['qualityExecutionTests', 'checked_in_character_stability_suite_passes_without_tauri', 'test the built-in suite without Tauri'],
  ['qualityExecutionTests', 'tideglass_quality_workflows_reach_full_coverage_without_tauri', 'test Tideglass Workflow coverage without Tauri'],
  ['qualityExecutionTests', 'failed_expectations_return_actionable_headless_evidence', 'test structured failure evidence without Tauri'],
  ['tauriQuality', 'execute_quality_suite(', 'delegate execution to the headless Quality domain'],
  ['tauriQuality', 'quality_suite_run_provenance', 'adapt build provenance for headless reports'],
]

const authoringRuntimeTraceRequirements = [
  ['runtime_safety_trace: Option<chat::ChatSafetyTrace>', 'export runtime safety traces in quality scenario reports'],
  ['runtime_safety_trace_required', 'let quality suites require runtime safety trace evidence'],
  ['required_runtime_guard_notes', 'let quality suites require specific guard notes'],
  ['runtime_guard_interventions', 'count runtime guard interventions in audit summaries'],
  ['scenario_runtime_safety_trace', 'centralize quality runtime trace construction'],
  ['chat::build_chat_safety_trace', 'reuse the chat safety trace contract in quality reports'],
  ['chat::build_event_trigger_decisions', 'reuse the chat story event decision contract in quality reports'],
  ['rule_fingerprint', 'carry story event rule fingerprints into quality reports'],
  ['expected.rule_fingerprint', 'let quality suites pin event rule fingerprints when needed'],
  ['pinned_knowledge_ref_ids', 'carry pinned knowledge ref ids into quality runtime traces'],
  ['guard_workflow_story_output', 'reuse runtime workflow LLM output finalization in quality reports'],
  ['workflow_output_equals', 'let workflow quality scenarios assert finalized workflow output text'],
  ['workflow_output: Option<String>', 'export finalized workflow output text in quality reports'],
  ['workflow_output_report', 'omit empty workflow output evidence from non-workflow scenarios'],
  ['QualitySuiteRunMetadata', 'export quality suite run metadata'],
  ['quality_suite_run_metadata', 'centralize quality suite run metadata generation'],
  ['suite_sha256', 'export quality suite content fingerprints in run metadata'],
]

const tauriRuntimeTraceRequirements = [
  ['pub struct QualitySuiteSummary', 'export quality suite summaries for the workbench'],
  ['LoadedQualitySuite', 'return backend-confirmed quality suite source paths with loaded suites'],
  ['source_sha256', 'return backend-confirmed quality suite content fingerprints with loaded suites'],
  ['quality_suite_source_path', 'normalize quality suite source paths for QA reports'],
  ['quality_suite_sha256', 'hash quality suite source content for QA reports'],
  ['quality_suite_loader_reports_relative_source_path', 'test quality suite report source paths stay project-relative'],
  ['quality_suite_summary_reports_source_fingerprint', 'test quality suite summaries expose source fingerprints'],
  ['pinned_knowledge_ref_count', 'carry pinned knowledge evidence into quality runtime traces'],
  ['CARGO_PKG_VERSION', 'bind quality suite run metadata to the engine package version'],
  ['MONOGATARI_GIT_COMMIT', 'bind quality suite run metadata to the build git commit'],
  ['MONOGATARI_GIT_SHORT_COMMIT', 'export a compact git commit for quality report UI evidence'],
  ['reports_workflow_output_finalization_mismatches', 'test finalized workflow output expectations fail loudly'],
]

export async function collectTauriQualityWorkflowEvidence(options = {}) {
  const { rustDirectory, tauriAppDirectory } = resolveBoundaries(options)
  const readFile = options.readTextFile ?? readFileFromDisk
  const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
  const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
  const sources = {
    qualityExecution: await readFile(path.join(authoringDirectory, 'quality_suite_execution.rs'), 'utf8'),
    qualityExecutionTests: await readFile(path.join(authoringDirectory, 'quality_suite_execution', 'tests.rs'), 'utf8'),
    qualitySuite: await readFile(path.join(authoringDirectory, 'quality_suite_validation.rs'), 'utf8'),
    workflow: await readFile(path.join(authoringDirectory, 'workflow_validation.rs'), 'utf8'),
    workflowPreview: await readFile(path.join(authoringDirectory, 'workflow_preview.rs'), 'utf8'),
    workflowPreviewTests: await readFile(path.join(authoringDirectory, 'workflow_preview', 'tests.rs'), 'utf8'),
    tauriQuality: await readFile(path.join(commandDirectory, 'quality_suite.rs'), 'utf8'),
    tauriWorkflow: await readFile(path.join(commandDirectory, 'workflow.rs'), 'utf8'),
  }
  const issues = []

  appendSourceRequirements(sources, qualityInputRequirements, 'Headless Quality input contracts', issues)
  if (/pub\s+struct\s+Quality(?:Suite|Scenario|Message|Expectation)\s*\{/.test(sources.tauriQuality)) {
    issues.push('Tauri Quality commands must not duplicate shared headless input models')
  }
  if (/pub\s+struct\s+WorkflowRunContext\s*\{/.test(sources.tauriWorkflow)) {
    issues.push('Tauri Workflow commands must not duplicate the shared run-context model')
  }
  const tauriQualityParserSource = sources.tauriQuality.match(
    /pub\(crate\) fn parse_quality_suite[\s\S]*?\n\}/,
  )?.[0] ?? ''
  if (
    !tauriQualityParserSource.includes('parse_quality_suite_document(content)')
    || tauriQualityParserSource.includes('serde_json::from_str')
  ) {
    issues.push('Tauri Quality parsing must delegate directly to the shared headless parser')
  }

  appendSourceRequirements(sources, workflowPreviewRequirements, 'Headless Workflow preview', issues)
  if (/struct\s+WorkflowPreviewState\s*\{/.test(sources.tauriWorkflow)) {
    issues.push('Tauri Workflow commands must not redeclare the headless preview state machine')
  }
  const qualityWorkflowCoverageSource = sources.qualityExecution.match(
    /fn scenario_workflow_coverage[\s\S]*?\n\}/,
  )?.[0] ?? ''
  if (
    !qualityWorkflowCoverageSource.includes('execute_workflow_preview(')
    || qualityWorkflowCoverageSource.includes('AppState::new()')
  ) {
    issues.push('Quality Workflow coverage must execute through the headless preview domain')
  }

  appendSourceRequirements(sources, qualityExecutionRequirements, 'Headless Quality execution', issues)
  if (/fn\s+(?:run_quality_scenario|scenario_knowledge_evidence|validate_scenario_expectations)\s*\(/.test(sources.tauriQuality)) {
    issues.push('Tauri Quality commands must not redeclare headless scenario execution or evidence logic')
  }

  appendRequirements(
    sources.qualityExecution,
    authoringRuntimeTraceRequirements,
    'Quality suite runtime safety tracing',
    issues,
  )
  appendRequirements(
    sources.tauriQuality,
    tauriRuntimeTraceRequirements,
    'Quality suite runtime safety tracing',
    issues,
  )

  return {
    issues,
    requirementCounts: {
      qualityInput: qualityInputRequirements.length,
      workflowPreview: workflowPreviewRequirements.length,
      qualityExecution: qualityExecutionRequirements.length,
      runtimeTrace:
        authoringRuntimeTraceRequirements.length + tauriRuntimeTraceRequirements.length,
    },
    structuralCheckCount: 6,
  }
}

function appendSourceRequirements(sources, requirements, label, issues) {
  for (const [sourceName, needle, description] of requirements) {
    if (!sources[sourceName].includes(needle)) issues.push(`${label} must ${description}`)
  }
}

function appendRequirements(source, requirements, label, issues) {
  for (const [needle, description] of requirements) {
    if (!source.includes(needle)) issues.push(`${label} must ${description}`)
  }
}

function resolveBoundaries(options) {
  const boundaries = {
    rustDirectory: options.rustDirectory,
    tauriAppDirectory: options.tauriAppDirectory,
  }
  for (const [name, value] of Object.entries(boundaries)) {
    if (typeof value !== 'string' || value.length === 0) {
      throw new Error(`Tauri Quality/Workflow policy requires ${name}.`)
    }
  }
  return boundaries
}
