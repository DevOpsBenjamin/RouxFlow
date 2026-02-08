import init, {
    WasmStorageManager,
    init_renderer,
    set_gyro_enabled,
    update_render_state,
    reset_gyro,
    find_cube_by_ble_name,
    all_scan_service_uuids,
    all_scan_name_prefixes,
    cm_init,
    cm_connect,
    cm_disconnect,
    cm_is_connected,
    cm_get_device_info,
    cm_process_ble_packet,
    cm_encode_command,
    cm_get_cube_state,
    cm_get_orientation,
    cm_get_facelets,
    cm_start_timer,
    cm_stop_timer,
    cm_update_timer,
    cm_get_timer_state,
    cm_is_timer_running,
    cm_get_current_time_ms,
    cm_get_flow_state,
    cm_set_active_session,
    cm_create_session,
    cm_start_scramble,
    cm_handle_scramble_move,
    cm_set_solving,
    cm_record_solve,
    cm_needs_mac_input,
    cm_requires_handshake,
    cm_handshake_data,
    cm_decrypt_hex,
    cm_encrypt_hex,
    cm_get_last_gyro_hex,
} from '../../wasm/rouxflow/rouxflow_wasm'
import { logger } from '../../utils/logger'
import type { SavedCube } from '../../stores/bluetooth'

// Callback to notify Vue store that WASM state changed (triggers computed re-evaluation)
let _onWasmStateChanged: (() => void) | null = null
export function onWasmStateChanged(cb: () => void) { _onWasmStateChanged = cb }

let wasmReady = false
let storageManager: WasmStorageManager | null = null
let storageInitPromise: Promise<WasmStorageManager> | null = null
let bleCharacteristic: BluetoothRemoteGATTCharacteristic | null = null
let commandCharacteristic: BluetoothRemoteGATTCharacteristic | null = null
let batteryPollInterval: ReturnType<typeof setInterval> | null = null

export async function ensureWasm() {
    if (wasmReady) return
    await init()
    cm_init()
    wasmReady = true
    logger.info('WASM initialized')

    // Expose gyro debug on window for console use
    ;(window as any).gyroDebug = {
        /** Print last decrypted gyro packet hex: gyroDebug.show() */
        show: () => {
            const hex = cm_get_last_gyro_hex()
            console.log('Last gyro hex:', hex)
            if (hex && !hex.startsWith('No') && !hex.startsWith('ERROR')) {
                const b = hex.split(' ').map(h => parseInt(h, 16))
                // LE int16 helper
                const i16 = (lo: number, hi: number) => { const v = lo | (hi << 8); return v > 32767 ? v - 65536 : v }
                // Accelerometer: bytes 1-2, 5-6, 9-10, 13-14 (LE int16)
                const ax = i16(b[1], b[2]), ay = i16(b[5], b[6]), az = i16(b[9], b[10]), a4 = i16(b[13], b[14])
                // Quaternion: bytes 3-4, 7-8, 11-12, 15-16 (LE int16 / 16384)
                const qw = i16(b[3], b[4]) / 16384, qx = i16(b[7], b[8]) / 16384
                const qy = i16(b[11], b[12]) / 16384, qz = i16(b[15], b[16]) / 16384
                const norm = Math.sqrt(qw*qw + qx*qx + qy*qy + qz*qz)
                console.log(`  Quaternion: w=${qw.toFixed(4)} x=${qx.toFixed(4)} y=${qy.toFixed(4)} z=${qz.toFixed(4)}  norm=${norm.toFixed(4)}`)
                console.log(`  Accel raw:  a1=${ax}  a2=${ay}  a3=${az}  a4=${a4}`)
                console.log(`  Accel /1000: a1=${(ax/1000).toFixed(2)}g  a2=${(ay/1000).toFixed(2)}g  a3=${(az/1000).toFixed(2)}g  a4=${(a4/1000).toFixed(2)}g`)
                console.log(`  Padding: ${b.slice(17).map(v => v.toString(16).padStart(2,'0')).join(' ')}`)
            }
            return hex
        },
    }

    // Expose debug tools on window for console use
    ;(window as any).cubeDebug = {
        /** Decrypt encrypted hex: cubeDebug.decode("90 5c 36 ...") */
        decode: (hex: string) => {
            const result = cm_decrypt_hex(hex)
            console.log('Decrypted:', result)
            return result
        },
        /** Encrypt plaintext hex: cubeDebug.encode("a4 00 00 ...") */
        encode: (hex: string) => {
            const result = cm_encrypt_hex(hex)
            console.log('Encrypted:', result)
            return result
        },
        /** Send raw encrypted hex to cube: cubeDebug.send("90 5c 36 ...") */
        send: async (hex: string) => {
            if (!commandCharacteristic) {
                console.error('No command characteristic — cube not connected')
                return
            }
            const bytes = new Uint8Array(hex.trim().split(/\s+/).map(h => parseInt(h, 16)))
            logBleSend('console', bytes)
            await commandCharacteristic.writeValue(bytes)
            console.log('Sent', bytes.length, 'bytes')
        },
    }
}

