import { readFile } from 'node:fs/promises'

import { expect, test } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('monogatari-locale', 'en')
    localStorage.setItem('monogatari-version-seen', '0.9.5')
  })
})

test('workspace navigation exposes the authoring surfaces', async ({ page }) => {
  await page.goto('/')

  await expect(page.getByRole('heading', { name: 'Choose a project' })).toBeVisible()
  await expect(page.getByText('Packaged Web Project')).toBeVisible()
  await page.getByRole('button', { name: 'Continue' }).click()
  await expect(page).toHaveURL(/\/workspace$/)
  await expect(page.getByRole('link', { name: 'Monogatari Engine' })).toBeVisible()
  await page.getByRole('link', { name: 'Story Flow' }).click()
  await expect(page).toHaveURL(/\/editor$/)
  await expect(page.getByRole('heading', { name: 'Workflow Editor' })).toBeVisible()

  await page.getByRole('link', { name: 'Live Roleplays' }).click()
  await expect(page).toHaveURL(/\/roleplay-editor$/)
  await expect(page.getByRole('heading', { name: 'Scene Roleplay' })).toBeVisible()

  await page.getByRole('link', { name: 'Dialogues' }).click()
  await expect(page).toHaveURL(/\/dialogue-editor$/)
  await expect(page.getByRole('heading', { name: 'Dialogue Graph' })).toBeVisible()
})

test('live roleplay authoring saves a structured browser draft and opens the LLM NPC playtest', async ({ page }) => {
  await page.goto('/roleplay-editor')

  await expect(page.getByRole('heading', { name: 'Scene Roleplay' })).toBeVisible()
  await expect(page.getByRole('tab')).toHaveCount(0)
  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByRole('button', { name: 'Story', exact: true }).click()
  await page.getByLabel('Title').fill('Agent-authored live scene')
  await page.getByRole('button', { name: 'Nodes', exact: true }).click()
  await expect(page.getByLabel('Observable situation')).toBeVisible()
  await page.getByLabel('Observable situation').fill('A player and one NPC face an observable scene problem.')
  await page.getByLabel('Player goal').fill('Use free-form conversation to establish one verifiable fact.')
  await page.getByLabel('Primary NPC goal').fill('Stay in character and test the player claim against pinned knowledge.')

  const save = page.getByRole('button', { name: 'Save', exact: true })
  await expect(save).toBeEnabled()
  await save.click()
  await expect(page.getByText('Live roleplay saved')).toBeVisible()

  const playtest = page.getByRole('button', { name: 'Playtest', exact: true })
  await expect(playtest).toBeEnabled()
  await playtest.click()
  await expect(page).toHaveURL(/previewRoleplay=new_roleplay/)
  await expect(page.getByTestId('scene-roleplay')).toBeVisible()
  await expect(page.locator('.dialogue-text')).toHaveCount(0)
  await expect(page.getByTestId('roleplay-runtime')).toBeVisible()
})

