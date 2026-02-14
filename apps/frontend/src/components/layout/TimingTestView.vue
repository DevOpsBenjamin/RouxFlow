<template>
  <div class="p-4 max-w-2xl mx-auto font-mono text-sm">
    <h1 class="text-xl font-bold mb-4">Timing Test</h1>
    <p class="text-zinc-400 mb-2">Pick up cube, do U, do U', put down. Events are logged with performance.now() timestamps.</p>

    <div class="flex gap-2 mb-4">
      <button
        class="px-3 py-1 rounded bg-blue-600 text-white"
        @click="startTest"
      >
        {{ listening ? 'Reset' : 'Start Listening' }}
      </button>
      <button
        v-if="events.length"
        class="px-3 py-1 rounded bg-zinc-700 text-white"
        @click="events = []; summary = null"
      >
        Clear
      </button>
    </div>

    <div v-if="listening" class="mb-4 text-green-400">
      Listening for events... Pick up the cube!
    </div>

    <!-- Event log -->
    <div class="space-y-1 mb-6">
      <div
        v-for="(e, i) in events"
        :key="i"
        class="flex gap-2"
        :class="{
          'text-yellow-400': e.type === 'pickup',
          'text-blue-400': e.type === 'putdown',
          'text-green-400': e.type === 'move',
        }"
      >
        <span class="w-24 text-right text-zinc-500">{{ e.perfMs.toFixed(1) }}ms</span>
        <span v-if="i > 0" class="w-20 text-right text-zinc-600">+{{ (e.perfMs - events[i-1]!.perfMs).toFixed(1) }}</span>
        <span v-else class="w-20 text-right text-zinc-600">---</span>
        <span class="w-16 font-bold">{{ e.type }}</span>
        <span class="text-zinc-400">{{ e.detail }}</span>
      </div>
    </div>

    <!-- Summary -->
    <div v-if="summary" class="border border-zinc-700 rounded p-3 space-y-2">
      <div class="font-bold text-lg mb-2">Summary</div>

      <div class="text-zinc-500 text-xs mb-2">JS timestamps (performance.now)</div>
      <div>Pickup → First move: <span class="text-yellow-300">{{ summary.pickupToFirstMove?.toFixed(1) ?? '?' }}ms</span></div>
      <div>First move → Last move: <span class="text-green-300">{{ summary.firstToLastMove?.toFixed(1) ?? '?' }}ms</span></div>
      <div>Last move → Putdown (JS event): <span class="text-blue-300">{{ summary.lastMoveToPutdown?.toFixed(1) ?? '?' }}ms</span></div>
      <div>Pickup → Putdown (JS): <span class="text-white font-bold">{{ summary.pickupToPutdown?.toFixed(1) ?? '?' }}ms</span></div>

      <div class="border-t border-zinc-700 pt-2 mt-2 text-zinc-500 text-xs">Cube internal timestamps</div>
      <div v-if="summary.cubeTsFirstMove != null && summary.cubeTsLastMove != null">
        First move → Last move (cube clock): <span class="text-green-300 font-bold">{{ (summary.cubeTsLastMove - summary.cubeTsFirstMove).toFixed(0) }}ms</span>
      </div>

      <div v-if="summary.wasmStableSince != null || summary.wasmCalcTime != null" class="border-t border-zinc-700 pt-2 mt-2">
        <div class="text-zinc-500 text-xs mb-1">WASM internal (stable_since = real putdown moment)</div>
        <div v-if="summary.wasmStableSince != null && summary.wasmPickup != null">
          Pickup (WASM): <span class="text-yellow-300">{{ (summary.wasmPickup * 1000).toFixed(1) }}ms</span>
        </div>
        <div v-if="summary.wasmPutdownTime != null">
          Putdown = stable_since (WASM): <span class="text-blue-300">{{ (summary.wasmPutdownTime * 1000).toFixed(1) }}ms</span>
        </div>
        <div v-if="summary.wasmNow != null">
          Putdown confirmed at (WASM): <span class="text-zinc-400">{{ (summary.wasmNow * 1000).toFixed(1) }}ms</span>
          <span class="text-zinc-600"> (+{{ ((summary.wasmNow - (summary.wasmPutdownTime ?? 0)) * 1000).toFixed(0) }}ms stability wait)</span>
        </div>
        <div v-if="summary.wasmCalcTime != null">
          Calc time (stable_since - pickup): <span class="text-white font-bold">{{ summary.wasmCalcTime.toFixed(0) }}ms</span>
        </div>
        <div v-if="summary.wasmPutdownTime != null && summary.wasmPickup != null && summary.lastMoveWasmTime != null">
          Last move → stable_since: <span class="text-blue-300 font-bold">{{ ((summary.wasmPutdownTime - summary.lastMoveWasmTime) * 1000).toFixed(0) }}ms</span>
          <span class="text-zinc-600"> (real putdown delay, no stability wait)</span>
        </div>
      </div>
    </div>
    <!-- Gyro Rate Stats -->
    <div class="border border-zinc-700 rounded p-3 mt-6">
      <div class="flex items-center gap-3 mb-3">
        <div class="font-bold text-lg">Gyro Packet Rate</div>
        <button
          class="px-3 py-1 rounded text-sm"
          :class="gyroPolling ? 'bg-red-600 text-white' : 'bg-purple-600 text-white'"
          @click="toggleGyroStats"
        >
          {{ gyroPolling ? 'Stop' : 'Start Measuring' }}
        </button>
        <button
          v-if="gyroStats"
          class="px-3 py-1 rounded bg-zinc-700 text-white text-sm"
          @click="resetGyroStats"
        >
          Reset
        </button>
      </div>

      <div v-if="gyroStats" class="space-y-2">
        <div class="flex gap-4">
          <div>Samples: <span class="text-white font-bold">{{ gyroStats.count }}</span></div>
          <div>Rate: <span class="text-purple-300 font-bold">{{ gyroStats.hz.toFixed(1) }} Hz</span></div>
        </div>

        <div class="flex gap-4 text-sm">
          <div>Avg: <span class="text-white">{{ gyroStats.avg_ms.toFixed(1) }}ms</span></div>
          <div>Min: <span class="text-green-300">{{ gyroStats.min_ms.toFixed(1) }}ms</span></div>
          <div>Max: <span class="text-red-300">{{ gyroStats.max_ms.toFixed(1) }}ms</span></div>
          <div>Median: <span class="text-yellow-300">{{ gyroStats.median_ms.toFixed(1) }}ms</span></div>
        </div>

        <div class="flex gap-4 text-sm text-zinc-400">
          <div>P5: {{ gyroStats.p5_ms.toFixed(1) }}ms</div>
          <div>P95: {{ gyroStats.p95_ms.toFixed(1) }}ms</div>
        </div>

        <!-- Histogram -->
        <div v-if="gyroStats.buckets && gyroStats.bucket_labels" class="mt-2">
          <div class="text-zinc-500 text-xs mb-1">Interval distribution (ms)</div>
          <div class="flex items-end gap-1 h-24">
            <div
              v-for="(count, i) in gyroStats.buckets"
              :key="i"
              class="flex flex-col items-center flex-1"
            >
              <div
                class="w-full rounded-t"
                :class="i < 3 ? 'bg-green-500' : i < 6 ? 'bg-yellow-500' : 'bg-red-500'"
                :style="{ height: maxBucket > 0 ? (count / maxBucket * 80) + 'px' : '0px', minHeight: count > 0 ? '2px' : '0px' }"
              ></div>
              <div class="text-[9px] text-zinc-600 mt-1 whitespace-nowrap">{{ gyroStats.bucket_labels[i] }}</div>
              <div v-if="count > 0" class="text-[9px] text-zinc-500">{{ count }}</div>
            </div>
          </div>
        </div>
      </div>

      <div v-else class="text-zinc-500 text-sm">
        Connect a cube with gyro and click "Start Measuring" to analyze packet intervals.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { cm_get_gyro_stats, cm_reset_gyro_stats } from '../../services/cube/bridge'

