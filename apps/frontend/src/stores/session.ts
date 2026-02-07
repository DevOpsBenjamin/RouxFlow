import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { cubeManager, getSessions, createSession as bridgeCreateSession } from '../services/cube/bridge'

export type SessionType = 'Free' | 'WCA'

export const useSessionStore = defineStore('session', () => {
    const sessions = ref<any[]>([])
    const activeSessionId = ref<string | null>(null)

    const activeSession = computed(() => {
        return sessions.value.find(s => s.id === activeSessionId.value) || null
    })

    async function loadSessions() {
        const data = await getSessions()
        sessions.value = data
        if (data.length > 0 && !activeSessionId.value) {
            activeSessionId.value = data[0].id
        }
    }

    async function createSession(name: string, type: SessionType) {
        // For now, create a simple session object
        // TODO: Use WASM create_session once API is updated
        const session = {
            id: crypto.randomUUID(),
            name: name || new Date().toLocaleDateString(),
            type,
            created_at: Date.now(),
            solves: []
        }

        // Save to storage
        await bridgeCreateSession(session)

        // Reload and set active
        await loadSessions()
        activeSessionId.value = session.id
    }

    async function switchSession(id: string) {
        if (!cubeManager) return

        const session = sessions.value.find(s => s.id === id)
        if (session) {
            cubeManager.set_active_session(JSON.stringify(session))
            activeSessionId.value = id
        }
    }

    return {
        sessions,
        activeSession,
        activeSessionId,
        createSession,
        switchSession,
        loadSessions
    }
})
