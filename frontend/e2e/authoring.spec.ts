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

test('character authoring persists a validated browser draft across reloads', async ({ page }) => {
  await page.goto('/character-editor')
  await page.getByTitle('Create Character').click()

  await page.getByLabel('Character ID').fill('agent_guide')
  await page.getByLabel('Name').fill('Agent Guide')
  await page.getByLabel('Description').fill('A browser-authored delivery fixture.')
  await page.getByRole('button', { name: 'Save', exact: true }).click()

  await expect(page.getByText('Browser draft active')).toBeVisible()
  await expect(page.getByRole('button', { name: /Agent Guide/ })).toBeVisible()
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
  await page.getByLabel('Dialogue text').fill('The browser delivery path is ready.')
  await page.getByRole('button', { name: 'Save', exact: true }).first().click()

  await expect(page.getByText('Dialogue created')).toBeVisible()
  await page.getByRole('button', { name: 'Playtest', exact: true }).click()
  await expect(page).toHaveURL(/\/game\?previewDialogue=agent_delivery_test&authoring=1$/)
  await expect(page.getByText('The browser delivery path is ready.')).toBeVisible()
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
