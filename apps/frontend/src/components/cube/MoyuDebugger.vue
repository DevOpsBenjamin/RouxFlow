<script setup lang="ts">
import { ref } from 'vue'
import CryptoJS from 'crypto-js'

const status = ref('Prêt')
const logs = ref<any[]>([])
const device = ref<any>(null)
let characteristicRead: any = null
let characteristicWrite: any = null

// CONFIG IDENTIQUE AU FICHIER HTML
const DEVICE_MAC = 'CF:30:16:01:C7:2F'
const BASE_KEY = [176, 81, 104, 224, 86, 180, 233, 8, 44, 0, 216, 152, 96, 156, 16, 172]
const BASE_IV = [22, 228, 96, 236, 12, 202, 210, 172, 208, 28, 212, 129, 96, 168, 21, 0]

const serviceUUID = '0783b03e-7735-b5a0-1760-a305d2795cb0'
const readUUID = '0783b03e-7735-b5a0-1760-a305d2795cb1'
const writeUUID = '0783b03e-7735-b5a0-1760-a305d2795cb2'

let aesKey: any = null
let aesIv: any = null

function log(msg: string, type = 'info') {
  logs.value.unshift({ time: new Date().toLocaleTimeString(), msg, type })
}

function initDecoder(mac: string) {
  log(`🔑 Dérivation Clé pour MAC: ${mac}`)
  const macBytes = mac.split(':').map(h => parseInt(h, 16))
  const key = [...BASE_KEY]
  const iv = [...BASE_IV]
  
  for (let i = 0; i < 6; i++) {
    key[i] = (key[i] + macBytes[5 - i]) % 255
    iv[i] = (iv[i] + macBytes[5 - i]) % 255
  }
  
  aesKey = CryptoJS.lib.WordArray.create(new Uint8Array(key) as any)
  aesIv = CryptoJS.lib.WordArray.create(new Uint8Array(iv) as any)
}

function decrypt(data: Uint8Array): Uint8Array {
  if (!aesKey) return data
  const ciphertext = CryptoJS.lib.WordArray.create(data as any)
  const decrypted = CryptoJS.AES.decrypt({ ciphertext } as any, aesKey, { 
    iv: aesIv, 
    mode: CryptoJS.mode.CBC, 
    padding: CryptoJS.pad.NoPadding 
  })
  
  const words = decrypted.words
  const result = []
  for (let i = 0; i < data.length; i++) {
     const w = words[Math.floor(i/4)]
     const b = (w >>> (24 - (i%4)*8)) & 0xff
     result.push(b)
  }
  return new Uint8Array(result)
}

async function connect() {
  try {
    status.value = 'Recherche...'
    device.value = await (navigator as any).bluetooth.requestDevice({
      filters: [{ namePrefix: 'WCU_MY32' }],
      optionalServices: [serviceUUID]
    })
    
    log(`✅ Appareil sélectionné: ${device.value.name}`)
    const server = await device.value.gatt!.connect()
    const service = await server.getPrimaryService(serviceUUID)
    
    characteristicRead = await service.getCharacteristic(readUUID)
    characteristicWrite = await service.getCharacteristic(writeUUID)
    
    initDecoder(DEVICE_MAC)
    
    let rxCount = 0
    await characteristicRead.startNotifications()
    characteristicRead.addEventListener('characteristicvaluechanged', (e: any) => {
      rxCount++
      const val = new Uint8Array(e.target.value.buffer)
      if (val.length >= 20) {
        const dec = decrypt(val)
        const hex = Array.from(dec).map(b => b.toString(16).padStart(2, '0')).join(' ')
        log(`RX #${rxCount}: ${hex}`, 'rx')
      } else {
        log(`RX RAW #${rxCount}: ${Array.from(val).map(b => b.toString(16).padStart(2, '0')).join(' ')}`, 'rx')
      }

      if (rxCount >= 100) {
        log("🛑 Limite de 100 paquets atteinte. Déconnexion automatique...", 'err')
        device.value.gatt.disconnect()
        status.value = 'Déconnecté (Auto-Stop 100 RX)'
      }
    })
    
    status.value = 'Connecté - Envoi Séquence D...'
    
    // Séquence D: 1, 2, 3, 4
    const p1 = new Uint8Array([0x95, 0x53, 0x0d, 0x6c, 0xdc, 0x06, 0xc3, 0x25, 0xbc, 0x21, 0xdb, 0x70, 0xa6, 0x4f, 0xe4, 0x00, 0x3d, 0x98, 0x0c, 0x5f]);
    const p2 = new Uint8Array([0x92, 0x93, 0xa7, 0xd6, 0x36, 0x62, 0x51, 0x7d, 0x8d, 0xdd, 0xa7, 0x53, 0x30, 0x3b, 0x9a, 0xa4, 0x69, 0xed, 0x6a, 0xa0]);
    const p3 = new Uint8Array([0xcd, 0xd8, 0x21, 0x93, 0x3e, 0x79, 0xa9, 0x6c, 0x92, 0x4f, 0x57, 0x4a, 0x1c, 0xc4, 0xa8, 0xd8, 0x09, 0xea, 0x8f, 0xee]);
    const p4 = new Uint8Array([0x90, 0x5c, 0x36, 0x16, 0x3b, 0x6c, 0xbf, 0x34, 0x8c, 0x8b, 0x54, 0xf7, 0xa4, 0xf3, 0x7f, 0xca, 0xa8, 0x61, 0x10, 0xff]);

    await sendRaw(p1);
    setTimeout(() => sendRaw(p2), 300);
    setTimeout(() => sendRaw(p3), 600);
    setTimeout(() => {
        sendRaw(p4);
        status.value = 'Flux Actif (Seq D) !';
    }, 900);

  } catch (e: any) {
    status.value = 'Erreur: ' + e.message
    log(e.message, 'err')
  }
}

