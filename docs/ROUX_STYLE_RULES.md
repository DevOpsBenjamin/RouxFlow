# Roux Method Style Detection & Evaluation Rules

This document defines the rules for analyzing Roux solves and detecting style quality.
It serves as the specification for the RouxFlow Style Analyzer.

---

## 1. Roux Method Overview

The Roux method consists of 4 main phases:

| Phase | Goal | Optimal Move Count |
|-------|------|-------------------|
| **FB (First Block)** | Build 1x2x3 block on left | 6-9 moves |
| **SB (Second Block)** | Build 1x2x3 block on right | 8-12 moves |
| **CMLL** | Orient & permute top corners | 7-11 moves (alg dependent) |
| **LSE** | Solve last 6 edges (4 U + 2 M-slice) | 10-16 moves |

**Total optimal solve: ~35-45 moves** (sub-40 for advanced)

---

## 2. Move Classification by Phase

### 2.1 First Block (FB)

**Purpose:** Build a 1x2x3 block on the LEFT side (L face + adjacent D/F/B edges).

#### ✅ Good Moves (Roux-native)
| Move | Reason |
|------|--------|
| `L, L', L2` | Direct manipulation of FB pieces |
| `U, U', U2` | Setup moves, very common |
| `B, B', B2` | Back insertion, common for DB edge |
| `D, D', D2` | Bottom setup, used for DL edge |
| `l, l', l2` | Wide left, keeps M-slice free |

#### ⚠️ Acceptable Moves (use sparingly)
| Move | Reason |
|------|--------|
| `F, F', F2` | Front setup, OK but often avoidable |
| `r, r', r2` | Usually not needed in FB, but not harmful |
| `M, M', M2` | Rare in FB, can be used for edge setup |

#### ❌ Bad Moves (anti-pattern)
| Move | Reason | Penalty |
|------|--------|---------|
| `R, R', R2` | Moves RIGHT side, not FB-related | HIGH |
| `S, S', S2` | Slice move, almost never needed | HIGH |
| `E, E', E2` | Equator slice, disrupts D layer | MEDIUM |
| `x, y, z` (excessive) | Rotations = lost time | MEDIUM per extra |

#### FB Quality Metrics
- **Move count:** Target ≤9, Flag if >12
- **R moves:** Should be 0, Flag any occurrence
- **S/E moves:** Should be 0, Flag any occurrence
- **Rotations (after x2 y setup):** Target ≤1, Flag if >2

---

### 2.2 Second Block (SB)

**Purpose:** Build a 1x2x3 block on the RIGHT side while keeping FB intact.

#### ✅ Good Moves (Roux-native)
| Move | Reason |
|------|--------|
| `R, R', R2` | Direct manipulation of SB pieces |
| `U, U', U2` | Setup, universal |
| `M, M', M2` | M-slice manipulation, core Roux technique |
| `r, r', r2` | Wide R = R + M', very Roux-native |

#### ⚠️ Acceptable Moves (use sparingly)
| Move | Reason |
|------|--------|
| `D, D', D2` | Can be used for DR edge, but risky |

#### ❌ Bad Moves (anti-pattern)
| Move | Reason | Penalty |
|------|--------|---------|
| `L, L', L2` | DESTROYS FB! | CRITICAL |
| `l, l', l2` | DESTROYS FB! | CRITICAL |
| `F, F', F2` | Usually breaks FB, CFOP-style | HIGH |
| `B, B', B2` | Usually breaks FB, CFOP-style | HIGH |
| `S, S', S2` | Slice move, disrupts both blocks | HIGH |
| `E, E', E2` | Equator slice, disrupts D layer | HIGH |
| `x, y, z` | Rotations = lost efficiency | MEDIUM per rotation |

#### SB Quality Metrics
- **Move count:** Target ≤12, Flag if >16
- **L/l moves:** CRITICAL error, should be 0
- **F/B moves:** Should be 0, Flag any occurrence
- **M/r usage ratio:** Higher = more Roux-native (target >30% of SB moves)
- **Rotations:** Target 0, Flag any occurrence

#### Common SB Anti-Patterns
| Anti-Pattern | Roux Alternative |
|--------------|------------------|
| `F R' F' R` | `M' U M` or `r U' r'` |
| `B' R B R'` | `M U' M'` or `r' U r` |
| `R B' R' B` | Use M-slice setup instead |
| `y R U R'` | Stay orientation, use `r U r'` |

---

### 2.3 CMLL (Corners of Last Layer)

**Purpose:** Orient and permute the 4 top corners while keeping M-slice free.

#### ✅ Good Moves
| Move | Reason |
|------|--------|
| `R, R', R2` | Standard CMLL triggers |
| `U, U', U2` | AUF and algorithm moves |
| `F, F'` | Part of many CMLL algs |
| `L, L'` | Some algs use L (Sune variants) |

#### ⚠️ Acceptable Moves
| Move | Reason |
|------|--------|
| `D, D'` | Rare, some algs use D |

#### ❌ Bad Moves
| Move | Reason | Penalty |
|------|--------|---------|
| `M, M', M2` | Should NOT appear in CMLL | HIGH |
| `r, r', r2` | Should NOT appear in CMLL | HIGH |
| `l, l', l2` | Should NOT appear in CMLL | HIGH |
| `B, B', B2` | Very rare in CMLL, likely wrong alg | MEDIUM |
| `S, E` | Never in CMLL | HIGH |

#### CMLL Quality Metrics
- **Move count:** Target 7-11 (alg dependent)
- **Recognition time:** (requires timestamps)
- **M/r/l moves:** Should be 0
- **Algorithm identification:** Match against known CMLL algs

---

### 2.4 LSE (Last Six Edges)

**Purpose:** Solve the remaining 6 edges using only M and U moves.

