# Bluetooth UX Redesign - Modern WASM Architecture

## Overview

Complete redesign of the Bluetooth cube connection UX to be modern, intuitive, and fully integrated with the new WASM-first architecture.

## New Components

### 1. **ConnectCubeButton.vue** - Smart Adaptive Button

**Location:** `apps/frontend/src/components/cube/ConnectCubeButton.vue`

**States & Behavior:**

#### State 1: No Cubes in Database
```vue
<!-- Big prominent button -->
🔵 Connect a Cube
```
- Gradient button (indigo → purple)
- Hover animation (scale up)
- Clicking opens Web Bluetooth picker

#### State 2: Has Saved Cubes, Not Connected
```vue
<!-- Status indicator -->
🔴 No Cube Connected ⚡
```
- Shows red dot + "No Cube Connected"
- Quick reconnect to last cube
- Opens connection flow on click

#### State 3: Connected
```vue
<!-- Cube status bar -->
🟢 [Cube Name] 🔋 87% →
```
- Green dot + cube name + battery
- Clicking opens Cube Manager drawer
- Shows connection is active

### 2. **DeviceSelectionModal.vue** - Simplified Connection Modal

**Purpose:** Shows connection status during Web Bluetooth pairing

**States:**

1. **Connecting:**
   - Animated spinner
   - "Connecting to Cube" message
   - "Initializing WASM protocol handler" detail

2. **Error:**
   - Red error icon
   - Error message
   - Troubleshooting tips:
     - Make sure cube is powered on
     - Check Bluetooth is enabled
     - Turn cube off and on
     - Make sure not connected elsewhere
   - "Try Again" and "Cancel" buttons

3. **Success:**
   - Green checkmark
   - "Connected!" message
   - Auto-closes after brief moment

**Note:** The native Web Bluetooth picker shows device list, so this modal only handles pre/post connection states.

### 3. **Cube Manager Drawer** - Integrated Management Panel

**Location:** Built into `ConnectCubeButton.vue` as slide-out drawer

**Features:**

#### Current Cube Info Card
```
┌────────────────────────────────┐
│ Connected Cube          🟢     │
│ MoYu WeiLong V10              │
├────────────────────────────────┤
│ Protocol: MoYuAi              │
│ MAC: CF:30:16:01:C7:2F        │
│ Battery: 87%                  │
│ Gyroscope: ✓ Supported        │
└────────────────────────────────┘
```

#### Gyro Controls (if supported)
- Explanation text
- "Reset Gyro Orientation" button
- Calls `reset_gyro()` from WASM bridge

#### Quick Actions
- "Disconnect Cube" button (red)
- Opens drawer, disconnects, closes

#### More Actions Menu (collapsible)
- **Connect Another Cube** - Opens Web Bluetooth picker
- **Switch Cube** (if multiple saved) - Quick switch UI

#### Saved Cubes List
- Shows all saved cubes from IndexedDB
- Each cube shows:
  - Name
  - MAC address
  - Delete button (forget cube)
- Integrates with auth (syncs with Supabase if online)

## WASM Integration

### Connection Flow

```
User clicks "Connect a Cube"
    ↓
[ConnectCubeButton] calls bt.startScan()
    ↓
[Bluetooth Store] calls bridge.connect()
    ↓
[Bridge] calls Web Bluetooth API
    ↓
Browser shows native device picker
    ↓
User selects cube from list
    ↓
[Bridge] calls find_cube_by_ble_name() (WASM)
    ↓
[Bridge] gets cube definition (protocol, UUIDs)
    ↓
[Bridge] calls cubeManager.connect() (WASM)
    ↓
[WASM] creates protocol handler via codec::create_protocol()
    ↓
[Bridge] sets up BLE notification listener
    ↓
Connected! ✅
```

### Protocol Detection (WASM)

```rust
// In bridge.ts
const { device, cubeDef } = await connect()
// cubeDef from find_cube_by_ble_name (WASM)

await finalizeConnection(device, cubeDef)
// Calls cubeManager.connect(name, mac, protocol)

// WASM creates protocol handler internally
let protocol = codec::create_protocol(protocol_version, mac_address)
```

**Key Point:** Protocol is determined by WASM based on BLE device name. No manual selection needed!

### MAC Address Handling

**Automatic Detection:**
- `device.id` from Web Bluetooth API = MAC address
- If not available, uses device name as fallback
- WASM uses MAC for encryption keys (cube-specific)

**User Never Sees This:**
- MAC is shown in Cube Manager for debugging
- But connection "just works" automatically

## User Experience Flow

### First-Time User (No Cubes)

1. Opens app → sees "Connect a Cube" button
2. Clicks button → browser shows device picker
3. Selects cube → modal shows "Connecting..."
4. Success! → modal shows "Connected!" briefly
5. Button transforms to cube status bar
6. Can start solving immediately

### Returning User (Has Saved Cube)

1. Opens app → sees "No Cube Connected" button
2. Clicks → auto-reconnects to last cube
3. Or opens Web Bluetooth if auto-reconnect fails
4. Button shows cube name + battery when connected

### Multi-Cube User

1. Connected to Cube A
2. Clicks cube status bar → drawer opens
3. Clicks "More Actions" → "Connect Another Cube"
4. Selects Cube B from browser picker
5. Cube A disconnects, Cube B connects
6. Both cubes saved in list for future quick switch

## Component Integration

### Navbar (Primary Location)

```vue
<template>
  <header class="navbar">
    <!-- Logo & Nav Links -->

    <div class="actions">
      <ConnectCubeButton />
      <UserProfile />
    </div>
  </header>
</template>
```