interface TimingEvent {
  type: 'pickup' | 'putdown' | 'move'
  perfMs: number
  wasmTime?: number  // WASM timestamp in seconds (performance.now/1000)
  detail: string
  cubeTs?: number  // cube internal timestamp (ms) for moves
}

interface Summary {
  pickupToFirstMove: number | null
  firstToLastMove: number | null
  lastMoveToPutdown: number | null
  pickupToPutdown: number | null
  cubeTsFirstMove: number | null
  cubeTsLastMove: number | null
  // WASM internal data
  wasmStableSince: number | null  // seconds
  wasmPutdownTime: number | null  // seconds (last_putdown_time = stable_since)
  wasmPickup: number | null       // seconds
  wasmCalcTime: number | null     // ms
  wasmNow: number | null          // seconds (when putdown was confirmed)
  lastMoveWasmTime: number | null // seconds (JS time of last move / 1000)
}

const events = ref<TimingEvent[]>([])
const listening = ref(false)
const summary = ref<Summary | null>(null)

// Cumulative cube timestamp tracker
let cubeTsAccum = 0

let origLog: typeof console.log | null = null
let origDebug: typeof console.debug | null = null
let origInfo: typeof console.info | null = null

// Parsed WASM timing from putdown debug line
let wasmPutdownData: {
  stableSince: number
  putdownTime: number
  pickup: number
  calcTime: number
  now: number
} | null = null

