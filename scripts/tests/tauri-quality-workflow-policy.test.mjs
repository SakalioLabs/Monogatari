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
    workflowPreview: 11,
    qualityExecution: 9,
    runtimeTrace: 29,
  })
  assert.equal(evidence.structuralCheckCount, 6)
})

test('input, preview, execution, trace, and adapter drift stays independently actionable', async () => {
  const qualitySuitePath = path.join(authoringDirectory, 'quality_suite_validation.rs')
  const workflowPath = path.join(authoringDirectory, 'workflow_validation.rs')
  const workflowPreviewPath = path.join(authoringDirectory, 'workflow_preview.rs')
  const qualityExecutionPath = path.join(authoringDirectory, 'quality_suite_execution.rs')
  const tauriQualityPath = path.join(commandDirectory, 'quality_suite.rs')
  const tauriWorkflowPath = path.join(commandDirectory, 'workflow.rs')
  const evidence = await collectTauriQualityWorkflowEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      const resolved = path.resolve(filePath)
      if (resolved === qualitySuitePath) {
        return source.replaceAll('pub struct QualitySuiteDocument', 'pub struct DriftedQualitySuiteDocument')
      }
      if (resolved === workflowPath) {
        return source.replaceAll('pub struct WorkflowRunContext', 'pub struct DriftedWorkflowRunContext')
      }
      if (resolved === workflowPreviewPath) {
        return source.replaceAll('struct DeterministicRandom', 'struct NondeterministicRandom')
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
            .replaceAll('pub struct QualitySuiteSummary', 'pub struct DriftedQualitySuiteSummary'),
          'pub struct QualityScenario {}',
          'fn run_quality_scenario() {}',
          '',
        ].join('\\n')
      }
      if (resolved === tauriWorkflowPath) {
        return [source, 'pub struct WorkflowRunContext {}', 'struct WorkflowPreviewState {}', ''].join('\\n')
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
    'Headless Workflow preview must make random branches reproducible',
    'Tauri Workflow commands must not redeclare the headless preview state machine',
    'Quality Workflow coverage must execute through the headless preview domain',
    'Headless Quality execution must own complete Quality Suite execution',
    'Tauri Quality commands must not redeclare headless scenario execution or evidence logic',
    'Quality suite runtime safety tracing must reuse the chat safety trace contract in quality reports',
    'Quality suite runtime safety tracing must export quality suite summaries for the workbench',
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