LSE has 3 sub-phases:
1. **EO (Edge Orientation):** Orient all 6 edges
2. **UL/UR:** Place UL and UR edges
3. **L4E (Last 4 Edges):** Solve remaining 4 M-slice edges

#### ✅ Good Moves (ONLY these)
| Move | Reason |
|------|--------|
| `M, M', M2` | M-slice manipulation |
| `U, U', U2` | Top layer manipulation |

#### ❌ Bad Moves (ALL others)
| Move | Reason | Penalty |
|------|--------|---------|
| `R, R', R2` | Destroys blocks | CRITICAL |
| `L, L', L2` | Destroys blocks | CRITICAL |
| `F, F', F2` | Destroys blocks | CRITICAL |
| `B, B', B2` | Destroys blocks | CRITICAL |
| `D, D', D2` | Destroys blocks | CRITICAL |
| `r, r', r2` | Destroys SB | CRITICAL |
| `l, l', l2` | Destroys FB | CRITICAL |
| `S, E` | Wrong slice | HIGH |
| `x, y, z` | Rotations in LSE = very bad | HIGH |

#### LSE Quality Metrics
- **EO move count:** Target 0-6, Flag if >8
- **UL/UR move count:** Target 3-6
- **L4E move count:** Target 4-8
- **Total LSE:** Target ≤16, Flag if >20
- **Non-M/U moves:** CRITICAL, should be 0
- **Bad edge count progression:** Should decrease monotonically

#### LSE Anti-Patterns
| Anti-Pattern | Issue |
|--------------|-------|
| Many `M M'` sequences | Wasted moves, hesitation |
| `M2 M2` | Cancels out, wasted |
| Rotations during LSE | Should never happen |

---

## 3. Global Solve Metrics

### 3.1 Roux Purity Score

Calculate a score from 0-100 based on:

```
Score = 100 - (Penalties)

Penalties:
- CRITICAL error: -20 points each
- HIGH error: -10 points each  
- MEDIUM error: -5 points each
- Move count over target: -1 point per extra move
```

### 3.2 Style Classification

| Score | Classification |
|-------|----------------|
| 90-100 | **Elite Roux** - Clean, efficient, native style |
| 75-89 | **Good Roux** - Minor inefficiencies |
| 60-74 | **Developing Roux** - Some CFOP habits |
| 40-59 | **Hybrid Style** - Mixed CFOP/Roux |
| 0-39 | **CFOP with Roux steps** - Needs fundamentals work |

### 3.3 Phase Balance

A balanced Roux solve should have roughly:
- FB: 15-20% of moves
- SB: 20-25% of moves
- CMLL: 18-22% of moves
- LSE: 30-35% of moves

Flag if any phase is significantly outside these ranges.

---

## 4. Rotation Analysis

### 4.1 Acceptable Rotations

| Phase | Acceptable | Flag if |
|-------|------------|---------|
| **Inspection → FB** | `x2 y` or `x2 y'` | More than 2 rotations |
| **FB → SB** | 0-1 | Any rotation |
| **SB → CMLL** | 0 | Any rotation |
| **CMLL → LSE** | 0 | Any rotation |
| **During LSE** | 0 | Any rotation (CRITICAL) |

### 4.2 Rotation Efficiency

- **Total rotations (excluding inspection):** Target ≤2, Flag if >4
- **Rotations per phase:** Flag any rotation after FB

---

## 5. Common Mistake Detection

### 5.1 "CFOP Disguised as Roux"

Patterns that indicate CFOP thinking:
- Using `F/B` moves during SB instead of `M/r`
- Heavy use of `D` moves during SB
- Rotations to set up pieces instead of M-slice manipulation
- `S` moves anywhere (very CFOP-like)

### 5.2 "Inefficient M-slice Usage"

Patterns that indicate poor M-slice understanding:
- `M M'` or `M' M` sequences (wastes 2 moves)
- `M2 M2` (wastes 4 moves!)
- Using `M` when `M'` would solve in fewer moves

### 5.3 "Block Destruction"

CRITICAL errors where blocks are broken:
- `L` moves during SB (breaks FB)
- `R` moves during LSE (breaks SB)
- `F/B/D` moves during LSE (breaks both blocks)

---

## 6. Implementation Notes

### 6.1 Detection Priority

1. **CRITICAL errors first** - Block destruction, non-M/U in LSE
2. **Phase validation** - Correct phase detection
3. **Move classification** - Good/acceptable/bad per phase
4. **Efficiency metrics** - Move count, rotations

### 6.2 Output Format

For each solve, output:
```
=== ROUX STYLE ANALYSIS ===
Roux Purity Score: 72/100 (Developing Roux)

Phase Breakdown:
- FB: 14 moves (⚠️ over target) - 1 suspicious move (S')
- SB: 20 moves (⚠️ over target) - 2 bad moves (F, F')
- CMLL: 26 moves (alg unknown)
- LSE: 24 moves (⚠️ over target)

Issues Detected:
[HIGH] SB: F move detected - consider using M' or r instead
[HIGH] SB: F' move detected - consider using M' or r instead
[MEDIUM] FB: S' move detected - unusual, check if necessary
[MEDIUM] SB: M M' sequence - wasted moves

Recommendations:
1. Practice SB with only R/U/M/r moves
2. Learn proper M-slice insertions for SB pairs
3. Review LSE algorithms for efficiency
```

---

## 7. References

- [Kian Mansour's Roux Tutorial](https://www.youtube.com/watch?v=example)
- [SpeedSolving Roux Method Wiki](https://www.speedsolving.com/wiki/index.php/Roux_method)
- [Roux Method subreddit FAQ](https://reddit.com/r/rouxcubing)

---

*Document version: 1.0*
*Last updated: 2026-02-03*
