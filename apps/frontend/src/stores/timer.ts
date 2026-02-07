import { defineStore } from 'pinia'
import { computed } from 'vue'
import { cm_get_current_time_ms, cm_is_timer_running, cm_get_flow_state, cm_get_timer_state } from '../services/cube/bridge'

export const useTimerStore = defineStore('timer', () => {
    // Query WASM for timer state (free functions, safe to call before init — returns defaults)
    const time = computed(() => cm_get_current_time_ms())

    const isRunning = computed(() => cm_is_timer_running())

    const flowState = computed(() => {
        const flowStateJson = cm_get_flow_state()
        if (!flowStateJson) return 'Idle'
        try {
            return JSON.parse(flowStateJson)
        } catch {
            return 'Idle'
        }
    })

    const currentMoves = computed(() => {
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

    return {
        time,
        isRunning,
        flowState,
        currentMoves,
        formattedTime,
    }
})
