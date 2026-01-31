<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '../../stores/auth'
import { useUIStore } from '../../stores/ui'

const auth = useAuthStore()
const ui = useUIStore()

const newDisplayName = ref(auth.user?.user_metadata?.display_name || auth.user?.user_metadata?.full_name || '')
const isSaving = ref(false)
const message = ref('')
const error = ref('')

async function handleSave() {
  if (!newDisplayName.value.trim()) return
  
  isSaving.value = true
  message.value = ''
  error.value = ''
  
  try {
    await auth.updateDisplayName(newDisplayName.value.trim())
    message.value = 'Profile updated successfully!'
  } catch (e: any) {
    error.value = e.message || 'Failed to update profile'
  } finally {
    isSaving.value = false
  }
}

function goBack() {
  ui.setHome()
}
</script>

<template>
  <div class="w-full max-w-2xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
    <div class="flex items-center gap-4">
      <button @click="goBack" class="p-2 hover:bg-white/5 rounded-full transition-colors text-slate-400 hover:text-white">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h2 class="text-3xl font-bold text-white">Your Profile</h2>
    </div>

    <div class="bg-slate-900/50 border border-white/5 rounded-3xl p-8 space-y-6 backdrop-blur-sm">
      <div v-if="message" class="p-4 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-xl text-sm">
        {{ message }}
      </div>
      <div v-if="error" class="p-4 bg-red-500/10 border border-red-500/20 text-red-400 rounded-xl text-sm">
        {{ error }}
      </div>

      <div class="space-y-2">
        <label class="text-sm font-medium text-slate-400 ml-1">Email</label>
        <div class="px-4 py-3 bg-slate-800/50 border border-white/5 rounded-xl text-slate-500 select-none">
          {{ auth.user?.email }}
        </div>
        <p class="text-xs text-slate-500 ml-1 italic">Email cannot be changed yet.</p>
      </div>

      <form @submit.prevent="handleSave" class="space-y-4">
        <div class="space-y-2">
          <label for="displayName" class="text-sm font-medium text-slate-400 ml-1">Display Name</label>
          <input 
            id="displayName"
            v-model="newDisplayName"
            type="text" 
            placeholder="How others see you..."
            class="w-full px-4 py-3 bg-slate-800 border border-white/10 rounded-xl text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all"
          />
        </div>

        <button 
          type="submit"
          :disabled="isSaving || !newDisplayName.trim()"
          class="w-full py-4 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white font-bold rounded-xl transition-all shadow-lg shadow-indigo-500/20 disabled:opacity-50 disabled:cursor-not-allowed transform active:scale-95"
        >
          {{ isSaving ? 'Saving...' : 'Save Profile' }}
        </button>
      </form>
    </div>

    <div class="bg-slate-900/30 border border-white/5 rounded-3xl p-6 flex items-center justify-between">
      <div class="space-y-1">
        <h3 class="text-white font-medium">Session Settings</h3>
        <p class="text-xs text-slate-500">More settings coming soon...</p>
      </div>
      <button @click="auth.signOut(); ui.setLanding()" class="px-6 py-2.5 bg-red-500/10 hover:bg-red-500/20 text-red-400 text-sm font-medium rounded-xl border border-red-500/20 transition-all">
        {{ auth.isAuthenticated ? 'Sign Out' : 'Exit Guest Mode' }}
      </button>
    </div>
  </div>
</template>
