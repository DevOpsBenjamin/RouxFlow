<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  show: boolean
  deviceName: string
}>()

const emit = defineEmits<{
  (e: 'submit', macAddress: string): void
  (e: 'cancel'): void
}>()

const macInput = ref('')
const error = ref('')

// Format MAC as user types (auto-add colons)
function formatMacInput(value: string) {
  // Remove all non-hex characters
  const cleaned = value.replace(/[^0-9A-Fa-f]/g, '').toUpperCase()

  // Add colons every 2 characters
  const parts = cleaned.match(/.{1,2}/g) || []
  macInput.value = parts.slice(0, 6).join(':')
}

const isValidMac = computed(() => {
  const macRegex = /^([0-9A-Fa-f]{2}:){5}([0-9A-Fa-f]{2})$/
  return macRegex.test(macInput.value)
})

function handleSubmit() {
  if (!isValidMac.value) {
    error.value = 'Invalid MAC address format. Expected: XX:XX:XX:XX:XX:XX'
    return
  }
  error.value = ''
  emit('submit', macInput.value)
}

function handleCancel() {
  macInput.value = ''
  error.value = ''
  emit('cancel')
}

// Auto-format on input
function onInput(event: Event) {
  const target = event.target as HTMLInputElement
  formatMacInput(target.value)
}
</script>

<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div
      v-if="show"
      class="fixed inset-0 z-[110] bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4"
      @click.self="handleCancel"
    >
      <div class="bg-slate-900 border border-slate-800 rounded-2xl p-8 max-w-md w-full shadow-2xl">
        <div class="text-center space-y-4 mb-6">
          <div class="w-16 h-16 mx-auto rounded-full bg-indigo-500/10 flex items-center justify-center">
            <svg class="w-8 h-8 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>

          <div>
            <h3 class="text-xl font-bold text-white mb-2">MAC Address Required</h3>
            <p class="text-sm text-slate-400">
              <span class="font-semibold text-indigo-400">{{ deviceName }}</span> uses encrypted communication.
            </p>
            <p class="text-sm text-slate-400 mt-2">
              Please enter your cube's MAC address for decryption.
            </p>
          </div>
        </div>

        <!-- MAC Input -->
        <div class="space-y-3">
          <label class="block">
            <span class="text-sm font-semibold text-slate-300 mb-2 block">MAC Address</span>
            <input
              v-model="macInput"
              @input="onInput"
              type="text"
              placeholder="XX:XX:XX:XX:XX:XX"
              maxlength="17"
              class="w-full px-4 py-3 rounded-xl bg-slate-800 border border-slate-700 text-white font-mono text-center text-lg focus:outline-none focus:border-indigo-500 focus:ring-2 focus:ring-indigo-500/20 transition-all"
              autofocus
            />
          </label>

          <!-- Helper Text -->
          <div class="bg-slate-800/50 rounded-lg p-3 text-xs text-slate-400 space-y-2">
            <p class="font-semibold text-slate-300">How to find your MAC address:</p>
            <ul class="list-disc list-inside space-y-1 ml-2">
              <li class="font-medium text-indigo-400">Chrome: Open <code class="bg-slate-700 px-1 rounded">chrome://bluetooth-internals/#devices</code></li>
              <li>Check the cube's packaging or manual</li>
              <li>Use a Bluetooth scanner app on your phone</li>
              <li>Look for a sticker on the cube itself</li>
              <li>Format: XX:XX:XX:XX:XX:XX (6 pairs of hex digits)</li>
            </ul>
            <p class="text-slate-500 italic mt-2">💡 You only need to enter this once - it will be saved for future connections</p>
          </div>

          <!-- Error Message -->
          <p v-if="error" class="text-sm text-red-400 text-center">
            {{ error }}
          </p>

          <!-- Status Indicator -->
          <div v-if="macInput && !error" class="flex items-center justify-center gap-2 text-sm">
            <svg v-if="isValidMac" class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <span :class="isValidMac ? 'text-emerald-400' : 'text-slate-500'">
              {{ isValidMac ? 'Valid format' : 'Keep typing...' }}
            </span>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex gap-3 mt-6">
          <button
            @click="handleCancel"
            class="flex-1 py-3 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 font-semibold text-sm transition-colors"
          >
            Cancel
          </button>
          <button
            @click="handleSubmit"
            :disabled="!isValidMac"
            class="flex-1 py-3 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:bg-slate-800 disabled:text-slate-600 disabled:cursor-not-allowed text-white font-semibold text-sm transition-colors"
          >
            Connect
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>
