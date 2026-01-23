import { defineStore } from 'pinia'
import { ref, computed, onMounted } from 'vue'
import { v4 as uuidv4 } from 'uuid'
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
            updateBridgeContext()
        }
    }

    onMounted(() => {
        loadSessions()
    })

    function updateBridgeContext() {
        (window as any).activeSessionId = activeSessionId.value
    }

    async function createSession(name: string, type: SessionType) {
        await ensureWasm()
        if (!sessionManager) return

        const id = uuidv4()
        const sessionJson = sessionManager.create_session(id, name || new Date().toLocaleDateString(), type as any)
        const session = JSON.parse(sessionJson)

        // Save via bridge
        await CubeBridge.createSession(session)

        await loadSessions()
        activeSessionId.value = id
        updateBridgeContext()
    }

    async function switchSession(id: string) {
        await ensureWasm()
        if (!sessionManager) return

        const session = sessions.value.find(s => s.id === id)
        if (session) {
            sessionManager.set_active_session(JSON.stringify(session))
            activeSessionId.value = id
            updateBridgeContext()
        }
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

        const actionJson = sessionManager.add_solve(JSON.stringify(solve))
        const action = JSON.parse(actionJson)

        // Process action via bridge (this handles database save)
        await CubeBridge.handleCoreAction(action)

        // Follow-up: if session was demoted, we need to refresh list
        if (action.type === 'DemoteSession') {
            await loadSessions()
        } else if (action.type === 'SaveSolve') {
            // Optimistic update or reload
            await loadSessions()
        }
    }

    return { sessions, activeSession, activeSessionId, createSession, switchSession, addSolveToActive, loadSessions }
})
