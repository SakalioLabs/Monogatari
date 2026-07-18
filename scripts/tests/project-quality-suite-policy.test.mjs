import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectProjectQualitySuiteEvidence,
  createProjectQualitySuitePolicy,
  verifyProjectQualitySuiteShape,
} from '../lib/project-content/quality-suite-policy.mjs'
import { requiredStoryEventRuleIds } from '../lib/project-content/story-event-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const boundaries = { repositoryRoot, rustDirectory }

test('checked-in Quality Suites return source-bound passing evidence', async () => {
  const messages = []
  const policy = createProjectQualitySuitePolicy({
    ...boundaries,
    log(message) {
      messages.push(message)
    },
  })
  const evidence = await policy.verifyQualitySuites()

  assert.deepEqual(evidence.issues, [])
  assert.deepEqual(
    {
      suiteCount: evidence.suiteCount,
      scenarioCount: evidence.scenarioCount,
      defaultScenarioCount: evidence.defaultScenarioCount,
      sourceDataRootLabel: evidence.sourceDataRootLabel,
    },
    {
      suiteCount: 4,
      scenarioCount: 44,
      defaultScenarioCount: 29,
      sourceDataRootLabel: 'data',
    },
  )
  assert.deepEqual(messages, ['[release] Quality suites OK (4 suite file(s))'])
})

test('pure Quality Suite shape validation reports schema, bounds, conflicts, and rule drift', () => {
  const issues = verifyProjectQualitySuiteShape({
    version: '',
    name: '',
    description: '',
    scenarios: [
      null,
      {
        id: 'duplicate',
        category: '',
        description: '',
        messages: {},
        workflow_path: '',
        workflow_max_steps: 0,
        workflow_run_contexts: {},
        workflow_choice_selections: [{ '../invalid': 129 }],
        expect: {
          min_friendliness: 1.1,
          max_friendliness: -0.1,
          min_relationship_delta: 0.6,
          max_relationship_delta: -0.6,
          expected_events: ['same'],
          forbidden_events: ['SAME'],
          expected_event_rules: [
            null,
            {
              event_id: '',
              event_type: '',
              rule_fingerprint: 'invalid',
              min_relationship: 'high',
              min_score: 'high',
              min_evaluation_count: 1.5,
            },
          ],
        },
      },
      {
        id: 'duplicate',
        category: 'scoring',
        description: 'duplicate',
        expect: [],
      },
    ],
  }, 'fixture.json')

  for (const issue of [
    'fixture.json: version is required',
    'fixture.json: name is required',
    'fixture.json: description is required',
    'fixture.json: scenario ids must be unique',
    'fixture.json:<missing-id>: scenario must be a JSON object',
    'fixture.json:duplicate: category is required',
    'fixture.json:duplicate: messages must be an array',
    'fixture.json:duplicate: workflow_path must be a non-empty string when provided',
    'fixture.json:duplicate: workflow_max_steps must be a positive integer when provided',
    'fixture.json:duplicate: workflow_run_contexts must be an array when provided',
    'fixture.json:duplicate: workflow choice run 0 contains invalid node id ../invalid',
    'fixture.json:duplicate: workflow choice run 0 selection for ../invalid must be an integer between 0 and 128',
    'fixture.json:duplicate: min_friendliness must be a finite number between 0 and 1',
    'fixture.json:duplicate: max_friendliness must be a finite number between 0 and 1',
    'fixture.json:duplicate: min_friendliness must be less than or equal to max_friendliness',
    'fixture.json:duplicate: min_relationship_delta must be a finite number between -0.5 and 0.5',
    'fixture.json:duplicate: max_relationship_delta must be a finite number between -0.5 and 0.5',
    'fixture.json:duplicate: min_relationship_delta must be less than or equal to max_relationship_delta',
    'fixture.json:duplicate: event "same" cannot appear in both expected_events and forbidden_events',
    'fixture.json:duplicate: event rule must be a JSON object',
    'fixture.json:duplicate: event rule id is required',
    'fixture.json:duplicate: event rule type is required',
    'fixture.json:duplicate: rule_fingerprint must be a 64-character SHA-256 hex string when provided',
    'fixture.json:duplicate: min_relationship must be numeric',
    'fixture.json:duplicate: min_score must be numeric',
    'fixture.json:duplicate: min_evaluation_count must be an integer',
    'fixture.json:duplicate: expect object is required',
  ]) {
    assert(issues.includes(issue), issue)
  }
})

