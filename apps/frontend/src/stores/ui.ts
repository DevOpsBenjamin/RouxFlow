import { defineStore } from 'pinia'
import { ref } from 'vue'

export type AppState = 'Landing' | 'Home' | 'Connecting' | 'ActiveSession' | 'Analysis' | 'Profile'

export const useUIStore = defineStore('ui', () => {
    const currentState = ref<AppState>('Landing')
    const selectedSolveId = ref<string | null>(null)

    function setLanding() {
        currentState.value = 'Landing'
    }

    function setHome() {
        currentState.value = 'Home'
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

    function openProfile() {
        currentState.value = 'Profile'
    }

    return {
        currentState,
        selectedSolveId,
        setLanding,
        setHome,
        startConnecting,
        setActiveSession,
        openAnalysis,
        openProfile
    }
})
