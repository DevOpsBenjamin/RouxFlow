# MoYu WeiLong V10 Bluetooth Protocol (API v3)

This document descibes the Bluetooth LE protocol reverse-engineered from the MoYu WeiLong V10 AI smart cube. This version, identified as "API v3", uses a unique custom double-pass AES encryption scheme.

## 1. Connectivity

- **Device Name**: `WCU_MY32_XXXX`
- **Service UUID**: `0783b03e-7735-b5a0-1760-a305d2795cb0`
- **Read Characteristic (Notify)**: `0783b03e-7735-b5a0-1760-a305d2795cb1`
- **Write Characteristic**: `0783b03e-7735-b5a0-1760-a305d2795cb2`

## 2. Encryption Keys

The protocol uses a static derivation from the device MAC address using two hardcoded master arrays found in the official driver.

### Master Constants
```rust
const MASTER_KEY: [u8; 16] = [21, 119, 58, 92, 103, 14, 45, 31, 23, 103, 42, 19, 155, 103, 82, 87];
const MASTER_IV: [u8; 16]  = [17, 35, 38, 37, 134, 42, 44, 59, 85, 6, 127, 49, 126, 103, 33, 87];
```

### Derivation Algorithm
For a MAC address `AA:BB:CC:DD:EE:FF`:
1. Parse MAC bytes in reverse order (FF, EE, DD...).
2. For each byte `i` (0 to 5):
   - `DeviceKey[i] = (MASTER_KEY[i] + MacByte[i]) % 255`
   - `DeviceIV[i]  = (MASTER_IV[i]  + MacByte[i]) % 255`
3. Identify the remaining bytes (6 to 15) directly from the Master arrays.

## 3. Double Overlapping Pass (The Secret)

Standard AES-CBC cannot decrypt MoYu V3 packets directly. The protocol uses a **two-pass decryption** with overlap on the 20-byte packets.

**Algorithm:**
1. **Pass 1 (Tail)**: Decrypt the **LAST 16 bytes** (indices 4 to 20) using AES-128-CBC with `DeviceKey` and `DeviceIV`.
   - *Crucial*: This modifies bytes 4-15 in place.
2. **Pass 2 (Head)**: Decrypt the **FIRST 16 bytes** (indices 0 to 16) using the same key and IV.
   - This uses the *already decrypted* bytes from Pass 1 as part of its ciphertext input.

**Implementation (Rust):**
```rust
// Pass 1: Tail
dec.decrypt_block(&mut buffer[4..20]);
// Pass 2: Head (overlapping)
dec.decrypt_block(&mut buffer[0..16]);
```

## 4. Handshake Sequence

To enable high-frequency data (Gyro/Moves), send these 4 commands (encrypted with the Double Pass algorithm):

1. `A0 00 ...` (Hello)
2. `A2 00 ...` (Setup)
3. `00 00 ...` (Sync)
4. `A6 00 ...` (Fast Mode Enable)

## 5. Packet Format (Decrypted)

All packets are 20 bytes. Byte 0 is the Opcode.

| Opcode | Name | Description |
| :--- | :--- | :--- |
| `0xA1` | **INFO** | Device name ("WCU_MY32...") |
| `0xA3` | **STATE** | Internal facelet state (compressed) |
| `0xA4` | **BATT** | Battery level (Byte 1 = %) |
| `0xA5` | **MOVE** | Face turn event |
| `0xAB` | **GYRO** | High-freq IMU data (Quaternion/Raw) |

### Move Packet (`0xA5`)
- **Byte 1**: Face (`00=U`, `01=D`, `02=L`, `03=R`, `04=F`, `05=B`) - *To be verified*
- **Byte 2**: Direction
- **Byte 3-10**: Timestamp / Latency data
- **Byte 11**: Global Move Counter (increments strictly)

## 6. Credits

Reverse-engineered for the **RouxFlow** project.
Released under MIT License.
