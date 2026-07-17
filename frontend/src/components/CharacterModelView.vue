<template>
  <div
    class="character-model-view"
    :class="`presentation-${presentation}`"
    ref="containerRef"
    :data-presentation="presentation"
    :data-model-path="props.modelPath || ''"
    :data-model-state="modelState"
    :data-model-animations="animationClipCount"
    :data-framing-excluded-objects="framingExcludedObjects"
    :data-canvas-signature="canvasSignature"
    :data-canvas-unique-colors="canvasUniqueColors"
    :data-canvas-luminance-range="canvasLuminanceRange"
    :data-canvas-non-background="canvasNonBackground"
    :data-canvas-bounds="canvasContentBounds"
    :data-canvas-motion="canvasMotionDetected"
    :data-canvas-preview="canvasPreviewDataUrl || undefined"
  >
    <div v-if="!modelLoaded && !modelError" class="model-loading" role="status">
      <span class="spinner"></span>
      <span>{{ t('renderer.model3d.loading', 'Loading 3D model...') }}</span>
    </div>
    <div v-else-if="modelError" class="model-error">
      <span class="error-mark">3D</span>
      <strong>{{ modelErrorDisplay }}</strong>
      <span class="error-hint">{{ t('renderer.model3d.hint', 'Supported formats: .glb, .gltf') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from '../lib/i18n'
import {
  calculateModelCameraDistance,
  selectPrimaryModelBounds,
  type ModelBounds3,
  type ModelPresentation,
} from '../lib/modelFraming'

const { t } = useI18n()

const props = defineProps<{
  modelPath?: string | null
  expression?: string
  motion?: string
  presentation?: ModelPresentation
}>()

const emit = defineEmits<{
  (e: 'load-error', payload: { path: string | null; message: string }): void
}>()

const containerRef = ref<HTMLDivElement>()
const modelLoaded = ref(false)
const modelError = ref<string | null>(null)
const animationClipCount = ref(0)
const framingExcludedObjects = ref(0)
const canvasSignature = ref('')
const canvasUniqueColors = ref(0)
const canvasLuminanceRange = ref('')
const canvasNonBackground = ref(0)
const canvasContentBounds = ref('')
const canvasMotionDetected = ref(false)
const canvasPreviewDataUrl = ref('')
const modelErrorDisplay = computed(() => t('renderer.model3d.load-failed', 'Could not load the 3D model'))
const presentation = computed(() => props.presentation || 'character')
const modelState = computed(() => {
  if (modelError.value) return 'error'
  if (!modelLoaded.value) return 'loading'
  return props.modelPath ? 'ready' : 'placeholder'
})

let renderer: any = null
let scene: any = null
let camera: any = null
let threeRuntime: any = null
let animationId: number | null = null
let mixer: any = null
let controls: any = null
let activeModel: any = null
let resizeObserver: ResizeObserver | null = null
let canvasProbeFrame = 0
let canvasProbeSamples = 0
let initialCanvasSignature = ''
let loadRequestSequence = 0
let disposed = false

async function initScene() {
  if (!containerRef.value) return
  const THREE = await import('three')
  const { GLTFLoader } = await import('three/addons/loaders/GLTFLoader.js')
  const { OrbitControls } = await import('three/addons/controls/OrbitControls.js')
  if (disposed || !containerRef.value) return
  threeRuntime = THREE

  scene = new THREE.Scene()
  scene.background = new THREE.Color(0x0f1115)

  const w = containerRef.value.clientWidth || 600
  const h = containerRef.value.clientHeight || 400
  camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 1000)
  camera.position.set(0, 1.2, 3)

  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
  renderer.setSize(w, h)
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.toneMapping = THREE.ACESFilmicToneMapping
  renderer.toneMappingExposure = 1.2
  renderer.outputColorSpace = THREE.SRGBColorSpace
  renderer.domElement.className = 'character-model-canvas'
  containerRef.value.appendChild(renderer.domElement)
  resizeObserver = new ResizeObserver(handleResize)
  resizeObserver.observe(containerRef.value)

  const ambient = new THREE.AmbientLight(0xffffff, 0.6)
  scene.add(ambient)
  const dirLight = new THREE.DirectionalLight(0xffffff, 0.8)
  dirLight.position.set(2, 3, 4)
  scene.add(dirLight)
  const fillLight = new THREE.DirectionalLight(0xd6d7d3, 0.3)
  fillLight.position.set(-2, 1, -2)
  scene.add(fillLight)

  controls = new OrbitControls(camera, renderer.domElement)
  controls.enableDamping = true
  controls.dampingFactor = 0.08
  controls.target.set(0, 1, 0)
  controls.minDistance = 1.5
  controls.maxDistance = 8

  function animate() {
    animationId = requestAnimationFrame(animate)
    controls.update()
    if (mixer) mixer.update(0.016)
    if (activeModel && !mixer && presentation.value === 'character') activeModel.rotation.y += 0.004
    renderer.render(scene, camera)
    if (shouldProbeCanvas()) {
      canvasProbeFrame += 1
      if (canvasProbeFrame === 1 || canvasProbeFrame % 20 === 0) probeCanvasPixels()
    }
  }
  animate()

  if (props.modelPath) {
    await loadModel(props.modelPath, THREE, GLTFLoader)
  } else {
    showPlaceholder(THREE)
  }
}

async function loadModel(path: string, THREE: any, GLTFLoader: any) {
  const requestId = ++loadRequestSequence
  modelError.value = null
  modelLoaded.value = false
  animationClipCount.value = 0
  framingExcludedObjects.value = 0
  resetCanvasProbe()
  const loader = new GLTFLoader()
  try {
    clearActiveModel()
    const gltf = await loader.loadAsync(path)
    if (disposed || requestId !== loadRequestSequence || !scene) {
      disposeObject(gltf.scene)
      return
    }
    const model = gltf.scene
    scene.add(model)
    activeModel = model
    normalizeAndFrameModel(model, THREE)
    animationClipCount.value = gltf.animations.length
    if (gltf.animations.length > 0) {
      mixer = new THREE.AnimationMixer(model)
      const action = mixer.clipAction(gltf.animations[0])
      action.play()
    }
    modelLoaded.value = true
  } catch (e) {
    if (disposed || requestId !== loadRequestSequence) return
    const message = 'Could not load 3D model'
    modelError.value = message
    emit('load-error', { path, message })
    showPlaceholder(THREE)
  }
}

function showPlaceholder(THREE: any) {
  loadRequestSequence += 1
  clearActiveModel()
  const group = new THREE.Group()
  const geo = new THREE.BoxGeometry(0.8, 1.6, 0.4)
  const mat = new THREE.MeshStandardMaterial({ color: 0x8b8d8a, transparent: true, opacity: 0.72 })
  const cube = new THREE.Mesh(geo, mat)
  cube.position.set(0, 0.8, 0)
  group.add(cube)
  const edges = new THREE.EdgesGeometry(geo)
  const line = new THREE.LineSegments(edges, new THREE.LineBasicMaterial({ color: 0xd0d2ce }))
  line.position.copy(cube.position)
  group.add(line)
  scene.add(group)
  activeModel = group
  normalizeAndFrameModel(group, THREE)
  modelLoaded.value = true
}

function normalizeAndFrameModel(model: any, THREE: any) {
  model.updateMatrixWorld(true)
  const sourceBox = modelFrameBounds(model, THREE)
  const sourceSize = sourceBox.getSize(new THREE.Vector3())
  const sourceExtent = Math.max(sourceSize.x, sourceSize.y, sourceSize.z)
  if (!Number.isFinite(sourceExtent) || sourceExtent <= 0.0001) {
    throw new Error('3D model has no visible bounds')
  }

  const targetExtent = presentation.value === 'scene' ? 2.8 : 1.8
  model.scale.multiplyScalar(targetExtent / sourceExtent)
  model.updateMatrixWorld(true)

  const scaledBox = modelFrameBounds(model, THREE)
  const center = scaledBox.getCenter(new THREE.Vector3())
  model.position.x -= center.x
  model.position.y -= presentation.value === 'scene' ? center.y : scaledBox.min.y
  model.position.z -= center.z
  model.updateMatrixWorld(true)
  frameModel(model, THREE)
}

function frameModel(model: any, THREE: any) {
  if (!camera || !controls || !containerRef.value) return
  const bounds = modelFrameBounds(model, THREE)
  const size = bounds.getSize(new THREE.Vector3())
  const targetY = bounds.min.y + size.y * 0.5
  const verticalFov = THREE.MathUtils.degToRad(camera.fov)
  const distance = calculateModelCameraDistance(
    [size.x, size.y, size.z],
    verticalFov,
    camera.aspect,
    presentation.value,
  )

  controls.target.set(0, targetY, 0)
  camera.position.set(0, targetY, distance)
  camera.near = Math.max(distance / 100, 0.01)
  camera.far = Math.max(distance * 20, 100)
  camera.updateProjectionMatrix()
  controls.minDistance = Math.max(distance * 0.45, 0.5)
  controls.maxDistance = Math.max(distance * 4, 8)
  controls.update()
}

function modelFrameBounds(model: any, THREE: any) {
  if (presentation.value !== 'scene') {
    framingExcludedObjects.value = 0
    return new THREE.Box3().setFromObject(model)
  }

  const meshBounds: ModelBounds3[] = []
  model.traverse((object: any) => {
    if (!object?.isMesh || object.visible === false) return
    const bounds = new THREE.Box3().setFromObject(object)
    if (bounds.isEmpty()) return
    meshBounds.push({
      min: [bounds.min.x, bounds.min.y, bounds.min.z],
      max: [bounds.max.x, bounds.max.y, bounds.max.z],
    })
  })
  const selection = selectPrimaryModelBounds(meshBounds)
  if (!selection) {
    framingExcludedObjects.value = 0
    return new THREE.Box3().setFromObject(model)
  }

  framingExcludedObjects.value = selection.excludedIndices.length
  return new THREE.Box3(
    new THREE.Vector3(...selection.bounds.min),
    new THREE.Vector3(...selection.bounds.max),
  )
}

function shouldProbeCanvas(): boolean {
  return Boolean(
    renderer
    && modelLoaded.value
    && props.modelPath
    && !modelError.value
    && rendererProbeEnabled()
    && canvasProbeSamples < 6
    && !canvasMotionDetected.value,
  )
}

function probeCanvasPixels() {
  const gl = renderer?.getContext?.()
  const width = gl?.drawingBufferWidth || 0
  const height = gl?.drawingBufferHeight || 0
  if (!gl || width < 2 || height < 2) return

  const pixels = new Uint8Array(width * height * 4)
  gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels)

  const columns = 24
  const rows = 24
  const colors = new Set<string>()
  let hash = 2166136261
  let minLuminance = 255
  let maxLuminance = 0
  let nonBackground = 0
  let minX = columns
  let minY = rows
  let maxX = -1
  let maxY = -1
  const background = [pixels[0], pixels[1], pixels[2]]

  for (let row = 0; row < rows; row += 1) {
    for (let column = 0; column < columns; column += 1) {
      const x = Math.min(width - 1, Math.floor((column + 0.5) * width / columns))
      const y = Math.min(height - 1, Math.floor((row + 0.5) * height / rows))
      const offset = (y * width + x) * 4
      const red = pixels[offset]
      const green = pixels[offset + 1]
      const blue = pixels[offset + 2]
      const alpha = pixels[offset + 3]
      colors.add(`${red},${green},${blue},${alpha}`)
      for (const channel of [red, green, blue, alpha]) {
        hash ^= channel
        hash = Math.imul(hash, 16777619)
      }
      const luminance = (red + green + blue) / 3
      minLuminance = Math.min(minLuminance, luminance)
      maxLuminance = Math.max(maxLuminance, luminance)
      const distance = Math.abs(red - background[0]) + Math.abs(green - background[1]) + Math.abs(blue - background[2])
      if (distance > 24) {
        nonBackground += 1
        minX = Math.min(minX, column)
        minY = Math.min(minY, row)
        maxX = Math.max(maxX, column)
        maxY = Math.max(maxY, row)
      }
    }
  }

  const signature = (hash >>> 0).toString(16).padStart(8, '0')
  canvasSignature.value = signature
  canvasUniqueColors.value = colors.size
  canvasLuminanceRange.value = `${Math.round(minLuminance)}-${Math.round(maxLuminance)}`
  canvasNonBackground.value = nonBackground
  canvasContentBounds.value = nonBackground > 0 ? `${minX},${minY},${maxX},${maxY}` : ''
  canvasProbeSamples += 1

  if (!initialCanvasSignature) initialCanvasSignature = signature
  else if (signature !== initialCanvasSignature) canvasMotionDetected.value = true

  publishCanvasProbe(width, height)
}

