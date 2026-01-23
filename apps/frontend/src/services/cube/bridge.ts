import init, { handle_ble_packet, SessionManager, SessionEvent } from '../../wasm/roux-core/roux_core'
import { useTimerStore } from '../../stores/timer'

let wasmInitialized = false
export let sessionManager: SessionManager | null = null

export async function ensureWasm() {
    if (!wasmInitialized) {
        await init()
        sessionManager = new SessionManager()
        wasmInitialized = true
    }
}

export class CubeBridge {
    static async processPacket(dataView: DataView) {
        await ensureWasm()
        if (!sessionManager) return

        const bytes = new Uint8Array(dataView.buffer)

        // Forward to Rust with session manager
        const event = handle_ble_packet(bytes, sessionManager) as SessionEvent | undefined

        if (event) {
            const timer = useTimerStore()
            timer.handleEvent(event.event_type, event.data)
        }
    }
}
