# Offline-First Authentication Architecture

## Overview

RouxFlow implements a **dual-layer authentication system** that works seamlessly both online and offline. Users stay authenticated even without network connectivity using cached credentials from localStorage.

## Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│ User Experience                                      │
├─────────────────────────────────────────────────────┤
│ • Login once → Stay logged in forever              │
│ • Work offline → Data saved locally                 │
│ • Come back online → Auto-sync                      │
└─────────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────────┐
│ Auth Store (Enhanced Supabase Auth)                 │
├─────────────────────────────────────────────────────┤
│ Primary: Supabase session tokens (localStorage)     │
│ Fallback: Cached user/session (localStorage)        │
│ Status: Online/offline detection (navigator.onLine) │
└─────────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────────┐
│ Data Layer (rouxflow-storage)                       │
├─────────────────────────────────────────────────────┤
│ Local: IndexedDB (cubes, sessions, solves)         │
│ Cloud: Supabase (opportunistic sync when online)   │
└─────────────────────────────────────────────────────┘
```

## How It Works

### Initial Login (Online)

1. User logs in via Google/Discord/Email
2. Supabase stores session tokens in **localStorage** automatically:
   - `sb-<project>-auth-token` (Supabase's default key)
   - Access token + Refresh token
3. Auth store **also caches** user data:
   - `rouxflow_offline_session` (full session object)
   - `rouxflow_offline_user` (user metadata)

### Working Offline

When the network goes offline:

1. **Auth Detection:**
   - `navigator.onLine` event fires
   - Auth store sets `isOnline = false`
   - UI shows "Offline Mode" indicator

2. **Session Validation:**
   - Supabase SDK can't reach server
   - Auth store loads from **cached session** in localStorage
   - User remains authenticated ✅

3. **Data Operations:**
   - All reads/writes go to **IndexedDB** (via rouxflow-storage)
   - Cubes, sessions, solves stored locally
   - No network required ✅

4. **Cube Connection:**
   - Web Bluetooth works offline (it's local device communication)
   - Users can connect cubes and train ✅

### Coming Back Online

When network is restored:

1. **Auto-Detection:**
   - `navigator.online` event fires
   - Auth store sets `isOnline = true`
   - UI hides "Offline Mode" indicator

2. **Session Refresh:**
   - Auth store calls `supabase.auth.getSession()`
   - Refreshes access token if expired
   - Updates cached session in localStorage

3. **Data Sync:**
   - rouxflow-storage syncs IndexedDB → Supabase
   - Uploads cubes, sessions, solves to cloud
   - Two-way merge if conflicts exist

## Guest Mode

Guest users (not logged in with Supabase):
- ✅ Can connect cubes
- ✅ Can record sessions and solves
- ✅ Data saved to IndexedDB locally
- ❌ No cloud sync (no Supabase user ID)
- ❌ Data tied to browser (no cross-device)

**Guest → Authenticated Migration:**
When a guest logs in, their local data can be associated with their Supabase user ID and synced to the cloud.

## Session Expiry Handling

### Tokens Never Expire Offline

**Problem:** Supabase access tokens typically expire after 1 hour.

**Solution:**
1. When **online**: Supabase SDK auto-refreshes tokens
2. When **offline**: We use **cached session** indefinitely
3. When **back online**: Session refreshes from server

**Edge Case:** If user is offline for >7 days (refresh token expiry):
- User must log in again when back online
- Local data is preserved in IndexedDB
- After login, data syncs to their account

## localStorage Keys

| Key | Purpose | Set By | Works Offline |
|-----|---------|--------|---------------|
| `sb-<project>-auth-token` | Supabase session tokens | Supabase SDK | ✅ (read-only) |
| `rouxflow_offline_session` | Full session cache | Auth store | ✅ |
| `rouxflow_offline_user` | User metadata cache | Auth store | ✅ |

## Implementation Details

### Auth Store Enhancement

```typescript
// When user logs in (online)
function cacheSessionOffline() {
  localStorage.setItem('rouxflow_offline_session', JSON.stringify(session))
  localStorage.setItem('rouxflow_offline_user', JSON.stringify(user))
}