function publishCanvasProbe(width: number, height: number) {
  if (!rendererProbeEnabled()) return
  canvasPreviewDataUrl.value = renderer.domElement.toDataURL('image/png')
  ;(window as Window & { __MONOGATARI_3D_PROBE__?: Record<string, unknown> }).__MONOGATARI_3D_PROBE__ = {
    state: modelState.value,
    animations: animationClipCount.value,
    signature: canvasSignature.value,
    initialSignature: initialCanvasSignature,
    motion: canvasMotionDetected.value,
    uniqueColors: canvasUniqueColors.value,
    luminanceRange: canvasLuminanceRange.value,
    nonBackground: canvasNonBackground.value,
    contentBounds: canvasContentBounds.value,
    samples: canvasProbeSamples,
    width,
    height,
    dataUrl: canvasPreviewDataUrl.value,
  }
}

function rendererProbeEnabled(): boolean {
  return typeof window !== 'undefined'
    && new URLSearchParams(window.location.search).get('rendererProbe') === '1'
}

function resetCanvasProbe() {
  canvasSignature.value = ''
  canvasUniqueColors.value = 0
  canvasLuminanceRange.value = ''
  canvasNonBackground.value = 0
  canvasContentBounds.value = ''
  canvasMotionDetected.value = false
  canvasPreviewDataUrl.value = ''
  canvasProbeFrame = 0
  canvasProbeSamples = 0
  initialCanvasSignature = ''
  if (typeof window !== 'undefined') {
    delete (window as Window & { __MONOGATARI_3D_PROBE__?: Record<string, unknown> }).__MONOGATARI_3D_PROBE__
  }
}

