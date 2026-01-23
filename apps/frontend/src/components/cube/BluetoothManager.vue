<script setup lang="ts">
import { ref } from 'vue'
import { CubeBridge } from '../../services/cube/bridge'

const isConnecting = ref(false)
const deviceName = ref('')

const GAN_SERVICE_UUID = '0000fe51-0000-1000-8000-00805f9b34fb'
const GAN_CHARACTERISTIC_UUID = '0000fe52-0000-1000-8000-00805f9b34fb'

async function connect() {
  try {
    isConnecting.value = true
    const device = await navigator.bluetooth.requestDevice({
      filters: [{ services: [GAN_SERVICE_UUID] }],
      optionalServices: [GAN_SERVICE_UUID]
    })

    deviceName.value = device.name || 'Smart Cube'
    const server = await device.gatt?.connect()
    const service = await server?.getPrimaryService(GAN_SERVICE_UUID)
    const characteristic = await service?.getCharacteristic(GAN_CHARACTERISTIC_UUID)

    await characteristic?.startNotifications()
    characteristic?.addEventListener('characteristicvaluechanged', (event: any) => {
      const value = event.target.value
      CubeBridge.processPacket(value)
    })

    console.log('Connected to', deviceName.value)
  } catch (error) {
    console.error('Bluetooth Error:', error)
  } finally {
    isConnecting.value = false
  }
}
</script>

<template>
  <button 
    @click="connect"
    :disabled="isConnecting"
    class="px-4 py-2 rounded-full border border-white/10 hover:bg-white/5 transition-all text-sm font-medium text-slate-200 disabled:opacity-50"
  >
    {{ isConnecting ? 'Connecting...' : deviceName || 'Connect Cube' }}
  </button>
</template>
