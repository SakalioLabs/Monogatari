<template>
  <div class="model-viewer" ref="container">
    <div v-if="!loaded" class="model-placeholder">
      <span class="empty-mark">3D</span>
      <span>{{ modelPath ? "Loading 3D model..." : "No 3D model configured" }}</span>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue"
const props = defineProps<{ modelPath?: string; emotion?: string }>()
const container = ref<HTMLElement>()
const loaded = ref(false)
let scene: any = null
let camera: any = null
let renderer: any = null
let animId: number | null = null
async function initThree() {
  if (!container.value || !props.modelPath) return
  try {
    const THREE = await import("three")
    scene = new THREE.Scene()
    camera = new THREE.PerspectiveCamera(50, container.value.clientWidth / container.value.clientHeight, 0.1, 1000)
    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
    renderer.setSize(container.value.clientWidth, container.value.clientHeight)
    renderer.setPixelRatio(window.devicePixelRatio)
    container.value.appendChild(renderer.domElement)
    scene.add(new THREE.DirectionalLight(0xffffff, 1))
    scene.add(new THREE.AmbientLight(0x404040, 0.6))
    camera.position.z = 3
    const cube = new THREE.Mesh(
      new THREE.BoxGeometry(1, 1.5, 0.5),
      new THREE.MeshStandardMaterial({ color: 0x2dd4bf })
    )
    scene.add(cube)
    loaded.value = true
    function animate() {
      animId = requestAnimationFrame(animate)
      cube.rotation.y += 0.005
      renderer.render(scene, camera)
    }
    animate()
  } catch { loaded.value = false }
}
onMounted(() => { if (props.modelPath) initThree() })
watch(() => props.modelPath, () => { if (props.modelPath && !loaded.value) initThree() })
onBeforeUnmount(() => { if (animId) cancelAnimationFrame(animId); if (renderer) renderer.dispose() })
</script>
<style scoped>
.model-viewer { width: 100%; height: 100%; min-height: 200px; position: relative; background: var(--surface-1); border-radius: var(--radius); overflow: hidden; }
.model-placeholder { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; gap: 8px; }
.empty-mark { font-size: 36px; font-weight: 900; color: var(--text-tertiary); }
</style>