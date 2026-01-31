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

    // Initialize auth state
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
        console.log('auth.signInWithGoogle called')
        const { data, error } = await supabase.auth.signInWithOAuth({
            provider: 'google',
            options: {
                redirectTo: window.location.origin,
                skipBrowserRedirect: true // Don't redirect in webview
            }
        })
        if (error) {
            console.error('Supabase OAuth error:', error)
            throw error
        }

        console.log('Supabase OAuth response data:', data)

        // Open in system browser for security
        if (data?.url) {
            console.log('URL found, calling openInBrowser:', data.url)
            await openInBrowser(data.url)
        } else {
            console.warn('No URL returned from Supabase OAuth')
        }
    }

    async function signInWithDiscord() {
        const { data, error } = await supabase.auth.signInWithOAuth({
            provider: 'discord',
            options: {
                redirectTo: window.location.origin,
                skipBrowserRedirect: true
            }
        })
        if (error) throw error

        if (data?.url) {
            await openInBrowser(data.url)
        }
    }

    async function openInBrowser(url: string) {
        console.log('openInBrowser called with:', url)

        // Robust check for Tauri (v1 or v2)
        const isTauri = !!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI__
        console.log('Is Tauri environment detected:', isTauri)

        if (isTauri) {
            try {
                console.log('Attempting to use Tauri shell plugin...')
                const { open } = await import('@tauri-apps/plugin-shell')
                await open(url)
                console.log('Tauri shell.open successful')
            } catch (err) {
                console.error('Tauri shell plugin failed, falling back to window.open:', err)
                window.open(url, '_blank')
            }
        } else {
            console.log('Non-Tauri environment, using window.open')
            window.open(url, '_blank')
        }
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

    return {
        user,
        session,
        loading,
        isAuthenticated,
        isGuest,
        init,
        signInWithGoogle,
        signInWithDiscord,
        signInWithEmail,
        signUpWithEmail,
        signOut
    }
})
