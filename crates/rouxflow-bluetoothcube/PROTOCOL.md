# Bluetooth Smart Cube Protocols

This document describes the Bluetooth Low Energy (BLE) protocols used by smart
Rubik's cubes supported by RouxFlow. All protocol constants, UUIDs, and
encryption keys are defined in the `src/protocol/` directory of this crate.

> **Status:** This is a living document. Not all protocols have been tested
> with physical hardware. Contributions, corrections, and new cube support
> are welcome.

---

## Table of Contents

- [Overview](#overview)
- [GAN v1](#gan-v1)
- [GAN Gen2](#gan-gen2)
- [GAN Gen3](#gan-gen3)
- [GAN Gen4](#gan-gen4)
- [MoYu AI](#moyu-ai)
- [MoYu V3](#moyu-v3)
- [Giiker (Xiaomi)](#giiker-xiaomi)
- [GoCube / Rubik's Connected](#gocube--rubiks-connected)
- [QiYi Smart](#qiyi-smart)
- [Encryption Reference](#encryption-reference)
- [Cube State Representation](#cube-state-representation)
- [Acknowledgements](#acknowledgements)

---

## Overview

| Protocol  | Manufacturer  | Encryption        | BLE Service UUID                         | Module        |
|-----------|---------------|-------------------|------------------------------------------|---------------|
| GAN v1    | GAN           | AES-CBC           | `0000fff0-0000-1000-8000-00805f9b34fb`   | `gan_v1.rs`   |
| GAN Gen2  | GAN           | AES-CBC           | `6e400001-b5a3-f393-e0a9-e50e24dc4179`   | `gan_v2.rs`   |
| GAN Gen3  | GAN           | AES-CBC           | `8653000a-43e6-47b7-9cb0-5fc21d4ae340`   | `gan_v3.rs`   |
| GAN Gen4  | GAN           | AES-CBC           | `00000010-0000-fff7-fff6-fff5fff4fff0`    | `gan_v4.rs`   |
| MoYu AI   | MoYu          | AES-CBC (alt key) | *(reuses GAN Gen2)*                      | `moyu_ai.rs`  |
| MoYu V3   | MoYu          | AES-CBC 2-pass    | `0783b03e-7735-b5a0-1760-a305d2795cb0`   | `moyu_v3.rs`  |
| Giiker    | Xiaomi        | XOR/ADD table     | `0000aadb-0000-1000-8000-00805f9b34fb`   | `giiker.rs`   |
| GoCube    | Particula     | None (plaintext)  | `6e400001-b5a3-f393-e0a9-e50e24dcca9e`   | `gocube.rs`   |
| QiYi      | QiYi          | AES-ECB           | `0000fff0-0000-1000-8000-00805f9b34fb`   | `qiyi.rs`     |

### BLE Name Prefixes (for scanning)

| Prefix               | Protocol |
|----------------------|----------|
| `GAN`                | GAN v1 / Gen2 / Gen3 / Gen4 (detected by service UUID) |
| `MG`                 | GAN Gen2 (Monster Go) |
| `AiCube`             | MoYu AI  |
| `MHC`                | MoYu AI v2 |
| `WCU_MY`             | MoYu V3  |
| `GiC`, `GiS`, `Gi`  | Giiker   |
| `Mi Smart Magic Cube`| Giiker   |
| `GoCube`             | GoCube   |
| `Rubiks`             | GoCube   |
| `QY-QYSC`           | QiYi SCS |
| `XMD-TornadoV4-i-`  | QiYi AI  |

---

## GAN v1

**File:** `src/protocol/gan_v1.rs`

The original GAN 356i protocol. Uses a single BLE service with multiple
characteristics for state, battery, and control.

### Cubes
- GAN 356i (China & International)
- GAN 356i Play
- GAN 356i 2 / 2 Play

### BLE
| UUID | Role |
|------|------|
| `0000fff0-...-00805f9b34fb` | Primary service |
| `0000fff2-...-00805f9b34fb` | Facelet status (notify) |
| `0000fff5-...-00805f9b34fb` | State / command (notify + write) |
| `0000fff7-...-00805f9b34fb` | Battery (notify) |

### Encryption
AES-128-CBC with MAC-salted keys. Same base keys as Gen2/3/4.

---

## GAN Gen2

**File:** `src/protocol/gan_v2.rs`

The most widely-used GAN protocol. Supports the largest number of cube models.

### Cubes
- GAN Mini ui FreePlay
- GAN 12 ui FreePlay / GAN 12 ui
- GAN 356 i Carry / Carry S
- GAN 356 i 3
- Monster Go 3Ai

### BLE
| UUID | Role |
|------|------|
| `6e400001-b5a3-f393-e0a9-e50e24dc4179` | Primary service |
| `28be4a4a-cd67-11e9-a32f-2a2ae2dbcce4` | Command (write) |
| `28be4cb6-cd67-11e9-a32f-2a2ae2dbcce4` | State (notify) |

### Encryption
AES-128-CBC. Key and IV are salted with the 6-byte MAC address (reversed):

```
salted_key[i] = (base_key[i] + mac_reversed[i]) % 255    for i in 0..6
```

Only the first and last 16-byte chunks are encrypted. Decryption order:
last chunk first, then first chunk.

### Packet Format (20 bytes, bit-packed)

First 4 bits = event type:

| Opcode | Event     | Key fields                                     |
|--------|-----------|------------------------------------------------|
| `0x01` | Gyro      | Quaternion (4 × 16-bit signed), velocity (3 × 4-bit) |
| `0x02` | Move      | Serial (8-bit), up to 7 moves (face 4-bit + dir 1-bit), elapsed (16-bit) |
| `0x04` | Facelets  | Serial (8-bit), CP (7×3-bit), CO (7×2-bit), EP (11×4-bit), EO (11×1-bit) |
| `0x05` | Hardware  | HW/SW version, 8-char name, gyro support flag  |
| `0x09` | Battery   | Level (8-bit, 0-100%)                           |
| `0x0D` | Disconnect|                                                |

### Commands

| Command          | Byte 0 |
|------------------|--------|
| Request facelets | `0x04` |
| Request hardware | `0x05` |
| Request battery  | `0x09` |
| Reset cube       | `0x0A` |

---

## GAN Gen3

**File:** `src/protocol/gan_v3.rs`

Introduced with the GAN 356 i Carry 2. Adds move history recovery for
handling lost BLE packets.

### Cubes
- GAN 356 i Carry 2

### BLE
| UUID | Role |
|------|------|
| `8653000a-43e6-47b7-9cb0-5fc21d4ae340` | Primary service |
| `8653000c-43e6-47b7-9cb0-5fc21d4ae340` | Command (write) |
| `8653000b-43e6-47b7-9cb0-5fc21d4ae340` | State (notify) |

### Encryption
Same as Gen2 (AES-128-CBC, same keys, same MAC salting).

### Packet Format (16 bytes)

Header: `[0x55, event_type, data_length, ...]`

| Opcode | Event        | Description                    |
|--------|--------------|--------------------------------|
| `0x01` | Move         | Single move + 32-bit timestamp |
| `0x02` | Facelets     | Full state (CP/CO/EP/EO)       |
| `0x06` | Move history | Recovery of missed moves       |
| `0x07` | Hardware     | HW/SW version                  |
| `0x10` | Battery      | Level (0-100%)                 |
| `0x11` | Disconnect   |                                |

### Move History Recovery

When a serial number gap is detected, the host requests missed moves:
```
Command: [0x68, 0x03, serial, 0x00, count, 0x00]
```
The response contains 2 moves per byte (4 bits each: 3-bit face + 1-bit direction).

> **Firmware bug:** Requesting history across the 255→0 serial boundary may
> produce spoofed 'D' moves. Clamp requests to stay within bounds.

---

## GAN Gen4

**File:** `src/protocol/gan_v4.rs`

Latest GAN protocol for flagship cubes. Adds multi-part hardware info and
gyroscope support.

### Cubes
- GAN 12 ui Maglev (gyroscope: yes)
- GAN 14 ui FreePlay (gyroscope: no)

### BLE
| UUID | Role |
|------|------|
| `00000010-0000-fff7-fff6-fff5fff4fff0` | Primary service |
| `0000fff5-0000-1000-8000-00805f9b34fb` | Command (write) |
| `0000fff6-0000-1000-8000-00805f9b34fb` | State (notify) |

### Encryption
Same as Gen2.

### Packet Format (20 bytes)

Header: `[event_type, data_length, ...]` (no magic byte).

| Opcode      | Event        | Description                     |
|-------------|--------------|---------------------------------|
| `0x01`      | Move         | Single move + 32-bit timestamp  |
| `0xD1`      | Move history | Recovery of missed moves        |
| `0xED`      | Facelets     | Full state (CP/CO/EP/EO)        |
| `0xEC`      | Gyro         | Quaternion + angular velocity   |
| `0xEF`      | Battery      | Level (0-100%)                  |
| `0xFA-0xFE` | Hardware     | Multi-part (see below)          |
| `0xEA`      | Disconnect   |                                 |

### Multi-Part Hardware Info

| Opcode | Field            | Format                   |
|--------|------------------|--------------------------|
| `0xFA` | Production date  | Year (16-bit) + month + day |
| `0xFC` | Hardware name    | ASCII string             |
| `0xFD` | Software version | Major.minor (4+4 bits)   |
| `0xFE` | Hardware version | Major.minor (4+4 bits)   |

All 4 parts must arrive before a complete hardware event can be emitted.
Gyroscope support is determined by `hardware_name == "GAN12uiM"`.

---

## MoYu AI

**File:** `src/protocol/moyu_ai.rs`

MoYu cubes that reuse the GAN Gen2 BLE service but with different encryption keys.
Detected at connection time by checking the device name prefix.

### Cubes
- MoYu AI 2023 (prefix: `AiCube`)
- MoYu AI v2 (prefix: `MHC`)

### BLE
Same as GAN Gen2.

### Encryption
Same AES-128-CBC scheme as GAN Gen2, but with MoYu-specific keys:
```
Key: [05 12 02 45 02 01 29 56 12 78 12 76 81 01 08 03]
IV:  [01 44 28 06 86 21 22 28 51 05 08 31 82 02 21 06]
```

### Packet Format
Identical to GAN Gen2.

---

## MoYu V3

**File:** `src/protocol/moyu_v3.rs`

Native MoYu protocol with a different BLE service and a unique double-pass
AES-CBC decryption scheme.

### Cubes
- MoYu WeiLong V10 (prefix: `WCU_MY`)

### BLE
| UUID | Role |
|------|------|
| `0783b03e-7735-b5a0-1760-a305d2795cb0` | Primary service |
| `0783b03e-7735-b5a0-1760-a305d2795cb1` | State (notify) |
| `0783b03e-7735-b5a0-1760-a305d2795cb2` | Command (write) |

### Encryption

AES-128-CBC with **double-pass** decryption.

**Key derivation** (MAC address reversed, modulo 255):
```
device_key[i] = (master_key[i] + mac_reversed[i]) % 255   for i in 0..6
device_iv[i]  = (master_iv[i]  + mac_reversed[i]) % 255   for i in 0..6
```

**Double-pass decryption:**
1. Decrypt 16 bytes at offset **4..20** (tail) with fresh AES-CBC context
2. Decrypt 16 bytes at offset **0..16** (head) with fresh AES-CBC context

Each pass uses its own CBC initialization (no IV chaining across passes).

### Handshake
A hello payload must be sent to the write characteristic after connecting,
before the cube will start sending notifications.

### Packet Format (20 bytes)

Byte 0 (after decryption) = opcode:

| Opcode | Event    | Description                             |
|--------|----------|-----------------------------------------|
| `0xA1` | Info     | Device info response                    |
| `0xA3` | State    | Full cube state (facelets)              |
| `0xA4` | Battery  | Battery level                           |
| `0xA5` | Move     | Face (0-5) + direction (CW/CCW/Double)  |
| `0xAB` | Gyro     | 4 × `f32` little-endian quaternion      |

Move face codes: `U=0, D=1, L=2, R=3, F=4, B=5`.
Direction codes: `CW=1, CCW=2, Double=3`.

---

## Giiker (Xiaomi)

**File:** `src/protocol/giiker.rs`

Proprietary protocol by Xiaomi for the Giiker smart cube line.

### Cubes
- Giiker i3 (prefix: `GiC`)
- Giiker i3S (prefix: `GiS`)
- Giiker i3Y (prefix: `Gi`)
- Mi Smart Magic Cube (prefix: `Mi Smart Magic Cube`)

### BLE
| UUID | Role |
|------|------|
| `0000aadb-0000-1000-8000-00805f9b34fb` | State service |
| `0000aadc-0000-1000-8000-00805f9b34fb` | Turn (notify) |
| `0000aaaa-0000-1000-8000-00805f9b34fb` | Request service |
| `0000aaab-0000-1000-8000-00805f9b34fb` | Request/response (write + notify) |

### Encryption

Proprietary XOR/ADD scheme using a 36-byte lookup table.

**Decryption:**
1. Verify byte 18 == `0xA7` (encryption marker)
2. Extract two 4-bit nibbles from byte 19 as offsets `o1`, `o2`
3. For each byte `i` in `0..18`:
   ```
   decrypted[i] = encrypted[i] + table[i + o1] + table[i + o2]
   ```

### Move Encoding

| Value | Move  | Value | Move  |
|-------|-------|-------|-------|
| 0     | D CW  | 6     | F CW  |
| 1     | D CCW | 7     | F CCW |
| 2     | U CW  | 8     | L CW  |
| 3     | U CCW | 9     | L CCW |
| 4     | B CW  | 10    | R CW  |
| 5     | B CCW | 11    | R CCW |

---

## GoCube / Rubik's Connected

**File:** `src/protocol/gocube.rs`

The simplest protocol — **no encryption**. Uses the Nordic UART Service (NUS)
with a unique base UUID.

### Cubes
- GoCube (prefix: `GoCube`)
- GoCube X (prefix: `GoCubeX`)
- Rubik's Connected (prefix: `Rubiks`)

### BLE
| UUID | Role |
|------|------|
| `6e400001-b5a3-f393-e0a9-e50e24dcca9e` | Nordic UART service |
| `6e400002-b5a3-f393-e0a9-e50e24dcca9e` | RX / state (notify) |
| `6e400003-b5a3-f393-e0a9-e50e24dcca9e` | TX / command (write) |

### Features
- Plaintext turn detection
- Battery level reporting
- Rotation tracking (can be enabled/disabled)
- Reset to solved state

---

## QiYi Smart

**File:** `src/protocol/qiyi.rs`

QiYi's smart cube protocol, using AES-ECB encryption and a handshake.

### Cubes
- QiYi Tornado V4 SCS (prefix: `QY-QYSC`)
- QiYi Tornado V4 AI (prefix: `XMD-TornadoV4-i-`)

### BLE
| UUID | Role |
|------|------|
| `0000fff0-0000-1000-8000-00805f9b34fb` | Primary service |
| `0000fff6-0000-1000-8000-00805f9b34fb` | Command + state (read/write + notify) |

### Encryption
AES-128-ECB with a fixed 16-byte key. No IV. Messages are padded to 16-byte
alignment before encryption.

### Handshake

After connecting, an `appHello` greeting must be sent:
```
SCS:  [0xCC, 0xA3, 0x00, 0x00, device_id_hi, device_id_lo, ...]
AI:   [0xCC, 0xA6, 0x00, 0x00, device_id_hi, device_id_lo, ...]
```
The device ID is parsed from the last 4 hex characters of the BLE name.

### Packet Format

Messages start with `0xFE`, followed by length and type.

| Type   | Description                |
|--------|----------------------------|
| `0x02` | CubeHello (38 bytes)       |
| `0x03` | State event                |
| `0x04` | Turn/move event            |
| `0x0A` | Battery / ACK              |

Certain messages require an ACK response.

Move face codes: `L=0, R=1, D=2, U=3, F=4, B=5`.
Direction: `CW=1, CCW=-1, 180°=2`.

---

## Encryption Reference

### AES-CBC MAC Salting (GAN / MoYu)

All GAN and MoYu protocols derive per-device keys from the cube's MAC address:

```
mac_bytes = parse("CF:30:16:01:C7:2F")  // 6 bytes
salt = reverse(mac_bytes)                // [0x2F, 0xC7, 0x01, 0x16, 0x30, 0xCF]

for i in 0..6:
    device_key[i] = (base_key[i] + salt[i]) % 255
    device_iv[i]  = (base_iv[i]  + salt[i]) % 255
```

Bytes 6..15 of the key and IV remain unchanged.

### Chunk Encryption (GAN)

GAN protocols only encrypt the first and last 16-byte chunks of each message:

**Encryption:** encrypt chunk at offset 0, then chunk at `len - 16`.
**Decryption:** decrypt chunk at `len - 16`, then chunk at offset 0.

For messages of exactly 16 bytes, only one chunk operation is performed.

### Double-Pass Encryption (MoYu V3)

MoYu V3 uses two independent AES-CBC operations per packet:

**Decryption:**
1. Decrypt bytes `[4..20]` (fresh CBC context)
2. Decrypt bytes `[0..16]` (fresh CBC context)

**Encryption:**
1. Encrypt bytes `[0..16]` (fresh CBC context)
2. Encrypt bytes `[4..20]` (fresh CBC context)

---

## Cube State Representation

All protocols encode cube state using Corner/Edge Permutation and Orientation:

- **CP** — Corner Permutation: 8 values (0-7)
- **CO** — Corner Orientation: 8 values (0-2)
- **EP** — Edge Permutation: 12 values (0-11)
- **EO** — Edge Orientation: 12 values (0-1)

The last element of each array is derived (not transmitted):
```
CP[7] = 28 - sum(CP[0..7])
CO[7] = (3 - sum(CO[0..7]) % 3) % 3
EP[11] = 66 - sum(EP[0..11])
EO[11] = (2 - sum(EO[0..11]) % 2) % 2
```

Corner order: URF, UFL, ULB, UBR, DFR, DLF, DBL, DRB.
Edge order: UR, UF, UL, UB, DR, DF, DL, DB, FR, FL, BL, BR.

Solved state in Kociemba notation:
```
UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
```

---

## Acknowledgements

Protocol specifications in this crate were built from the following
open-source references:

- **GAN Gen2/3/4** — [gan-web-bluetooth](https://github.com/afedotov/gan-web-bluetooth) by Andy Fedotov (MIT)
- **Giiker** — [giiker](https://github.com/hakatashi/giiker) by hakatashi (MIT)
- **GoCube** — [gocube-protocol](https://github.com/oddpetersson/gocube-protocol) by oddpetersson (MIT)
- **Multi-cube** — [cstimer](https://github.com/cs0x7f/cstimer) by Shuang Chen (GPLv3) — used as cross-reference for all protocols