### Home View (Alternative)

```vue
<template>
  <div class="home">
    <h1>Welcome to RouxFlow</h1>

    <!-- Prominent placement if no cube -->
    <ConnectCubeButton v-if="!bt.isConnected" />

    <SessionList />
  </div>
</template>
```

## Styling & Design

**Color Scheme:**
- 🔵 **Not Connected:** Indigo/Purple gradient
- 🟢 **Connected:** Emerald/Cyan gradient
- 🔴 **Error:** Red accents
- 🟡 **Connecting:** Indigo with animation

**Animations:**
- Button hover: Scale 1.05
- Button active: Scale 0.95
- Drawer slide: 300ms ease-out
- Status dot: Pulse animation (when disconnected)
- Spinner: Smooth rotation

**Responsive:**
- Desktop: Full button text + all info
- Mobile: Compact with icons only
- Drawer: Full-width on mobile, 24rem on desktop

## State Management

### Bluetooth Store (Dumb Proxy)

```typescript
// Queries WASM for all state
const isConnected = computed(() => cubeManager?.is_connected() ?? false)
const deviceInfo = computed(() => cubeManager?.get_device_info() ?? null)
const orientation = computed(() => cubeManager?.get_orientation() ?? [0,0,0,1])

// Only local state:
- savedCubes (IndexedDB cache)
- showPicker (modal visibility)
- isConnecting (connection in progress)
- error (last error message)
```

### Bridge (Connection Handler)

```typescript
// Single BLE listener
function blePacketHandler(event) {
  const bytes = new Uint8Array(event.target.value.buffer)
  const timestamp = performance.now() / 1000.0
  cubeManager.process_ble_packet(bytes, timestamp)
}

// Connection lifecycle
async function connect() → { device, cubeDef }
async function finalizeConnection(device, cubeDef)
async function disconnect()
```

### WASM CubeManager (Single Source of Truth)

```rust
pub struct WasmCubeManager {
    inner: CubeManager,
    protocol: Option<Box<dyn CubeProtocol>>,
}

// All state lives here:
- connection_state (Connected/Disconnected)
- device_info (name, MAC, protocol, gyro, battery)
- cube_state (orientation, facelets, moves)
- timer_state (running, time, moves)
- session_manager (scramble, flow state)
```

## Differences from Old Implementation

### Old (Removed) ❌

- Manual device scanning with custom UI
- Device list stored in Vue
- Multiple connection modals
- Separate cube manager page
- Complex state machine in stores
- Business logic scattered across components

### New (Current) ✅

- Web Bluetooth native picker
- WASM handles device detection
- Single adaptive button component
- Integrated drawer for management
- Dumb proxy stores
- All logic in WASM CubeManager

## Benefits

1. **Cleaner Code:**
   - 300+ lines removed
   - Single component for all connection UX
   - No duplicate state

2. **Better UX:**
   - Fewer clicks to connect
   - Native device picker (familiar)
   - Adaptive UI based on state
   - All actions in one place

3. **WASM Integration:**
   - Protocol detection automatic
   - MAC address handled by WASM
   - Encryption keys managed properly
   - No manual configuration

4. **Offline-First:**
   - Saved cubes in IndexedDB
   - Auto-reconnect to last cube
   - Works without network
   - Syncs when online

## Future Enhancements

### Auto-Reconnect on App Load

```typescript
// In App.vue onMounted
const lastCube = localStorage.getItem('last_connected_cube')
if (lastCube && navigator.bluetooth) {
  await bt.autoReconnect(lastCube)
}
```

### Quick Switch Between Cubes

```vue
<!-- In Cube Manager -->
<div v-for="cube in savedCubes">
  <button @click="switchToCube(cube)">
    {{ cube.name }}
  </button>
</div>
```

### Connection Quality Indicator

```vue
<!-- Show signal strength -->
<div class="signal">
  <span v-for="bar in signalBars">📶</span>
</div>
```

### Battery Alerts

```typescript
watch(() => bt.deviceInfo?.battery_level, (level) => {
  if (level && level < 20) {
    toast.warning('Cube battery low!')
  }
})
```

## Testing Checklist

- [ ] First-time connection (no saved cubes)
- [ ] Reconnection (saved cube exists)
- [ ] Multi-cube switching
- [ ] Gyro reset (if supported)
- [ ] Disconnect and reconnect
- [ ] Error handling (cube off, out of range)
- [ ] Offline operation (saved cubes available)
- [ ] Online sync (cubes sync to Supabase)
- [ ] Mobile responsive
- [ ] Browser compatibility (Chrome, Edge)

## Browser Compatibility

**Web Bluetooth API Support:**
- ✅ Chrome/Chromium (Desktop & Android)
- ✅ Edge (Desktop & Android)
- ✅ Opera (Desktop & Android)
- ❌ Firefox (not supported)
- ❌ Safari (not supported)
- ❌ iOS browsers (not supported)

**Detection:**
```typescript
if (!navigator.bluetooth) {
  // Show error: "Web Bluetooth not supported"
  // Suggest using Chrome or Edge
}
```

## Summary

The new Bluetooth UX is:
- ✅ Modern and intuitive
- ✅ Fully integrated with WASM
- ✅ Offline-first with IndexedDB
- ✅ Adaptive to connection state
- ✅ Single component, no page navigation
- ✅ Native browser picker (no custom scanning)
- ✅ Protocol detection automatic
- ✅ All features in one drawer

**Result:** Users can connect cubes in 2 clicks and manage everything from one place! 🎉
