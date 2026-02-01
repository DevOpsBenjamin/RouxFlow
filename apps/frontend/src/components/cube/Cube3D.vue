<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

const canvasRef = ref<HTMLCanvasElement | null>(null)
let renderer: any = null

onMounted(async () => {
  if (!canvasRef.value) return

  try {
    // Dynamic import of the WASM module
    const wasm = await import('../../wasm/rouxflow-render/rouxflow_render.js')
    await wasm.default() // Initialize WASM memory if needed (standard for wasm-pack)

    // Initialize the renderer on our canvas
    // We use a unique ID for the canvas to be safe
    const canvasId = canvasRef.value.id
    renderer = new wasm.RouxRenderer(canvasId)
    
    console.log('RouxRenderer initialized via WASM')
  } catch (e) {
    console.error('Failed to initialize 3D renderer:', e)
  }
})

onUnmounted(() => {
  if (renderer) {
    // Ideally we would free memory here, but WASM struct is dummy for now
    // renderer.free() // wasm-bindgen generates this automatically
    renderer = null
  }
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
    <div v-if="!renderer" class="absolute inset-0 flex items-center justify-center bg-black/50 text-white">
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