function clearActiveModel() {
  if (!activeModel || !scene) return
  mixer?.stopAllAction?.()
  scene.remove(activeModel)
  disposeObject(activeModel)
  activeModel = null
  mixer = null
  animationClipCount.value = 0
}

function disposeObject(objectRoot: any) {
  objectRoot?.traverse?.((object: any) => {
    object.geometry?.dispose?.()
    if (Array.isArray(object.material)) {
      object.material.forEach(disposeMaterial)
    } else {
      disposeMaterial(object.material)
    }
  })
}

function disposeMaterial(material: any) {
  if (!material) return
  Object.values(material).forEach((value: any) => {
    if (value?.isTexture) value.dispose?.()
  })
  material.dispose?.()
}

function handleResize() {
  if (!containerRef.value || !renderer || !camera) return
  const w = Math.max(containerRef.value.clientWidth, 1)
  const h = Math.max(containerRef.value.clientHeight, 1)
  camera.aspect = w / h
  camera.updateProjectionMatrix()
  renderer.setSize(w, h)
  if (activeModel && threeRuntime) {
    frameModel(activeModel, threeRuntime)
    if (modelLoaded.value && props.modelPath && !modelError.value) resetCanvasProbe()
  }
}

onMounted(() => {
  disposed = false
  initScene()
})