async function getStorage(): Promise<WasmStorageManager> {
    await ensureWasm()

    if (storageManager) {
        return storageManager
    }

    if (storageInitPromise) {
        return storageInitPromise
    }

    storageInitPromise = (async () => {
        const url = import.meta.env.VITE_SUPABASE_URL || ''
        const key = import.meta.env.VITE_SUPABASE_ANON_KEY || ''
        storageManager = await new WasmStorageManager(
            url || undefined,
            key || undefined
        )
        return storageManager
    })()

    return storageInitPromise
}

// Re-export WASM functions directly — WASM is guaranteed loaded before Vue mounts
export {
    init_renderer,
    set_gyro_enabled,
    update_render_state,
    reset_gyro,
    cm_is_connected,
    cm_get_device_info,
    cm_get_orientation,
    cm_get_facelets,
    cm_get_cube_state,
    cm_get_timer_state,
    cm_is_timer_running,
    cm_get_current_time_ms,
    cm_get_flow_state,
    cm_needs_mac_input,
    cm_connect,
    cm_disconnect,
    cm_process_ble_packet,
    cm_encode_command,
    cm_start_timer,
    cm_stop_timer,
    cm_update_timer,
    cm_set_active_session,
    cm_create_session,
    cm_start_scramble,
    cm_handle_scramble_move,
    cm_set_solving,
    cm_record_solve,
}

// Wrapper functions that parse JSON from WASM (avoids wasm_bindgen alloc churn)
export function findCubeByBleName(deviceName: string): any | null {
    const json = find_cube_by_ble_name(deviceName)
    if (!json) return null
    try { return JSON.parse(json) } catch { return null }
}

export function getScanServiceUuids(): string[] {
    return JSON.parse(all_scan_service_uuids())
}

export function getScanNamePrefixes(): string[] {
    return JSON.parse(all_scan_name_prefixes())
}

/// Send a reset command to the cube (MoYu V3: 0xAC 0x00 0x01)
export async function resetCube(): Promise<void> {
    if (!commandCharacteristic) {
        logger.warn('Cannot reset: no command characteristic')
        return
    }
    try {
        const bytesJson = cm_encode_command('reset')
        const bytes = new Uint8Array(JSON.parse(bytesJson))
        logBleSend('reset', bytes)
        await commandCharacteristic.writeValue(bytes)
        logger.info('Reset command sent')
    } catch (e) {
        logger.error('Reset command failed:', e)
    }
}

/// Single BLE event listener that forwards packets to WASM
function blePacketHandler(event: Event) {
    const target = event.target as unknown as BluetoothRemoteGATTCharacteristic
    const value = target.value
    if (!value) return

    const bytes = new Uint8Array(value.buffer)
    const timestamp = performance.now() / 1000.0

    // Forward to WASM CubeManager (free function, no struct)
    const actionsJson = cm_process_ble_packet(bytes, timestamp)

    // Notify Vue that WASM state changed (facelets, orientation, device info, etc.)
    _onWasmStateChanged?.()

    if (actionsJson) {
        logger.info(`Core Actions: ${actionsJson}`)
        try {
            const actions = actionsJson.startsWith('[') ? JSON.parse(actionsJson) : [JSON.parse(actionsJson)]
            for (const action of actions) {
                handleCoreAction(action)
            }
        } catch (e) {
            logger.error('Failed to parse Core actions', e)
        }
    }
}

