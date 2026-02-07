import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { cm_is_connected, cm_get_device_info, cm_get_orientation, cm_get_facelets, cm_needs_mac_input, connect, finalizeConnection, disconnect as bridgeDisconnect, getCubes, saveCube as bridgeSaveCube, deleteCube as bridgeDeleteCube, syncCubes as bridgeSyncCubes, reconnectSavedCube, onWasmStateChanged } from '../services/cube/bridge'
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

    // Saved cubes from IndexedDB (loaded on startup)
    const savedCubes = ref<SavedCube[]>([])

    // Reactive tick — bumped after WASM state changes so computed() re-evaluates
    const _wasmTick = ref(0)
    function bumpWasm() { _wasmTick.value++ }

    // Register callback so bridge.ts BLE packet handler can trigger Vue reactivity
    onWasmStateChanged(bumpWasm)

    // ========== Query WASM for all state ==========
    // Each computed reads _wasmTick to create a reactive dependency on WASM changes

    const isConnected = computed(() => { _wasmTick.value; return cm_is_connected() })

    const connectedDeviceName = computed(() => {
        _wasmTick.value
        const deviceInfoJson = cm_get_device_info()
        if (!deviceInfoJson) return null
        try {
            const info = JSON.parse(deviceInfoJson)
            return info.name || null
        } catch {
            return null
        }
    })

    const deviceInfo = computed(() => {
        _wasmTick.value
        const deviceInfoJson = cm_get_device_info()
        if (!deviceInfoJson) return null
        try {
            return JSON.parse(deviceInfoJson)
        } catch {
            return null
        }
    })

    const orientation = computed(() => {
        _wasmTick.value
        try {
            const [x, y, z, w] = JSON.parse(cm_get_orientation())
            return { x, y, z, w }
        } catch { return { x: 0, y: 0, z: 0, w: 1 } }
    })

    const facelets = computed(() => {
        _wasmTick.value
        try {
            return JSON.parse(cm_get_facelets())
        } catch { return new Array(54).fill(0) }
    })

    // ========== Connection Flow (Browser API only) ==========

    async function startScan() {
        isConnecting.value = true
        showPicker.value = true
        error.value = null

        try {
            // Browser API: show Web Bluetooth picker
            const { device, cubeDef } = await connect()

            // Ask WASM: do we need MAC input? (free function)
            const needsMacInput = cm_needs_mac_input(device.id || '', cubeDef.protocol)

            if (needsMacInput) {
                // Check if we already have a saved MAC for this cube
                const existing = savedCubes.value.find(c => c.name === device.name)
                if (existing) {
                    logger.info(`Found saved MAC for ${device.name}, skipping MAC input`)
                    await finalizeConnection(device, cubeDef, existing.mac_address)
                    bumpWasm()
                    logger.info(`Connected to ${device.name} using saved MAC`)
                    showPicker.value = false
                    return
                }

                logger.warn(`Protocol ${cubeDef.protocol} requires MAC address`)
                pendingConnection.value = { device, cubeDef }
                showMacInput.value = true
                showPicker.value = false
                isConnecting.value = false
                return
            }

            // Don't need MAC - proceed with connection
            await finalizeConnection(device, cubeDef, device.id)
            bumpWasm()
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
            bumpWasm()
            logger.info(`Connected to ${device.name} with provided MAC`)

            // Save cube with MAC to offline storage (skip if already saved)
            const auth = useAuthStore()
            const existing = savedCubes.value.find(
                c => c.name === device.name && c.mac_address === macAddress
            )
            if (!existing) {
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
                await loadSavedCubes(auth.user?.id || null)
            } else {
                logger.info(`Cube ${device.name} already saved, skipping duplicate`)
            }

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

    async function reconnectCube(cube: SavedCube) {
        isConnecting.value = true
        error.value = null
        try {
            const { device, cubeDef } = await reconnectSavedCube(cube)
            await finalizeConnection(device, cubeDef, cube.mac_address)
            bumpWasm()
            logger.info(`Reconnected to ${cube.name}`)
        } catch (e: any) {
            logger.warn('Quick reconnect failed, falling back to scan:', e.message)
            await startScan()
        } finally {
            isConnecting.value = false
        }
    }

    async function disconnect() {
        await bridgeDisconnect()
        bumpWasm()
    }

    // ========== Storage Operations (delegate to WASM via bridge) ==========

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
        // Saved cubes (from IndexedDB)
        savedCubes,
        // Actions (browser APIs + WASM calls)
        startScan,
        reconnectCube,
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