function startTest() {
  events.value = []
  summary.value = null
  cubeTsAccum = 0
  wasmPutdownData = null
  listening.value = true

  // Hook console.log/debug/info to capture WASM logs
  // (console_log crate maps log::debug! to console.log)
  if (!origLog) {
    origLog = console.log.bind(console)
    origDebug = console.debug.bind(console)
    origInfo = console.info.bind(console)

    const hook = (orig: Function) => (...args: any[]) => {
      orig(...args)
      if (!listening.value) return
      const msg = args.map((a: any) => typeof a === 'string' ? a : JSON.stringify(a)).join(' ')
      processLogLine(msg)
    }

    console.log = hook(origLog) as typeof console.log
    console.debug = hook(origDebug) as typeof console.debug
    console.info = hook(origInfo) as typeof console.info
  }
}

function processLogLine(msg: string) {
  const now = performance.now()

  // Pickup — also capture the WASM timestamp from the log if present
  if (msg.includes('[gyro] pickup detected')) {
    events.value.push({ type: 'pickup', perfMs: now, detail: 'gyro pickup' })
  }

  // Putdown — parse WASM debug data
  if (msg.includes('[gyro] putdown detected')) {
    // Parse: stable_since=X.XXXs last_putdown=X.XXXs pickup=X.XXXs solve_pickup=... calc_time=XXXms now=X.XXXs
    const ssMatch = msg.match(/stable_since=([\d.]+)s/)
    const pdMatch = msg.match(/last_putdown=([\d.]+)s/)
    const pkMatch = msg.match(/pickup=([\d.]+)s/)
    const ctMatch = msg.match(/calc_time=([\d.]+)ms/)
    const nowMatch = msg.match(/now=([\d.]+)s/)

    if (pdMatch && pkMatch) {
      wasmPutdownData = {
        stableSince: ssMatch ? parseFloat(ssMatch[1]!) : 0,
        putdownTime: parseFloat(pdMatch[1]!),
        pickup: parseFloat(pkMatch[1]!),
        calcTime: ctMatch ? parseFloat(ctMatch[1]!) : 0,
        now: nowMatch ? parseFloat(nowMatch[1]!) : 0,
      }
    }

    events.value.push({
      type: 'putdown',
      perfMs: now,
      detail: wasmPutdownData
        ? `stable_since=${(wasmPutdownData.putdownTime * 1000).toFixed(0)}ms pickup=${(wasmPutdownData.pickup * 1000).toFixed(0)}ms calc=${wasmPutdownData.calcTime.toFixed(0)}ms`
        : 'gyro putdown'
    })
    computeSummary()
  }

  // Move from MoYu V3 emit line — has cube internal timestamp
  const moveMatch = msg.match(/\[MoYu V3 MOVE\] emit: slot=\d+ code=\d+ → (\S+) (CW|CCW) \(ts=(\d+)ms\)/)
  if (moveMatch) {
    const cubeTs = parseInt(moveMatch[3]!)
    cubeTsAccum += cubeTs
    events.value.push({
      type: 'move',
      perfMs: now,
      wasmTime: now / 1000,
      detail: `${moveMatch[1]} ${moveMatch[2]} (cube_ts=${cubeTs}ms, accum=${cubeTsAccum}ms)`,
      cubeTs: cubeTsAccum
    })
  }

  // Move from interpreter [move] line
  const interpMatch = msg.match(/\[move\] (.+)/)
  if (interpMatch && !msg.includes('MoYu')) {
    const last = events.value[events.value.length - 1]
    if (last && last.type === 'move') {
      last.detail += ` → ${interpMatch[1]}`
    }
  }
}

