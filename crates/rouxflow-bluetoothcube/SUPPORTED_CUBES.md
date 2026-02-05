# Supported Bluetooth Smart Cubes

This document lists all 3x3 Bluetooth smart cubes known to RouxFlow, along with
their current verification status.

> **RouxFlow is a Roux method trainer.** Only 3x3 cubes are supported.

---

## Verification Status

| Status | Meaning |
|--------|---------|
| Verified | Tested and confirmed working with physical hardware |
| Unverified | Protocol implemented based on documentation; awaiting hardware confirmation |

The maintainer currently owns:
- **MoYu WeiLong V10** (verified)
- **GoCube** by Particula (owned, protocol not yet tested)

All other cubes are listed based on protocol analysis and community documentation.
If you own one of these cubes and can help test, contributions are very welcome!

---

## Cube List

### GAN

| Cube | Protocol | Features | Status |
|------|----------|----------|--------|
| GAN 356i | GAN v1 | Battery, HW Info | Unverified |
| GAN 356i Play | GAN v1 | Battery, HW Info | Unverified |
| GAN 356i 2 | GAN v1 | Battery, HW Info | Unverified |
| GAN 356i 2 Play | GAN v1 | Battery, HW Info | Unverified |
| GAN Mini ui FreePlay | GAN Gen2 | Gyro, Battery, HW Info | Unverified |
| GAN 12 ui FreePlay | GAN Gen2 | Gyro, Battery, HW Info | Unverified |
| GAN 12 ui | GAN Gen2 | Gyro, Battery, HW Info | Unverified |
| GAN 356 i Carry S | GAN Gen2 | Battery, HW Info | Unverified |
| GAN 356 i Carry | GAN Gen2 | Battery, HW Info | Unverified |
| GAN 356 i Carry E | GAN Gen2 | Battery, HW Info | Unverified |
| GAN 356 i 3 | GAN Gen2 | Gyro, Battery, HW Info | Unverified |
| GAN 356 i Carry 2 | GAN Gen3 | Battery, HW Info, Move History | Unverified |
| GAN 12 ui Maglev | GAN Gen4 | Gyro, Battery, HW Info, Move History | Unverified |
| GAN 14 ui FreePlay | GAN Gen4 | Battery, HW Info, Move History | Unverified |
| GAN 356 i Carry 4 | GAN Gen4 | Battery, HW Info, Move History | Unverified |

### Monster Go (GAN sub-brand)

| Cube | Protocol | Features | Status |
|------|----------|----------|--------|
| Monster Go 3Ai | GAN Gen2 | Battery, HW Info | Unverified |

### MoYu

| Cube | Protocol | Features | Status |
|------|----------|----------|--------|
| MoYu AI 2023 | MoYu AI | Gyro, Battery, HW Info | Unverified |
| MoYu AI v2 | MoYu AI | Battery, HW Info | Unverified |
| MoYu WeiLong V10 | MoYu V3 | Gyro, Battery, HW Info | **Verified** |

### Xiaomi / Giiker

| Cube | Protocol | Features | Status |
|------|----------|----------|--------|
| Giiker i3 | Giiker v1 | Battery | Unverified |
| Giiker i3S | Giiker v1 | Battery | Unverified |
| Giiker i3Y | Giiker v1 | Battery | Unverified |
| Mi Smart Magic Cube | Giiker v1 | Battery | Unverified |

### Particula

| Cube | Protocol | Features | Status |
|------|----------|----------|--------|
| GoCube | GoCube | Battery | Unverified |
| Rubik's Connected | GoCube | Battery | Unverified |

### QiYi

| Cube | Protocol | Features | Status |
|------|----------|----------|--------|
| QiYi Tornado V4 (SCS) | QiYi Smart | Battery | Unverified |
| QiYi Tornado V4 AI | QiYi Smart | Battery | Unverified |
| QiYi AI 3x3 | QiYi Smart | Battery | Unverified |

---

## How to Help

If you own a Bluetooth smart cube and want to help verify support:

1. Connect your cube with RouxFlow
2. Check if moves are detected correctly
3. Check if battery reporting works
4. Open an issue or PR with your results

Even a quick "it works" / "it doesn't work" report is valuable!
