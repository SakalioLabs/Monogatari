import { expect, test, type Page, type TestInfo } from '@playwright/test'

test.describe.configure({ timeout: 120_000 })

interface BlueFrameViewport {
  name: string
  size: { width: number; height: number }
}

interface SceneProbeExpectation {
  fileName: string
  excludedObjects: number
  minimumNonBackground: number
}

const viewports: BlueFrameViewport[] = [
  { name: 'desktop', size: { width: 1440, height: 900 } },
  { name: 'mobile', size: { width: 390, height: 844 } },
]

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('monogatari-locale', 'zh-CN')
    localStorage.setItem('monogatari-version-seen', '0.9.5')
  })
})

for (const viewport of viewports) {
  test(`Blue Frame ${viewport.name} starts the dynamic roleplay on the main stage`, async ({ page }, testInfo) => {
    await page.setViewportSize(viewport.size)
    const runtimeErrors = captureRuntimeErrors(page)

    await page.goto('/game?previewRoleplay=blue_frame_roleplay&authoring=1&rendererProbe=1')
    const roleplay = page.getByTestId('scene-roleplay')
    await expect(roleplay).toBeVisible({ timeout: 15_000 })
    await expect(roleplay).toHaveAttribute('data-roleplay-status', 'active')
    await expect(roleplay).toContainText('潮镜：蓝色定格')
    await expect(roleplay).toContainText('通过自由对话提出可复做的核验方法')
    await expect(roleplay.locator('.narration-entry')).toContainText('你可以直接向它提问')
    await expect(roleplay.locator('.score-item')).toHaveCount(3)
    await expect(page.getByTestId('npc-trigger')).toHaveCount(0)

    const composer = roleplay.locator('textarea')
    await expect(composer).toBeVisible()
    await expect(composer).toHaveAttribute('maxlength', '4000')
    await composer.fill('先核验坐标和时间戳，再讨论你是谁。')
    await expect(roleplay.locator('.send-button')).toBeEnabled()
    await expectRoleplayLayoutInsideViewport(page)

    await testInfo.attach(`blue-frame-roleplay-${viewport.name}`, {
      body: await page.screenshot(),
      contentType: 'image/png',
    })
    expect(runtimeErrors, runtimeErrors.join('\n')).toEqual([])
  })

  test(`Blue Frame ${viewport.name} route renders all authored 3D scenes`, async ({ page }, testInfo) => {
    test.slow()
    await page.setViewportSize(viewport.size)
    const runtimeErrors = captureRuntimeErrors(page)

    await openDialogueNode(page, 'start', '信标警报解除后的第七天')
    await expectSceneModel(page, {
      fileName: 'blue_frame_earth.glb',
      excludedObjects: 3,
      minimumNonBackground: 40,
    })
    await expectNpcConversationPanel(page)

    await openDialogueNode(page, 'first_test_choice', '记录本停在空白页')
    await expectChoicesInsideViewport(page, 3)

    await openDialogueNode(page, 'classroom_01', '探针穿过变形的门框')
    await expectSceneModel(page, {
      fileName: 'blue_frame_classroom.glb',
      excludedObjects: 1,
      minimumNonBackground: 80,
    })
    await expectLayoutInsideViewport(page)

    await openDialogueNode(page, 'classroom_response_choice', '九号回声的轮廓在两张桌面之间重叠')
    await expectChoicesInsideViewport(page, 3)
    await openDialogueNode(page, 'evidence_form_choice', '铁盒中的名字没有替声音作证')
    await expectChoicesInsideViewport(page, 2)

    await openDialogueNode(page, 'still_01', '蓝色定格室没有墙')
    await expectSceneModel(page, {
      fileName: 'blue_frame_still.glb',
      excludedObjects: 0,
      minimumNonBackground: 100,
    })
    await expectLayoutInsideViewport(page)

    await openDialogueNode(page, 'publication_choice', '你把手放在发布控制台上')
    await expectChoicesInsideViewport(page, 3)
    await openDialogueNode(page, 'truth_10', '黎明照进潮镜站')
    await expect(page.locator('.dialogue-text')).toContainText('无法独占身份的证词完整进入了公共记录')
    await expectLayoutInsideViewport(page)

    await testInfo.attach(`blue-frame-${viewport.name}`, {
      body: await page.screenshot(),
      contentType: 'image/png',
    })
    expect(runtimeErrors, runtimeErrors.join('\n')).toEqual([])
  })
}

