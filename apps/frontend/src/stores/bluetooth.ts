import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { cubeManager, connect, finalizeConnection, disconnect as bridgeDisconnect, getCubes, saveCube as bridgeSaveCube, deleteCube as bridgeDeleteCube, syncCubes as bridgeSyncCubes } from '../services/cube/bridge'
import { logger } from '../utils/logger'

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
    const savedCubes = ref<SavedCube[]>([])
    const showPicker = ref(false)
    const error = ref<string | null>(null)
    const isConnecting = ref(false)

    // Query WASM for cube connection state
    const isConnected = computed(() => cubeManager?.is_connected() ?? false)

    const connectedDeviceName = computed(() => {
        if (!cubeManager) return null
        const deviceInfoJson = cubeManager.get_device_info()
        if (!deviceInfoJson) return null
        try {
            const info = JSON.parse(deviceInfoJson)
            return info.name || null
        } catch {
            return null
        }
    })

    const deviceInfo = computed(() => {
        if (!cubeManager) return null
        const deviceInfoJson = cubeManager.get_device_info()
        if (!deviceInfoJson) return null
        try {
            return JSON.parse(deviceInfoJson)
        } catch {
            return null
        }
    })

    const orientation = computed(() => {
        if (!cubeManager) return { x: 0, y: 0, z: 0, w: 1 }
        const [x, y, z, w] = cubeManager.get_orientation()
        return { x, y, z, w }
    })

    const facelets = computed(() => {
        if (!cubeManager) return new Array(54).fill(0)
        return cubeManager.get_facelets()
    })

    async function startScan() {
        isConnecting.value = true
        showPicker.value = true
        error.value = null

        try {
            const { device, cubeDef } = await connect()
            await finalizeConnection(device, cubeDef)
            logger.info(`Connected to ${device.name}`)
        } catch (e: any) {
            logger.error('Connection failed:', e)
            error.value = e.message || 'Connection failed'
        } finally {
            isConnecting.value = false
            showPicker.value = false
        }
    }

    function setError(msg: string) {
        error.value = msg
        isConnecting.value = false
    }

    async function disconnect() {
        await bridgeDisconnect()
    }

    // DB Operations
    async function loadSavedCubes(userId: string | null = null) {
        try {
            savedCubes.value = await getCubes(userId)
        } catch (e) {
            logger.error('Failed to load saved cubes:', e)
        }
    }

    async function saveCube(cube: Omit<SavedCube, 'created_at'>) {
        const newCube: SavedCube = {
            ...cube,
            created_at: Date.now()
        }
        try {
            await bridgeSaveCube(newCube)
            await loadSavedCubes(cube.user_id)
        } catch (e) {
            logger.error('Failed to save cube:', e)
            throw e
        }
    }

    async function deleteCube(id: string, userId: string | null = null) {
        try {
            await bridgeDeleteCube(id, userId)
            await loadSavedCubes(userId)
        } catch (e) {
            logger.error('Failed to delete cube:', e)
        }
    }

    async function sync(userId: string) {
        try {
            await bridgeSyncCubes(userId)
            await loadSavedCubes(userId)
        } catch (e) {
            logger.error('Failed to sync cubes:', e)
        }
    }

    return {
        savedCubes,
        isConnecting,
        isConnected,
        connectedDeviceName,
        deviceInfo,
        orientation,
        facelets,
        showPicker,
        error,
        startScan,
        setError,
        disconnect,
        loadSavedCubes,
        saveCube,
        deleteCube,
        sync
    }
})
