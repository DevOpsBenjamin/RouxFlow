<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { ensureWasm, init_renderer, update_render_state, cubeManager } from '../../services/cube/bridge'
import { logger } from '../../utils/logger'

const canvasRef = ref<HTMLCanvasElement | null>(null)
const isLoaded = ref(false)
let rafHandle: number | null = null

function renderLoop() {
  if (!cubeManager) {
    rafHandle = requestAnimationFrame(renderLoop)
    return
  }

  // Query WASM for latest state
  const facelets = cubeManager.get_facelets()
  const [x, y, z, w] = cubeManager.get_orientation()

  // Update WASM render state
  update_render_state(facelets, x, y, z, w)

  // Timer update (WASM calculates time)
  if (cubeManager.is_timer_running()) {
    const timestamp = performance.now() / 1000.0
    cubeManager.update_timer(timestamp)
  }

  rafHandle = requestAnimationFrame(renderLoop)
}

onMounted(async () => {
  if (!canvasRef.value) return

  // IMPORTANT: Set initial canvas dimensions BEFORE winit takes control.
  // On the web, the canvas element often starts with default 300x150 attributes
  // even if CSS sizes it differently. Winit reads the canvas attributes at init,
  // so we must sync them with the actual displayed size ONCE before init.
  // After winit starts, it will manage resizing on its own.
  const rect = canvasRef.value.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1
  canvasRef.value.width = Math.round(rect.width * dpr)
  canvasRef.value.height = Math.round(rect.height * dpr)

  logger.debug(`Initial canvas size: ${canvasRef.value.width}x${canvasRef.value.height} (CSS: ${rect.width}x${rect.height}, DPR: ${dpr})`)

  try {
    await ensureWasm()
    const canvasId = canvasRef.value.id

    try {
        init_renderer(canvasId)
    } catch (e: any) {
        // winit throws this error on the web to break control flow and start the loop
        if (typeof e === 'string' && e.includes("Using exceptions for control flow")) {
             logger.info("Loop started successfully (caught control flow exception)")
        } else if (e instanceof Error && e.message.includes("Using exceptions for control flow")) {
             logger.info("Loop started successfully (caught control flow exception)")
        } else {
             logger.error("Initialization error:", e)
             throw e
        }
    }

    isLoaded.value = true

    // Start render loop - THE ONLY RAF LOOP IN THE APP
    rafHandle = requestAnimationFrame(renderLoop)
  } catch (e) {
      logger.error("Failed to load 3D engine:", e)
  }
})

onUnmounted(() => {
  if (rafHandle !== null) {
    cancelAnimationFrame(rafHandle)
    rafHandle = null
  }
})
</script>

<template>
  <div class="cube-container relative">
    <canvas
      ref="canvasRef"
      id="roux-render-canvas"
      class="w-full h-full block touch-none"
      oncontextmenu="return false;"
    ></canvas>

    <!-- Loading overlay if needed -->
    <div v-if="!isLoaded" class="absolute inset-0 flex items-center justify-center bg-black/50 text-white transform transition-opacity duration-500">
      Loading 3D Engine...
    </div>
  </div>
</template>

<style scoped>
.cube-container {
  /* Minimal styles, detailed sizing handled by parent */
  min-height: 300px;
}
</style>