test('Blue Frame ending previews start their dedicated epilogues', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1280, height: 720 })
  const runtimeErrors = captureRuntimeErrors(page)
  const endings = [
    {
      id: 'blue_frame_truth_ending',
      opening: '开放档案上线后的第一个清晨',
      final: '真相没有获得一张脸，却获得了公开、可追溯、允许反驳的未来。',
    },
    {
      id: 'blue_frame_beacon_ending',
      opening: '旧信标再次亮起时，九个名字沿海岸依次出现',
      final: '纪念没有补完任何人的人生，却让他们留下的方向照亮了后来者。',
    },
    {
      id: 'blue_frame_silence_ending',
      opening: '公共网络只收到坐标、证据等级与避难路线',
      final: '沉默成为有期限的责任，而不是遗忘。',
    },
  ]

  for (const ending of endings) {
    await page.goto(`/game?previewEnding=${ending.id}&authoring=1&rendererProbe=1`)
    await advanceUntilText(page, ending.opening)
    await expect(page.locator('.dialogue-text')).not.toContainText('信标警报解除后的第七天')
    await expectSceneModel(page, {
      fileName: 'blue_frame_still.glb',
      excludedObjects: 0,
      minimumNonBackground: 100,
    })
    await advanceUntilText(page, ending.final)
    await expectLayoutInsideViewport(page)
  }

  await testInfo.attach('blue-frame-dedicated-epilogues', {
    body: await page.screenshot(),
    contentType: 'image/png',
  })
  expect(runtimeErrors, runtimeErrors.join('\n')).toEqual([])
})

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

async function openDialogueNode(page: Page, nodeId: string, text: string): Promise<void> {
  const query = new URLSearchParams({
    previewDialogue: 'blue_frame_dialogue',
    previewNode: nodeId,
    authoring: '1',
    rendererProbe: '1',
  })
  await page.goto(`/game?${query}`)
  const dialogue = page.locator('.dialogue-text')
  await expect(dialogue).toBeVisible({ timeout: 15_000 })
  await page.keyboard.press('Enter')
  await expect(dialogue).toContainText(text, { timeout: 15_000 })
}

async function advanceUntilText(page: Page, text: string): Promise<void> {
  const dialogue = page.locator('.dialogue-text')
  for (let step = 0; step < 240; step += 1) {
    if ((await dialogue.textContent())?.includes(text)) return
    const choiceCount = await page.locator('.choice-btn').count()
    if (choiceCount > 0) {
      await page.keyboard.press('Enter')
      await expect(dialogue).toContainText(text, { timeout: 5_000 })
      return
    }
    const advance = page.locator('.advance-hint')
    await expect(advance).toBeVisible({ timeout: 15_000 })
    await page.keyboard.press('Enter')
    await page.waitForTimeout(5)
  }
  throw new Error(`Dialogue text did not become visible: ${text}`)
}

async function expectSceneModel(page: Page, expected: SceneProbeExpectation): Promise<void> {
  const model = page.locator('.scene-model-backdrop')
  await expect(model).toHaveCount(1)
  await expect(model).toHaveAttribute('data-model-path', new RegExp(expected.fileName), { timeout: 15_000 })
  await expect(model).toHaveAttribute('data-model-state', 'ready', { timeout: 15_000 })
  await expect(model).toHaveAttribute('data-framing-excluded-objects', String(expected.excludedObjects))
  await expect.poll(async () => Number(await model.getAttribute('data-canvas-non-background')), {
    message: `${expected.fileName} did not render enough non-background canvas samples`,
    timeout: 15_000,
  }).toBeGreaterThanOrEqual(expected.minimumNonBackground)
  await expect.poll(async () => Number(await model.getAttribute('data-canvas-unique-colors')), {
    message: `${expected.fileName} canvas did not contain enough color variation`,
    timeout: 15_000,
  }).toBeGreaterThan(20)

  const evidence = await model.evaluate((element) => {
    const canvas = element.querySelector('canvas')
    const bounds = (element.getAttribute('data-canvas-bounds') || '').split(',').map(Number)
    const rect = canvas?.getBoundingClientRect()
    return {
      bounds,
      canvas: rect ? { width: rect.width, height: rect.height } : null,
      signature: element.getAttribute('data-canvas-signature') || '',
    }
  })
  expect(evidence.signature).toMatch(/^[0-9a-f]{8}$/)
  expect(evidence.bounds).toHaveLength(4)
  expect(evidence.bounds[2] - evidence.bounds[0]).toBeGreaterThanOrEqual(8)
  expect(evidence.bounds[3] - evidence.bounds[1]).toBeGreaterThanOrEqual(8)
  expect(evidence.canvas?.width).toBeGreaterThan(300)
  expect(evidence.canvas?.height).toBeGreaterThan(600)
}

async function expectChoicesInsideViewport(page: Page, expectedCount: number): Promise<void> {
  const choices = page.locator('.choice-btn')
  await expect(choices).toHaveCount(expectedCount)
  const evidence = await choices.evaluateAll((elements) => elements.map((element) => {
    const box = element.getBoundingClientRect()
    return {
      left: box.left,
      right: box.right,
      top: box.top,
      bottom: box.bottom,
      scrollWidth: element.scrollWidth,
      clientWidth: element.clientWidth,
    }
  }))
  const viewport = await page.evaluate(() => ({ width: window.innerWidth, height: window.innerHeight }))
  for (const box of evidence) {
    expect(box.left).toBeGreaterThanOrEqual(-1)
    expect(box.right).toBeLessThanOrEqual(viewport.width + 1)
    expect(box.top).toBeGreaterThanOrEqual(-1)
    expect(box.bottom).toBeLessThanOrEqual(viewport.height + 1)
    expect(box.scrollWidth).toBeLessThanOrEqual(box.clientWidth + 1)
  }
}

