<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function goBack() {
  router.push({ name: 'Landing' })
}
</script>

<template>
  <div class="w-full min-h-[100dvh] flex flex-col bg-gradient-to-b from-slate-950 to-slate-900">
    <!-- Header -->
    <header class="sticky top-0 z-10 bg-slate-950/90 backdrop-blur-sm border-b border-white/5">
      <div class="max-w-3xl mx-auto px-6 py-4 flex items-center gap-4">
        <button
          @click="goBack"
          class="flex items-center gap-2 text-slate-400 hover:text-white transition-colors text-sm font-medium"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          Back
        </button>
        <h1 class="text-lg font-bold text-white">Terms of Use</h1>
      </div>
    </header>

    <!-- Content -->
    <main class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto px-6 py-8 space-y-6 text-slate-300 text-sm leading-relaxed">
        <p class="text-slate-500 text-xs uppercase tracking-widest font-bold">Last updated: February 2026</p>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">1. Acceptance of Terms</h2>
          <p>
            By accessing or using RouxFlow ("the Service"), you agree to be bound by these Terms of Use.
            If you do not agree to all the terms and conditions, you must not use the Service.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">2. Description of Service</h2>
          <p>
            RouxFlow is a web-based training platform for Roux method speedcubing. The Service provides
            Bluetooth smart cube connectivity, solve timing, phase analysis, and training tools.
            The Service requires a compatible Bluetooth smart cube and a supported web browser.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">3. User Accounts</h2>
          <p>
            You may use RouxFlow as a guest or create an account using supported authentication providers.
            If you create an account, you are responsible for maintaining the security of your credentials.
            You agree to provide accurate information and to update it as necessary.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">4. Data Ownership and Usage</h2>
          <p>
            By using RouxFlow, you grant RouxFlow a perpetual, non-exclusive, royalty-free license to use,
            store, and process the solve data you generate through the Service. This data may be used to
            improve algorithms, training tools, and the overall Service experience.
          </p>
          <p>
            Solve data includes, but is not limited to: solve times, move sequences, phase breakdowns,
            scramble sequences, and cube state information.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">5. Data Retention and Deletion</h2>
          <p>
            If you request account deletion under GDPR or any applicable data protection law, we will
            remove all personally identifiable data linked to your account. However, anonymized solve data
            that has been processed for training purposes will be retained, as it can no longer be linked
            to your identity.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">6. Privacy</h2>
          <p>
            RouxFlow stores data locally on your device using IndexedDB. If you create an account, your
            data may also be synced to our cloud infrastructure (Supabase). We do not sell your personal
            information to third parties.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">7. Bluetooth and Device Access</h2>
          <p>
            RouxFlow requires access to your device's Bluetooth capabilities via the Web Bluetooth API.
            By granting Bluetooth permissions, you authorize RouxFlow to communicate with your smart cube
            for the purpose of tracking solves and cube state.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">8. Limitation of Liability</h2>
          <p>
            RouxFlow is provided "as is" without warranty of any kind. We are not responsible for any
            damage to your devices, data loss, or any other issues arising from the use of the Service.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">9. Modifications to Terms</h2>
          <p>
            We reserve the right to modify these Terms of Use at any time. Continued use of the Service
            after changes constitutes acceptance of the updated terms.
          </p>
        </section>

        <section class="space-y-3">
          <h2 class="text-xl font-bold text-white">10. Contact</h2>
          <p>
            For questions about these Terms of Use, please reach out through the project's GitHub repository.
          </p>
        </section>

        <div class="pt-8 pb-12 text-center">
          <button
            @click="goBack"
            class="px-6 py-2.5 rounded-xl border border-slate-700 text-slate-300 hover:border-slate-500 hover:text-white transition-all text-sm font-medium"
          >
            Back to Login
          </button>
        </div>
      </div>
    </main>
  </div>
</template>
