<script setup lang="ts">
defineProps<{ open: boolean }>()
const emit = defineEmits<{ (e: 'close'): void }>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200" @click.self="emit('close')">
      <div class="bg-slate-900 border border-amber-500/20 rounded-[3vmin] p-[4vmin] max-w-[60vw] max-h-[80vh] overflow-y-auto shadow-2xl animate-in zoom-in-95 duration-300">
        <div class="flex items-center justify-between mb-[3vh]">
          <h2 class="text-[3vmin] font-black text-amber-400 uppercase tracking-wider">WCA Integrity Mode</h2>
          <button @click="emit('close')" class="text-slate-500 hover:text-white transition-colors p-[1vmin]">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-[3vmin] h-[3vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <p class="text-slate-400 text-[1.8vmin] mb-[1.5vh] leading-relaxed">
          RouxFlow's leaderboard features Ao5 averages from WCA sessions. To keep results fair and comparable for everyone, WCA Integrity Mode enforces a set of rules inspired by real competition conditions.
        </p>
        <p class="text-slate-500 text-[1.5vmin] mb-[1.5vh] leading-relaxed">
          In a real WCA competition, you never scramble your own cube — a trained scrambler does it for you behind a screen. You only see the puzzle once it's handed to you, and then you have 15 seconds to inspect before solving.
        </p>
        <p class="text-slate-500 text-[1.5vmin] mb-[3vh] leading-relaxed">
          Since RouxFlow requires you to scramble yourself, these rules recreate that separation. The goal is to make the scramble phase a quick, mechanical execution — not an opportunity to study the puzzle. Every rule below exists so that when your Ao5 appears on the leaderboard, it carries the same weight as everyone else's.
        </p>

        <div class="space-y-[2.5vh]">
          <!-- Bluetooth Cube Required -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-[2.5vmin] h-[2.5vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M8.288 15.038a5.25 5.25 0 017.424 0M5.106 11.856c3.807-3.808 9.98-3.808 13.788 0M1.924 8.674c5.565-5.565 14.587-5.565 20.152 0M12.53 18.22l-.53.53-.53-.53a.75.75 0 011.06 0z" />
              </svg>
            </div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">Bluetooth Cube Verified</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">Every move is read directly from your smart cube via Bluetooth. The app knows exactly which moves you make and when — scrambles, solves, and timing are all hardware-verified.</div>
            </div>
          </div>

          <!-- One Move at a Time -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-[2.5vmin] h-[2.5vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">Guided Scramble</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">Moves are revealed one at a time and validated as you go. Since you have to scramble yourself, seeing the full scramble upfront would let you plan your solve while scrambling. One-at-a-time keeps it blind — just like receiving a pre-scrambled cube from a judge.</div>
            </div>
          </div>

          <!-- 3s Move Timer -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 font-black text-[2vmin] shrink-0">3s</div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">3-Second Move Window</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">Each scramble move must be completed within 3 seconds. A competition scrambler doesn't pause between moves to think — they execute at a steady pace. This keeps you doing the same: read, turn, next.</div>
            </div>
          </div>

          <!-- Error Tracking -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-[2.5vmin] h-[2.5vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
              </svg>
            </div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">Error Tracking</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">Every wrong move is counted — even if corrected immediately. 3 total mistakes and the scramble is invalidated. This prevents stalling during scramble (e.g. doing random moves back and forth to buy thinking time). The scramble should be a clean, uninterrupted execution.</div>
            </div>
          </div>

          <!-- 15s Inspection -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 font-black text-[1.8vmin] shrink-0">15s</div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">15-Second Inspection</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">After scrambling, you get exactly 15 seconds to inspect the cube before solving — matching WCA Regulation A3a1. This is your only window to plan. Exceeding it results in a DNF, just like in competition.</div>
            </div>
          </div>

          <!-- Ao5 -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 font-black text-[2vmin] shrink-0">5</div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">Average of 5</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">Each WCA session is exactly 5 solves — the standard competition format. Best and worst are trimmed, the middle 3 are averaged. This is what goes on the leaderboard.</div>
            </div>
          </div>

          <!-- 1 Hour -->
          <div class="flex gap-[2vmin]">
            <div class="w-[5vmin] h-[5vmin] rounded-[1vmin] bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400 shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-[2.5vmin] h-[2.5vmin]" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <div class="text-[1.8vmin] font-bold text-slate-200">1-Hour Session Window</div>
              <div class="text-[1.4vmin] text-slate-500 leading-relaxed">All 5 solves must be completed within one hour of your first solve. Just like a competition round has a time window — you sit down, you solve, you're done.</div>
            </div>
          </div>
        </div>

        <div class="mt-[4vh] pt-[2vh] border-t border-slate-800 text-center">
          <p class="text-[1.3vmin] text-slate-600 mb-[2vh] leading-relaxed">
            Together, these rules ensure the scramble is a quick mechanical step — not part of your solve. Your inspection starts when the scramble ends, and your Ao5 on the leaderboard means the same thing as everyone else's.
          </p>
          <button @click="emit('close')" class="w-full py-[1.5vh] bg-amber-500 hover:bg-amber-400 text-black font-bold rounded-[1.5vmin] text-[1.8vmin] transition-all">
            Got it
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
