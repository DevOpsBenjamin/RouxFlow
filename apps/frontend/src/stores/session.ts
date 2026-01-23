import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { sessionManager, ensureWasm } from '../services/cube/bridge'

export type SessionType = 'Free' | 'WCA'

export const useSessionStore = defineStore('session', () => {
    const rawSessionsJson = ref('[]')
    const rawActiveSessionJson = ref('')

    const sessions = computed(() => {
        try {
            return JSON.parse(rawSessionsJson.value)
        } catch (e) {
            return []
        }
    })

    const activeSession = computed(() => {
        if (!rawActiveSessionJson.value) return null
        try {
            return JSON.parse(rawActiveSessionJson.value)
        } catch (e) {
            return null
        }
    })

    const activeSessionId = computed(() => activeSession.value?.id || null)

    async function createSession(name: string, type: SessionType) {
        await ensureWasm()
        if (!sessionManager) return

        const id = uuidv4()
        sessionManager.create_session(id, name || new Date().toLocaleDateString(), type as any)
        updateLocalState()
    }

    async function switchSession(id: string) {
        await ensureWasm()
        if (!sessionManager) return
        sessionManager.switch_session(id)
        updateLocalState()
    }

    async function addSolveToActive(time: number, moves: string[]) {
        await ensureWasm()
        if (!sessionManager) return

        const solve = {
            id: uuidv4(),
            time,
            moves,
            date: Date.now(),
            is_valid: true
        }

        try {
            sessionManager.add_solve(JSON.stringify(solve))
            updateLocalState()
        } catch (e) {
            console.error('WASM error:', e)
        }
    }

    function updateLocalState() {
        if (!sessionManager) return
        rawSessionsJson.value = sessionManager.get_sessions_json()
        rawActiveSessionJson.value = sessionManager.get_active_session_json()
    }

    return { sessions, activeSession, activeSessionId, createSession, switchSession, addSolveToActive, updateLocalState }
})
