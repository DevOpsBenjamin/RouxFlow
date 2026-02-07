import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { supabase } from '../lib/supabase'
import type { User, Session } from '@supabase/supabase-js'
import { logger } from '../utils/logger'

const OFFLINE_SESSION_KEY = 'rouxflow_offline_session'
const OFFLINE_USER_KEY = 'rouxflow_offline_user'

export const useAuthStore = defineStore('auth', () => {
    const user = ref<User | null>(null)
    const session = ref<Session | null>(null)
    const loading = ref(true)
    const isOnline = ref(navigator.onLine)

    const isAuthenticated = computed(() => !!user.value)
    const isGuest = computed(() => !user.value)
    const displayName = computed(() => {
        if (!user.value) return 'Guest'
        return user.value.user_metadata?.display_name || user.value.user_metadata?.full_name || user.value.email || 'User'
    })

    // Track online/offline status
    function setupOnlineDetection() {
        window.addEventListener('online', () => {
            isOnline.value = true
            logger.info('Network connection restored')
            // Try to refresh session when coming back online
            refreshSessionOnline()
        })

        window.addEventListener('offline', () => {
            isOnline.value = false
            logger.warn('Network connection lost - working in offline mode')
        })
    }

    // Save session to localStorage for offline access
    function cacheSessionOffline() {
        if (session.value && user.value) {
            try {
                localStorage.setItem(OFFLINE_SESSION_KEY, JSON.stringify(session.value))
                localStorage.setItem(OFFLINE_USER_KEY, JSON.stringify(user.value))
            } catch (e) {
                logger.error('Failed to cache session for offline use:', e)
            }
        }
    }

    // Load session from localStorage when offline
    function loadCachedSession() {
        try {
            const cachedSession = localStorage.getItem(OFFLINE_SESSION_KEY)
            const cachedUser = localStorage.getItem(OFFLINE_USER_KEY)

            if (cachedSession && cachedUser) {
                session.value = JSON.parse(cachedSession)
                user.value = JSON.parse(cachedUser)
                logger.info('Loaded cached session for offline use')
                return true
            }
        } catch (e) {
            logger.error('Failed to load cached session:', e)
        }
        return false
    }

    // Clear offline cache
    function clearOfflineCache() {
        try {
            localStorage.removeItem(OFFLINE_SESSION_KEY)
            localStorage.removeItem(OFFLINE_USER_KEY)
        } catch (e) {
            logger.error('Failed to clear offline cache:', e)
        }
    }

    // Refresh session when coming back online
    async function refreshSessionOnline() {
        if (!isOnline.value) return

        try {
            const { data: { session: currentSession } } = await supabase.auth.getSession()
            if (currentSession) {
                session.value = currentSession
                user.value = currentSession.user
                cacheSessionOffline()
                logger.info('Session refreshed from server')
            }
        } catch (e) {
            logger.warn('Failed to refresh session from server:', e)
        }
    }

    async function init() {
        loading.value = true
        setupOnlineDetection()

        try {
            // Try to get session from Supabase (works if online OR if tokens cached)
            const { data: { session: currentSession } } = await supabase.auth.getSession()

            if (currentSession) {
                session.value = currentSession
                user.value = currentSession.user
                cacheSessionOffline()
                logger.info('Session loaded from Supabase')
            } else {
                // No session from Supabase, try offline cache
                const loaded = loadCachedSession()
                if (loaded) {
                    logger.info('Using cached offline session')
                } else {
                    logger.info('No session found - user will be guest')
                }
            }

            // Listen for auth changes (only works when online)
            supabase.auth.onAuthStateChange((_event, newSession) => {
                session.value = newSession
                user.value = newSession?.user ?? null

                if (newSession) {
                    cacheSessionOffline()
                } else {
                    clearOfflineCache()
                }
            })
        } catch (e) {
            logger.warn('Supabase auth init failed, checking offline cache:', e)

            // If Supabase init fails (offline), try to load from cache
            const loaded = loadCachedSession()
            if (loaded) {
                logger.info('Working in offline mode with cached credentials')
            } else {
                logger.info('No cached session - user will be guest')
            }
        }

        loading.value = false
    }

    async function signInWithGoogle() {
        if (!isOnline.value) {
            throw new Error('Cannot sign in while offline. Please check your network connection.')
        }

        const { error } = await supabase.auth.signInWithOAuth({
            provider: 'google',
            options: {
                redirectTo: window.location.origin
            }
        })
        if (error) throw error
    }

    async function signInWithDiscord() {
        if (!isOnline.value) {
            throw new Error('Cannot sign in while offline. Please check your network connection.')
        }

        const { error } = await supabase.auth.signInWithOAuth({
            provider: 'discord',
            options: {
                redirectTo: window.location.origin
            }
        })
        if (error) throw error
    }

    async function signInWithEmail(email: string, password: string) {
        if (!isOnline.value) {
            throw new Error('Cannot sign in while offline. Please check your network connection.')
        }

        const { error } = await supabase.auth.signInWithPassword({
            email,
            password
        })
        if (error) throw error
    }

    async function signUpWithEmail(email: string, password: string) {
        if (!isOnline.value) {
            throw new Error('Cannot sign up while offline. Please check your network connection.')
        }

        const { error } = await supabase.auth.signUp({
            email,
            password
        })
        if (error) throw error
    }

    async function signOut() {
        // Clear local state first
        user.value = null
        session.value = null
        clearOfflineCache()

        // Try to sign out from Supabase if online
        if (isOnline.value) {
            try {
                const { error } = await supabase.auth.signOut()
                if (error) {
                    logger.warn('Supabase sign out error (but local session cleared):', error)
                }
            } catch (e) {
                logger.warn('Failed to sign out from server (but local session cleared):', e)
            }
        } else {
            logger.info('Signed out locally (offline mode)')
        }
    }

    async function updateDisplayName(name: string) {
        if (!user.value) return

        if (!isOnline.value) {
            throw new Error('Cannot update profile while offline. Please check your network connection.')
        }

        const { data, error } = await supabase.auth.updateUser({
            data: { display_name: name }
        })
        if (error) throw error

        user.value = data.user
        cacheSessionOffline()
    }

    return {
        user,
        session,
        loading,
        isOnline,
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
