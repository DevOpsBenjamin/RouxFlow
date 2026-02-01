<script setup lang="ts">
type CubeStatus = 'supported' | 'planned' | 'community'

interface Protocol {
  name: string
  status: CubeStatus
  cubes: string[]
}

const protocols: Protocol[] = [
  { 
    name: 'GAN Gen2', 
    status: 'supported',
    cubes: ['MoYu AI 2023', 'GAN 356i v1', 'GAN 356i Carry', 'Monster Go 3Ai']
  },
  { 
    name: 'GoCube BLE', 
    status: 'planned',
    cubes: ['GoCube Edge', 'GoCube 2x2']
  },
  { 
    name: 'GAN Gen3', 
    status: 'community',
    cubes: ['GAN 356i Carry 2']
  },
  { 
    name: 'GAN Gen4', 
    status: 'community',
    cubes: ['GAN12 ui Maglev', 'GAN14 ui FreePlay']
  },
  { 
    name: 'QiYi BLE', 
    status: 'community',
    cubes: ['QiYi AI Smart Cube']
  },
  { 
    name: 'Giiker BLE', 
    status: 'community',
    cubes: ['Giiker i3s']
  },
]

const statusConfig: Record<CubeStatus, { text: string, class: string, icon: string }> = {
  supported: { text: 'Supported', class: 'text-emerald-400 bg-emerald-400/10 border-emerald-400/20', icon: '✅' },
  planned: { text: 'Planned', class: 'text-amber-400 bg-amber-400/10 border-amber-400/20', icon: '🔜' },
  community: { text: 'PR Welcome', class: 'text-slate-400 bg-slate-400/10 border-slate-400/20', icon: '🤝' },
}
</script>

<template>
  <div class="w-full max-w-4xl mx-auto space-y-6 pb-12">
    <!-- Header -->
    <div class="text-center space-y-2">
      <h2 class="text-3xl font-bold text-white">🔌 Currently Supported Cubes</h2>
      <p class="text-slate-400 text-sm">Grouped by Bluetooth protocol</p>
    </div>

    <!-- Legend -->
    <div class="flex flex-wrap justify-center gap-3">
      <div v-for="(cfg, key) in statusConfig" :key="key" 
           class="px-3 py-1.5 rounded-full text-xs font-bold border" :class="cfg.class">
        {{ cfg.icon }} {{ cfg.text }}
      </div>
    </div>

    <!-- Protocols List -->
    <div class="space-y-4">
      <div v-for="protocol in protocols" :key="protocol.name" 
           class="bg-slate-900/50 border border-white/5 rounded-2xl overflow-hidden">
        <!-- Protocol Header -->
        <div class="flex items-center justify-between p-4 border-b border-white/5">
          <div class="flex items-center gap-3">
            <span class="text-lg font-bold text-white">{{ protocol.name }}</span>
            <span class="px-2 py-0.5 rounded-md text-[10px] font-bold border" :class="statusConfig[protocol.status].class">
              {{ statusConfig[protocol.status].icon }} {{ statusConfig[protocol.status].text }}
            </span>
          </div>
        </div>
        
        <!-- Cubes List -->
        <div class="p-4">
          <div class="flex flex-wrap gap-2">
            <span v-for="cube in protocol.cubes" :key="cube"
                  class="px-3 py-1.5 bg-white/5 rounded-lg text-sm text-slate-300 border border-white/5">
              {{ cube }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Contribute CTA -->
    <div class="bg-indigo-500/10 border border-indigo-500/20 rounded-2xl p-5 flex items-start gap-4">
      <div class="text-2xl">🤝</div>
      <div class="space-y-1">
        <h4 class="text-base font-bold text-white">Want to add your cube?</h4>
        <p class="text-slate-400 text-sm">
          Add a protocol file (e.g., <code class="text-indigo-400">gan_v3.rs</code>) and submit a PR!
        </p>
        <a href="https://github.com/DevOpsBenjamin/RouxFlow" target="_blank" 
           class="inline-flex items-center gap-2 text-indigo-400 hover:text-indigo-300 font-medium text-sm">
          View on GitHub →
        </a>
      </div>
    </div>
  </div>
</template>
