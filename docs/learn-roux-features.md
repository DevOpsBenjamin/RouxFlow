# Learn Roux — Feature Design Document

> This document captures the brainstormed vision for RouxFlow's Learn feature.
> It is a living roadmap — not all sections will ship at once. See **V1 Scope** at the bottom.

---

## Architecture Overview

### Navigation

Learn is a **multi-section area** with a persistent **left sidebar** inside the Learn route.
The top navbar shows a single "Learn" button. Clicking it opens the Learn area.
The sidebar lists all sections; clicking a section shows its content in the main area on the right.

```
┌─────────────────────────────────────────────────┐
│  ROUXFLOW     Dashboard  Timer  Learn  ...      │
├──────────┬──────────────────────────────────────┤
│ Sidebar  │                                      │
│          │  Content area                        │
│ ▸ Tutorial│  (changes based on sidebar section) │
│ ▸ Sample │                                      │
│   Solves │                                      │
│ ▸ Drills │                                      │
│ ▸ Guides │                                      │
│          │                                      │
└──────────┴──────────────────────────────────────┘
```

Mobile: sidebar collapses to a hamburger/sheet.

### Cube Requirement

Learn **always requires a connected smart cube**. All content is designed around
having a real cube in hand. Users without a cube see the BluetoothRequired page.

### Data Flow

- **User solves** are stored locally (IndexedDB) and synced to Supabase.
- **Sample solves** are stored in Supabase and synced to IndexedDB on visit (offline-first).
- **Step analysis results** are cached in a separate IndexedDB table. Analysis runs lazily
  in WASM when the user visits stats/learn pages — detects unanalyzed solves and processes
  them in the foreground (not a real background worker, just async WASM while page is open).
- **Admin tool** for inputting sample solves is a **separate app** (not in this repo).
  This repo only contains the viewing/display code. Connects to the same Supabase DB.

---

## Roux Method Reference

| Step | Name | Goal | Moves (advanced) |
|------|------|------|-------------------|
| 1 | **First Block (FB)** | Build a 1x2x3 block on the left | 6–9 |
| 2 | **Second Block (SB)** | Build a 1x2x3 block on the right | 7–11 |
| 3 | **CMLL** | Orient + permute last-layer corners | 1 algorithm |
| 4a | **EO** | Orient the last 6 edges | 3–7 M/U moves |
| 4b | **ULUR** | Solve UL and UR edges | 1–5 M/U moves |
| 4c | **L4E / 4c** | Solve the last 4 M-slice edges | 0–5 M/U moves |

Total: ~45–55 moves average. Fewer rotations than CFOP. Heavy M-slice usage.

---

## Section 1 — Interactive Roux Tutorial

**V1 Status: Working**

### Concept

A guided walkthrough where the user solves a cube with Roux while the app explains each step.

**Layout:**
- **Left:** 3D cube showing live state from the connected smart cube
- **Right:** Collapsible step cards (FB, SB, CMLL, LSE)

### Step Cards Behavior

1. On page load, the **Scramble** card is expanded. It shows a scramble to apply.
2. Once the cube matches the scrambled state (or user manually confirms), **FB card** expands.
3. FB card shows:
   - Goal: "Build a 1x2x3 block on the left side"
   - Target pieces: DL edge, FL edge, DBL corner, DFL corner
   - Move counter for this step
4. When step detection (from rouxflow-ai) detects FB is solved → FB card collapses, **SB card** expands.
5. Same pattern for SB → CMLL → LSE.
6. After full solve: summary card with per-step move count and time.

### Notes on Step Detection

Step detection will be implemented in `rouxflow-ai` crate by checking actual cube state:
- **FB solved:** left 1x2x3 block (DL, FL, DBL, DFL + centers aligned)
- **SB solved:** right 1x2x3 block (DR, FR, DBR, DFR + centers aligned)
- **CMLL solved:** all U-layer corners correctly oriented and permuted
- **LSE solved:** cube is solved

For V1, step detection may be a simple heuristic. Full detection is a separate implementation task.

### EO Explanation Challenge

The LSE card needs to explain Edge Orientation which is conceptually hard for beginners.
This is acceptable to be imperfect — the tutorial is a fallback for curious users. Most users
are expected to already know Roux basics when they use the app.

---

## Section 2 — Sample Solves

**V1 Status: Working (structure + initial content)**

### Concept

A library of curated example solves from top Roux solvers, viewable with a playback system.

### Data Sources

1. **Kian Mansour solves** — manually transcribed from video by admin (move-by-move input)
2. **Speedsolving.com thread** — "Roux example solves" (80+ pages). Text format:
   scramble + solution in notation. Can be bulk-imported after validation.
3. **User-submitted solves** — future: verified community contributions

### Data Model (Supabase table: `sample_solves`)

