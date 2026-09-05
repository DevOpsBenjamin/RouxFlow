import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
    cm_get_sessions_json,
    cm_get_active_session_json,
    cm_get_active_session_solves_json,
    cm_get_active_session_id,
    cm_switch_session,
    cm_create_session_persist,
    cm_load_active_session_solves,
    cm_get_session_stats_json,
    cm_get_solve_list_json,
    cm_get_solve_by_id_json,
    onWasmStateChanged,
} from '../services/cube/bridge'
import { logger } from '../utils/logger'

export type SessionType = 'Free' | 'WCA'

export const useSessionStore = defineStore('session', () => {
    // Reactive tick — bumped after WASM state changes so computed() re-evaluates
    const _wasmTick = ref(0)
    function bumpWasm() { _wasmTick.value++ }

    // Register callback so WASM state changes trigger Vue reactivity
    onWasmStateChanged(bumpWasm)

    // ========== Query WASM for all state ==========

    const sessions = computed(() => {
        _wasmTick.value
        try {
            return JSON.parse(cm_get_sessions_json())
        } catch { return [] }
    })

    const activeSessionId = computed(() => {
        _wasmTick.value
        return cm_get_active_session_id() ?? null
    })

    const activeSession = computed(() => {
        _wasmTick.value
        try {
            const json = cm_get_active_session_json()
            return json && json !== 'null' ? JSON.parse(json) : null
        } catch { return null }
    })

    const activeSessionSolves = computed(() => {
        _wasmTick.value
        try {
            return JSON.parse(cm_get_active_session_solves_json())
        } catch { return [] }
    })

    const sessionStats = computed(() => {
        _wasmTick.value
        try {
            return JSON.parse(cm_get_session_stats_json())
        } catch { return null }
    })

    const solveList = computed(() => {
        _wasmTick.value
        try {
            return JSON.parse(cm_get_solve_list_json())
        } catch { return [] }
    })

    const selectedSolveId = ref<string | null>(null)

    const selectedSolve = computed(() => {
        if (!selectedSolveId.value) return null
        try {
            const json = cm_get_solve_by_id_json(selectedSolveId.value)
            return json && json !== 'null' ? JSON.parse(json) : null
        } catch { return null }
    })

    function selectSolve(id: string | null) {
        selectedSolveId.value = id
    }

    // ========== Actions ==========

    async function createSession(name: string, type: SessionType) {
        try {
            await cm_create_session_persist(name || new Date().toLocaleDateString(), type)
            bumpWasm()
        } catch (e) {
            logger.error('Failed to create session:', e)
        }
    }

    async function switchSession(id: string) {
        const ok = cm_switch_session(id)
        if (ok) {
            // Load solves for the new active session from IndexedDB
            try {
                await cm_load_active_session_solves()
            } catch (e) {
                logger.error('Failed to load solves for session:', e)
            }
            bumpWasm()
        }
    }

    return {
        sessions,
        activeSession,
        activeSessionId,
        activeSessionSolves,
        sessionStats,
        solveList,
        selectedSolveId,
        selectedSolve,
        selectSolve,
        createSession,
        switchSession,
        bumpWasm,
    }
})
