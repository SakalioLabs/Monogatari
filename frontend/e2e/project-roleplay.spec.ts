import { expect, test, type Locator, type Page, type TestInfo } from '@playwright/test'

const roleplayId = process.env.MONOGATARI_E2E_ROLEPLAY_ID || ''
const expectedTitle = process.env.MONOGATARI_E2E_ROLEPLAY_TITLE || ''
const expectedRuntime = process.env.MONOGATARI_E2E_EXPECT_RUNTIME || ''
const liveAiEnabled = process.env.MONOGATARI_E2E_LIVE_AI === '1'
const liveMessage = process.env.MONOGATARI_E2E_LIVE_MESSAGE
  || 'Keep the plan observable, assign a watch rotation, and agree on the signal for regrouping.'

const viewports = [
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'mobile', width: 390, height: 844 },
]

test.describe('configured project roleplay', () => {
  test.skip(!roleplayId, 'Set MONOGATARI_E2E_ROLEPLAY_ID to validate a project roleplay.')
  test.describe.configure({ timeout: 300_000 })

  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('monogatari-locale', 'zh-CN')
      localStorage.setItem('monogatari-version-seen', '0.9.5')
    })
  })

  for (const viewport of viewports) {
    test(`${viewport.name} opens the free-form roleplay as the primary interaction`, async ({
      page,
    }, testInfo) => {
      await page.setViewportSize(viewport)
      const runtimeErrors = captureRuntimeErrors(page)
      const roleplay = await openRoleplay(page)

      await expect(roleplay).toHaveAttribute('data-roleplay-status', 'active')
      await expect(roleplay.locator('.node-kicker')).not.toBeEmpty()
      await expect(roleplay.locator('.narration-entry')).toBeVisible()
      await expectNarrationInsideTranscript(roleplay)
      await expect(roleplay.locator('textarea')).toBeVisible()
      await expect(page.getByTestId('npc-trigger')).toHaveCount(0)
      await expect(page.locator('.dialogue-text')).toHaveCount(0)
      await expect(page.getByTestId('npc-trigger')).toHaveCount(0)
      if (expectedTitle) await expect(roleplay).toContainText(expectedTitle)

      const runtime = page.getByTestId('roleplay-runtime')
      await expect(runtime).not.toHaveAttribute('data-runtime-kind', 'loading')
      if (expectedRuntime) {
        await expect(runtime).toHaveAttribute('data-runtime-kind', expectedRuntime)
      }
      await expectLayoutInsideViewport(page, roleplay)
      await attachScreenshot(page, testInfo, `project-roleplay-${viewport.name}`)
      expect(runtimeErrors, runtimeErrors.join('\n')).toEqual([])
    })
  }

  test('stale scripted preview links recover into the primary live story', async ({ page }) => {
    await page.goto('/game?previewDialogue=blue_frame_dialogue&authoring=1')

    await expect(page.getByTestId('scene-roleplay')).toBeVisible({ timeout: 30_000 })
    await expect(page).not.toHaveURL(/previewDialogue=/)
    await expect(page).toHaveURL(/preview(?:Campaign|Roleplay)=/)
    await expect(page.getByTestId('roleplay-runtime')).toBeVisible()
  })

  test('uses separate live NPC and evaluator calls before deterministic commit', async ({ page }) => {
    test.skip(!liveAiEnabled, 'Set MONOGATARI_E2E_LIVE_AI=1 to call the configured model.')
    const runtimeErrors = captureRuntimeErrors(page)
    const inferenceStatuses: number[] = []
    page.on('response', (response) => {
      if (response.url().includes('/authoring-api/chat/completions')) {
        inferenceStatuses.push(response.status())
      }
    })

    const roleplay = await openRoleplay(page)
    await expect(page.getByTestId('roleplay-runtime')).toHaveAttribute('data-runtime-kind', 'api')
    await roleplay.locator('textarea').fill(liveMessage)
    await roleplay.locator('.send-button').click()

    await expect(roleplay).toHaveAttribute(
      'data-evaluation-source',
      /^authoring_api_model(?:_reconciled)?$/,
      { timeout: 240_000 },
    )
    await expect(roleplay.locator('.turn-entry:not(.pending)')).toHaveCount(2)
    await expect(page.getByTestId('roleplay-degraded')).toHaveCount(0)
    expect(inferenceStatuses).toEqual([200, 200])
    expect(runtimeErrors, runtimeErrors.join('\n')).toEqual([])
  })
})

async function openRoleplay(page: Page) {
  await page.goto(`/game?previewRoleplay=${encodeURIComponent(roleplayId)}&authoring=1`)
  const roleplay = page.getByTestId('scene-roleplay')
  await expect(roleplay).toBeVisible({ timeout: 30_000 })
  return roleplay
}

function captureRuntimeErrors(page: Page): string[] {
  const errors: string[] = []
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(`console: ${message.text()}`)
  })
  page.on('pageerror', (error) => errors.push(`page: ${error.message}`))
  page.on('requestfailed', (request) => {
    errors.push(`request: ${request.url()} (${request.failure()?.errorText || 'failed'})`)
  })
  return errors
}

async function expectLayoutInsideViewport(page: Page, roleplay: Locator) {
  const viewport = page.viewportSize()
  const bounds = await roleplay.boundingBox()
  expect(viewport).not.toBeNull()
  expect(bounds).not.toBeNull()
  expect(bounds!.x).toBeGreaterThanOrEqual(-1)
  expect(bounds!.y).toBeGreaterThanOrEqual(-1)
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(viewport!.width + 1)
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(viewport!.height + 1)
}

async function expectNarrationInsideTranscript(roleplay: Locator) {
  await expect.poll(async () => {
    const transcript = await roleplay.locator('.roleplay-transcript').boundingBox()
    const narration = await roleplay.locator('.narration-entry p').boundingBox()
    if (!transcript || !narration) return 0
    return Math.max(
      0,
      Math.min(transcript.y + transcript.height, narration.y + narration.height)
        - Math.max(transcript.y, narration.y),
    )
  }).toBeGreaterThan(16)
}

async function attachScreenshot(
  page: Page,
  testInfo: TestInfo,
  name: string,
): Promise<void> {
  await testInfo.attach(name, {
    body: await page.screenshot(),
    contentType: 'image/png',
  })
}
