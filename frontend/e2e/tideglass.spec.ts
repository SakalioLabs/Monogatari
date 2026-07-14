import { expect, test, type Page, type TestInfo } from '@playwright/test'

interface TideglassRoute {
  name: string
  viewport: { width: number; height: number }
  openingChoice: string
  investigationChoice: string
  branchEvidence: string
  branchChoice: string
  finalChoice: string
  terminalText: string
  endingId: string
  endingBackground: string
  endingOpening: string
  endingVoice: string
  endingFinal: string
}

const routes: TideglassRoute[] = [
  {
    name: 'beacon',
    viewport: { width: 1440, height: 900 },
    openingChoice: '先稳定讯号。无论是谁，至少让它把话说完。',
    investigationChoice: '查阅守塔日志，寻找过去留下的同类记录。',
    branchEvidence: '1897年，信标曾收到',
    branchChoice: '空白不等于罪。被救下的人仍然真实。',
    finalChoice: '点亮信标。先救下能够看见这束光的人。',
    terminalText: '我们守住的不是未来，是愿意为陌生人留一盏灯的现在。',
    endingId: 'tideglass_beacon_ending',
    endingBackground: 'tideglass_lantern_chamber.png',
    endingOpening: '风暴过去后，潮镜站的光每晚都会准时越过外海。',
    endingVoice: '港口没有为我们立碑。很好，灯本来就不是为了被感谢。',
    endingFinal: '在不再需要求救的未来，九号回声留下最后一句：有人回家了。',
  },
  {
    name: 'truth',
    viewport: { width: 1280, height: 720 },
    openingChoice: '先切断发射端，只保留接收。我们不能让未知讯号控制信标。',
    investigationChoice: '拆开接收机，验证讯号是否真的来自未来。',
    branchEvidence: '频率偏移与浪高同步，却提前了四十七秒。',
    branchChoice: '物理上说得通。相信讯号，但不相信它对因果的解释。',
    finalChoice: '公开讯号。把事实与风险一起交给整片海岸。',
    terminalText: '真相不会替我们负责。但从今晚起，责任不再只锁在这座塔里。',
    endingId: 'tideglass_truth_ending',
    endingBackground: 'tideglass_control_room.png',
    endingOpening: '整片海岸都听见了来自未来的讯号，也听见了它无法证明的部分。',
    endingVoice: '他们争论、核查、修堤。真相没有带来安静，但带来了共同承担。',
    endingFinal: '九号回声散去前说：终于，不只有一座塔记得。',
  },
  {
    name: 'silence-mobile',
    viewport: { width: 390, height: 844 },
    openingChoice: '先稳定讯号。无论是谁，至少让它把话说完。',
    investigationChoice: '直接询问九号回声：它记得怎样的未来？',
    branchEvidence: '我那时九岁。后来他们用幸存者的声音训练发射协议',
    branchChoice: '无论你是什么，害怕消失都是真的。',
    finalChoice: '清除讯号。拒绝让一个未经证实的未来劫持现在。',
    terminalText: '讯号消失了。可我们已经听见。沉默不再等于什么都没发生。',
    endingId: 'tideglass_silence_ending',
    endingBackground: 'tideglass_lantern_chamber.png',
    endingOpening: '磁鼓被清空后，控制室只剩真实的浪声。坐标却留在你们的纸上。',
    endingVoice: '我们没有服从未来，也没有假装没听见。明天去检查东堤。',
    endingFinal: '没有声音回答。远海的黑暗里，一艘船平安转向。',
  },
]

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('monogatari-locale', 'en')
    localStorage.setItem('monogatari-version-seen', '0.9.5')
    localStorage.setItem('monogatari.activeScene', JSON.stringify({
      id: 'tideglass_causeway',
      name: '潮镜站外海石堤',
      background_path: 'assets/backgrounds/tideglass_causeway.png',
      bgm_path: null,
      weather: 'storm',
      time_of_day: 'night',
      tags: ['tideglass', 'coast', 'storm', 'arrival'],
      source: 'web_project',
      background_exists: true,
      absolute_background_path: null,
    }))
  })
})

for (const route of routes) {
  test(`Tideglass ${route.name} route and ending render without runtime errors`, async ({ page }, testInfo) => {
    await page.setViewportSize(route.viewport)
    const runtimeErrors = captureRuntimeErrors(page)

    await page.goto('/game?previewDialogue=tideglass_signal&authoring=1')
    await advanceUntilText(page, '午夜十二点前，潮水已经漫过堤道两次。')
    await expectBackground(page, 'tideglass_causeway.png')

    await advanceUntilText(page, '踩我走过的石缝。黑色的地方不是积水，是海。')
    await expectLoadedSprite(page, '澜音')
    await advanceUntilText(page, '这里是九号回声。我们只剩十七分钟。')
    await expectLoadedSprite(page, '九号回声')

    await choose(page, route.openingChoice)
    await choose(page, route.investigationChoice)
    await advanceUntilText(page, route.branchEvidence)
    await choose(page, route.branchChoice)
    await choose(page, route.finalChoice)
    await advanceUntilText(page, route.terminalText)
    await expectLoadedSprite(page, '澜音')
    await expectReadableDialogue(page)
    await expectLayoutInsideViewport(page)

    await page.goto(`/game?previewEnding=${route.endingId}&authoring=1`)
    await advanceUntilText(page, route.endingOpening)
    await expectBackground(page, route.endingBackground)
    await advanceUntilText(page, route.endingVoice)
    await expectLoadedSprite(page, '澜音')
    await advanceUntilText(page, route.endingFinal)
    await expectLoadedSprite(page, '九号回声')
    await expectReadableDialogue(page)
    await expectLayoutInsideViewport(page)

    await testInfo.attach(`tideglass-${route.name}`, {
      body: await page.screenshot(),
      contentType: 'image/png',
    })
    expect(runtimeErrors, runtimeErrors.join('\n')).toEqual([])
  })
}

