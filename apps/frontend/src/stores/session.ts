import { defineStore } from 'pinia'
import { ref, computed, onMounted } from 'vue'
import { sessionManager, ensureWasm, CubeBridge } from '../services/cube/bridge'

export type SessionType = 'Free' | 'WCA'

export const useSessionStore = defineStore('session', () => {
    const sessions = ref<any[]>([])
    const activeSessionId = ref<string | null>(null)

    const activeSession = computed(() => {
        return sessions.value.find(s => s.id === activeSessionId.value) || null
    })

    async function loadSessions() {
        const data = await CubeBridge.getSessions()
        sessions.value = data
        if (data.length > 0 && !activeSessionId.value) {
            activeSessionId.value = data[0].id
        }
    }

    onMounted(() => {
        loadSessions()
    })

    async function createSession(name: string, type: SessionType) {
        await ensureWasm()
        if (!sessionManager) return

        const sessionJson = sessionManager.create_session(name || new Date().toLocaleDateString(), type)
        const session = JSON.parse(sessionJson)

        await CubeBridge.createSession(session)

        await loadSessions()
        activeSessionId.value = session.id
    }

    async function switchSession(id: string) {
        await ensureWasm()
        if (!sessionManager) return

        const session = sessions.value.find(s => s.id === id)
        if (session) {
            sessionManager.set_active_session(JSON.stringify(session))
            activeSessionId.value = id
        }
    }

    async function addSolveToActive(time: number, moves: string[]) {
        await ensureWasm()
        if (!sessionManager) return

        const actionJson = sessionManager.record_solve(time, JSON.stringify(moves))
        const action = JSON.parse(actionJson)

        await CubeBridge.handleCoreAction(action)
    }

    return { sessions, activeSession, activeSessionId, createSession, switchSession, addSolveToActive, loadSessions }
})
