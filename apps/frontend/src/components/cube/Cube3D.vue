<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

const canvasRef = ref<HTMLCanvasElement | null>(null)
const isLoaded = ref(false)


onMounted(async () => {
  if (!canvasRef.value) return

  try {
    // Dynamic import of the WASM module
    const wasm = await import('../../wasm/rouxflow-render/rouxflow_render.js')
    await wasm.default() 

    const canvasId = canvasRef.value.id
    
    try {
        wasm.init_renderer(canvasId)
    } catch (e: any) {
        // winit throws this error on the web to break control flow and start the loop
        if (typeof e === 'string' && e.includes("Using exceptions for control flow")) {
             console.log("[RouxRenderer] Loop started successfully (caught control flow exception)")
        } else if (e instanceof Error && e.message.includes("Using exceptions for control flow")) {
             console.log("[RouxRenderer] Loop started successfully (caught control flow exception)")
        } else {
             console.error("[RouxRenderer] Initialization error:", e)
             throw e
        }
    }
    
    isLoaded.value = true
  } catch (e) {
      console.error("Failed to load 3D engine:", e)
  }
})

onUnmounted(() => {
  // TODO: Implement cleaner shutdown if possible. 
  // currently we rely on the browser checking canvas existence.
})
</script>

<template>
  <div class="cube-container w-full h-full relative">
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
