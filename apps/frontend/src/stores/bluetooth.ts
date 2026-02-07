import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { cubeManager, connect, finalizeConnection, disconnect as bridgeDisconnect, getCubes, saveCube as bridgeSaveCube, deleteCube as bridgeDeleteCube, syncCubes as bridgeSyncCubes } from '../services/cube/bridge'
import { logger } from '../utils/logger'
import { useAuthStore } from './auth'

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
    // UI state only
    const showPicker = ref(false)
    const error = ref<string | null>(null)
    const isConnecting = ref(false)
    const showMacInput = ref(false)
    const pendingConnection = ref<{ device: any; cubeDef: any } | null>(null)

    // ========== Query WASM for all state ==========

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

    // ========== Connection Flow (Browser API only) ==========

    async function startScan() {
        isConnecting.value = true
        showPicker.value = true
        error.value = null

        try {
            // Browser API: show Web Bluetooth picker
            const { device, cubeDef } = await connect()

            // Ask WASM: do we need MAC input?
            const needsMacInput = cubeManager?.needs_mac_input(device.id || '', cubeDef.protocol) ?? false

            if (needsMacInput) {
                // Load saved cubes to check for saved MAC
                const auth = useAuthStore()
                const savedCubes = await getCubes(auth.user?.id || null)

                // Check if we have this cube saved with a MAC
                const savedCube = savedCubes.find(
                    cube => cube.name === device.name && cube.device_type === cubeDef.protocol
                )

                if (savedCube && savedCube.mac_address) {
                    // Use saved MAC - proceed with connection
                    logger.info(`Using saved MAC for ${device.name}`)
                    await finalizeConnection(device, cubeDef, savedCube.mac_address)
                    logger.info(`Connected to ${device.name}`)
                    showPicker.value = false
                    return
                }

                // Need to ask for MAC address
                logger.warn(`Protocol ${cubeDef.protocol} requires MAC address, but auto-detection failed`)
                pendingConnection.value = { device, cubeDef }
                showMacInput.value = true
                showPicker.value = false
                isConnecting.value = false
                return
            }

            // Don't need MAC - proceed with connection
            await finalizeConnection(device, cubeDef, device.id)
            logger.info(`Connected to ${device.name}`)
            showPicker.value = false
        } catch (e: any) {
            logger.error('Connection failed:', e)
            error.value = e.message || 'Connection failed'
            showPicker.value = false
        } finally {
            if (!showMacInput.value) {
                isConnecting.value = false
            }
        }
    }

    async function submitMacAddress(macAddress: string) {
        if (!pendingConnection.value) return

        isConnecting.value = true
        showMacInput.value = false
        showPicker.value = true

        try {
            const { device, cubeDef } = pendingConnection.value
            await finalizeConnection(device, cubeDef, macAddress)
            logger.info(`Connected to ${device.name} with provided MAC`)

            // Save cube with MAC to offline storage
            const auth = useAuthStore()
            const cubeId = `${device.name}_${Date.now()}`
            await bridgeSaveCube({
                id: cubeId,
                user_id: auth.user?.id || null,
                name: device.name || 'Unknown Cube',
                device_type: cubeDef.protocol,
                mac_address: macAddress,
                created_at: Date.now()
            })
            logger.info(`Saved cube ${device.name} with MAC to offline storage`)

            pendingConnection.value = null
        } catch (e: any) {
            logger.error('Connection failed:', e)
            error.value = e.message || 'Connection failed'
        } finally {
            isConnecting.value = false
            showPicker.value = false
        }
    }

    function cancelMacInput() {
        showMacInput.value = false
        pendingConnection.value = null
        error.value = 'Connection cancelled - MAC address required'
    }

    function setError(msg: string) {
        error.value = msg
        isConnecting.value = false
    }

    async function disconnect() {
        await bridgeDisconnect()
    }

    // ========== Storage Operations (delegate to WASM via bridge) ==========

    async function loadSavedCubes(userId: string | null = null) {
        try {
            return await getCubes(userId)
        } catch (e) {
            logger.error('Failed to load saved cubes:', e)
            return []
        }
    }

    async function saveCube(cube: Omit<SavedCube, 'created_at'>) {
        const newCube: SavedCube = {
            ...cube,
            created_at: Date.now()
        }
        try {
            await bridgeSaveCube(newCube)
        } catch (e) {
            logger.error('Failed to save cube:', e)
            throw e
        }
    }

    async function deleteCube(id: string, userId: string | null = null) {
        try {
            await bridgeDeleteCube(id, userId)
        } catch (e) {
            logger.error('Failed to delete cube:', e)
        }
    }

    async function sync(userId: string) {
        try {
            await bridgeSyncCubes(userId)
        } catch (e) {
            logger.error('Failed to sync cubes:', e)
        }
    }

    return {
        // UI state
        showPicker,
        showMacInput,
        pendingConnection,
        error,
        isConnecting,
        // WASM state queries
        isConnected,
        connectedDeviceName,
        deviceInfo,
        orientation,
        facelets,
        // Actions (browser APIs + WASM calls)
        startScan,
        submitMacAddress,
        cancelMacInput,
        setError,
        disconnect,
        // Storage operations
        loadSavedCubes,
        saveCube,
        deleteCube,
        sync
    }
})
