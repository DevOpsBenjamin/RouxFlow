<script setup lang="ts">
</script>

<template>
  <div class="min-h-screen p-8">
    <h1 class="text-3xl font-bold mb-2">RouxFlow Admin</h1>
    <p class="text-slate-400 mb-8">Local-only admin panel — requires .env with Supabase service key</p>
    <router-view />
  </div>
</template>