```sql
sample_solves:
  id              UUID PRIMARY KEY
  solver_name     TEXT            -- "Kian Mansour", "Sean Patrick Villanueva"
  scramble        TEXT            -- "R U R' F2 D' L ..."
  solution        TEXT            -- "r U R2 F ..."  (full solution notation)
  step_boundaries JSON            -- { fb: 8, sb: 16, cmll: 17, lse: 24 }  (move indices)
  time_ms         INTEGER NULL    -- solve time if known (null for untimed reconstructions)
  tps             FLOAT NULL      -- turns per second if known
  source          TEXT            -- "video", "speedsolving.com", "competition"
  source_url      TEXT NULL       -- link to original
  notes           TEXT NULL       -- admin commentary per step
  verified        BOOLEAN DEFAULT FALSE  -- only verified solves shown to users
  created_at      TIMESTAMPTZ
```

### Sync Strategy

- Supabase → IndexedDB on first visit to Sample Solves page
- On subsequent visits: check for new solves (compare latest `created_at`)
- Once cached locally, works offline
- Same pattern as user solve sync (already implemented in rouxflow-storage)

### Viewing Experience

Each sample solve shows:
- Solver name, time, move count, TPS
- Step breakdown bar (colored segments: FB | SB | CMLL | LSE)
- Click to open → **Playback view**

### Admin Tool (separate app)

The admin tool (NOT in this repo) provides:
- Input: scramble + move list
- Auto-detect step boundaries via rouxflow-ai
- Playback preview to verify transcription accuracy (critical for manual video input)
- Set `verified = true` to publish
- Connects to same Supabase DB

---

## Section 3 — Drills

**V1 Status: Placeholder (Coming Soon)**

### Planned Drill Modules

#### 3a — Block Building Trainer (FB/SB)

- Targeted scrambles that set up specific FB/SB cases
- Difficulty levels (easy: 2–4 move FB, medium: 6–9, hard: worst-case)
- Metrics: move count vs optimal, time, rotation count, efficiency ratio
- 3D cube highlights the 5 target pieces
- After solve: show optimal solution from rouxflow-ai

#### 3b — CMLL Algorithm Trainer

42 CMLL cases grouped into 8 sets (O, H, Pi, U, T, S, AS, L).

| Set | Cases | Description |
|-----|:-----:|-------------|
| O (Oriented) | 6 | All corners oriented, just permute |
| H | 4 | Opposite colors on top |
| Pi | 6 | Two bars on opposite sides |
| U | 6 | One bar, headlights opposite |
| T | 6 | One bar, headlights adjacent |
| S (Sune) | 6 | Sune-type patterns |
| AS (Anti-Sune) | 6 | Mirror of Sune |
| L | 6 | L-shape oriented corners |

Training modes:
- Learn: show case → algorithm → practice on cube → verify
- Recognition drill: identify the case quickly (time-based)
- Execution drill: solve it fast (track per-case times)
- Weak case focus: prioritize slowest cases

#### 3c — LSE Trainer

- EO recognition (arrow system)
- ULUR isolated practice
- L4E/4c cases + parity
- EOLR (advanced: EO + ULUR simultaneously)
- One-look LSE training

#### 3d — Ergonomics Lab (Gyro-Based)

- M-slice speed benchmark (20 M-moves, track time + consistency)
- Regrip detection (gyro change without face turns = regrip)
- MU generator speed drill (M U M U... at increasing speed)
- Cube stability score during algorithms

---

## Playback System (Cross-Cutting Feature)

**V1 Status: Placeholder**

Used by: Sample Solves, Solve Analysis, Tutorial, Admin Tool.

### Design

The 3D renderer currently works on a single `CubeState`. For playback, we need the ability
to create and manipulate **multiple independent cube states** in WASM.

### Playback Modes

| Mode | Timeline Scale | Use Case |
|------|---------------|----------|
| **Timed** | Real timestamps (ms) | User's own solves, timed sample solves |
| **Constant** | Fixed interval per move | Untimed reconstructions, admin preview |

### Controls (Cubeast-inspired)

- **Play/Pause** at real speed (timed) or constant rate (untimed)
- **Step forward/back** one move at a time
- **Timeline scrubber** — horizontal bar, click to jump to any point
  - Timed mode: scale = time in ms (long pauses show as wide gaps)
  - Constant mode: scale = move index (even spacing)
- **Speed control:** 0.25x, 0.5x, 1x, 2x
- **Step-colored segments** on the timeline (FB=blue, SB=green, CMLL=yellow, LSE=purple)

### WASM Requirements

```rust
// New: standalone cube state for playback (no BLE connection)
pub struct PlaybackCube {
    facelets: FaceletCube,
    moves: Vec<TimedMove>,
    current_index: usize,
}

impl PlaybackCube {
    fn from_scramble(scramble: &str) -> Self;
    fn apply_move_forward(&mut self) -> bool;
    fn apply_move_backward(&mut self) -> bool;
    fn jump_to_move(&mut self, index: usize);
    fn get_facelets(&self) -> Vec<u8>;
    fn current_step(&self) -> RouxStep;  // FB, SB, CMLL, LSE
}
```

