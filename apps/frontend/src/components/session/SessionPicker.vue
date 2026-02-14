<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSessionStore } from '../../stores/session'
import type { SessionType } from '../../stores/session'
import WcaRulesModal from './WcaRulesModal.vue'
import SessionSettingsModal from './SessionSettingsModal.vue'

const sessionStore = useSessionStore()
const isMenuOpen = ref(false)
const newSessionName = ref('')
const newSessionType = ref<SessionType>('Free')
const showWcaRules = ref(false)
const showSettings = ref(false)

const isActiveWCA = computed(() => sessionStore.activeSession?.session_type === 'WCA')

function handleCreate() {
  const isWCA = newSessionType.value === 'WCA'
  sessionStore.createSession(newSessionName.value, newSessionType.value)
  newSessionName.value = ''
  isMenuOpen.value = false

  // Show WCA rules on first WCA session creation
  if (isWCA && !localStorage.getItem('rouxflow_wca_rules_seen')) {
    localStorage.setItem('rouxflow_wca_rules_seen', '1')
    showWcaRules.value = true
  }
}
</script>

<template>
  <div class="relative flex items-center gap-[1vmin]">
    <button
      @click="isMenuOpen = !isMenuOpen"
      class="flex items-center gap-2 px-4 py-2 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition-all text-sm font-medium"
    >
      <span class="text-indigo-400">📁</span>
      <span>{{ sessionStore.activeSession?.name || 'Sessions' }}</span>
      <span class="text-[10px] opacity-50 px-1.5 py-0.5 rounded-md border border-white/10">
        {{ sessionStore.activeSession?.session_type }}
      </span>
    </button>

    <!-- WCA info icon -->
    <button
      v-if="isActiveWCA"
      @click="showWcaRules = true"
      class="w-[3.5vmin] h-[3.5vmin] flex items-center justify-center rounded-[0.8vmin] bg-amber-500/10 border border-amber-500/20 text-amber-400 hover:bg-amber-500/20 transition-all"
      title="WCA Rules"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-[2vmin] h-[2vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
        <path stroke-linecap="round" stroke-linejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
      </svg>
    </button>

    <!-- Free session settings icon -->
    <button
      v-else
      @click="showSettings = true"
      class="w-[3.5vmin] h-[3.5vmin] flex items-center justify-center rounded-[0.8vmin] bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 transition-all"
      title="Session Settings"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-[2vmin] h-[2vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 010-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28z" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
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

    <!-- Modals -->
    <WcaRulesModal :open="showWcaRules" @close="showWcaRules = false" />
    <SessionSettingsModal :open="showSettings" @close="showSettings = false" />
  </div>
</template>