let _lastGyroLog = 0

/// Log every BLE TX as hex (for comparing with CubeEast)
function logBleSend(label: string, bytes: Uint8Array) {
    const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join(' ')
    logger.info(`BLE SEND [${label}] → ${hex}`)
}

/// Handle side effects from Core actions (storage writes only)
async function handleCoreAction(action: any) {
    switch (action.type) {
        case 'SaveSolve': {
            logger.debug('SaveSolve action received', action.data)
            break
        }
        case 'DemoteSession': {
            const storage = await getStorage()
            await storage.demote_session(action.data)
            logger.debug('Session demoted', action.data)
            break
        }
        case 'WriteBack': {
            if (commandCharacteristic) {
                const wbBytes = new Uint8Array(action.data)
                logBleSend('WriteBack', wbBytes)
                await commandCharacteristic.writeValue(wbBytes)
            }
            break
        }
        case 'Battery':
            logger.info(`Battery: ${action.data}%`)
            break
        case 'Hardware':
            logger.info(`Hardware: ${action.data.name} fw=${action.data.swVersion} hw=${action.data.hwVersion} gyro=${action.data.gyroSupported}`)
            break
        case 'RawFacelets':
            logger.info(`Facelets: ${action.data}`)
            break
        case 'Pickup':
            logger.info('Cube picked up')
            break
        case 'GyroRaw': {
            const now = performance.now()
            if (now - _lastGyroLog > 1000) {
                _lastGyroLog = now
                const d = action.data
                logger.info(`GyroRaw hex=[${d.hex}] x=${d.x.toFixed(4)} y=${d.y.toFixed(4)} z=${d.z.toFixed(4)} w=${d.w.toFixed(4)} norm=${d.norm.toFixed(4)}`)
            }
            break
        }
        case 'Error':
            logger.error('Core logic error:', action.data)
            break
        default:
            break
    }
}

/// Connect to a cube via Web Bluetooth
export async function connect(): Promise<{ device: BluetoothDevice; cubeDef: any }> {
    await ensureWasm()
    const serviceUuids = getScanServiceUuids()
    const prefixes = getScanNamePrefixes()
    const nameFilters = prefixes.map((p: string) => ({ namePrefix: p }))

    logger.debug('Scanning with prefixes:', prefixes)

    const device = await (navigator as any).bluetooth.requestDevice({
        filters: nameFilters.length > 0 ? nameFilters : [{ services: [serviceUuids[0]] }],
        optionalServices: serviceUuids
    })

    logger.info(`Selected device: ${device.name} (${device.id})`)

    const cubeDef = device.name ? findCubeByBleName(device.name) : null

    if (!cubeDef) {
        throw new Error(`Unknown cube: ${device.name}. Please ensure your cube is supported.`)
    }

    logger.debug('Cube definition:', cubeDef)
    return { device, cubeDef }
}

