# WCA-Official 3x3 Scramble Generation: Algorithm Reference

## Goal

Reimplement the WCA-official 3x3 scramble generator in Rust (for `rouxflow-ai`), using the same Kociemba two-phase algorithm and WCA parameters as TNoodle.

## License

| Source | License | Notes |
|---|---|---|
| [TNoodle](https://github.com/thewca/tnoodle) | AGPL-3.0 | WCA's official scramble program |
| [tnoodle-lib](https://github.com/thewca/tnoodle-lib) | GPL-3.0 | Contains min2phase Java solver |
| [min2phase](https://github.com/cs0x7f/min2phase) | GPL-3.0 | Optimized Kociemba implementation |
| [Kociemba's Two-Phase Algorithm](https://kociemba.org/math/twophase.htm) | Published math | The algorithm behind all of the above |

**GPL protects code, not algorithms.** We reimplement the same algorithm + WCA parameters in Rust from the mathematical description. MIT-licensed.

---

## How WCA Scrambles Work

1. **Generate a uniformly random valid cube state** (~4.3×10¹⁹ possible)
2. **Solve it** with Kociemba's two-phase algorithm (≤21 moves)
3. **Invert the solution** — the inverse IS the scramble

This guarantees every scramble produces a truly random position.

---

## Part 1: Cube Representation (CubieCube)

### Piece Numbering

**8 Corners** (index → name → facelets):
```
0: URF  (U9, R1, F3)
1: UFL  (U7, F1, L3)
2: ULB  (U1, L1, B3)
3: UBR  (U3, B1, R3)
4: DFR  (D3, F9, R7)
5: DLF  (D1, L9, F7)
6: DBL  (D7, B9, L7)
7: DRB  (D9, R9, B7)
```

**12 Edges** (index → name → facelets):
```
 0: UR  (U6, R2)
 1: UF  (U8, F2)
 2: UL  (U4, L2)
 3: UB  (U2, B2)
 4: DR  (D6, R8)
 5: DF  (D2, F8)
 6: DL  (D4, L8)
 7: DB  (D8, B8)
 8: FR  (F6, R4)
 9: FL  (F4, L6)
10: BL  (B6, L4)
11: BR  (B4, R6)
```

**Facelet numbering per face** (looking at the face):
```
1 2 3
4 5 6
7 8 9
```
Centers (5) are fixed. Face order: U=0, R=1, F=2, D=3, L=4, B=5.

### Data Structure

```rust
struct CubieCube {
    cp: [u8; 8],   // corner permutation: cp[i] = which corner is in slot i
    co: [u8; 8],   // corner orientation: co[i] = twist of corner in slot i (0, 1, 2)
    ep: [u8; 12],  // edge permutation: ep[i] = which edge is in slot i
    eo: [u8; 12],  // edge orientation: eo[i] = flip of edge in slot i (0, 1)
}
```

Identity (solved) state: `cp = [0,1,2,3,4,5,6,7]`, `co = [0;8]`, `ep = [0,1,2,3,4,5,6,7,8,9,10,11]`, `eo = [0;12]`.

### Corner Orientation Convention

When a corner is in its home position with correct orientation, `co = 0`.
- Twist CW (120° clockwise looking at the corner from outside) → `co = 1`
- Twist CCW (120° counter-clockwise) → `co = 2`

Specifically: The U/D facelet of each corner defines orientation. If U/D facelet is on U or D face → 0. If U/D facelet is rotated CW → 1. If CCW → 2.

### Edge Orientation Convention

An edge is correctly oriented (`eo = 0`) if it can be solved using only {U, D, R2, L2, F2, B2} moves (the G1 subgroup). Otherwise `eo = 1`.

Practical rule: An edge is flipped if an F or B quarter-turn was needed to put it in place.

---

## Part 2: Cube Multiplication (State Composition)

Applying move B to state A: `C = A * B`

```rust
fn multiply(a: &CubieCube, b: &CubieCube) -> CubieCube {
    let mut c = CubieCube::default();
    for i in 0..8 {
        // Corner in slot i of C: take from slot b.cp[i] of A
        c.cp[i] = a.cp[b.cp[i] as usize];
        // Orientation: add A's orientation at that slot + B's orientation, mod 3
        c.co[i] = (a.co[b.cp[i] as usize] + b.co[i]) % 3;
    }
    for i in 0..12 {
        c.ep[i] = a.ep[b.ep[i] as usize];
        // Edge flip: XOR (add mod 2)
        c.eo[i] = (a.eo[b.ep[i] as usize] + b.eo[i]) % 2;
    }
    c
}
```

---

## Part 3: The 18 Basic Moves as CubieCubes

Move index encoding: `face * 3 + power` where face ∈ {U=0, R=1, F=2, D=3, L=4, B=5}, power ∈ {0=90°CW, 1=180°, 2=90°CCW(=270°CW)}.

So: U=0, U2=1, U'=2, R=3, R2=4, R'=5, F=6, F2=7, F'=8, D=9, D2=10, D'=11, L=12, L2=13, L'=14, B=15, B2=16, B'=17.

Only 6 base moves (90° CW) need defining. 180° = apply twice, 270° = apply three times.

### U (rotate top face 90° CW looking from top)
```
cp: [3, 0, 1, 2, 4, 5, 6, 7]   // URF←UBR, UFL←URF, ULB←UFL, UBR←ULB
co: [0, 0, 0, 0, 0, 0, 0, 0]   // U doesn't change corner orientation
ep: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11]  // UR←UB, UF←UR, UL←UF, UB←UL
eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

### R (rotate right face 90° CW looking from right)
```
cp: [4, 1, 2, 0, 7, 5, 6, 3]   // URF←DFR, UBR←URF, DFR←DRB, DRB←UBR
co: [2, 0, 0, 1, 1, 0, 0, 2]   // corners twist
ep: [8, 1, 2, 3, 11, 5, 6, 7, 4, 9, 10, 0]  // UR←FR, FR←DR, DR←BR, BR←UR
eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]   // R doesn't flip edges
```

### F (rotate front face 90° CW looking from front)
```
cp: [1, 5, 2, 3, 0, 4, 6, 7]   // URF←UFL, UFL←DLF, DFR←URF, DLF←DFR
co: [1, 2, 0, 0, 2, 1, 0, 0]
ep: [0, 9, 2, 3, 4, 1, 6, 7, 5, 8, 10, 11]  // UF←FL, FL←DF, DF←FR, FR←UF
eo: [0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0]   // F flips the 4 edges it touches
```

### D (rotate bottom face 90° CW looking from bottom)
```
cp: [0, 1, 2, 3, 5, 6, 7, 4]   // DFR←DLF, DLF←DBL, DBL←DRB, DRB←DFR
co: [0, 0, 0, 0, 0, 0, 0, 0]
ep: [0, 1, 2, 3, 5, 6, 7, 4, 8, 9, 10, 11]  // DR←DF, DF←DL, DL←DB, DB←DR
eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

### L (rotate left face 90° CW looking from left)
```
cp: [0, 2, 6, 3, 4, 1, 5, 7]   // UFL←ULB, ULB←DBL, DLF←UFL, DBL←DLF
co: [0, 1, 2, 0, 0, 2, 1, 0]
ep: [0, 1, 10, 3, 4, 5, 2, 7, 8, 6, 9, 11]  // UL←BL, BL←DL, DL←FL, FL←UL
eo: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

### B (rotate back face 90° CW looking from back)
```
cp: [0, 1, 3, 7, 4, 5, 2, 6]   // ULB←UBR, UBR←DRB, DRB←DBL, DBL←ULB
co: [0, 0, 1, 2, 0, 0, 2, 1]
ep: [0, 1, 2, 11, 4, 5, 6, 3, 8, 9, 7, 10]  // UB←BR, BR←DB, DB←BL, BL←UB
eo: [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1]   // B flips the 4 edges it touches
```

**Key pattern:** U and D don't change any orientations. R and L don't flip edges but twist corners. F and B both twist corners AND flip edges.

---

## Part 4: Coordinate Computation

### twist (corner orientation) — range [0, 2187)

```rust
fn get_twist(co: &[u8; 8]) -> u16 {
    let mut twist: u16 = 0;
    for i in 0..7 {  // only 7 corners, 8th is determined
        twist = twist * 3 + co[i] as u16;
    }
    twist
}

fn set_twist(co: &mut [u8; 8], mut twist: u16) {
    let mut sum = 0u8;
    for i in (0..7).rev() {
        co[i] = (twist % 3) as u8;
        sum += co[i];
        twist /= 3;
    }
    co[7] = (3 - sum % 3) % 3;  // 8th corner determined by constraint
}
```

### flip (edge orientation) — range [0, 2048)

```rust
fn get_flip(eo: &[u8; 12]) -> u16 {
    let mut flip: u16 = 0;
    for i in 0..11 {  // only 11 edges, 12th is determined
        flip = flip * 2 + eo[i] as u16;
    }
    flip
}

fn set_flip(eo: &mut [u8; 12], mut flip: u16) {
    let mut sum = 0u8;
    for i in (0..11).rev() {
        eo[i] = (flip % 2) as u8;
        sum += eo[i];
        flip /= 2;
    }
    eo[11] = (2 - sum % 2) % 2;
}
```

### udslice (UD-slice edge positions, unordered) — range [0, 495)

The 4 UD-slice edges are FR(8), FL(9), BL(10), BR(11). We track which 4 of the 12 edge slots contain slice edges, ignoring order.

This is a combination number. We rank which 4 slots (out of 12) contain slice edges using the combinatorial number system.

```rust
fn get_udslice(ep: &[u8; 12]) -> u16 {
    // Which slots contain slice edges (ep[slot] >= 8)?
    let mut occupied = [false; 12];
    for i in 0..12 {
        if ep[i] >= 8 { occupied[i] = true; }
    }
    // Rank using combinatorial number system
    // Count combinations: choose k items from positions 0..n
    let mut k = 4i32;
    let mut val: u16 = 0;
    for i in (0..12).rev() {
        if occupied[i] {
            k -= 1;
        } else if k >= 0 {
            val += c_nk(i as u32, k as u32) as u16;
        }
    }
    val
}
```

Where `c_nk(n, k)` = binomial coefficient C(n, k). Precompute a small table.

**Goal:** udslice = 0 means all 4 slice edges are in slots 8, 9, 10, 11 (the slice).

### Lehmer Code for Permutations — cperm, eperm

```rust
/// Encode a permutation of n elements as an integer in [0, n!)
fn perm_to_index(perm: &[u8], n: usize) -> u32 {
    let mut idx: u32 = 0;
    for i in 0..n {
        idx *= (n - i) as u32;
        for j in (i+1)..n {
            if perm[j] < perm[i] {
                idx += 1;
            }
        }
    }
    idx
}

/// Decode an integer in [0, n!) to a permutation of n elements
fn index_to_perm(mut idx: u32, n: usize, perm: &mut [u8]) {
    let mut available: Vec<u8> = (0..n as u8).collect();
    for i in 0..n {
        let fact = factorial(n - 1 - i) as u32;
        let k = (idx / fact) as usize;
        idx %= fact;
        perm[i] = available[k];
        available.remove(k);
    }
}
```

**cperm:** `perm_to_index(&cube.cp, 8)` → range [0, 40320)

**eperm (phase 2):** Permutation of the 8 non-slice edges only (indices 0–7 of ep). `perm_to_index(&cube.ep[0..8], 8)` → range [0, 40320)

**udslice2 (phase 2):** Permutation of the 4 slice edges. Take `cube.ep[8..12]`, subtract 8 from each value, then `perm_to_index(_, 4)` → range [0, 24)

### Parity

```rust
fn perm_parity(perm: &[u8], n: usize) -> u8 {
    let mut parity = 0u8;
    for i in 0..n {
        for j in (i+1)..n {
            if perm[i] > perm[j] {
                parity ^= 1;
            }
        }
    }
    parity  // 0 = even, 1 = odd
}
```

---

## Part 5: Move Tables

For each coordinate, precompute the result of applying each move.

### Construction Algorithm

```rust
// Example: twist_move table
let mut twist_move = [[0u16; 18]; 2187];
for twist in 0..2187 {
    let mut cube = CubieCube::default();
    set_twist(&mut cube.co, twist as u16);
    for move_idx in 0..18 {
        let moved = multiply(&cube, &MOVES[move_idx]);
        twist_move[twist][move_idx] = get_twist(&moved.co);
    }
}
```

Same pattern for flip_move, udslice_move, cperm_move, eperm_move, udslice2_move.

**Phase 2 move indices:** Only 10 of the 18 moves are valid in phase 2.
```
Phase 2 moves (indices into the 18-move array):
  U=0, U2=1, U'=2,         (U quarter-turns OK)
  R2=4,                      (R half-turn only)
  F2=7,                      (F half-turn only)
  D=9, D2=10, D'=11,        (D quarter-turns OK)
  L2=13,                     (L half-turn only)
  B2=16                      (B half-turn only)
```
Map: phase2_move_idx [0..10] → [0, 1, 2, 4, 7, 9, 10, 11, 13, 16]

---

## Part 6: Pruning Tables

### Construction via BFS

Each pruning table maps a coordinate pair → minimum moves to reach goal (0).

```rust
// Example: twist_udslice_prun[2187 * 495]
// 4-bit packed: two entries per byte
let table_size = 2187 * 495;
let mut prun = vec![0xFFu8; (table_size + 1) / 2];  // init to 0xF (=15, "not visited")

fn get_prun(table: &[u8], idx: usize) -> u8 {
    if idx & 1 == 0 { table[idx / 2] & 0x0F } else { table[idx / 2] >> 4 }
}

fn set_prun(table: &mut [u8], idx: usize, val: u8) {
    if idx & 1 == 0 {
        table[idx / 2] = (table[idx / 2] & 0xF0) | (val & 0x0F);
    } else {
        table[idx / 2] = (table[idx / 2] & 0x0F) | (val << 4);
    }
}

// BFS from goal state
set_prun(&mut prun, 0, 0);  // twist=0, udslice=0 → distance 0
let mut depth = 0u8;
let mut filled = 1;
while filled < table_size {
    for twist in 0..2187 {
        for udslice in 0..495 {
            let idx = twist * 495 + udslice;
            if get_prun(&prun, idx) != depth { continue; }
            // Try all 18 moves
            for m in 0..18 {
                let new_twist = twist_move[twist][m] as usize;
                let new_udslice = udslice_move[udslice][m] as usize;
                let new_idx = new_twist * 495 + new_udslice;
                if get_prun(&prun, new_idx) == 0x0F {
                    set_prun(&mut prun, new_idx, depth + 1);
                    filled += 1;
                }
            }
        }
    }
    depth += 1;
}
```

Same pattern for flip_udslice_prun, cperm_udslice2_prun (using phase-2 moves only), eperm_udslice2_prun (phase-2 moves only).

**Optimization:** For large tables, reverse the BFS direction at higher depths (search from unvisited entries looking for `depth` neighbors). min2phase switches at depth 3.

---

## Part 7: IDA* Search

### Move Filtering

Never apply the same face twice consecutively. Never apply opposite faces three times (e.g., U then D then U — reorder to U U D instead).

Faces 0&3 (U&D), 1&4 (R&L), 2&5 (F&B) are opposite pairs. They share the same axis: `face % 3 == opposite_face % 3`. Opposite faces commute (U D = D U), so we enforce canonical ordering to avoid searching duplicates.

```rust
/// Returns true if move `m` is allowed after `last_move`.
/// face = m / 3. Opposite faces share axis: face % 3.
fn is_move_allowed(m: usize, last_move: i8) -> bool {
    if last_move < 0 { return true; }  // no previous move
    let face = m / 3;
    let last_face = last_move as usize / 3;
    if face == last_face { return false; }  // same face consecutively
    // Opposite faces commute: enforce face > last_face to avoid e.g. U D U = U U D
    if face % 3 == last_face % 3 && face < last_face { return false; }
    true
}
```

### Phase 1 Search

```rust
fn phase1_search(
    twist: usize, flip: usize, udslice: usize,
    depth: u8, max_depth: u8, last_move: i8,
    solution: &mut Vec<u8>,
    // ... move tables, pruning tables
) -> bool {
    // Pruning
    let prun = max(
        get_prun(&twist_udslice_prun, twist * 495 + udslice),
        get_prun(&flip_udslice_prun, flip * 495 + udslice),
    );
    if prun > max_depth - depth { return false; }  // can't reach goal in remaining moves

    // Goal check
    if depth == max_depth {
        return twist == 0 && flip == 0 && udslice == 0;
    }

    for m in 0..18u8 {
        if !is_move_allowed(m as usize, last_move) { continue; }

        let new_twist = twist_move[twist][m as usize] as usize;
        let new_flip = flip_move[flip][m as usize] as usize;
        let new_udslice = udslice_move[udslice][m as usize] as usize;

        solution.push(m);
        if phase1_search(new_twist, new_flip, new_udslice,
                         depth + 1, max_depth, m as i8, solution, ..) {
            return true;  // don't return yet — try Phase 2
        }
        solution.pop();
    }
    false
}
```

### Phase 1 → Phase 2 Transition

When phase 1 reaches goal (twist=0, flip=0, udslice=0), we need phase-2 coordinates. At this point the cube is in G1 but we need cperm, eperm, udslice2.

**How:** Reconstruct the full CubieCube by applying the phase-1 solution moves to the initial state, then compute phase-2 coordinates from it.

```rust
// After phase 1 finds a solution of length `n`:
let mut cube = initial_cube.clone();
for &m in &solution[0..n] {
    cube = multiply(&cube, &MOVES[m as usize]);
}
let cperm = perm_to_index(&cube.cp, 8) as usize;
let eperm = perm_to_index(&cube.ep[0..8], 8) as usize;
let udslice2 = perm_to_index_4(&cube.ep[8..12]) as usize;  // subtract 8 first
// Now search phase 2 from (cperm, eperm, udslice2)
```

### Phase 2 Search

Identical structure to phase 1 but:
- Uses cperm, eperm, udslice2 coordinates
- Uses cperm_move, eperm_move, udslice2_move tables
- Uses cperm_udslice2_prun, eperm_udslice2_prun tables
- Only 10 allowed moves (not 18)
- Goal: cperm=0, eperm=0, udslice2=0

### Outer Loop

```rust
fn solve(cube: &CubieCube, max_total: u8) -> Vec<u8> {
    let twist = get_twist(&cube.co) as usize;
    let flip = get_flip(&cube.eo) as usize;
    let udslice = get_udslice(&cube.ep) as usize;

    let mut best_solution: Option<Vec<u8>> = None;

    for phase1_depth in 0..=max_total {
        let mut solution = Vec::new();
        // Try phase 1 at this depth
        // When phase 1 succeeds, try phase 2 with remaining depth budget
        // If total < best, keep it
        // ...
    }

    best_solution.unwrap()
}
```

The solver tries increasing phase-1 depths. For each successful phase-1, it searches phase-2 with `max_depth = max_total - phase1_len`. The first complete solution found at minimum total depth wins. Optionally keep searching longer (WCA: min 200ms) for shorter solutions.

---

## Part 8: Random State Generation

```rust
fn random_cube(rng: &mut impl Rng) -> CubieCube {
    let mut cube = CubieCube::default();

    // Random corner permutation
    let cp_idx = rng.gen_range(0..40320u32);
    index_to_perm(cp_idx, 8, &mut cube.cp);
    let cp_parity = perm_parity(&cube.cp, 8);

    // Random edge permutation with matching parity
    loop {
        let ep_idx = rng.gen_range(0..479001600u32);  // 12!
        index_to_perm(ep_idx, 12, &mut cube.ep);
        if perm_parity(&cube.ep, 12) == cp_parity { break; }
    }

    // Random corner orientation (7 independent, 8th forced)
    let twist = rng.gen_range(0..2187u16);
    set_twist(&mut cube.co, twist);

    // Random edge orientation (11 independent, 12th forced)
    let flip = rng.gen_range(0..2048u16);
    set_flip(&mut cube.eo, flip);

    cube
}
```

**For WASM:** Use `js_sys::Math::random()` or `getrandom` crate as RNG source.

---

## Part 9: Scramble = Inverse of Solution

```rust
fn invert_solution(solution: &[u8]) -> Vec<u8> {
    solution.iter().rev().map(|&m| {
        let face = m / 3;
        let power = m % 3;
        // Inverse: 90°CW(0) ↔ 90°CCW(2), 180°(1) stays
        let inv_power = match power { 0 => 2, 2 => 0, _ => 1 };
        face * 3 + inv_power
    }).collect()
}

fn solution_to_string(moves: &[u8]) -> String {
    let face_names = ["U", "R", "F", "D", "L", "B"];
    let suffixes = ["", "2", "'"];
    moves.iter()
        .map(|&m| format!("{}{}", face_names[(m / 3) as usize], suffixes[(m % 3) as usize]))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Full scramble generation
fn generate_scramble(rng: &mut impl Rng) -> String {
    let cube = random_cube(rng);
    let solution = solve(&cube, 21);        // Kociemba, max 21 moves
    let scramble = invert_solution(&solution);
    solution_to_string(&scramble)
}
```

---

## Part 10: WCA Parameters (from TNoodle)

| Parameter | Value | Notes |
|---|---|---|
| Max scramble length | 21 moves | God's number for half-turn metric |
| Solver timeout | 60,000 ms | Abort if no solution found |
| Min solve time | 200 ms | Keep searching for shorter solutions |
| Solution flag | INVERSE_SOLUTION | Return scramble, not solve |
| Axis restrictions | Optional | First/last move axis for FMC scrambles (not needed for normal) |

---

## Part 11: Implementation Plan

### Target: `rouxflow-ai` crate (rlib, pure Rust)

#### Files
```
rouxflow-ai/src/
├── scramble/
│   ├── mod.rs           // pub fn generate_scramble() -> String
│   ├── cubie.rs         // CubieCube struct, multiply, move definitions
│   ├── coord.rs         // coordinate getters/setters, Lehmer encode/decode
│   ├── tables.rs        // move tables + pruning tables (build or embed)
│   └── solver.rs        // IDA* two-phase search
```

#### Table Strategy

**Option A: Build at init** (~200-500ms on first call)
- Compute move tables + pruning tables in `lazy_static` / `OnceLock`
- Pro: no binary blobs, smaller WASM
- Con: 200-500ms startup cost on first scramble

**Option B: Embed as binary** (build.rs generates .bin, include_bytes!)
- Pro: instant startup
- Con: ~3.8 MB added to WASM binary
- This is the right choice for the analyzer WASM module which already budgets ~5 MB for tables

**Recommended: Option A for first impl, Option B later.** Build tables lazily on first `generate_scramble()` call. The 200-500ms delay is invisible since the analyzer WASM is already lazy-loaded.

#### Table Sizes (no symmetry)

| Table | Entries | Bytes |
|---|---|---|
| twist_move | 2187 × 18 | ~79 KB |
| flip_move | 2048 × 18 | ~74 KB |
| udslice_move | 495 × 18 | ~18 KB |
| cperm_move | 40320 × 10 | ~806 KB |
| eperm_move | 40320 × 10 | ~806 KB |
| udslice2_move | 24 × 10 | ~0.5 KB |
| twist_udslice_prun | 2187 × 495 / 2 | ~541 KB |
| flip_udslice_prun | 2048 × 495 / 2 | ~507 KB |
| cperm_udslice2_prun | 40320 × 24 / 2 | ~484 KB |
| eperm_udslice2_prun | 40320 × 24 / 2 | ~484 KB |
| **Total** | | **~3.8 MB** |

#### Validation

1. Solved cube → solution is empty string
2. Single move state (e.g., apply U) → solution should contain U' (or equivalent)
3. Generate 1000 scrambles → all should be ≤21 moves, average ~18-19
4. Apply scramble to solved cube → result should NOT be solved
5. Apply scramble then solve → should return to solved

---

## References

- [Kociemba's Two-Phase Algorithm](https://kociemba.org/math/twophase.htm) — canonical algorithm description
- [Implementation Details](https://kociemba.org/math/imptwophase.htm) — move tables, pruning tables, search
- [Cube Definitions](https://kociemba.org/math/CubeDefs.htm) — corner/edge numbering, facelets
- [Symmetries](https://kociemba.org/math/symmetric.htm) — symmetry reduction (optional optimization)
- [min2phase](https://github.com/cs0x7f/min2phase) — reference Java impl (GPL-3.0, algorithm reference only)
- [tnoodle-lib](https://github.com/thewca/tnoodle-lib) — WCA parameters reference (GPL-3.0, algorithm reference only)
