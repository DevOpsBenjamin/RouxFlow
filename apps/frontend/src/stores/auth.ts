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

    // Initialize auth state
    async function init() {
        loading.value = true

        try {
            // Handle initial deep link if app was opened with one
            if (window.__TAURI__) {
                const { onOpenUrl } = await import('@tauri-apps/plugin-deep-link')
                await onOpenUrl((urls) => {
                    console.log('Received deep link urls:', urls)
                    const firstUrl = urls[0]
                    if (firstUrl) {
                        handleDeepLink(firstUrl)
                    }
                })

                // Also listen for links sent from a second instance (single-instance plugin)
                const { listen } = await import('@tauri-apps/api/event')
                await listen<string[]>('deep-link://new-url', (event) => {
                    console.log('Received deep link from second instance:', event.payload)
                    if (event.payload && event.payload.length > 0) {
                        // On Windows, event.payload[0] is often the exe path, [1] is the URL
                        const url = event.payload.find(arg => arg.startsWith('rouxflow://'))
                        if (url) handleDeepLink(url)
                    }
                })
            }

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

    async function handleDeepLink(url: string) {
        if (!url.startsWith('rouxflow://')) return

        console.log('Handling RouxFlow deep link:', url)
        const urlObj = new URL(url.replace('rouxflow://', 'http://localhost/'))
        const hash = urlObj.hash.substring(1) // remove #

        if (hash) {
            const { data, error } = await supabase.auth.setSession({
                access_token: new URLSearchParams(hash).get('access_token') || '',
                refresh_token: new URLSearchParams(hash).get('refresh_token') || '',
            })
            if (error) console.error('Error setting session from deep link:', error)
            else {
                console.log('Session set successfully from deep link')
                user.value = data.user
                session.value = data.session
            }
        }
    }

    async function signInWithGoogle() {
        console.log('auth.signInWithGoogle called')
        const isTauri = !!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI__
        const redirectUrl = isTauri ? 'rouxflow://auth-callback' : window.location.origin

        const { data, error } = await supabase.auth.signInWithOAuth({
            provider: 'google',
            options: {
                redirectTo: redirectUrl,
                skipBrowserRedirect: isTauri // Don't redirect in webview
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
        console.log('auth.signInWithDiscord called')
        const isTauri = !!(window as any).__TAURI_INTERNALS__ || !!(window as any).__TAURI__
        const redirectUrl = isTauri ? 'rouxflow://auth-callback' : window.location.origin

        const { data, error } = await supabase.auth.signInWithOAuth({
            provider: 'discord',
            options: {
                redirectTo: redirectUrl,
                skipBrowserRedirect: isTauri
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
