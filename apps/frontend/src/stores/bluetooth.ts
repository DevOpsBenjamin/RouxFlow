import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface BluetoothDevice {
    id: string
    name: string
    rssi: number
}

export const useBluetoothStore = defineStore('bluetooth', () => {
    const scannedDevices = ref<BluetoothDevice[]>([])
    const isScanning = ref(false)
    const isConnecting = ref(false)
    const showPicker = ref(false)
    const error = ref<string | null>(null)

    function setDevices(devices: BluetoothDevice[]) {
        scannedDevices.value = devices
    }

    function startScan() {
        isScanning.value = true
        showPicker.value = true
        error.value = null
    }

    function stopScan() {
        isScanning.value = false
    }

    function setError(msg: string) {
        error.value = msg
        isScanning.value = false
        isConnecting.value = false
    }

    return {
        scannedDevices,
        isScanning,
        isConnecting,
        showPicker,
        error,
        setDevices,
        startScan,
        stopScan,
        setError
    }
})
