import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useTimerStore = defineStore('timer', () => {
    const time = ref(0)
    const isRunning = ref(false)
    const solves = ref<any[]>([])

    const formattedTime = computed(() => {
        const seconds = (time.value / 1000).toFixed(2)
        return seconds
    })

    function startTimer() {
        isRunning.value = true
    }

    function stopTimer() {
        isRunning.value = false
        solves.value.push({ time: time.value, date: new Date() })
    }

    return { time, isRunning, solves, formattedTime, startTimer, stopTimer }
})