test('live roleplay playtest generates an NPC turn then evaluates deterministic story state', async ({ page }) => {
  test.setTimeout(90_000)
  let generationCalls = 0
  await page.route('**/authoring-inference-runtime.json', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      schema: 'monogatari-authoring-inference-runtime/v1',
      provider: 'api',
      endpoint: '/authoring-api/chat/completions',
      model: 'e2e-roleplay-model',
      max_new_tokens: 128,
      temperature: 0.7,
      top_p: 0.9,
    }),
  }))
  await page.route('**/authoring-api/chat/completions', async (route) => {
    generationCalls += 1
    const content = generationCalls === 1
      ? 'I check the visible gate, then answer your proposal in character.'
      : JSON.stringify({
          score_deltas: [{
            dimension_id: 'story_progress',
            delta: 1,
            reason: 'The player made a concrete scene-bound proposal.',
          }],
          evidence: [],
          relationship_delta: 0,
          relationship_reason: 'No relationship rule is active.',
          npc_emotion: 'attentive',
          summary: 'The proposal advances the current scene objective.',
        })
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ choices: [{ message: { content } }] }),
    })
  })

  await page.goto('/roleplay-editor')
  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  await expect(page.getByText('Live roleplay saved')).toBeVisible()
  await page.getByRole('button', { name: 'Playtest', exact: true }).click()
  await expect(page).toHaveURL(/previewRoleplay=new_roleplay/)

  const roleplay = page.getByTestId('scene-roleplay')
  await expect(roleplay).toBeVisible()
  await expect(page.getByTestId('roleplay-runtime')).toBeVisible()
  const input = page.getByLabel('Say or do something in this scene...')
  await input.fill('I point to the gate and offer to inspect it with you.')
  const send = page.getByRole('button', { name: 'Send', exact: true })
  await expect(send).toBeEnabled()
  await send.click()

  await expect(roleplay).toHaveAttribute('data-evaluation-source', 'authoring_api_model')
  await expect(roleplay).toHaveAttribute('data-evaluation-deltas', '1')
  await expect(roleplay).toContainText('I check the visible gate')
  await expect(roleplay.locator('.score-label')).toContainText('+1.0')
  await expect(page.locator('.dialogue-text')).toHaveCount(0)
  expect(generationCalls).toBe(2)
})

test('live roleplay authoring remains selectable and contained on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/roleplay-editor')

  const picker = page.getByLabel('Roleplay', { exact: true })
  await expect(picker).toBeVisible()
  await expect.poll(() => picker.inputValue()).not.toBe('')
  const originalId = await picker.inputValue()

  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  await expect(page.getByText('Live roleplay saved')).toBeVisible()
  await expect.poll(() => picker.locator('option').count()).toBeGreaterThan(1)
  await picker.selectOption(originalId)

  await expect(page.getByLabel('Roleplay ID')).toHaveValue(originalId)
  await expect.poll(() => page.evaluate(() => document.body.scrollWidth)).toBe(390)
  const geometry = await page.locator('.roleplay-editor').evaluate((element) => {
    const rect = element.getBoundingClientRect()
    return { left: rect.left, right: rect.right, bottom: rect.bottom }
  })
  expect(geometry.left).toBeGreaterThanOrEqual(0)
  expect(geometry.right).toBeLessThanOrEqual(390)
  expect(geometry.bottom).toBeLessThanOrEqual(784)
})

test('Workflow execution renders deterministic trace evidence across desktop and mobile', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto('/editor')

  await expect(page.getByRole('heading', { name: 'Workflow Editor' })).toBeVisible()
  await expect(page.locator('.workflow-node')).toHaveCount(2)
  await page.locator('.toolbar-right').getByRole('button', { name: 'Run', exact: true }).click()

  await expect(page.locator('.execution-summary')).toContainText('Completed')
  await expect(page.locator('.coverage-row')).toContainText('100%')
  await expect(page.locator('.trace-item')).toHaveCount(2)
  await expect(page.locator('.workflow-node.run-executed')).toHaveCount(2)
  await expect(page.locator('.workflow-node.run-pass')).toHaveCount(1)
  await expect(page.locator('.workflow-node.run-current')).toHaveCount(1)

  await page.setViewportSize({ width: 390, height: 844 })
  await expect.poll(() => page.evaluate(() => document.body.scrollWidth)).toBe(390)
  const compactGeometry = await page.evaluate(() => {
    const inspector = document.querySelector('.properties-panel')?.getBoundingClientRect()
    return {
      viewportWidth: window.innerWidth,
      inspectorLeft: inspector?.left ?? -1,
      inspectorRight: inspector?.right ?? -1,
      inspectorWidth: inspector?.width ?? -1,
    }
  })
  expect(compactGeometry.inspectorLeft).toBeGreaterThanOrEqual(0)
  expect(compactGeometry.inspectorRight).toBeLessThanOrEqual(compactGeometry.viewportWidth)
  expect(compactGeometry.inspectorWidth).toBeGreaterThan(0)
})

