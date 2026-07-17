import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  collectProjectWorkflowEvidence,
  createProjectWorkflowPolicy,
  verifyProjectWorkflowShape,
} from '../lib/project-content/workflow-policy.mjs'

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..')
const rustDirectory = path.join(repositoryRoot, 'rust-engine')
const boundaries = { repositoryRoot, rustDirectory }

test('checked-in Workflow catalogs return cross-root passing evidence', async () => {
  const evidence = await collectProjectWorkflowEvidence(boundaries)
  assert.deepEqual(evidence.issues, [])
  assert.equal(evidence.workflowCount, 10)
})

test('pure Workflow shape validation reports graph, state, condition, and Event drift', () => {
  const storyEvents = new Map([
    ['known_event', {
      event_type: 'known_type',
      character_ids: ['sakura'],
    }],
  ])
  const issues = verifyProjectWorkflowShape({
    id: 'drifted_workflow',
    name: 'Drifted Workflow',
    start_node_id: 'start',
    nodes: [
      { id: 'start', node_type: 'start', config: {}, connections: ['start', 'missing'] },
      {
        id: 'evaluation',
        node_type: 'evaluation',
        config: { criteria: 'score', variable_name: '../score' },
        connections: [],
      },
      {
        id: 'condition',
        node_type: 'condition',
        config: { condition: 'allowed\u0000' },
        connections: [],
      },
      {
        id: 'wrong_type',
        node_type: 'trigger_event',
        config: { event_id: 'known_event', event_type: 'wrong_type' },
        connections: [],
      },
      {
        id: 'wrong_character',
        node_type: 'trigger_event',
        config: { event_id: 'known_event', character_id: 'luna' },
        connections: [],
      },
      { id: 'unknown', node_type: 'mystery_node', config: {}, connections: [] },
    ],
  }, 'fixture.json', storyEvents)

  for (const issue of [
    'fixture.json:start: node cannot connect to itself',
    'fixture.json:start: connection target missing does not exist',
    'fixture.json:evaluation: state key field variable_name is invalid: can contain only ASCII letters, numbers, dots, underscores, or hyphens',
    'fixture.json:condition: condition field is invalid: cannot contain control characters',
    'fixture.json:wrong_type: story event known_event does not use type wrong_type',
    'fixture.json:wrong_character: story event known_event is not available for character luna',
    'fixture.json:unknown: unknown node_type mystery_node',
  ]) {
    assert(issues.includes(issue), issue)
  }
})

test('Workflow file drift uses the active Story Event catalog and explicit roots', async () => {
  const workflowPath = path.join(repositoryRoot, 'data', 'workflows', 'score_gate_demo.json')
  const evidence = await collectProjectWorkflowEvidence({
    ...boundaries,
    async readTextFile(filePath, encoding) {
      const source = await readFile(filePath, encoding)
      if (path.resolve(filePath) !== workflowPath) return source

      const workflow = JSON.parse(source)
      workflow.start_node_id = 'missing_start'
      workflow.nodes.find((node) => node.id === 'start').connections.push('missing_node')
      workflow.nodes.find((node) => node.id === 'engagement_gate').config.variable_name = '../score'
      workflow.nodes.find((node) => node.id === 'trigger_high_engagement').config.event_id = 'missing_event'
      return JSON.stringify(workflow)
    },
  })

  for (const issue of [
    'data/workflows/score_gate_demo.json: start_node_id does not match any node',
    'data/workflows/score_gate_demo.json:start: connection target missing_node does not exist',
    'data/workflows/score_gate_demo.json:engagement_gate: state key field variable_name is invalid: can contain only ASCII letters, numbers, dots, underscores, or hyphens',
    'data/workflows/score_gate_demo.json:trigger_high_engagement: story event missing_event is not in the project catalog',
  ]) {
    assert(evidence.issues.includes(issue), issue)
  }
})

test('Project Workflow policy requires explicit roots and unique data-root labels', () => {
  assert.throws(() => createProjectWorkflowPolicy(), /requires repositoryRoot/)
  assert.throws(
    () => createProjectWorkflowPolicy({ repositoryRoot }),
    /requires rustDirectory/,
  )
  assert.throws(
    () => createProjectWorkflowPolicy({
      ...boundaries,
      dataRoots: [
        { label: 'project', dir: path.join(repositoryRoot, 'data') },
        { label: 'project', dir: path.join(rustDirectory, 'data') },
      ],
    }),
    /label is duplicated/,
  )
})

test('release runner delegates Workflow policy without retaining node rules', async () => {
  const runnerSource = await readFile(path.join(repositoryRoot, 'scripts', 'verify-release.mjs'), 'utf8')
  const policySource = await readFile(
    path.join(repositoryRoot, 'scripts', 'lib', 'project-content', 'workflow-policy.mjs'),
    'utf8',
  )

  assert(runnerSource.includes('createProjectWorkflowPolicy'))
  assert(runnerSource.includes('storyEventPolicy: projectStoryEventPolicy'))
  assert(!runnerSource.includes('async function verifyWorkflowFiles'))
  assert(!runnerSource.includes('function verifyWorkflowShape'))
  assert(!runnerSource.includes('workflowStateKeyMaxChars'))
  assert(policySource.includes('async function collectWorkflowEvidence'))
  assert(policySource.includes('export function verifyProjectWorkflowShape'))
  assert(policySource.includes('workflowStateKeyMaxChars'))
})
