import init, {
    handle_ble_packet,
    encode_cube_command,
    SessionManager,
    WasmStorageManager,
    init_renderer,
    set_gyro_enabled,
    update_render_state,
    reset_gyro,
    find_cube_by_ble_name,
    all_scan_service_uuids,
    all_scan_name_prefixes,
} from '../../wasm/rouxflow/rouxflow_wasm'
import { useTimerStore } from '../../stores/timer'
import { useSessionStore } from '../../stores/session'
import type { SavedCube } from '../../stores/bluetooth'

let wasmInitialized = false
export let sessionManager: SessionManager | null = null
let storageManager: WasmStorageManager | null = null

export async function ensureWasm() {
    if (!wasmInitialized) {
        await init()
        sessionManager = new SessionManager()
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
    encode_cube_command,
    find_cube_by_ble_name,
    all_scan_service_uuids,
    all_scan_name_prefixes,
}

export class CubeBridge {
    static async processPacket(dataView: DataView) {
        const bytes = new Uint8Array(dataView.buffer)
        await this.processRawPacket(bytes)
    }

    static async processRawPacket(bytes: Uint8Array, deviceId: string = 'web-bluetooth-device') {
        await ensureWasm()
        if (!sessionManager) return

        const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join(' ')
        console.log(`[Bridge] Raw Packet (${deviceId}): ${hex}`)

        const eventJson = handle_ble_packet(bytes, deviceId, sessionManager)

        if (eventJson) {
            console.log(`[Bridge] Core Result: ${eventJson}`)
            try {
                const action = JSON.parse(eventJson)
                await this.handleCoreAction(action)
            } catch (e) {
                console.error('Failed to parse Core event', e)
            }
        }
    }

    static async handleCoreAction(action: any) {
        const timer = useTimerStore()
        const sessionStore = useSessionStore()

        switch (action.type) {
            case 'FlowStateChanged':
                timer.flowState = action.data
                if (action.data === 'Solving') {
                    timer.startTimer()
                } else if (action.data === 'Finished') {
                    timer.stopTimer()
                }
                break
            case 'SaveSolve': {
                const storage = await getStorage()
                const activeId = sessionStore.activeSessionId
                if (activeId) {
                    await storage.save_solve_json(activeId, JSON.stringify(action.data))
                    await sessionStore.loadSessions()
                }
                break
            }
            case 'DemoteSession': {
                const storage = await getStorage()
                await storage.demote_session(action.data)
                await sessionStore.loadSessions()
                break
            }
            case 'Pickup':
                if (timer.flowState === 'Ready') {
                    await ensureWasm()
                    const actionJson = sessionManager?.set_solving()
                    if (actionJson) this.handleCoreAction(JSON.parse(actionJson))
                }
                timer.handleEvent('pickup')
                break
            case 'Putdown':
                if (timer.flowState === 'Solving') {
                    const actionJson = sessionManager?.record_solve(timer.time, JSON.stringify(timer.currentMoves))
                    if (actionJson) this.handleCoreAction(JSON.parse(actionJson))
                }
                timer.handleEvent('putdown')
                break
            case 'Move':
                timer.handleEvent('move', action.data)
                break
            case 'Error':
                console.error('Core logic error:', action.data)
                break
        }
    }

    // Web Bluetooth connection
    static async connect(): Promise<string> {
        await ensureWasm()
        const serviceUuids = all_scan_service_uuids()

        // Build filters from known cube name prefixes
        const prefixes = all_scan_name_prefixes()
        const nameFilters = prefixes.map((p: string) => ({ namePrefix: p }))

        const device = await (navigator as any).bluetooth.requestDevice({
            filters: nameFilters.length > 0 ? nameFilters : [{ services: [serviceUuids[0]] }],
            optionalServices: serviceUuids
        })

        return device.name || 'Smart Cube'
    }

    static async finalConnect(device: any): Promise<void> {
        await ensureWasm()

        // Look up the cube definition from its BLE name
        const cubeDef = device.name ? find_cube_by_ble_name(device.name) : null
        const serviceUuid = cubeDef?.serviceUuid || '0000fe51-0000-1000-8000-00805f9b34fb'
        const charUuid = cubeDef?.stateCharacteristic || '0000fe52-0000-1000-8000-00805f9b34fb'

        const server = await device.gatt?.connect()
        const service = await server?.getPrimaryService(serviceUuid)
        const characteristic = await service?.getCharacteristic(charUuid)

        await characteristic?.startNotifications()
        characteristic?.addEventListener('characteristicvaluechanged', (event: any) => {
            const value = event.target.value
            this.processPacket(value)
        })
    }

    // --- Storage operations (delegated to WASM StorageManager) ---

    static async getSessions(): Promise<any[]> {
        const storage = await getStorage()
        const json = await storage.get_sessions_json()
        return JSON.parse(json)
    }

    static async createSession(session: any) {
        const storage = await getStorage()
        await storage.create_session_json(JSON.stringify(session))
    }

    static async getCubes(userId: string | null = null): Promise<SavedCube[]> {
        const storage = await getStorage()
        const json = await storage.get_cubes_json(userId ?? undefined)
        return JSON.parse(json)
    }

    static async saveCube(cube: SavedCube) {
        const storage = await getStorage()
        await storage.save_cube_json(JSON.stringify(cube))
    }

    static async deleteCube(id: string, userId: string | null = null) {
        const storage = await getStorage()
        await storage.delete_cube(id, userId || '')
    }

    static async syncCubes(userId: string) {
        const storage = await getStorage()
        await storage.sync_cubes(userId)
    }
}
