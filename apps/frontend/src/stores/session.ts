import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { cm_set_active_session, getSessions, createSession as bridgeCreateSession } from '../services/cube/bridge'

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
        const session = {
            id: crypto.randomUUID(),
            name: name || new Date().toLocaleDateString(),
            type,
            created_at: Date.now(),
            solves: []
        }

        await bridgeCreateSession(session)

        await loadSessions()
        activeSessionId.value = session.id
    }

    async function switchSession(id: string) {
        const session = sessions.value.find(s => s.id === id)
        if (session) {
            cm_set_active_session(JSON.stringify(session))
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