test('default Quality baseline drift stays actionable across policy groups', async () => {
  const defaultSuitePath = path.join(
    repositoryRoot,
    'data',
    'quality_suites',
    'character_stability.json',
  )
  const evidence = await collectProjectQualitySuiteEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      if (path.resolve(filePath) !== defaultSuitePath) return source

      const suite = JSON.parse(source)
      suite.scenarios = suite.scenarios.filter(
        (scenario) => scenario.id !== 'warm-creative-conversation',
      )
      const scenario = (id) => suite.scenarios.find((candidate) => candidate.id === id)
      scenario('knowledge-anchor-safe-response').expect.required_knowledge_refs = []
      scenario('workflow-output-sanitized').expect.workflow_output_leak_detected = true
      scenario('multilingual-warm-creative-conversation').messages = []
      scenario('score-gate-workflow-coverage').workflow_path = 'workflows/wrong.json'
      scenario('memory-prompt-replay-sanitized').expect.memory_prompt_leak_detected = true
      scenario('overrange-score-clamped').mock_evaluation_response.friendliness = '1.0'
      scenario('style-drift-sanitized-response').expect.style_drift_detected = true
      scenario('fallback-injection-score-contained').expect.max_overall = 0.5
      scenario('tool-role-injection-contained').expect.prompt_injection_detected = false
      scenario('group-chat-runtime-trace-contained').category = 'chat'
      scenario('block-body-prompt-injection-contained').category = 'scoring'
      scenario('relationship-injection-delta-contained').relationship = 0.3
      scenario('multilingual-prompt-injection-contained').category = 'scoring'
      scenario('unicode-obfuscated-injection-contained').category = 'scoring'
      return JSON.stringify(suite)
    },
  })

  for (const issue of [
    'Missing required quality scenario: warm-creative-conversation',
    'Knowledge anchor scenario must require sakura_nature',
    'Knowledge anchor scenario must require sakura_art_knowledge',
    'Workflow output scenario must expect workflow_output_leak_detected=false',
    'Multilingual warm conversation scenario must include localized scoring marker 谢谢',
    'Workflow coverage scenario must target workflows/score_gate_demo.json',
    'Memory prompt replay scenario must expect memory_prompt_leak_detected=false',
    'Overrange score scenario must include an above-100% friendliness score',
    'Style drift scenario must expect style_drift_detected=false',
    'Fallback injection score scenario must cap overall fallback score at 0.4',
    'Tool-role injection scenario must expect prompt_injection_detected=true',
    'Group chat runtime trace scenario must use category group_chat',
    'Block-body prompt injection scenario must use category injection',
    'Relationship injection scenario must start just below first_friend at relationship 0.29',
    'Multilingual injection scenario must use category injection',
    'Unicode obfuscated injection scenario must use category injection',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('Quality event snapshots use the injected Story Event policy', async () => {
  const requestedRoots = []
  const evidence = await collectProjectQualitySuiteEvidence({
    ...boundaries,
    storyEventPolicy: {
      async loadStoryEventCatalog(dataRoot) {
        requestedRoots.push(dataRoot.label)
        return { events: new Map() }
      },
    },
  })

  assert.deepEqual(requestedRoots, ['data'])
  for (const id of requiredStoryEventRuleIds) {
    assert(
      evidence.issues.includes(
        'Event rule snapshot fingerprint does not match data/events for ' + id,
      ),
      id,
    )
  }
})

test('Project Quality Suite policy requires explicit roots, labels, and Story Event interface', () => {
  assert.throws(() => createProjectQualitySuitePolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createProjectQualitySuitePolicy({ repositoryRoot }),
    /requires rustDirectory/,
  )
  assert.throws(
    () => createProjectQualitySuitePolicy({
      ...boundaries,
      dataRoots: [
        { label: 'project', dir: path.join(repositoryRoot, 'data') },
        { label: 'project', dir: path.join(rustDirectory, 'data') },
      ],
    }),
    /label is duplicated/,
  )
  assert.throws(
    () => createProjectQualitySuitePolicy({
      ...boundaries,
      storyEventPolicy: {},
    }),
    /requires storyEventPolicy\.loadStoryEventCatalog/,
  )
})

test('release runner delegates Quality policy without retaining suite rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(
      repositoryRoot,
      'scripts',
      'lib',
      'project-content',
      'quality-suite-policy.mjs',
    ),
    'utf8',
  )

  assert(runnerSource.includes('createProjectQualitySuitePolicy'))
  assert(runnerSource.includes('storyEventPolicy: projectStoryEventPolicy'))
  assert(runnerSource.includes('verifyQualitySuites'))
  assert(!runnerSource.includes('async function verifyQualitySuites'))
  assert(!runnerSource.includes('function verifyQualitySuiteShape'))
  assert(!runnerSource.includes('requiredQualityScenarios'))
  assert(!runnerSource.includes('Missing required quality scenario'))
  assert(!runnerSource.includes('Multilingual injection scenario'))
  assert(policySource.includes('async function collectQualitySuiteEvidence'))
  assert(policySource.includes('export function verifyProjectQualitySuiteShape'))
  assert(policySource.includes('export function verifyDefaultProjectQualitySuite'))
  assert(policySource.includes('character_stability.json'))
})