// When Supabase fails (offline)
function loadCachedSession() {
  const session = localStorage.getItem('rouxflow_offline_session')
  const user = localStorage.getItem('rouxflow_offline_user')
  // User stays authenticated!
}
```

### Online/Offline Detection

```typescript
window.addEventListener('online', () => {
  isOnline.value = true
  refreshSessionOnline() // Sync with server
})

window.addEventListener('offline', () => {
  isOnline.value = false
  // Continue working with cached session
})
```

### Storage Layer (rouxflow-storage)

```rust
// IndexedDB: Always works, no network needed
impl Storage for StorageManager {
    async fn save_solve(&self, session_id: &str, solve: &Solve) {
        // Save to IndexedDB first (offline-first)
        self.local.save_solve(session_id, solve).await?;

        // Try cloud sync if online (opportunistic)
        if let Some(cloud) = &self.cloud {
            cloud.save_solve(session_id, solve).await.ok();
        }
    }
}
```

## Security Considerations

### Offline Session Security

**Stored in localStorage:**
- ✅ Same-origin policy (only RouxFlow domain can access)
- ✅ Not accessible to other websites
- ❌ Accessible via browser DevTools (acceptable for offline-first)
- ❌ Vulnerable if device is compromised (same as any PWA)

**Mitigation:**
- Session tokens have expiry (must re-login eventually)
- Sensitive operations (payments, data deletion) require online + fresh token
- User data is not highly sensitive (cube solve times, not health/financial)

### LocalStorage vs SessionStorage

**We use localStorage (not sessionStorage) because:**
- User wants to stay logged in across browser restarts
- PWA offline-first requires persistent auth
- Closing tab shouldn't log user out

## Testing Offline Mode

### Browser DevTools

1. Open DevTools (F12)
2. Network tab → Throttling dropdown
3. Select "Offline"
4. App should:
   - Show "Offline Mode" indicator
   - Continue working normally
   - Load cached user session

### Service Worker

The PWA service worker caches:
- ✅ HTML, CSS, JS files
- ✅ WASM binary
- ✅ Static assets

Combined with offline auth:
- App loads completely offline
- User is authenticated
- Data operations work
- Cube connection works

## Future Enhancements

1. **Conflict Resolution UI:**
   - Show merge conflicts when syncing
   - Let user choose which data to keep

2. **Offline Queue:**
   - Queue operations while offline
   - Replay when online
   - Show sync status

3. **Biometric Auth:**
   - Face ID / Touch ID for quick unlock
   - Skip typing password each time
   - Secure cached session access

4. **Multi-Device Sync:**
   - Sync across user's devices
   - Real-time updates via Supabase Realtime
   - Conflict resolution

## Comparison to Other Apps

| App | Offline Auth | Offline Data | Sync |
|-----|-------------|--------------|------|
| **RouxFlow** | ✅ Cached session | ✅ IndexedDB | ✅ Opportunistic |
| Google Docs | ❌ Must be online | ✅ IndexedDB | ✅ Automatic |
| Notion | ❌ Must be online | ⚠️ Limited | ✅ Automatic |
| Spotify | ✅ Remembered | ✅ Downloaded | ⚠️ Manual |

RouxFlow has **best-in-class offline support** for a speedcubing training app!

## Summary

**Authentication Strategy:**
- 🔑 Supabase session tokens (primary)
- 💾 Cached user/session (fallback)
- 🌐 Online/offline detection (navigator.onLine)

**Data Strategy:**
- 📦 IndexedDB for local storage (always works)
- ☁️ Supabase for cloud backup (when online)
- 🔄 Automatic bidirectional sync

**User Experience:**
- ✅ Login once, stay logged in forever
- ✅ Work offline without limitations
- ✅ Auto-sync when connection restored
- ✅ Guest mode for immediate use