test('browser Playtest does not expose or schedule desktop-only saves', async ({ page }) => {
  await page.clock.install()
  const runtimeErrors = captureRuntimeErrors(page)
  await page.goto('/game?previewDialogue=tideglass_signal&authoring=1')
  await expect(page.locator('.dialogue-box')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Save', exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Load', exact: true })).toHaveCount(0)
  await expect(page.locator('.auto-save-badge')).toHaveCount(0)

  await page.clock.fastForward(120_001)
  await page.evaluate(() => true)
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

async function choose(page: Page, text: string): Promise<void> {
  const choice = page.locator('.choice-btn').filter({ hasText: text })
  for (let step = 0; step < 80; step += 1) {
    if (await choice.isVisible()) {
      await choice.click()
      return
    }
    await chooseAdvance(page)
  }
  throw new Error(`Choice did not become available: ${text}`)
}

async function advanceUntilText(page: Page, text: string): Promise<void> {
  const dialogue = page.locator('.dialogue-text')
  for (let step = 0; step < 80; step += 1) {
    if ((await dialogue.textContent())?.includes(text)) return
    await chooseAdvance(page)
  }
  throw new Error(`Dialogue text did not become visible: ${text}`)
}

async function chooseAdvance(page: Page): Promise<void> {
  const advance = page.locator('.advance-hint')
  await expect(advance).toBeVisible({ timeout: 10_000 })
  await advance.click()
  await page.waitForTimeout(10)
}

async function expectBackground(page: Page, fileName: string): Promise<void> {
  const dimensions = await page.locator('.scene-backdrop').evaluate(async (element) => {
    const backgroundImage = getComputedStyle(element).backgroundImage
    const url = backgroundImage.match(/url\(["']?(.*?)["']?\)/)?.[1] || ''
    if (!url) return { backgroundImage, width: 0, height: 0 }
    const image = new Image()
    image.src = url
    await image.decode()
    return { backgroundImage, width: image.naturalWidth, height: image.naturalHeight }
  })
  expect(dimensions.backgroundImage).toContain(fileName)
  expect(dimensions.width).toBeGreaterThan(1000)
  expect(dimensions.height).toBeGreaterThan(700)
}

async function expectLoadedSprite(page: Page, alt: string): Promise<void> {
  const directSprite = page.locator(`.sprite-stage img[alt="${alt}"]`)
  await expect(directSprite).toBeVisible()
  await expect.poll(
    () => directSprite.evaluate((image: HTMLImageElement) => (
      image.complete && image.naturalWidth > 700 && image.naturalHeight > 1200
    )),
    { message: `Sprite ${alt} did not finish decoding at its authored resolution`, timeout: 10_000 },
  ).toBe(true)
}

async function expectReadableDialogue(page: Page): Promise<void> {
  const color = await page.locator('.dialogue-text').evaluate((element) => getComputedStyle(element).color)
  const channels = color.match(/\d+(?:\.\d+)?/g)?.slice(0, 3).map(Number) || []
  expect(channels).toHaveLength(3)
  expect(Math.min(...channels), `dialogue color ${color} is too dark`).toBeGreaterThan(180)
}

async function expectLayoutInsideViewport(page: Page): Promise<void> {
  const layout = await page.evaluate(() => {
    const viewport = { width: window.innerWidth, height: window.innerHeight }
    const appMain = document.querySelector('.app-main')
    const game = document.querySelector('.game-container')
    const stage = document.querySelector('.stage')
    const boxes = ['.game-topbar', '.dialogue-box', '.sprite-stage img'].map((selector) => {
      const element = document.querySelector(selector)
      if (!element) return { selector, box: null }
      const box = element.getBoundingClientRect()
      return { selector, box: { left: box.left, top: box.top, right: box.right, bottom: box.bottom } }
    })
    const metrics = (element: Element | null) => element ? {
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
      scrollTop: element.scrollTop,
      top: element.getBoundingClientRect().top,
      bottom: element.getBoundingClientRect().bottom,
    } : null
    return {
      viewport,
      boxes,
      scrollY: window.scrollY,
      documentHeight: document.documentElement.scrollHeight,
      bodyHeight: document.body.scrollHeight,
      appMain: metrics(appMain),
      game: metrics(game),
      stage: metrics(stage),
    }
  })
  const layoutEvidence = JSON.stringify(layout)
  for (const entry of layout.boxes) {
    expect(entry.box, `${entry.selector} is missing: ${layoutEvidence}`).not.toBeNull()
    expect(entry.box!.left, `${entry.selector} escaped left: ${layoutEvidence}`).toBeGreaterThanOrEqual(-1)
    expect(entry.box!.right, `${entry.selector} escaped right: ${layoutEvidence}`).toBeLessThanOrEqual(layout.viewport.width + 1)
    expect(entry.box!.top, `${entry.selector} escaped top: ${layoutEvidence}`).toBeGreaterThanOrEqual(-1)
    expect(entry.box!.bottom, `${entry.selector} escaped bottom: ${layoutEvidence}`).toBeLessThanOrEqual(layout.viewport.height + 1)
  }
}
