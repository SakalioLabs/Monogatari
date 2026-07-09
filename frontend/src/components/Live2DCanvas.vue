<template>
  <div class="live2d-container" ref="containerRef">
    <canvas ref="canvasRef" class="live2d-canvas"></canvas>
    <div v-if="loading" class="loading-overlay">
      <p>Loading model...</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps<{
  modelPath: string | null
  expression?: string
  motion?: string
}>()

const containerRef = ref<HTMLDivElement>()
const canvasRef = ref<HTMLCanvasElement>()
const loading = ref(false)

// Live2D model state
let app: any = null
let model: any = null
let resizeObserver: ResizeObserver | null = null

async function initPixi() {
  if (!canvasRef.value || !containerRef.value) return

  // Dynamically import pixi.js and live2d-display
  try {
    const PIXI = await import('pixi.js')
    const { Live2DModel } = await import('pixi-live2d-display')

    // Register the Live2D ticker
    ;(window as any).PIXI = PIXI

    const rect = containerRef.value.getBoundingClientRect()
    app = new PIXI.Application({
      view: canvasRef.value,
      width: rect.width,
      height: rect.height,
      backgroundAlpha: 0,
      antialias: true,
    })
    resizeObserver = new ResizeObserver(resizeCanvas)
    resizeObserver.observe(containerRef.value)

    if (props.modelPath) {
      await loadModel(props.modelPath)
    }
  } catch (e) {
    console.error('Failed to initialize PixiJS/Live2D:', e)
  }
}

async function loadModel(path: string) {
  if (!app) return
  loading.value = true

  try {
    const { Live2DModel } = await import('pixi-live2d-display')

    // Remove existing model
    if (model) {
      app.stage.removeChild(model)
      model.destroy()
    }

    // Load new model
    model = await Live2DModel.from(path)

    // Scale and position
    const scale = Math.min(
      (app.screen.width * 0.8) / model.width,
      (app.screen.height * 0.8) / model.height
    )
    model.scale.set(scale)
    model.x = (app.screen.width - model.width * scale) / 2
    model.y = app.screen.height - model.height * scale

    // Enable interaction
    model.interactive = true
    model.on('hit', () => {
      // Trigger tap motion
      if (model.internalModel?.motionManager) {
        model.motion('Tap')
      }
    })

    app.stage.addChild(model)
    fitModelToStage()
  } catch (e) {
    console.error('Failed to load Live2D model:', e)
  } finally {
    loading.value = false
  }
}

function fitModelToStage() {
  if (!app || !model) return
  const scale = Math.min(
    (app.screen.width * 0.8) / model.width,
    (app.screen.height * 0.8) / model.height
  )
  model.scale.set(scale)
  model.x = (app.screen.width - model.width * scale) / 2
  model.y = app.screen.height - model.height * scale
}

function resizeCanvas() {
  if (!app || !containerRef.value) return
  const rect = containerRef.value.getBoundingClientRect()
  app.renderer.resize(Math.max(rect.width, 1), Math.max(rect.height, 1))
  fitModelToStage()
}

function setExpression(expression: string) {
  if (model?.internalModel?.expressionManager) {
    model.expression(expression)
  }
}

function setMotion(motion: string) {
  if (model?.internalModel?.motionManager) {
    model.motion(motion)
  }
}

// Watch for prop changes
watch(() => props.modelPath, (newPath) => {
  if (newPath) loadModel(newPath)
})

watch(() => props.expression, (newExpr) => {
  if (newExpr) setExpression(newExpr)
})

watch(() => props.motion, (newMotion) => {
  if (newMotion) setMotion(newMotion)
})

onMounted(() => {
  initPixi()
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  if (model) {
    model.destroy()
  }
  if (app) {
    app.destroy(true)
  }
})
</script>

<style scoped>
.live2d-container {
  width: 100%;
  height: 100%;
  position: relative;
}

.live2d-canvas {
  width: 100%;
  height: 100%;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0,0,0,0.5);
  color: var(--text-muted);
}
</style>