onUnmounted(() => {
  disposed = true
  loadRequestSequence += 1
  resizeObserver?.disconnect()
  if (animationId) cancelAnimationFrame(animationId)
  clearActiveModel()
  if (renderer) {
    renderer.dispose()
    renderer.domElement?.remove()
  }
  threeRuntime = null
  resetCanvasProbe()
})

watch(() => props.modelPath, (newPath) => {
  if (scene && !disposed) {
    import('three').then(THREE => {
      if (disposed || !scene) return
      if (!newPath) {
        modelError.value = null
        showPlaceholder(THREE)
        return
      }
      import('three/addons/loaders/GLTFLoader.js').then(({ GLTFLoader }) => {
        if (!disposed && scene) loadModel(newPath, THREE, GLTFLoader)
      })
    })
  }
})
</script>

<style scoped>
.character-model-view {
  position: relative; width: 100%; height: 100%;
  min-height: 300px; background: var(--surface-0);
  border-radius: var(--radius); overflow: hidden;
}

.character-model-view.presentation-scene {
  min-height: 0;
  border-radius: 0;
}

:deep(.character-model-canvas) {
  width: 100%;
  height: 100%;
  display: block;
}
.model-loading, .model-error {
  position: absolute; inset: 0;
  display: flex; flex-direction: column;
  align-items: center; justify-content: center; gap: 12px;
  color: var(--text-tertiary); font-size: 13px;
}
.error-mark {
  display: inline-flex; align-items: center; justify-content: center;
  width: 48px; height: 48px; border: 1px solid var(--border);
  border-radius: var(--radius); background: var(--surface-2);
  color: var(--brand-light); font-family: var(--font-mono); font-weight: 900;
}
.error-hint { color: var(--text-tertiary); font-size: 11px; }
.spinner {
  display: inline-block; width: 20px; height: 20px;
  border: 2px solid rgba(255,255,255,0.2); border-top-color: var(--brand);
  border-radius: 50%; animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
