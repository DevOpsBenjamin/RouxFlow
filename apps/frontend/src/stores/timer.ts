import { defineStore } from 'pinia'
import { computed } from 'vue'
import { cubeManager } from '../services/cube/bridge'

export const useTimerStore = defineStore('timer', () => {
    // Query WASM for timer state
    const time = computed(() => cubeManager?.get_current_time_ms() ?? 0)

    const isRunning = computed(() => cubeManager?.is_timer_running() ?? false)

    const flowState = computed(() => {
        if (!cubeManager) return 'Idle'
        const flowStateJson = cubeManager.get_flow_state()
        if (!flowStateJson) return 'Idle'
        try {
            // FlowState is returned as a JSON string
            return JSON.parse(flowStateJson)
        } catch {
            return 'Idle'
        }
    })

    const currentMoves = computed(() => {
        if (!cubeManager) return []
        const timerStateJson = cubeManager.get_timer_state()
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
