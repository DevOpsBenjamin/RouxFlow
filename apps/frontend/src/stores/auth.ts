import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { supabase } from '../lib/supabase'
import type { User, Session } from '@supabase/supabase-js'

export const useAuthStore = defineStore('auth', () => {
    const user = ref<User | null>(null)
    const session = ref<Session | null>(null)
    const loading = ref(true)

    const isAuthenticated = computed(() => !!user.value)
    const isGuest = computed(() => !user.value)
    const displayName = computed(() => {
        if (!user.value) return 'Guest'
        return user.value.user_metadata?.display_name || user.value.user_metadata?.full_name || user.value.email || 'User'
    })

    async function init() {
        loading.value = true

        try {
            // Get current session
            const { data: { session: currentSession } } = await supabase.auth.getSession()
            session.value = currentSession
            user.value = currentSession?.user ?? null

            // Listen for auth changes
            supabase.auth.onAuthStateChange((_event, newSession) => {
                session.value = newSession
                user.value = newSession?.user ?? null
            })
        } catch (e) {
            console.warn('Supabase auth init failed (working in offline mode):', e)
        }

        loading.value = false
    }

    async function signInWithGoogle() {
        const { error } = await supabase.auth.signInWithOAuth({
            provider: 'google',
            options: {
                redirectTo: window.location.origin
            }
        })
        if (error) throw error
    }

    async function signInWithDiscord() {
        const { error } = await supabase.auth.signInWithOAuth({
            provider: 'discord',
            options: {
                redirectTo: window.location.origin
            }
        })
        if (error) throw error
    }

    async function signInWithEmail(email: string, password: string) {
        const { error } = await supabase.auth.signInWithPassword({
            email,
            password
        })
        if (error) throw error
    }

    async function signUpWithEmail(email: string, password: string) {
        const { error } = await supabase.auth.signUp({
            email,
            password
        })
        if (error) throw error
    }

    async function signOut() {
        const { error } = await supabase.auth.signOut()
        if (error) throw error
        user.value = null
        session.value = null
    }

    async function updateDisplayName(name: string) {
        if (!user.value) return
        const { data, error } = await supabase.auth.updateUser({
            data: { display_name: name }
        })
        if (error) throw error
        user.value = data.user
    }

    return {
        user,
        session,
        loading,
        isAuthenticated,
        isGuest,
        displayName,
        init,
        signInWithGoogle,
        signInWithDiscord,
        signInWithEmail,
        signUpWithEmail,
        signOut,
        updateDisplayName
    }
})
