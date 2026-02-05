import { defineStore } from 'pinia'
import { ref } from 'vue'
import { CubeBridge } from '../services/cube/bridge'

export interface SavedCube {
    id: string
    user_id: string | null
    name: string
    device_type: string
    mac_address: string
    created_at: number
}

export interface BluetoothDevice {
    id: string
    name: string
    rssi: number
}

export const useBluetoothStore = defineStore('bluetooth', () => {
    const scannedDevices = ref<BluetoothDevice[]>([])
    const savedCubes = ref<SavedCube[]>([])
    const isScanning = ref(false)
    const isConnecting = ref(false)
    const isConnected = ref(false)
    const connectedDeviceName = ref<string | null>(null)
    const showPicker = ref(false)
    const error = ref<string | null>(null)

    function setDevices(devices: BluetoothDevice[]) {
        scannedDevices.value = devices
    }

    function startScan() {
        if (!isScanning.value) {
            console.log('[Store] Starting scan...');
            isScanning.value = true
        }
        showPicker.value = true
        error.value = null
        CubeBridge.connect().catch(e => {
            console.error('[Store] Scan initiation failed:', e);
            setError(e.message || 'Scan failed');
        });
    }

    function stopScan() {
        isScanning.value = false
    }

    function setError(msg: string) {
        error.value = msg
        isScanning.value = false
        isConnecting.value = false
    }

    // DB Operations
    async function loadSavedCubes(userId: string | null = null) {
        try {
            savedCubes.value = await CubeBridge.getCubes(userId)
        } catch (e) {
            console.error('Failed to load saved cubes:', e)
        }
    }

    async function saveCube(cube: Omit<SavedCube, 'created_at'>) {
        const newCube: SavedCube = {
            ...cube,
            created_at: Date.now()
        }
        try {
            await CubeBridge.saveCube(newCube)
            await loadSavedCubes(cube.user_id)
        } catch (e) {
            console.error('Failed to save cube:', e)
            throw e
        }
    }

    async function deleteCube(id: string, userId: string | null = null) {
        try {
            await CubeBridge.deleteCube(id, userId)
            await loadSavedCubes(userId)
        } catch (e) {
            console.error('Failed to delete cube:', e)
        }
    }

    async function sync(userId: string) {
        try {
            await CubeBridge.syncCubes(userId)
            await loadSavedCubes(userId)
        } catch (e) {
            console.error('Failed to sync cubes:', e)
        }
    }

    const connectedDeviceInfo = ref<{ name: string; address: string; protocol: string; features?: { gyro: boolean } } | null>(null)
    const orientation = ref({ x: 0, y: 0, z: 0, w: 1 })
    const facelets = ref<number[]>(new Array(54).fill(0))

    function setOrientation(x: number, y: number, z: number, w: number) {
        orientation.value = { x, y, z, w }
    }

    function setFacelets(newFacelets: number[]) {
        facelets.value = newFacelets
    }

    function disconnect() {
        isConnected.value = false
        connectedDeviceName.value = null
        connectedDeviceInfo.value = null
    }

    return {
        scannedDevices,
        savedCubes,
        isScanning,
        isConnecting,
        isConnected,
        connectedDeviceName,
        deviceInfo: connectedDeviceInfo,
        orientation,
        facelets,
        showPicker,
        error,
        setDevices,
        startScan,
        stopScan,
        setError,
        setOrientation,
        setFacelets,
        loadSavedCubes,
        saveCube,
        deleteCube,
        disconnect,
        sync
    }
})
