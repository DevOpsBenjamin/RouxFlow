<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'

const router = useRouter()
const auth = useAuthStore()

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
  router.push({ name: 'Home' })
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

    <div class="bg-slate-900/40 border border-white/5 rounded-3xl p-6 space-y-4">
      <div class="flex items-start justify-between gap-4">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="text-lg">💬</span>
            <h3 class="text-white font-bold">Feedback & Support</h3>
          </div>
          <p class="text-xs text-slate-400 max-w-md leading-relaxed">
            Real cuber using RouxFlow? We want your feedback! Report bugs, suggest features, or request smart cube support to help prioritize development.
          </p>
        </div>
      </div>
      <div class="flex flex-wrap gap-3 pt-2">
        <a
          href="https://github.com/DevOpsBenjamin/RouxFlow/issues/new"
          target="_blank"
          rel="noopener noreferrer"
          class="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition-all shadow-md shadow-indigo-600/20 flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
          </svg>
          Open a GitHub Issue
        </a>
        <a
          href="https://github.com/DevOpsBenjamin/RouxFlow"
          target="_blank"
          rel="noopener noreferrer"
          class="px-4 py-2.5 bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 hover:text-white text-xs font-semibold rounded-xl transition-all flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"/>
          </svg>
          GitHub Repository
        </a>
      </div>
    </div>

    <div class="bg-slate-900/30 border border-white/5 rounded-3xl p-6 flex items-center justify-between">
      <div class="space-y-1">
        <h3 class="text-white font-medium">Session Settings</h3>
        <p class="text-xs text-slate-500">More settings coming soon...</p>
      </div>
      <button @click="auth.signOut(); router.push({ name: 'Landing' })" class="px-6 py-2.5 bg-red-500/10 hover:bg-red-500/20 text-red-400 text-sm font-medium rounded-xl border border-red-500/20 transition-all">
        {{ auth.isAuthenticated ? 'Sign Out' : 'Exit Guest Mode' }}
      </button>
    </div>
  </div>
</template>