test('Workflow canvas delegates drag and connection gestures', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto('/editor')

  const startNode = page.locator('.workflow-node.node-type-start')
  const endNode = page.locator('.workflow-node.node-type-end')
  await expect(startNode).toHaveCount(1)
  await expect(startNode.locator('.node-port.input')).toHaveCount(0)
  await expect(startNode.locator('.node-port.output')).toHaveCount(1)
  await expect(endNode.locator('.node-port.input')).toHaveCount(1)
  await expect(endNode.locator('.node-port.output')).toHaveCount(0)
  const beforeDrag = await startNode.boundingBox()
  expect(beforeDrag).not.toBeNull()
  await page.mouse.move(beforeDrag!.x + 80, beforeDrag!.y + 40)
  await page.mouse.down()
  await page.mouse.move(beforeDrag!.x + 120, beforeDrag!.y + 70)
  await page.mouse.up()
  await expect.poll(async () => (await startNode.boundingBox())?.x ?? -1)
    .toBeGreaterThan(beforeDrag!.x + 20)

  await expect(page.locator('.connections path')).toHaveCount(1)
  await page.locator('.palette-node').filter({ hasText: 'Narration' }).click()
  await expect(page.locator('.workflow-node')).toHaveCount(3)

  const narrationPort = page.locator('.workflow-node.node-type-narration .node-port.output')
  const endInput = endNode.locator('.node-port.input')
  const [portBox, inputBox] = await Promise.all([narrationPort.boundingBox(), endInput.boundingBox()])
  expect(portBox).not.toBeNull()
  expect(inputBox).not.toBeNull()
  const portCenter = {
    x: portBox!.x + portBox!.width / 2,
    y: portBox!.y + portBox!.height / 2,
  }
  expect(await page.evaluate(({ x, y }) => (
    document.elementFromPoint(x, y) as HTMLElement | null
  )?.className ?? '', portCenter)).toContain('node-port')
  await page.mouse.move(portCenter.x, portCenter.y)
  await page.mouse.down()
  await page.mouse.move(inputBox!.x + inputBox!.width / 2, inputBox!.y + inputBox!.height / 2)
  await page.mouse.up()

  await expect(page.locator('.connections path')).toHaveCount(2)

  await page.locator('.palette-node').filter({ hasText: 'Scene Change' }).click()
  const sceneField = page.locator('.property-group').filter({ hasText: 'Scene ID' }).locator('select')
  await expect(sceneField).toBeVisible()
  await expect(sceneField.locator('option')).not.toHaveCount(1)
})