async function expectNpcConversationPanel(page: Page): Promise<void> {
  await page.getByTestId('npc-trigger').click()
  const panel = page.getByTestId('npc-panel')
  await expect(panel).toBeVisible()
  await expect(panel).toHaveAttribute('data-npc-runtime', 'webgpu')
  await expect(panel).toContainText('九号回声')
  const composer = page.getByTestId('npc-input')
  await expect(composer).toBeVisible()
  const bounds = await composer.boundingBox()
  const viewport = page.viewportSize()
  expect(bounds).not.toBeNull()
  expect(viewport).not.toBeNull()
  expect(bounds!.x).toBeGreaterThanOrEqual(0)
  expect(bounds!.y).toBeGreaterThanOrEqual(0)
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(viewport!.width)
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(viewport!.height)
  await page.getByTestId('npc-close').click()
  await expect(panel).toHaveCount(0)
}

async function expectLayoutInsideViewport(page: Page): Promise<void> {
  const layout = await page.evaluate(() => {
    const viewport = { width: window.innerWidth, height: window.innerHeight }
    const boxes = ['.game-topbar', '.dialogue-box', '.sprite-stage img'].map((selector) => {
      const element = document.querySelector(selector)
      if (!element) return { selector, box: null }
      const box = element.getBoundingClientRect()
      return { selector, box: { left: box.left, top: box.top, right: box.right, bottom: box.bottom } }
    })
    return {
      viewport,
      boxes,
      documentWidth: document.documentElement.scrollWidth,
      documentHeight: document.documentElement.scrollHeight,
    }
  })
  const evidence = JSON.stringify(layout)
  expect(layout.documentWidth, evidence).toBe(layout.viewport.width)
  expect(layout.documentHeight, evidence).toBe(layout.viewport.height)
  for (const entry of layout.boxes) {
    expect(entry.box, `${entry.selector} is missing: ${evidence}`).not.toBeNull()
    expect(entry.box!.left, `${entry.selector} escaped left: ${evidence}`).toBeGreaterThanOrEqual(-1)
    expect(entry.box!.right, `${entry.selector} escaped right: ${evidence}`).toBeLessThanOrEqual(layout.viewport.width + 1)
    expect(entry.box!.top, `${entry.selector} escaped top: ${evidence}`).toBeGreaterThanOrEqual(-1)
    expect(entry.box!.bottom, `${entry.selector} escaped bottom: ${evidence}`).toBeLessThanOrEqual(layout.viewport.height + 1)
  }
}

async function expectRoleplayLayoutInsideViewport(page: Page): Promise<void> {
  const layout = await page.evaluate(() => {
    const viewport = { width: window.innerWidth, height: window.innerHeight }
    const boxes = ['.game-topbar', '.model-area', '.roleplay-shell', '.roleplay-composer textarea'].map((selector) => {
      const element = document.querySelector(selector)
      if (!element) return { selector, box: null, scrollWidth: 0, clientWidth: 0 }
      const box = element.getBoundingClientRect()
      return {
        selector,
        box: { left: box.left, top: box.top, right: box.right, bottom: box.bottom },
        scrollWidth: element.scrollWidth,
        clientWidth: element.clientWidth,
      }
    })
    return {
      viewport,
      boxes,
      documentWidth: document.documentElement.scrollWidth,
      documentHeight: document.documentElement.scrollHeight,
    }
  })
  const evidence = JSON.stringify(layout)
  expect(layout.documentWidth, evidence).toBe(layout.viewport.width)
  expect(layout.documentHeight, evidence).toBe(layout.viewport.height)
  for (const entry of layout.boxes) {
    expect(entry.box, `${entry.selector} is missing: ${evidence}`).not.toBeNull()
    expect(entry.box!.left, `${entry.selector} escaped left: ${evidence}`).toBeGreaterThanOrEqual(-1)
    expect(entry.box!.right, `${entry.selector} escaped right: ${evidence}`).toBeLessThanOrEqual(layout.viewport.width + 1)
    expect(entry.box!.top, `${entry.selector} escaped top: ${evidence}`).toBeGreaterThanOrEqual(-1)
    expect(entry.box!.bottom, `${entry.selector} escaped bottom: ${evidence}`).toBeLessThanOrEqual(layout.viewport.height + 1)
    expect(entry.scrollWidth, `${entry.selector} overflowed horizontally: ${evidence}`).toBeLessThanOrEqual(entry.clientWidth + 1)
  }
}