function computeSummary() {
  const pickup = events.value.find(e => e.type === 'pickup')
  const moves = events.value.filter(e => e.type === 'move')
  const putdown = events.value.filter(e => e.type === 'putdown').pop()

  if (!pickup || moves.length === 0 || !putdown) {
    summary.value = null
    return
  }

  const firstMove = moves[0]!
  const lastMove = moves[moves.length - 1]!

  summary.value = {
    pickupToFirstMove: firstMove.perfMs - pickup.perfMs,
    firstToLastMove: lastMove.perfMs - firstMove.perfMs,
    lastMoveToPutdown: putdown.perfMs - lastMove.perfMs,
    pickupToPutdown: putdown.perfMs - pickup.perfMs,
    cubeTsFirstMove: firstMove.cubeTs ?? null,
    cubeTsLastMove: lastMove.cubeTs ?? null,
    // WASM data from debug line
    wasmStableSince: wasmPutdownData?.stableSince ?? null,
    wasmPutdownTime: wasmPutdownData?.putdownTime ?? null,
    wasmPickup: wasmPutdownData?.pickup ?? null,
    wasmCalcTime: wasmPutdownData?.calcTime ?? null,
    wasmNow: wasmPutdownData?.now ?? null,
    lastMoveWasmTime: lastMove.wasmTime ?? null,
  }
}

// ========== Gyro Rate Stats ==========

interface GyroStats {
  count: number
  hz: number
  avg_ms: number
  min_ms: number
  max_ms: number
  median_ms: number
  p5_ms: number
  p95_ms: number
  buckets: number[]
  bucket_labels: string[]
}

const gyroStats = ref<GyroStats | null>(null)
const gyroPolling = ref(false)
let gyroPollTimer: ReturnType<typeof setInterval> | null = null

const maxBucket = computed(() => {
  if (!gyroStats.value?.buckets) return 0
  return Math.max(...gyroStats.value.buckets)
})

function toggleGyroStats() {
  if (gyroPolling.value) {
    stopGyroStats()
  } else {
    startGyroStats()
  }
}

function startGyroStats() {
  cm_reset_gyro_stats()
  gyroStats.value = null
  gyroPolling.value = true
  gyroPollTimer = setInterval(() => {
    const json = cm_get_gyro_stats()
    if (json && json !== '{}') {
      try { gyroStats.value = JSON.parse(json) } catch {}
    }
  }, 500)
}

function stopGyroStats() {
  gyroPolling.value = false
  if (gyroPollTimer) {
    clearInterval(gyroPollTimer)
    gyroPollTimer = null
  }
}

function resetGyroStats() {
  cm_reset_gyro_stats()
  gyroStats.value = null
}

onUnmounted(() => {
  listening.value = false
  stopGyroStats()
  if (origLog) {
    console.log = origLog
    origLog = null
  }
  if (origDebug) {
    console.debug = origDebug
    origDebug = null
  }
  if (origInfo) {
    console.info = origInfo
    origInfo = null
  }
})
</script>
