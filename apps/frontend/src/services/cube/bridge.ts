import init, { handle_ble_packet, SessionManager } from '../../wasm/roux-core/roux_core'
import { SupabaseStorage } from '../../wasm/roux-storage-cloud/roux_storage_cloud'
import { useTimerStore } from '../../stores/timer'
import { useSessionStore } from '../../stores/session'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { SavedCube } from '../../stores/bluetooth'

let wasmInitialized = false
export let sessionManager: SessionManager | null = null
export let cloudStorage: SupabaseStorage | null = null

// Platform detection
export const isTauri = !!(window as any).__TAURI_INTERNALS__

async function getSupabaseConfig() {
    return {
        url: import.meta.env.VITE_SUPABASE_URL,
        key: import.meta.env.VITE_SUPABASE_ANON_KEY
    }
}

export async function ensureWasm() {
    if (!wasmInitialized) {
        await init()
        sessionManager = new SessionManager()
        wasmInitialized = true
    }
}

// --- DRIVERS (Internal Implementation) ---

const tauriDriver = {
    async getCubes(userId: string | null) {
        const res = await invoke<string>('db_get_cubes', { userId })
        return JSON.parse(res) as SavedCube[]
    },
    async saveCube(cube: SavedCube) {
        await invoke('db_save_cube', { cubeJson: JSON.stringify(cube) })
    },
    async deleteCube(id: string) {
        await invoke('db_delete_cube', { id })
    },
    async syncCubes(userId: string) {
        const config = await getSupabaseConfig()
        await invoke('db_sync_cubes', {
            userId,
            url: config.url,
            key: config.key
        })
    },
    async getSessions() {
        const json = await invoke('db_get_sessions') as string
        return JSON.parse(json)
    },
    async createSession(session: any) {
        await invoke('db_create_session', { sessionJson: JSON.stringify(session) })
    }
}

const wasmDriver = {
    async getClient() {
        await ensureWasm()
        if (!cloudStorage) {
            const config = await getSupabaseConfig()
            cloudStorage = new SupabaseStorage(config.url, config.key)
        }
        return cloudStorage
    },
    async getCubes(userId: string) {
        const client = await this.getClient()
        const json = await client.get_cubes_json(userId)
        return JSON.parse(json) as SavedCube[]
    },
    async saveCube(cube: SavedCube) {
        const client = await this.getClient()
        await client.save_cube_json(JSON.stringify(cube))
    },
    async deleteCube(id: string, userId: string) {
        const client = await this.getClient()
        await client.delete_cube_json(id, userId)
    }
}

export class CubeBridge {
    static async processPacket(dataView: DataView) {
        const bytes = new Uint8Array(dataView.buffer)
        await this.processRawPacket(bytes)
    }

