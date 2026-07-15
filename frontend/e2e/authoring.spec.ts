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

  await expect(page.getByRole('link', { name: 'Monogatari Engine' })).toBeVisible()
  await page.getByRole('link', { name: 'Story Flow' }).click()
  await expect(page).toHaveURL(/\/editor$/)
  await expect(page.getByRole('heading', { name: 'Workflow Editor' })).toBeVisible()

  await page.getByRole('link', { name: 'Dialogues' }).click()
  await expect(page).toHaveURL(/\/dialogue-editor$/)
  await expect(page.getByRole('heading', { name: 'Dialogue Graph' })).toBeVisible()
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
  await page.setViewportSize({ width: 1280, height: 720 })
  await page.goto('/settings')

  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible()
  await expect(page.locator('.context-notice')).toContainText('keeps project changes in memory')
  await page.getByLabel('Project title').fill('Agent Settings Audit')

  await page.locator('.settings-nav .nav-item').filter({ hasText: /^AI/ }).click()
  await page.getByRole('button', { name: 'Development API' }).click()
  const runtimeCredential = `sk-${'A'.repeat(30)}`
  await page.getByLabel('API key').fill(runtimeCredential)
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
