import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import { collectTauriQualityWorkflowEvidence } from '../lib/tauri-packaging/quality-workflow-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const tauriAppDirectory = path.join(rustDirectory, 'crates', 'tauri-app')
const authoringDirectory = path.join(rustDirectory, 'crates', 'authoring', 'src')
const commandDirectory = path.join(tauriAppDirectory, 'src', 'commands')
const boundaries = { rustDirectory, tauriAppDirectory }

test('checked-in Quality and Workflow headless contracts return passing evidence', async () => {
  const evidence = await collectTauriQualityWorkflowEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(evidence.requirementCounts, {
    qualityInput: 11,
    qualitySource: 18,
    workflowPreview: 11,
    workflowProjectPreview: 14,
    workflowExecutionPolicy: 27,
    qualityExecution: 9,
    runtimeTrace: 22,
  })
  assert.equal(evidence.structuralCheckCount, 15)
})

test('input, preview, execution, trace, and adapter drift stays independently actionable', async () => {
  const qualitySuitePath = path.join(authoringDirectory, 'quality_suite_validation.rs')
  const qualitySuiteTestsPath = path.join(authoringDirectory, 'quality_suite_validation', 'tests.rs')
  const workflowPath = path.join(authoringDirectory, 'workflow_validation.rs')
  const workflowDocumentsPath = path.join(authoringDirectory, 'workflow_documents.rs')
  const workflowDocumentsTestsPath = path.join(authoringDirectory, 'workflow_documents', 'tests.rs')
  const workflowExecutionPolicyPath = path.join(authoringDirectory, 'workflow_execution_policy.rs')
  const workflowExecutionPolicyTestsPath = path.join(authoringDirectory, 'workflow_execution_policy', 'tests.rs')
  const workflowPreviewPath = path.join(authoringDirectory, 'workflow_preview.rs')
  const qualityExecutionPath = path.join(authoringDirectory, 'quality_suite_execution.rs')
  const tauriQualityPath = path.join(commandDirectory, 'quality_suite.rs')
  const tauriWorkflowPath = path.join(commandDirectory, 'workflow.rs')
  const mcpServerPath = path.join(rustDirectory, 'crates', 'mcp-server', 'src', 'server.rs')
  const mcpProtocolPath = path.join(rustDirectory, 'crates', 'mcp-server', 'src', 'protocol.rs')
  const mcpE2ePath = path.join(rustDirectory, 'crates', 'mcp-server', 'tests', 'stdio_e2e.rs')
  const evidence = await collectTauriQualityWorkflowEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === qualitySuitePath) {
        return source
          .replaceAll('pub struct QualitySuiteDocument', 'pub struct DriftedQualitySuiteDocument')
          .replaceAll('pub fn load_project_quality_suite_document', 'pub fn load_drifted_quality_suite_document')
          .replaceAll('read_project_json(project_root, requested_path)?', 'read_drifted_project_json(project_root, requested_path)?')
      }
      if (resolved === qualitySuiteTestsPath) {
        return source.replaceAll(
          'rejects_non_quality_oversized_and_case_aliased_sources',
          'drifts_non_quality_oversized_and_case_aliased_sources',
        )
      }
      if (resolved === workflowPath) {
        return source.replaceAll('pub struct WorkflowRunContext', 'pub struct DriftedWorkflowRunContext')
      }
      if (resolved === workflowDocumentsPath) {
        return source
          .replaceAll('pub struct LoadedWorkflowDocument', 'pub struct DriftedLoadedWorkflowDocument')
          .replaceAll('pub async fn load_project_workflow_document', 'pub async fn load_drifted_workflow_document')
      }
      if (resolved === workflowDocumentsTestsPath) {
        return source.replaceAll('loaded_source.source_sha256', 'loaded_source.drifted_sha256')
      }
      if (resolved === workflowExecutionPolicyPath) {
        return source.replaceAll('pub fn workflow_next_node', 'pub fn drifted_workflow_next_node')
      }
      if (resolved === workflowExecutionPolicyTestsPath) {
        return source.replaceAll(
          'resolves_node_transitions_for_every_branching_contract',
          'drifts_node_transitions_for_every_branching_contract',
        )
      }
      if (resolved === workflowPreviewPath) {
        return [
          source
            .replaceAll('struct DeterministicRandom', 'struct NondeterministicRandom')
            .replaceAll('pub async fn execute_project_workflow_preview', 'pub async fn execute_drifted_project_workflow_preview')
            .replaceAll('load_project_workflow_document(', 'load_desktop_workflow_document('),
          'pub struct WorkflowExecutionReport {}',
          '',
        ].join('\n')
      }
      if (resolved === qualityExecutionPath) {
        return source
          .replaceAll('pub fn execute_quality_suite', 'pub fn execute_drifted_quality_suite')
          .replaceAll('execute_workflow_preview(', 'execute_desktop_workflow_preview(')
          .replaceAll('chat::build_chat_safety_trace', 'chat::build_drifted_safety_trace')
      }
      if (resolved === tauriQualityPath) {
        return [
          source
            .replace(
              'parse_quality_suite_document(content)',
              'serde_json::from_str(content).map_err(|error| error.to_string())',
            )
            .replaceAll('QualitySuiteSummary', 'DriftedQualitySuiteSummary')
            .replaceAll('list_project_quality_suite_summaries', 'list_drifted_quality_suite_summaries')
            .replaceAll('load_project_quality_suite_document', 'load_drifted_quality_suite_document')
            .replace(
              'pub(crate) fn parse_quality_suite',
              'fn drifted_quality_file_read() { let _ = std::fs::read_to_string("suite.json"); }\n\npub(crate) fn parse_quality_suite',
            ),
          'pub struct QualitySuiteSummary {}',
          'pub struct QualityScenario {}',
          'fn quality_suite_sha256() {}',
          'fn run_quality_scenario() {}',
          '',
        ].join('\\n')
      }
      if (resolved === tauriWorkflowPath) {
        return [
          source,
          'pub struct WorkflowRunContext {}',
          'struct WorkflowPreviewState {}',
          'fn workflow_next_node() {}',
          '',
        ].join('\\n')
      }
      if (resolved === mcpServerPath) {
        return source
          .replaceAll('load_project_quality_suite_document', 'read_project_json')
          .replaceAll('execute_project_workflow_preview(', 'execute_desktop_workflow_preview(')
          .replace(
            'let loaded = read_project_json(&self.project_root, &request.path)',
            'let source = serde_json::to_string(&request.path).unwrap();\n        let loaded = read_project_json(&self.project_root, &request.path)',
          )
      }
      if (resolved === mcpProtocolPath) {
        return source.replaceAll('MCP_WORKFLOW_PREVIEW_SCHEMA_V1', 'MCP_DRIFTED_WORKFLOW_PREVIEW_SCHEMA_V1')
      }
      if (resolved === mcpE2ePath) {
        return source.replaceAll('CallToolRequestParams::new("preview_workflow")', 'CallToolRequestParams::new("drifted_workflow")')
      }
      return source
    },
  })

  for (const issue of [
    'Headless Quality input contracts must own the Quality Suite document model',
    'Headless Quality input contracts must own the Workflow run-context model',
    'Tauri Quality commands must not duplicate shared headless input models',
    'Tauri Workflow commands must not duplicate the shared run-context model',
    'Tauri Quality parsing must delegate directly to the shared headless parser',
    'Headless Quality sources must own exact bounded Quality source loading',
    'Headless Quality sources must test catalog, size, and exact-case boundaries',
    'Headless Quality sources must reuse the shared Quality summary model',
    'Headless Quality sources must delegate Quality listing to the shared source domain',
    'Headless Quality sources must delegate MCP Quality loading to the shared source domain',
    'Shared Quality source loading must reuse the exact bounded JSON catalog reader',
    'Tauri Quality commands must not redeclare the shared summary model',
    'Tauri Quality commands must not redeclare shared source path, hash, or summary helpers',
    'Tauri Quality commands must not read project suite files outside the shared source domain',
    'MCP Quality execution must consume the shared loaded source without reserialization',
    'Headless Workflow preview must make random branches reproducible',
    'Tauri Workflow commands must not redeclare the headless preview state machine',
    'Quality Workflow coverage must execute through the headless preview domain',
    'Project Workflow preview must own loaded Workflow source evidence',
    'Project Workflow preview must load validated Workflows with source evidence',
    'Project Workflow preview must test Workflow source fingerprints without a transport',
    'Project Workflow preview must own project-backed provider-free previews',
    'Project Workflow preview must version MCP Workflow preview evidence',
    'Project Workflow preview must delegate MCP Workflow previews to the headless project domain',
    'Project Workflow preview must execute Workflow preview through a real MCP child process',
    'Project Workflow preview must compose shared source loading and headless execution',
    'MCP Workflow preview must delegate without filesystem or parser ownership',
    'Shared Workflow execution policy must own next-node and stop-reason decisions',
    'Shared Workflow execution policy must test every branching transition contract',
    'Workflow adapters must not redeclare shared execution report or coverage models',
    'Workflow adapters must not redeclare shared execution policy functions',
    'Headless Quality execution must own complete Quality Suite execution',
    'Tauri Quality commands must not redeclare headless scenario execution or evidence logic',
    'Quality suite runtime safety tracing must reuse the chat safety trace contract in quality reports',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('Quality and Workflow policy requires both Rust filesystem boundaries', async () => {
  await assert.rejects(
    () => collectTauriQualityWorkflowEvidence(),
    /requires rustDirectory/,
  )
  await assert.rejects(
    () => collectTauriQualityWorkflowEvidence({ rustDirectory }),
    /requires tauriAppDirectory/,
  )
})
