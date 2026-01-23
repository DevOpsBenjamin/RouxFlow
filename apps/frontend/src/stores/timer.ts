import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useSessionStore } from './session'

export const useTimerStore = defineStore('timer', () => {
    const time = ref(0)
    const isRunning = ref(false)
    const currentMoves = ref<string[]>([])
    const isHandheld = ref(false)
    const isSynced = ref(false)
    const useGyroTiming = ref(true)
    const flowState = ref<'Idle' | 'Scrambling' | 'Solving' | 'Summary'>('Idle')
    const lastReceivedMove = ref<{ face: number; amount: number } | null>(null)

    const sessionStore = useSessionStore()

    const formattedTime = computed(() => {
        const seconds = (time.value / 1000).toFixed(2)
        return seconds
    })

    function handleEvent(type: string, data?: string) {
        if (type === 'pickup' && useGyroTiming.value) {
            isHandheld.value = true
            console.log('[Timer] Pickup detected')
        } else if (type === 'putdown' && useGyroTiming.value) {
            isHandheld.value = false
            if (isRunning.value) {
                stopTimer()
                console.log('[Timer] Putdown detected - stopping solve')
            }
        } else if (type === 'move') {
            if (flowState.value === 'Idle' || flowState.value === 'Summary') {
                flowState.value = 'Scrambling'
            } else if (flowState.value === 'Scrambling') {
                // If the scramble is ready, any move starts the timer
                // This will be triggered by handleScrambleComplete usually
                // but let's allow it here if it's the start of a solve
            }

            if (isRunning.value) {
                if (data) currentMoves.value.push(data)
            }
        } else if (type === 'sync') {
            isSynced.value = true
            console.log('[Timer] Cube state synchronized!')
        }
    }

    let startTime = 0
    let timerId: number | null = null

    function updateTime() {
        if (isRunning.value) {
            time.value = performance.now() - startTime
            timerId = requestAnimationFrame(updateTime)
        }
    }

    function startTimer() {
        isRunning.value = true
        flowState.value = 'Solving'
        startTime = performance.now()
        time.value = 0
        currentMoves.value = []
        updateTime()
    }

    function stopTimer() {
        isRunning.value = false
        flowState.value = 'Summary'
        if (timerId !== null) {
            cancelAnimationFrame(timerId)
            timerId = null
        }
        sessionStore.addSolveToActive(time.value, [...currentMoves.value])
    }

    function reset() {
        flowState.value = 'Idle'
        time.value = 0
        currentMoves.value = []
    }

    return { time, isRunning, isHandheld, isSynced, useGyroTiming, currentMoves, lastReceivedMove, flowState, formattedTime, handleEvent, startTimer, stopTimer, reset }
})