    static async processRawPacket(bytes: Uint8Array) {
        await ensureWasm()
        if (!sessionManager) return

        // 1. Action: Vue -> Bridge -> Core (Logic Execution)
        const eventJson = handle_ble_packet(bytes, sessionManager)

        if (eventJson) {
            try {
                // 2. Logic: Core returns an ActionRequest
                const action = JSON.parse(eventJson)

                // 3. Execution: Bridge executes the result
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
                break;
            case 'SaveSolve':
                // Bridge delegates to Storage Implementation
                if (isTauri) {
                    await invoke('db_save_solve', {
                        sessionId: (window as any).activeSessionId,
                        solveJson: JSON.stringify(action.data)
                    })
                    await sessionStore.loadSessions()
                } else {
                    // Web API fallback here
                    console.log('[Web] Would save solve via API:', action.data)
                }
                break;
            case 'DemoteSession':
                if (isTauri) {
                    await invoke('db_demote_session', { id: action.data })
                    await sessionStore.loadSessions()
                }
                break;
            case 'Pickup':
                if (timer.flowState === 'Ready') {
                    await ensureWasm()
                    const actionJson = sessionManager?.set_solving()
                    if (actionJson) this.handleCoreAction(JSON.parse(actionJson))
                }
                timer.handleEvent('pickup')
                break;
            case 'Putdown':
                if (timer.flowState === 'Solving') {
                    const actionJson = sessionManager?.record_solve(timer.time, JSON.stringify(timer.currentMoves))
                    if (actionJson) this.handleCoreAction(JSON.parse(actionJson))
                }
                timer.handleEvent('putdown')
                break;
            case 'Move':
                timer.handleEvent('move', action.data)
                break;
            case 'Error':
                console.error('Core logic error:', action.data)
                break;
        }
    }

    // Bluetooth abstraction
    static async connect(): Promise<string> {
        const { useBluetoothStore } = await import('../../stores/bluetooth')
        const bt = useBluetoothStore()

        if (isTauri) {
            console.log('[Bridge] Starting Tauri Native Scan')
            try {
                bt.startScan()

                // First check if Bluetooth adapter is available
                // We do this AFTER starting scan UI so the error can be shown in the modal
                await invoke('ble_check_available')

                await invoke('ble_scan')

                // Timeout after 10 seconds of scanning
                const timeoutId = setTimeout(() => {
                    if (bt.isScanning && bt.scannedDevices.length === 0) {
                        bt.stopScan()
                    }
                }, 10000)

                // Poll for devices every 2 seconds while scanner is up
                const pollInterval = setInterval(async () => {
                    if (!bt.showPicker) {
                        clearInterval(pollInterval)
                        clearTimeout(timeoutId)
                        return
                    }
                    try {
                        const devices: any = await invoke('ble_list_devices')
                        bt.setDevices(devices)

                        // If devices found, we can stop the automatic "no devices" timeout
                        if (devices.length > 0) {
                            clearTimeout(timeoutId)
                        }
                    } catch (e) {
                        console.error('Scan polling failed', e)
                    }
                }, 2000)

                return 'Scanning...'
            } catch (e: any) {
                const errorMessage = typeof e === 'string' ? e : (e.message || 'Bluetooth initialization failed')
                bt.setError(errorMessage)
                throw e
            }
        } else {
            // Web Bluetooth
            const GAN_SERVICE_UUID = '0000fe51-0000-1000-8000-00805f9b34fb'
            const GAN_CHARACTERISTIC_UUID = '0000fe52-0000-1000-8000-00805f9b34fb'

            const device = await (navigator as any).bluetooth.requestDevice({
                filters: [{ services: [GAN_SERVICE_UUID] }],
                optionalServices: [GAN_SERVICE_UUID]
            })

            const server = await device.gatt?.connect()
            const service = await server?.getPrimaryService(GAN_SERVICE_UUID)
            const characteristic = await service?.getCharacteristic(GAN_CHARACTERISTIC_UUID)

            await characteristic?.startNotifications()
            characteristic?.addEventListener('characteristicvaluechanged', (event: any) => {
                const value = event.target.value
                this.processPacket(value)
            })

            return device.name || 'Smart Cube'
        }
    }

    static async getSessions(): Promise<any[]> {
        if (isTauri) return tauriDriver.getSessions()
        return []
    }

    static async createSession(session: any) {
        if (isTauri) return tauriDriver.createSession(session)
    }

    static handleSyncEvent(callback: (devices: any[]) => void) {
        if (isTauri) {
            listen('ble-devices', (event: any) => {
                callback(event.payload)
            })
        }
    }

    static async finalConnect(device: any): Promise<void> {
        await ensureWasm()
        if (isTauri) {
            await invoke('ble_connect', { id: device.id })
            listen('ble-packet', (event: any) => {
                this.processRawPacket(new Uint8Array(event.payload as number[]))
            })
        }
    }

    static async getCubes(userId: string | null = null): Promise<SavedCube[]> {
        if (isTauri) return tauriDriver.getCubes(userId)
        if (userId) return wasmDriver.getCubes(userId)
        return []
    }

    static async saveCube(cube: SavedCube) {
        return isTauri ? tauriDriver.saveCube(cube) : (cube.user_id ? wasmDriver.saveCube(cube) : null)
    }

    static async deleteCube(id: string, userId: string | null = null) {
        return isTauri ? tauriDriver.deleteCube(id) : (userId ? wasmDriver.deleteCube(id, userId) : null)
    }

    static async syncCubes(userId: string) {
        if (isTauri) return tauriDriver.syncCubes(userId)
        console.log('[Bridge] Sync not needed on Web')
    }
}
