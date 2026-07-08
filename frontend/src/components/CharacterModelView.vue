<template>
  <div class="character-model-view" ref="containerRef">
    <div v-if="!modelLoaded" class="model-loading">
      <span class="spinner"></span>
      <span>Loading 3D model...</span>
    </div>
    <div v-if="modelError" class="model-error">
      <span class="error-mark">3D</span>
      <strong>{{ modelError }}</strong>
      <span class="error-hint">Supported formats: .glb, .gltf</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps<{
  modelPath?: string | null
  expression?: string
  motion?: string
}>()

const containerRef = ref<HTMLDivElement>()
const modelLoaded = ref(false)
const modelError = ref<string | null>(null)

let renderer: any = null
let scene: any = null
let camera: any = null
let animationId: number | null = null
let mixer: any = null
let controls: any = null
let activeModel: any = null
let resizeObserver: ResizeObserver | null = null

async function initScene() {
  if (!containerRef.value) return
  const THREE = await import('three')
  const { GLTFLoader } = await import('three/addons/loaders/GLTFLoader.js')
  const { OrbitControls } = await import('three/addons/controls/OrbitControls.js')

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
  renderer.domElement.className = 'character-model-canvas'
  containerRef.value.appendChild(renderer.domElement)
  resizeObserver = new ResizeObserver(handleResize)
  resizeObserver.observe(containerRef.value)

  const ambient = new THREE.AmbientLight(0xffffff, 0.6)
  scene.add(ambient)
  const dirLight = new THREE.DirectionalLight(0xffffff, 0.8)
  dirLight.position.set(2, 3, 4)
  scene.add(dirLight)
  const fillLight = new THREE.DirectionalLight(0x6aa5ff, 0.3)
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
    if (activeModel && !mixer) activeModel.rotation.y += 0.004
    renderer.render(scene, camera)
  }
  animate()

  if (props.modelPath) {
    await loadModel(props.modelPath, THREE, GLTFLoader)
  } else {
    showPlaceholder(THREE)
  }
}

async function loadModel(path: string, THREE: any, GLTFLoader: any) {
  modelError.value = null
  modelLoaded.value = false
  const loader = new GLTFLoader()
  try {
    clearActiveModel()
    const gltf = await loader.loadAsync(path)
    const model = gltf.scene
    model.scale.set(1, 1, 1)
    scene.add(model)
    activeModel = model
    if (gltf.animations.length > 0) {
      mixer = new THREE.AnimationMixer(model)
      const action = mixer.clipAction(gltf.animations[0])
      action.play()
    }
    modelLoaded.value = true
  } catch (e) {
    modelError.value = 'Could not load model'
    showPlaceholder(THREE)
  }
}

function showPlaceholder(THREE: any) {
  clearActiveModel()
  const group = new THREE.Group()
  const geo = new THREE.BoxGeometry(0.8, 1.6, 0.4)
  const mat = new THREE.MeshStandardMaterial({ color: 0x2dd4bf, transparent: true, opacity: 0.7 })
  const cube = new THREE.Mesh(geo, mat)
  cube.position.set(0, 0.8, 0)
  group.add(cube)
  const edges = new THREE.EdgesGeometry(geo)
  const line = new THREE.LineSegments(edges, new THREE.LineBasicMaterial({ color: 0x2dd4bf }))
  line.position.copy(cube.position)
  group.add(line)
  scene.add(group)
  activeModel = group
  modelLoaded.value = true
}

function clearActiveModel() {
  if (!activeModel || !scene) return
  scene.remove(activeModel)
  activeModel.traverse?.((object: any) => {
    object.geometry?.dispose?.()
    if (Array.isArray(object.material)) {
      object.material.forEach((material: any) => material.dispose?.())
    } else {
      object.material?.dispose?.()
    }
  })
  activeModel = null
  mixer = null
}

function handleResize() {
  if (!containerRef.value || !renderer || !camera) return
  const w = Math.max(containerRef.value.clientWidth, 1)
  const h = Math.max(containerRef.value.clientHeight, 1)
  camera.aspect = w / h
  camera.updateProjectionMatrix()
  renderer.setSize(w, h)
}

onMounted(() => {
  initScene()
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  if (animationId) cancelAnimationFrame(animationId)
  clearActiveModel()
  if (renderer) {
    renderer.dispose()
    renderer.domElement?.remove()
  }
})

watch(() => props.modelPath, (newPath) => {
  if (scene) {
    import('three').then(THREE => {
      if (!newPath) {
        modelError.value = null
        showPlaceholder(THREE)
        return
      }
      import('three/addons/loaders/GLTFLoader.js').then(({ GLTFLoader }) => loadModel(newPath, THREE, GLTFLoader))
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
