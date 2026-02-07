import init, {
    WasmCubeManager,
    WasmStorageManager,
    init_renderer,
    set_gyro_enabled,
    update_render_state,
    reset_gyro,
    find_cube_by_ble_name,
    all_scan_service_uuids,
    all_scan_name_prefixes,
} from '../../wasm/rouxflow/rouxflow_wasm'
import { logger } from '../../utils/logger'
import type { SavedCube } from '../../stores/bluetooth'

let wasmInitialized = false
export let cubeManager: WasmCubeManager | null = null
let storageManager: WasmStorageManager | null = null
let bleCharacteristic: BluetoothRemoteGATTCharacteristic | null = null

export async function ensureWasm() {
    if (!wasmInitialized) {
        await init()
        cubeManager = new WasmCubeManager()
        wasmInitialized = true
    }
}

async function getStorage(): Promise<WasmStorageManager> {
    await ensureWasm()
    if (!storageManager) {
        const url = import.meta.env.VITE_SUPABASE_URL || ''
        const key = import.meta.env.VITE_SUPABASE_ANON_KEY || ''
        storageManager = await new WasmStorageManager(
            url || undefined,
            key || undefined
        )
    }
    return storageManager
}

// Re-export render functions for components
export {
    init_renderer,
    set_gyro_enabled,
    update_render_state,
    reset_gyro,
    find_cube_by_ble_name,
    all_scan_service_uuids,
    all_scan_name_prefixes,
}

/// Single BLE event listener that forwards packets to WASM
function blePacketHandler(event: Event) {
    const target = event.target as unknown as BluetoothRemoteGATTCharacteristic
    const value = target.value
    if (!value || !cubeManager) return

    const bytes = new Uint8Array(value.buffer)
    const timestamp = performance.now() / 1000.0

    const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join(' ')
    logger.debug(`BLE Packet: ${hex}`)

    // Forward to WASM CubeManager
    const actionsJson = cubeManager.process_ble_packet(bytes, timestamp)

    if (actionsJson) {
        logger.debug(`Core Actions: ${actionsJson}`)
        try {
            // Parse single action or array of actions
            const actions = actionsJson.startsWith('[') ? JSON.parse(actionsJson) : [JSON.parse(actionsJson)]
            for (const action of actions) {
                handleCoreAction(action)
            }
        } catch (e) {
            logger.error('Failed to parse Core actions', e)
        }
    }
}

/// Handle side effects from Core actions (storage writes only)
async function handleCoreAction(action: any) {
    switch (action.type) {
        case 'SaveSolve': {
            // Get active session from somewhere - for now we'll need to store this
            // This is a limitation we'll address in the refactor
            // For now, skip the save if we don't have a session ID
            logger.debug('SaveSolve action received', action.data)
            // TODO: Implement once we have session management in place
            break
        }
        case 'DemoteSession': {
            const storage = await getStorage()
            await storage.demote_session(action.data)
            logger.debug('Session demoted', action.data)
            break
        }
        case 'Error':
            logger.error('Core logic error:', action.data)
            break
        default:
            // Other actions are handled by WASM state updates
            logger.debug('Action processed by WASM:', action.type)
            break
    }
}

/// Connect to a cube via Web Bluetooth
export async function connect(): Promise<{ device: BluetoothDevice; cubeDef: any }> {
    await ensureWasm()
    const serviceUuids = all_scan_service_uuids()

    logger.debug('Available service UUIDs:', serviceUuids)

    // Build filters from known cube name prefixes
    const prefixes = all_scan_name_prefixes()
    const nameFilters = prefixes.map((p: string) => ({ namePrefix: p }))

    logger.debug('Name prefixes for scanning:', prefixes)

    const device = await (navigator as any).bluetooth.requestDevice({
        filters: nameFilters.length > 0 ? nameFilters : [{ services: [serviceUuids[0]] }],
        optionalServices: serviceUuids
    })

    logger.info(`Selected device: ${device.name} (${device.id})`)

    // Look up the cube definition from its BLE name
    const cubeDef = device.name ? find_cube_by_ble_name(device.name) : null

    if (!cubeDef) {
        logger.error(`No cube definition found for: ${device.name}`)
        throw new Error(`Unknown cube: ${device.name}. Please ensure your cube is supported.`)
    }

    logger.debug('Cube definition loaded:', cubeDef)

    return { device, cubeDef }
}

/// Finalize connection and set up BLE listener
export async function finalizeConnection(device: BluetoothDevice, cubeDef: any, macAddress: string): Promise<void> {
    if (!cubeManager) {
        throw new Error('WASM not initialized')
    }

    logger.debug('Cube definition:', cubeDef)

    const serviceUuid = cubeDef.serviceUuid
    const charUuid = cubeDef.stateCharacteristic
    const protocolName = cubeDef.protocol

    logger.debug(`MAC Address: ${macAddress}`)
    logger.debug(`Connecting to GATT service ${serviceUuid}, characteristic ${charUuid}`)

    try {
        // Connect GATT
        const server = await device.gatt?.connect()
        if (!server) {
            throw new Error('Failed to connect to GATT server')
        }
        logger.debug('GATT server connected')

        const service = await server.getPrimaryService(serviceUuid)
        logger.debug('GATT service obtained')

        const characteristic = await service.getCharacteristic(charUuid)
        logger.debug('GATT characteristic obtained')

        if (!characteristic) {
            throw new Error('Failed to get BLE characteristic')
        }

        // Remove old listener if exists
        if (bleCharacteristic) {
            bleCharacteristic.removeEventListener('characteristicvaluechanged', blePacketHandler)
        }

        // Set up new listener
        await characteristic.startNotifications()
        logger.debug('Notifications started')

        characteristic.addEventListener('characteristicvaluechanged', blePacketHandler)
        bleCharacteristic = characteristic

        // Connect in WASM
        cubeManager.connect(device.name || 'Unknown Cube', macAddress, protocolName)

        logger.info(`Connected to ${device.name} (${protocolName})`)
    } catch (error) {
        logger.error('GATT connection error:', error)
        logger.error(`Device: ${device.name}, Service: ${serviceUuid}, Char: ${charUuid}`)
        throw error
    }
}

/// Disconnect from cube
export async function disconnect(): Promise<void> {
    if (bleCharacteristic) {
        try {
            bleCharacteristic.removeEventListener('characteristicvaluechanged', blePacketHandler)
            await bleCharacteristic.stopNotifications()
        } catch (e) {
            logger.warn('Error stopping BLE notifications', e)
        }
        bleCharacteristic = null
    }

    if (cubeManager) {
        cubeManager.disconnect()
    }

    logger.info('Disconnected from cube')
}

// --- Storage operations (delegated to WASM StorageManager) ---

export async function getSessions(): Promise<any[]> {
    const storage = await getStorage()
    const json = await storage.get_sessions_json()
    return JSON.parse(json)
}

export async function createSession(session: any) {
    const storage = await getStorage()
    await storage.create_session_json(JSON.stringify(session))
}

export async function getCubes(userId: string | null = null): Promise<SavedCube[]> {
    const storage = await getStorage()
    const json = await storage.get_cubes_json(userId ?? undefined)
    return JSON.parse(json)
}

export async function saveCube(cube: SavedCube) {
    const storage = await getStorage()
    await storage.save_cube_json(JSON.stringify(cube))
}

export async function deleteCube(id: string, userId: string | null = null) {
    const storage = await getStorage()
    await storage.delete_cube(id, userId || '')
}

export async function syncCubes(userId: string) {
    const storage = await getStorage()
    await storage.sync_cubes(userId)
}
