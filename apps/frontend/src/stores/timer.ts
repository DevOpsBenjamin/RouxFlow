import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import {
    cm_get_current_time_ms,
    cm_is_timer_running,
    cm_get_flow_state,
    cm_get_timer_state,
    cm_is_cube_solved,
    cm_is_wca_session_full,
    cm_get_scramble_state,
    cm_get_inspection_remaining,
    cm_get_pending_scramble,
    cm_reset_flow,
    cm_generate_new_scramble,
    cm_is_cube_stable,
    updateTimer,
    onWasmStateChanged,
} from '../services/cube/bridge'

export const useTimerStore = defineStore('timer', () => {
    // Reactive tick — bumped after WASM state changes so computed() re-evaluates
    const _wasmTick = ref(0)
    function bumpWasm() { _wasmTick.value++ }

    // Register callback so WASM state changes trigger Vue reactivity
    onWasmStateChanged(bumpWasm)

    // Query WASM for timer state
    const time = computed(() => {
        _wasmTick.value
        return cm_get_current_time_ms()
    })

    const isRunning = computed(() => {
        _wasmTick.value
        return cm_is_timer_running()
    })

    const flowState = computed(() => {
        _wasmTick.value
        const flowStateJson = cm_get_flow_state()
        if (!flowStateJson) return 'Idle'
        try {
            return JSON.parse(flowStateJson)
        } catch {
            return 'Idle'
        }
    })

    const currentMoves = computed(() => {
        _wasmTick.value
        const timerStateJson = cm_get_timer_state()
        if (!timerStateJson) return []
        try {
            const state = JSON.parse(timerStateJson)
            return state.moves || []
        } catch {
            return []
        }
    })

    const formattedTime = computed(() => {
        const timeNum = typeof time.value === 'bigint' ? Number(time.value) : time.value
        const seconds = (timeNum / 1000).toFixed(2)
        return seconds
    })

    const isCubeSolved = computed(() => {
        _wasmTick.value
        return cm_is_cube_solved()
    })

    const inspectionRemaining = computed(() => {
        _wasmTick.value
        return cm_get_inspection_remaining(performance.now() / 1000.0)
    })

    const scrambleState = computed(() => {
        _wasmTick.value
        try {
            return JSON.parse(cm_get_scramble_state(performance.now() / 1000.0))
        } catch {
            return { scramble: [], index: 0, total: 0, is_ready: false, is_invalid: false, expected_move: null, correction_move: null }
        }
    })

    const isWcaFull = computed(() => {
        _wasmTick.value
        return cm_is_wca_session_full()
    })

    const pendingScramble = computed(() => {
        _wasmTick.value
        return cm_get_pending_scramble()
    })

    const isCubeStable = computed(() => {
        _wasmTick.value
        return cm_is_cube_stable()
    })

    // ========== Actions ==========

    function reset() {
        cm_reset_flow()
        bumpWasm()
    }

    function generateScramble() {
        const scramble = cm_generate_new_scramble()
        bumpWasm()
        return scramble
    }

    // ========== Animation Loop ==========

    let rafId: number | null = null

    function startTick() {
        if (rafId) return
        const tick = () => {
            const now = performance.now() / 1000.0
            updateTimer(now)
            bumpWasm()
            rafId = requestAnimationFrame(tick)
        }
        rafId = requestAnimationFrame(tick)
    }

    function stopTick() {
        if (rafId) {
            cancelAnimationFrame(rafId)
            rafId = null
        }
    }

    // Watch flowState to start/stop animation loop
    watch(flowState, (state) => {
        if (state === 'Scrambling' || state === 'Inspection' || state === 'Solving') {
            startTick()
        } else {
            stopTick()
        }
    })

    // Auto-generate scramble when cube is solved and flow is Idle.
    // immediate: true fires on initial load (WASM is already initialized by this point).
    watch([flowState, isCubeSolved], ([flow, solved]) => {
        if (flow === 'Idle' && solved) {
            generateScramble()
        }
    }, { immediate: true })

    return {
        time,
        isRunning,
        flowState,
        currentMoves,
        formattedTime,
        isCubeSolved,
        isWcaFull,
        inspectionRemaining,
        scrambleState,
        pendingScramble,
        isCubeStable,
        reset,
        generateScramble,
    }
})
