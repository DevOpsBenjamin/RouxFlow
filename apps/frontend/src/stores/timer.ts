import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useSessionStore } from './session'

export const useTimerStore = defineStore('timer', () => {
    const time = ref(0)
    const isRunning = ref(false)
    const isHandheld = ref(false)
    const isSynced = ref(false)
    const useGyroTiming = ref(true)
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
            if (!isRunning.value) {
                startTimer()
            }
            if (data) {
                try {
                    const m = JSON.parse(data)
                    const faceNames = ['U', 'R', 'F', 'D', 'L', 'B']
                    const amountStr = m.amount === 1 ? '' : m.amount === -1 ? "'" : '2'
                    currentMoves.value.push(`${faceNames[m.face]}${amountStr}`)
                } catch (e) {
                    console.error('Failed to parse move data', e)
                }
            }
        } else if (type === 'sync') {
            isSynced.value = true
            console.log('[Timer] Cube state synchronized!')
        }
    }

    function startTimer() {
        isRunning.value = true
        time.value = 0
        currentMoves.value = []
    }

    function stopTimer() {
        isRunning.value = false
        sessionStore.addSolveToActive(time.value, [...currentMoves.value])
    }

    return { time, isRunning, isHandheld, isSynced, useGyroTiming, currentMoves, lastReceivedMove, formattedTime, handleEvent, startTimer, stopTimer }
})
