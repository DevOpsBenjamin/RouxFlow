import { defineStore } from 'pinia'
import { ref } from 'vue'

export type AppState = 'Landing' | 'Connecting' | 'ActiveSession' | 'Analysis'

export const useUIStore = defineStore('ui', () => {
    const currentState = ref<AppState>('Landing')
    const selectedSolveId = ref<string | null>(null)

    function setLanding() {
        currentState.value = 'Landing'
    }

    function startConnecting() {
        currentState.value = 'Connecting'
    }

    function setActiveSession() {
        currentState.value = 'ActiveSession'
    }

    function openAnalysis(solveId: string) {
        selectedSolveId.value = solveId
        currentState.value = 'Analysis'
    }

    return {
        currentState,
        selectedSolveId,
        setLanding,
        startConnecting,
        setActiveSession,
        openAnalysis
    }
})
