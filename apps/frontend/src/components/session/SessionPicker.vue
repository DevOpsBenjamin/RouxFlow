<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSessionStore } from '../../stores/session'
import type { SessionType } from '../../stores/session'

const sessionStore = useSessionStore()
const isMenuOpen = ref(false)
const newSessionName = ref('')
const newSessionType = ref<SessionType>('Free')

onMounted(() => {
  // Create a default session if none exist
  if (sessionStore.sessions.length === 0) {
    sessionStore.createSession('General Session', 'Free')
  }
})

function handleCreate() {
  sessionStore.createSession(newSessionName.value, newSessionType.value)
  newSessionName.value = ''
  isMenuOpen.value = false
}
</script>

<template>
  <div class="relative">
    <button 
      @click="isMenuOpen = !isMenuOpen"
      class="flex items-center gap-2 px-4 py-2 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition-all text-sm font-medium"
    >
      <span class="text-indigo-400">📁</span>
      <span>{{ sessionStore.activeSession?.name || 'Sessions' }}</span>
      <span class="text-[10px] opacity-50 px-1.5 py-0.5 rounded-md border border-white/10">
        {{ sessionStore.activeSession?.type }}
      </span>
    </button>

    <!-- Dropdown Menu -->
    <div v-if="isMenuOpen" 
         class="absolute top-full left-0 mt-2 w-64 bg-slate-900 border border-white/10 rounded-2xl shadow-2xl p-4 z-[100] animate-in fade-in slide-in-from-top-2 duration-200">
      
      <div class="space-y-4">
        <div class="text-xs font-bold text-slate-500 uppercase tracking-widest">Select Session</div>
        
        <div class="max-h-48 overflow-y-auto space-y-1">
          <button 
            v-for="session in sessionStore.sessions" 
            :key="session.id"
            @click="sessionStore.switchSession(session.id); isMenuOpen = false"
            :class="['w-full text-left px-3 py-2 rounded-lg text-sm transition-colors', 
                     session.id === sessionStore.activeSessionId ? 'bg-indigo-500/20 text-indigo-300' : 'hover:bg-white/5 text-slate-400']"
          >
            <div class="flex justify-between items-center">
              <span>{{ session.name }}</span>
              <span class="text-[10px] opacity-40">{{ session.session_type }}</span>
            </div>
            <div class="text-[10px] opacity-40">{{ session.solves?.length || 0 }} solves</div>
          </button>
        </div>

        <div class="pt-4 border-t border-white/5 space-y-3">
          <div class="text-xs font-bold text-slate-500 uppercase tracking-widest">New Session</div>
          <input 
            v-model="newSessionName"
            type="text" 
            placeholder="Name (e.g. AO5 Practice)"
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-indigo-500/50 transition-colors"
          />
          <div class="flex gap-2">
            <button 
              @click="newSessionType = 'Free'"
              :class="['flex-1 px-3 py-1.5 rounded-lg text-xs font-bold transition-all', 
                       newSessionType === 'Free' ? 'bg-indigo-500 text-white' : 'bg-white/5 text-slate-500']"
            >
              FREE
            </button>
            <button 
              @click="newSessionType = 'WCA'"
              :class="['flex-1 px-3 py-1.5 rounded-lg text-xs font-bold transition-all', 
                       newSessionType === 'WCA' ? 'bg-amber-500 text-white' : 'bg-white/5 text-slate-500']"
            >
              WCA
            </button>
          </div>
          <button 
            @click="handleCreate"
            class="w-full py-2 bg-indigo-500 hover:bg-indigo-400 text-white rounded-lg text-sm font-bold transition-all"
          >
            Create Session
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
