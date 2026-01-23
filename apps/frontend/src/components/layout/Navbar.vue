<template>
  <header class="p-6 flex justify-between items-center border-b border-white/5 backdrop-blur-md sticky top-0 z-50">
    <div class="flex items-center gap-3">
      <span class="text-3xl">🧊</span>
      <h1 class="text-2xl font-bold tracking-tight bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">
        RouxFlow
      </h1>
    </div>
    <nav class="hidden md:flex gap-8 text-sm font-medium text-slate-400">
      <a href="#" class="hover:text-indigo-400 transition-colors">Session</a>
      <a href="#" class="hover:text-indigo-400 transition-colors">Training</a>
      <a href="#" class="hover:text-indigo-400 transition-colors">Stats</a>
    </nav>
    <div class="flex gap-4">
      <slot name="actions"></slot>
    </div>
  </header>
</template>