---

## Stats Page (Separate Route — /stats)

**V1 Status: Placeholder**

Not under Learn. Standalone page with sub-step analytics.

### Features

- **Sub-step split** for all analyzed solves: FB time, SB time, CMLL time, LSE time
- **Per-day average** chart (line graph over time)
- **Weak step identification:** "Your SB averages 5.1s — your biggest opportunity"
  with a link to the SB drill in Learn
- **Move efficiency per step:** your moves vs optimal
- **Rotation count trends:** are you reducing rotations over time?
- **TPS per step:** identify where you're fast and where you're slow

### Lazy Analysis

When user visits Stats or Learn, the app checks for unanalyzed solves:

```
1. Query solves from IndexedDB that have timed_moves but no step_analysis
2. For each: run step detector in WASM (rouxflow-ai)
3. Store results in a 'solve_analysis' IndexedDB table
4. Display results on the Stats page
```

This is NOT a background worker. It runs in the foreground as async WASM while the page
is open. If the user navigates away, analysis pauses and resumes on next visit.

### Data Model (IndexedDB table: `solve_analysis`)

```
solve_analysis:
  solve_id        TEXT PRIMARY KEY
  fb_moves        INTEGER
  fb_time_ms      INTEGER
  sb_moves        INTEGER
  sb_time_ms      INTEGER
  cmll_moves      INTEGER
  cmll_time_ms    INTEGER
  lse_moves       INTEGER
  lse_time_ms     INTEGER
  total_rotations INTEGER
  step_boundaries JSON    -- move indices where each step starts/ends
  analyzed_at     INTEGER -- timestamp
```

---

## V1 Scope

### Working in V1

| Feature | Description |
|---------|-------------|
| Learn layout | Left sidebar + content area, mobile responsive |
| Sidebar navigation | All sections listed, clicking navigates |
| Roux Tutorial page | 3D cube (live) + expandable step cards (FB/SB/CMLL/LSE) |
| Sample Solves page | List of curated solves, viewing individual solve details |
| Sample Solves data | Supabase table + IndexedDB sync + display |

### Placeholder in V1

| Feature | Shows |
|---------|-------|
| Drills | Coming soon card with module descriptions |
| Guides | Coming soon card with planned guide list |
| Playback | Coming soon — referenced from Sample Solves and Tutorial |
| Stats page | Coming soon — separate route /stats |

### Not in This Repo

| Feature | Notes |
|---------|-------|
| Admin tool | Separate app for inputting/verifying sample solves |
| Step detector | Implemented in rouxflow-ai crate (separate task) |
| Optimal solver | Implemented in rouxflow-ai crate (separate task) |

---

## Gamification (Future)

### Progression System

- **Belt system** (inspired by martial arts):
  - White: Can solve with Roux (any speed)
  - Yellow: Sub-2 minutes
  - Green: Sub-1 minute, knows all CMLL
  - Blue: Sub-30, efficient FB/SB
  - Red: Sub-20, one-look LSE
  - Black: Sub-15, competition-ready
- **Achievements:** "100 solves", "First sub-30", "All CMLL learned", "Zero-rotation solve"
- **Daily streak:** Solve at least once per day
- **Daily scramble:** One scramble per day, ranked globally

### Community Features (Future)

- Share solves with reconstruction
- Compare stats with friends
- Algorithm voting (best CMLL algorithms)
- Challenge friends to same-scramble races

---

## Technical Summary

### Crates Involved

| Crate | Role |
|-------|------|
| `rouxflow-core` | PlaybackCube struct, TimedMove, solve data model |
| `rouxflow-ai` | Step detector, CMLL case identifier, optimal solver |
| `rouxflow-wasm` | Expose PlaybackCube + step detection to frontend |
| `rouxflow-storage` | IndexedDB tables for solve_analysis + sample_solves cache |
| `rouxflow-render` | 3D cube rendering for playback (multiple cube states) |

### Supabase Tables

| Table | Purpose | Sync |
|-------|---------|------|
| `sample_solves` | Curated example solves | Supabase → IndexedDB |
| `solve_analysis` | Per-solve step breakdown cache | Local only (computed from solve data) |

### Frontend Routes

| Route | Component | Sidebar |
|-------|-----------|---------|
| `/learn` | LearnLayout.vue | Yes — Tutorial, Sample Solves, Drills, Guides |
| `/learn/tutorial` | RouxTutorial.vue | |
| `/learn/sample-solves` | SampleSolves.vue | |
| `/learn/drills` | DrillsPlaceholder.vue | |
| `/learn/guides` | GuidesPlaceholder.vue | |
| `/stats` | StatsView.vue | No — standalone page |