async function sendRaw(data: Uint8Array) {
    if (!characteristicWrite) return
    log(`TX RAW: ${Array.from(data).map(b => b.toString(16).padStart(2, '0')).join(' ')}`, 'tx')
    await characteristicWrite.writeValue(data)
}

const customKeyInput = ref(BASE_KEY.join(', '))

function updateKey() {
    try {
        const kStr = customKeyInput.value;
        const clean = kStr.replace(/[\[\]\s]/g, ',').split(',').filter(x => x).map(x => {
            if (x.startsWith('0x')) return parseInt(x, 16);
            return parseInt(x.trim());
        })
        if (clean.length === 16) {
             const key = new Uint8Array(clean)
             aesKey = CryptoJS.lib.WordArray.create(key as any)
             log("Nouvelle clé AES appliquée !", 'info')
        } else {
            log(`Erreur: Clé invalide (${clean.length} octets)`, 'err')
        }
    } catch(e: any) {
        log(`Erreur: ${e.message}`, 'err')
    }
}

</script>

<template>
  <div class="p-8 max-w-2xl mx-auto bg-slate-900 text-white min-h-screen">
    <h1 class="text-3xl font-bold mb-6 text-blue-400 text-center">MoYu V10 Debugger (V3)</h1>
    
    <div class="bg-slate-800 p-6 rounded-lg border border-slate-700 mb-6">
      <div class="flex justify-between items-center mb-4">
        <div>
           <div class="text-sm text-slate-400 uppercase tracking-wider mb-1">Status</div>
           <div class="font-mono text-xl" :class="status.includes('Actif') ? 'text-green-400' : 'text-blue-300'">{{ status }}</div>
        </div>
        <button @click="connect" class="bg-blue-600 hover:bg-blue-500 px-6 py-3 rounded-md font-bold transition-all shadow-lg active:scale-95">
          CONNECTER LE CUBE
        </button>
      </div>
      <div class="text-xs text-slate-500 font-mono">MAC: {{ DEVICE_MAC }}</div>
    </div>

    <div class="bg-slate-800 p-6 rounded-lg border border-slate-700 mb-6">
        <label class="block text-sm font-semibold text-slate-300 mb-2">Clé AES Custom</label>
        <div class="flex gap-2">
            <input 
              v-model="customKeyInput"
              type="text" 
              class="bg-slate-950 border border-slate-700 text-white p-3 rounded-md flex-grow font-mono text-sm focus:border-blue-500 outline-none" 
              placeholder="176, 81, 104..."
            >
            <button @click="updateKey" class="bg-slate-700 hover:bg-slate-600 px-4 py-2 rounded-md text-sm transition-colors">
                Appliquer
            </button>
        </div>
        <p class="text-[10px] text-slate-500 mt-2 italic">Format: décimal (176, 81...) ou hex (0xb0, 0x51...)</p>
    </div>

    <div class="bg-slate-950 rounded-lg border border-slate-800 p-4 h-[500px] overflow-y-auto font-mono text-xs shadow-inner">
      <div v-for="(l, i) in logs" :key="i" class="mb-1 py-1 border-b border-slate-900 last:border-0" :class="{
        'text-green-400': l.type === 'rx',
        'text-red-400': l.type === 'err',
        'text-blue-400': l.type === 'tx',
        'text-slate-400': l.type === 'info'
      }">
        <span class="text-slate-600">[{{ l.time }}]</span> 
        <span class="font-bold mr-2">{{ l.type.toUpperCase() }}:</span>
        {{ l.msg }}
      </div>
      <div v-if="logs.length === 0" class="text-slate-700 text-center mt-20">En attente de connexion...</div>
    </div>
  </div>
</template>