test('character authoring persists a validated browser draft across reloads', async ({ page }) => {
  await page.goto('/character-editor')
  const createCharacter = page.getByTitle('Create Character')
  await expect(createCharacter).toBeEnabled()
  await createCharacter.click()

  await page.getByLabel('Character ID').fill('agent_guide')
  await page.getByLabel('Name').fill('Agent Guide')
  await page.getByLabel('Description').fill('A browser-authored delivery fixture.')
  await page.getByRole('button', { name: 'Save', exact: true }).click()

  await expect(page.getByText('Browser draft active')).toBeVisible()
  await expect(page.getByRole('button', { name: /Agent Guide/ })).toBeVisible()

  await page.getByTitle('Create Character').click()
  await page.getByLabel('Character ID').fill('AGENT_GUIDE')
  await page.getByLabel('Name').fill('Duplicate Guide')
  await expect(page.getByText('This character ID already exists.')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeDisabled()

  await page.reload()
  await expect(page.getByRole('button', { name: /Agent Guide/ })).toBeVisible()
  await expect(page.getByText('Browser draft active')).toBeVisible()
})

test('dialogue authoring saves a graph and opens it in browser Playtest', async ({ page }) => {
  await page.goto('/dialogue-editor')
  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByRole('button', { name: 'Script', exact: true }).click()
  await page.getByLabel('Dialogue ID').fill('agent_delivery_test')
  await page.getByLabel('Title', { exact: true }).fill('Agent Delivery Test')
  await page.getByRole('button', { name: 'Node', exact: true }).click()

  await page.getByRole('button', { name: 'Add Node', exact: true }).click()
  await page.locator('.rename-row input').fill('agent_delivery_end')
  await page.getByRole('button', { name: 'Rename', exact: true }).click()
  await page.getByLabel('Dialogue text').fill('The browser delivery path is ready.')
  await page.getByRole('group', { name: 'Node flow mode' }).getByRole('button', { name: 'End', exact: true }).click()

  await page.locator('.node-card').filter({ has: page.locator('.node-heading b', { hasText: /^start$/ }) }).click()
  await page.getByLabel('Dialogue text').fill('The browser delivery route begins.')
  await page.getByRole('group', { name: 'Node flow mode' }).getByRole('button', { name: 'Linear', exact: true }).click()
  await expect(page.getByLabel('Next node')).toHaveValue('agent_delivery_end')
  await page.getByRole('button', { name: 'Save', exact: true }).first().click()

  await expect(page.getByText('Dialogue created')).toBeVisible()
  await page.getByRole('button', { name: 'Playtest', exact: true }).click()
  await expect(page).toHaveURL(/\/game\?previewDialogue=agent_delivery_test&authoring=1$/)
  await expect(page.getByText('The browser delivery route begins.')).toBeVisible()
  const continueButton = page.getByRole('button', { name: 'Continue', exact: true })
  await expect(continueButton).toBeVisible()
  await continueButton.click()
  await expect(page.getByText('The browser delivery path is ready.')).toBeVisible()
})

test('Story Event authoring preserves metadata-only edits and reactive duplication', async ({ page }) => {
  await page.goto('/story-events')

  await expect(page.getByRole('heading', { name: 'Story Events' })).toBeVisible()
  await expect(page.locator('.status-strip')).toContainText('Loaded')
  const saveButton = page.getByRole('button', { name: 'Save catalog', exact: true })
  await expect(saveButton).toBeDisabled()

  const selectedHeading = page.locator('.event-inspector h2')
  const originalId = (await selectedHeading.textContent())?.trim()
  if (!originalId) throw new Error('Story Event editor did not select a loaded event')
  const metadata = page.locator('.metadata-input')
  await metadata.fill('{"agent":"ready"}')
  await expect(saveButton).toBeEnabled()

  await page.getByRole('button', { name: 'Duplicate', exact: true }).click()
  const duplicateId = `${originalId}_copy`
  await expect(selectedHeading).toHaveText(duplicateId)
  await expect(metadata).toHaveValue('{\n  "agent": "ready"\n}')
  await saveButton.click()
  await expect(page.locator('.status-strip')).toContainText('Saved')

  await page.reload()
  await page.getByLabel('Search events').fill(duplicateId)
  await expect(page.locator('.event-row')).toHaveCount(1)
  await page.locator('.event-row').click()
  await expect(metadata).toHaveValue('{\n  "agent": "ready"\n}')
})

test('Ending authoring saves real references, previews, and rejects portable ID collisions', async ({ page }) => {
  await page.goto('/endings')

  await expect(page.getByRole('heading', { name: 'Ending Routes' })).toBeVisible()
  await expect(page.locator('.ending-item').first()).toBeVisible()
  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByLabel('Ending ID').fill('agent_ending_test')
  await page.getByLabel('Title', { exact: true }).fill('Agent Ending Test')
  await page.getByLabel('Description').fill('A browser-authored ending delivery fixture.')
  await expect(page.getByRole('combobox', { name: 'Scene', exact: true })).not.toHaveValue('')
  await expect(page.getByRole('combobox', { name: 'Dialogue', exact: true })).not.toHaveValue('')
  await page.getByRole('button', { name: 'Save', exact: true }).click()

  await expect(page.getByText('Ending saved')).toBeVisible()
  await page.getByRole('button', { name: 'Preview', exact: true }).click()
  await expect(page).toHaveURL(/\/game\?previewEnding=agent_ending_test&authoring=1$/)
  await expect(page.locator('.dialogue-box')).toBeVisible()

  await page.goto('/endings')
  await expect(page.locator('.ending-item').first()).toBeVisible()
  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByLabel('Ending ID').fill('AGENT_ENDING_TEST')
  await expect(page.getByRole('alert')).toContainText('already exists')
  await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeDisabled()
})

test('Scene authoring saves a real background, previews, and rejects portable ID collisions', async ({ page }) => {
  await page.goto('/scene-editor')

  await expect(page.getByRole('heading', { name: 'Scene Catalog' })).toBeVisible()
  await expect(page.locator('.scene-item').first()).toBeVisible()
  const projectBackground = await page.getByLabel('Background path').inputValue()
  expect(projectBackground).not.toBe('')

  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByLabel('Scene ID').fill('agent_scene_test')
  await page.getByLabel('Name', { exact: true }).fill('Agent Scene Test')
  await page.getByLabel('Background path').fill(projectBackground)
  await page.getByLabel('Tags').fill('agent, delivery, agent')
  await page.locator('.header-actions').getByRole('button', { name: 'Save', exact: true }).click()

  await expect(page.locator('.notice.success')).toContainText('Scene created')
  await page.getByRole('button', { name: 'Playtest', exact: true }).click()
  await expect(page).toHaveURL(/\/game\?previewScene=agent_scene_test&authoring=1$/)
  await expect(page.locator('.scene-meta strong')).toHaveText('Agent Scene Test')
  await expect(page.locator('.model-placeholder')).toBeVisible()
  await expect(page.locator('.scene-empty p')).toContainText(projectBackground)
  await expect(page.locator('.scene-backdrop')).toHaveAttribute('style', /url\(/)

  await page.goto('/scene-editor')
  await expect(page.locator('.scene-item').first()).toBeVisible()
  await page.getByRole('button', { name: 'New', exact: true }).click()
  await page.getByLabel('Scene ID').fill('AGENT_SCENE_TEST')
  await page.getByLabel('Name', { exact: true }).fill('Portable Collision')
  await expect(page.getByRole('alert')).toContainText('already exists')
  await expect(page.locator('.header-actions').getByRole('button', { name: 'Save', exact: true })).toBeDisabled()
})

test('Scene Asset diagnostics uses the project catalog and persists browser runtime selection', async ({ page }) => {
  await page.goto('/assets')

  await expect(page.getByRole('heading', { name: 'Scene Assets' })).toBeVisible()
  const sceneRows = page.locator('.scene-row')
  await expect.poll(() => sceneRows.count()).toBeGreaterThan(2)
  const target = page.locator('.scene-row:not(.active)').first()
  const targetId = (await target.locator('.scene-copy small').textContent())?.trim()
  const targetName = (await target.locator('.scene-copy strong').textContent())?.trim()
  if (!targetId || !targetName) throw new Error('Scene Asset diagnostics did not expose a non-active project Scene')

  await target.click()
  await page.getByRole('button', { name: 'Set Active', exact: true }).click()
  await expect(page.locator('.status-toast.success')).toContainText(`Active scene: ${targetName}`)
  const activated = page.locator('.scene-row').filter({ hasText: targetId })
  await expect(activated).toHaveClass(/active/)

  await page.reload()
  const persisted = page.locator('.scene-row').filter({ hasText: targetId })
  await expect(persisted).toHaveClass(/active/)
  await page.getByRole('button', { name: 'Active', exact: true }).first().click()
  await expect(page.locator('.scene-row')).toHaveCount(1)
  await expect(page.locator('.scene-row .scene-copy small')).toHaveText(targetId)
})

test('Knowledge authoring persists Agent context and protects browser character references', async ({ page }) => {
  await page.goto('/knowledge')

  await expect(page.getByRole('heading', { name: 'Knowledge Base' })).toBeVisible()
  await page.getByRole('button', { name: 'Create Entry', exact: true }).click()
  await page.getByLabel('Entry ID').fill('agent_knowledge_test')
  await page.getByLabel('Name', { exact: true }).fill('Agent Knowledge Test')
  await page.getByLabel('Content', { exact: true }).fill('Canonical browser-authored context for Agent delivery.')
  await page.getByLabel('Tags (comma-separated)').fill('agent, delivery, agent')
  await page.getByLabel('Related entries (comma-separated)').fill('location_park')
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  await expect(page.locator('.status-toast.success')).toContainText('Knowledge entry saved')

  await page.reload()
  await page.getByPlaceholder('Search knowledge...').fill('agent_knowledge_test')
  const knowledgeRow = page.locator('.entry-row').filter({ hasText: 'Agent Knowledge Test' })
  await expect(knowledgeRow).toHaveCount(1)
  await knowledgeRow.click()
  await expect(page.locator('.related-list')).toContainText('location_park')

  await page.goto('/character-editor')
  const createCharacter = page.getByTitle('Create Character')
  await expect(createCharacter).toBeEnabled()
  await createCharacter.click()
  await page.getByLabel('Character ID').fill('agent_knowledge_guard')
  await page.getByLabel('Name').fill('Agent Knowledge Guard')
  await page.getByLabel('Description').fill('Protects browser-authored canonical context.')
  await page.getByRole('tab', { name: 'Knowledge', exact: true }).click()
  await page.getByLabel('Pinned knowledge references').fill('agent_knowledge_test')
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  await expect(page.getByText('Browser draft active')).toBeVisible()

  await page.goto('/knowledge')
  await page.getByPlaceholder('Search knowledge...').fill('agent_knowledge_test')
  const protectedRow = page.locator('.entry-row').filter({ hasText: 'Agent Knowledge Test' })
  await protectedRow.click()
  await page.locator('.inspector-actions').getByRole('button', { name: 'Delete', exact: true }).click()
  const deleteDialog = page.getByRole('dialog', { name: 'Confirm' })
  await deleteDialog.getByRole('button', { name: 'Delete', exact: true }).click()
  await expect(page.locator('.status-toast.error')).toContainText('character:agent_knowledge_guard')
  await expect.poll(() => page.evaluate(() => (
    localStorage.getItem('monogatari:knowledge-authoring-catalog:v1') || ''
  ))).toContain('agent_knowledge_test')
})

test('Quality Suite workbench presents generated evidence across desktop and mobile', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto('/quality')

  await expect(page.getByRole('heading', { name: 'Quality Suites' })).toBeVisible()
  await expect(page.locator('.toolbar-metrics')).toContainText('29')
  await expect(page.locator('.scenario-row')).toHaveCount(29)
  await expect(page.locator('.diagnostics-panel')).toContainText('warm-creative-conversation')

  await page.getByLabel('Search scenarios').fill('score-gate-workflow-coverage')
  await expect(page.locator('.scenario-row')).toHaveCount(1)
  await page.getByRole('button', { name: /score-gate-workflow-coverage/ }).click()
  await expect(page.locator('.diagnostics-panel')).toContainText('Workflow Coverage')
  await expect(page.locator('.diagnostics-panel')).toContainText('100%')

  await page.getByRole('tab', { name: 'Audit' }).click()
  await expect(page.locator('.category-audit-row')).toHaveCount(8)
  await expect(page.locator('.safety-signal-list')).toContainText('Runtime guards')
  await expect(page.locator('.workflow-audit-list')).toContainText('Score Gate Demo')

  await page.setViewportSize({ width: 390, height: 844 })
  await expect.poll(() => page.evaluate(() => ({
    bodyWidth: document.body.scrollWidth,
    viewportWidth: window.innerWidth,
    toolbarBottom: document.querySelector('.quality-toolbar')?.getBoundingClientRect().bottom ?? 0,
    bodyTop: document.querySelector('.quality-body')?.getBoundingClientRect().top ?? 0,
  }))).toEqual({
    bodyWidth: 390,
    viewportWidth: 390,
    toolbarBottom: expect.any(Number),
    bodyTop: expect.any(Number),
  })
  const compactGeometry = await page.evaluate(() => ({
    toolbarBottom: document.querySelector('.quality-toolbar')?.getBoundingClientRect().bottom ?? 0,
    bodyTop: document.querySelector('.quality-body')?.getBoundingClientRect().top ?? 0,
  }))
  expect(compactGeometry.bodyTop).toBeGreaterThanOrEqual(compactGeometry.toolbarBottom)

  const diagnostics = page.locator('.diagnostics-panel')
  await diagnostics.getByRole('button', { name: 'Close' }).click()
  await expect(diagnostics).not.toHaveClass(/compact-open/)
  await page.getByRole('button', { name: 'Open diagnostics' }).click()
  await expect(diagnostics).toHaveClass(/compact-open/)
})

test('Settings keeps runtime credentials out of saved browser manifests across desktop and mobile', async ({ page }) => {
  let sessionRequestBody = ''
  await page.route('**/authoring-api/session', async (route) => {
    sessionRequestBody = route.request().postData() || ''
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        schema: 'monogatari-authoring-inference-runtime/v1',
        provider: 'api',
        endpoint: '/authoring-api/chat/completions',
        model: 'browser-session-model',
        ready: true,
        issue: null,
        max_new_tokens: 128,
        temperature: 0.7,
        top_p: 0.9,
      }),
    })
  })
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto('/settings')

  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible()
  await expect(page.locator('.context-notice')).toContainText('keeps project changes in memory')
  await page.getByLabel('Project title').fill('Agent Settings Audit')

  await page.locator('.settings-nav .nav-item').filter({ hasText: /^AI/ }).click()
  await page.getByRole('button', { name: 'Development API' }).click()
  const runtimeCredential = `sk-${'A'.repeat(30)}`
  await page.getByLabel('API key').fill(runtimeCredential)
  await page.getByRole('button', { name: 'Connect', exact: true }).click()
  await expect(page.locator('.settings-toast')).toContainText('AI backend connected')
  await expect(page.getByLabel('API key')).toHaveValue('')
  expect(JSON.parse(sessionRequestBody)).toEqual({ api_key: runtimeCredential })
  expect(await page.evaluate(key => Object.values(localStorage).includes(key), runtimeCredential)).toBe(false)

  await page.getByRole('button', { name: 'Save project' }).click()
  await expect(page.locator('.settings-toast')).toContainText('settings updated for this session')

  await page.locator('.settings-nav .nav-item').filter({ hasText: /^Project/ }).click()
  const downloadPromise = page.waitForEvent('download')
  await page.getByRole('button', { name: 'Export manifest' }).click()
  const download = await downloadPromise
  const downloadPath = await download.path()
  if (!downloadPath) throw new Error('Settings manifest download did not produce a readable file')
  const manifest = JSON.parse(await readFile(downloadPath, 'utf8')) as {
    schema: string
    settings: {
      render: { title: string }
      ai: { provider: string; api: { api_key: string } }
    }
  }

  expect(manifest.schema).toBe('monogatari-project-export@1')
  expect(manifest.settings.render.title).toBe('Agent Settings Audit')
  expect(manifest.settings.ai.provider).toBe('api')
  expect(manifest.settings.ai.api.api_key).toBe('<redacted>')
  expect(JSON.stringify(manifest)).not.toContain(runtimeCredential)

  await page.setViewportSize({ width: 390, height: 844 })
  await expect.poll(() => page.evaluate(() => document.body.scrollWidth)).toBe(390)
  const compactGeometry = await page.evaluate(() => ({
    navBottom: document.querySelector('.settings-nav')?.getBoundingClientRect().bottom ?? 0,
    workspaceTop: document.querySelector('.settings-workspace')?.getBoundingClientRect().top ?? 0,
  }))
  expect(compactGeometry.workspaceTop).toBeGreaterThanOrEqual(compactGeometry.navBottom)
})