/// Finalize connection and set up BLE listener
export async function finalizeConnection(device: BluetoothDevice, cubeDef: any, macAddress: string): Promise<void> {
    const serviceUuid = cubeDef.serviceUuid
    const charUuid = cubeDef.stateCharacteristic
    const cmdCharUuid = cubeDef.commandCharacteristic
    const protocolName = cubeDef.protocol

    logger.debug(`MAC Address: ${macAddress}`)

    // Step 1: Set up protocol in WASM FIRST (before any GATT/BLE operations)
    cm_connect(device.name || 'Unknown Cube', macAddress, protocolName)
    logger.info(`WASM protocol ready: ${protocolName}`)

    // Step 2: Connect GATT and start BLE notifications
    logger.debug(`Connecting to GATT service ${serviceUuid}, characteristic ${charUuid}`)

    try {
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

        // Get command characteristic for WriteBack and handshake
        if (cmdCharUuid) {
            try {
                commandCharacteristic = await service.getCharacteristic(cmdCharUuid)
                logger.debug('Command characteristic obtained')
            } catch (e) {
                logger.warn('Command characteristic not available:', e)
                commandCharacteristic = null
            }
        }

        // Remove old listener if exists
        if (bleCharacteristic) {
            bleCharacteristic.removeEventListener('characteristicvaluechanged', blePacketHandler)
        }

        // Start notifications and register handler
        await characteristic.startNotifications()
        logger.debug('Notifications started')

        characteristic.addEventListener('characteristicvaluechanged', blePacketHandler)
        bleCharacteristic = characteristic

        // Send handshake if protocol requires it (e.g. QiYi)
        if (cm_requires_handshake()) {
            const handshakeData = cm_handshake_data()
            if (handshakeData.length > 0 && commandCharacteristic) {
                const encrypted = new Uint8Array(JSON.parse(cm_encode_command('handshake')))
                logBleSend('handshake', encrypted)
                await commandCharacteristic.writeValue(encrypted)
                logger.info(`Handshake sent for ${protocolName}`)
            }
        }

        // Send initial request commands to activate the cube's notification stream
        // (matches bt-test sequence: wait, then request battery/hardware/facelets)
        if (commandCharacteristic) {
            const delay = (ms: number) => new Promise(r => setTimeout(r, ms))
            await delay(400)
            for (const cmd of ['battery', 'hardware', 'facelets']) {
                try {
                    const bytesJson = cm_encode_command(cmd)
                    const bytes = new Uint8Array(JSON.parse(bytesJson))
                    logBleSend(cmd, bytes)
                    await commandCharacteristic.writeValue(bytes)
                    await delay(200)
                } catch (e) {
                    logger.debug(`Init command ${cmd} not supported: ${e}`)
                }
            }
        }

        // Start periodic battery polling (cube won't push battery on its own)
        startBatteryPoll()

        logger.info(`Connected to ${device.name} (${protocolName})`)
    } catch (error) {
        // GATT failed — disconnect WASM state so we're not in a half-connected state
        cm_disconnect()
        commandCharacteristic = null
        logger.error('GATT connection error:', error)
        logger.error(`Device: ${device.name}, Service: ${serviceUuid}, Char: ${charUuid}`)
        throw error
    }
}

/// Reconnect to a previously paired saved cube via getDevices() (no BLE picker)
export async function reconnectSavedCube(savedCube: SavedCube): Promise<{ device: BluetoothDevice; cubeDef: any }> {
    await ensureWasm()

    const bt = (navigator as any).bluetooth
    if (!bt?.getDevices) {
        throw new Error('getDevices() not supported in this browser')
    }

    const devices: BluetoothDevice[] = await bt.getDevices()
    const device = devices.find((d: BluetoothDevice) => d.name === savedCube.name)
    if (!device) {
        throw new Error('Device not found — needs fresh BLE scan')
    }

    // Connect GATT
    if (!device.gatt) {
        throw new Error('GATT not available on device')
    }
    await device.gatt.connect()

    const cubeDef = findCubeByBleName(device.name || '')
    if (!cubeDef) {
        throw new Error(`Unknown cube: ${device.name}`)
    }

    return { device, cubeDef }
}

async function sendBatteryRequest() {
    if (!commandCharacteristic) return
    try {
        const bytesJson = cm_encode_command('battery')
        const bytes = new Uint8Array(JSON.parse(bytesJson))
        logBleSend('battery-poll', bytes)
        await commandCharacteristic.writeValue(bytes)
    } catch (e) {
        logger.debug('Battery poll failed:', e)
    }
}

function startBatteryPoll() {
    stopBatteryPoll()
    // Request immediately, then every 30s
    sendBatteryRequest()
    batteryPollInterval = setInterval(sendBatteryRequest, 30_000)
}

function stopBatteryPoll() {
    if (batteryPollInterval) {
        clearInterval(batteryPollInterval)
        batteryPollInterval = null
    }
}

/// Disconnect from cube
export async function disconnect(): Promise<void> {
    stopBatteryPoll()

    if (bleCharacteristic) {
        try {
            bleCharacteristic.removeEventListener('characteristicvaluechanged', blePacketHandler)
            await bleCharacteristic.stopNotifications()
        } catch (e) {
            logger.warn('Error stopping BLE notifications', e)
        }
        bleCharacteristic = null
    }

    commandCharacteristic = null
    cm_disconnect()

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
