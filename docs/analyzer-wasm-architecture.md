# Analyzer WASM Architecture

**Status:** 🔮 Future implementation (not started yet)
**Priority:** Low (after core app features)
**Estimated size:** ~3-5 MB (with lookup tables)

## Overview

Second WASM module for advanced solve analysis and optimal solving, loaded lazily in a Web Worker to avoid impacting initial load time.

## Why a Separate WASM?

### Main WASM (`rouxflow-wasm`)
- **Purpose:** Core app functionality
- **Size:** ~500 KB
- **Loading:** Immediate (blocking)
- **Features:**
  - Session management
  - BLE protocol handling
  - 3D cube rendering
  - Basic move tracking

### Analyzer WASM (`rouxflow-wasm-analyzer`)
- **Purpose:** Advanced analysis & optimal solving
- **Size:** ~3-5 MB (includes lookup tables)
- **Loading:** Lazy (only when needed)
- **Features:**
  - Optimal solve generation
  - FB/SB efficiency analysis
  - CMLL recognition time analysis
  - Alternative solution suggestions
  - Step-by-step breakdown

## Architecture

```
User Opens App
  └─> Main WASM loads (~500KB, fast)
      └─> App fully functional

User Finishes First Solve
  └─> Analyzer Worker starts
      └─> Analyzer WASM loads in background (~3-5MB)
          └─> Lookup tables loaded into memory
          └─> Analysis ready for subsequent solves
```

## Crate Structure

```
crates/
├── rouxflow-wasm/              # Main entry point (existing)
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── rouxflow-wasm-analyzer/     # Analyzer entry point (FUTURE)
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── rouxflow-ai/                # Solver logic (existing, being developed)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── solver.rs          # Roux solver implementation
│   │   ├── tables.rs          # Lookup table loading
│   │   └── analysis.rs        # Solve analysis algorithms
│   └── tables/                # Binary lookup tables
│       ├── fb_pruning.bin     # First Block pruning (~1 MB)
│       ├── sb_pruning.bin     # Second Block pruning (~1 MB)
│       ├── cmll_moves.bin     # CMLL algorithms (~500 KB)
│       └── eolr_table.bin     # EOLR cases (~500 KB)
│
└── rouxflow-core/              # Core types (existing)
    └── src/
        └── analysis.rs         # Analysis data structures
```

## Lookup Tables

### FB Pruning Table
- **Size:** ~1 MB
- **Format:** Binary (u8 array)
- **Purpose:** Optimal First Block solving
- **Generation:** Pre-computed offline

### SB Pruning Table
- **Size:** ~1 MB
- **Format:** Binary (u8 array)
- **Purpose:** Optimal Second Block solving
- **Generation:** Pre-computed offline

### CMLL Move Table
- **Size:** ~500 KB
- **Format:** Binary (encoded move sequences)
- **Purpose:** All CMLL algorithms and recognition
- **Content:** 42 cases × multiple algorithms

### EOLR Table
- **Size:** ~500 KB
- **Format:** Binary (case → moves mapping)
- **Purpose:** Edge Orientation + Last 6 edges
- **Content:** Common EOLR cases and solutions

**Total:** ~3-4 MB compiled into WASM

## API Design

### Entry Point

```rust
// crates/rouxflow-wasm-analyzer/src/lib.rs
use wasm_bindgen::prelude::*;
use rouxflow_ai::{RouxSolver, SolveAnalysis, OptimalSolution};

thread_local! {
    static SOLVER: RouxSolver = RouxSolver::new();
}

/// Initialize the solver and load lookup tables into memory.
/// This is called once when the Worker first loads.
#[wasm_bindgen]
pub fn init_solver() -> Result<(), JsValue> {
    SOLVER.with(|_| ());
    Ok(())
}

/// Find an optimal solution for the given scramble.
///
/// Returns:
/// - Solution steps (FB, SB, CMLL, LSE)
/// - Move count for each step
/// - Total move count
/// - Estimated solve time (based on TPS)
#[wasm_bindgen]
pub fn solve_optimal(scramble: String) -> JsValue {
    SOLVER.with(|solver| {
        let solution = solver.solve_optimal(&scramble);
        serde_wasm_bindgen::to_value(&solution).unwrap()
    })
}

/// Analyze a completed solve.
///
/// Parameters:
/// - scramble: The scramble used
/// - moves: All moves performed
/// - timestamps: Timestamp after each move (ms)
///
/// Returns:
/// - FB efficiency (actual moves / optimal moves)
/// - SB efficiency
/// - CMLL recognition time
/// - Average TPS per step
/// - Suggestions for improvement
#[wasm_bindgen]
pub fn analyze_solve(
    scramble: String,
    moves: Vec<String>,
    timestamps: Vec<u32>
) -> JsValue {
    SOLVER.with(|solver| {
        let analysis = solver.analyze_solve(&scramble, &moves, &timestamps);
        serde_wasm_bindgen::to_value(&analysis).unwrap()
    })
}

/// Find alternative solutions for a given step.
///
/// Example: User completed FB in 15 moves, show them a 10-move alternative.
#[wasm_bindgen]
pub fn find_alternatives(
    cube_state: String,
    step: String // "FB", "SB", "CMLL", "LSE"
) -> JsValue {
    SOLVER.with(|solver| {
        let alternatives = solver.find_alternatives(&cube_state, &step);
        serde_wasm_bindgen::to_value(&alternatives).unwrap()
    })
}

/// Get recognition hints for CMLL case.
///
/// Returns:
/// - Case name
/// - Recognition features
/// - Recommended algorithms
/// - Alternative algorithms
#[wasm_bindgen]
pub fn recognize_cmll(cube_state: String) -> JsValue {
    SOLVER.with(|solver| {
        let cmll_info = solver.recognize_cmll(&cube_state);
        serde_wasm_bindgen::to_value(&cmll_info).unwrap()
    })
}
```

## Data Structures

```rust
// crates/rouxflow-core/src/analysis.rs

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptimalSolution {
    pub scramble: String,
    pub total_moves: u32,
    pub steps: Vec<SolutionStep>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolutionStep {
    pub name: String,        // "FB", "SB", "CMLL", "LSE"
    pub moves: Vec<String>,
    pub move_count: u32,
    pub cube_state: String,  // Facelet string after this step
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolveAnalysis {
    pub total_time: u32,     // milliseconds
    pub total_moves: u32,
    pub average_tps: f32,

    // Step-by-step breakdown
    pub fb_analysis: StepAnalysis,
    pub sb_analysis: StepAnalysis,
    pub cmll_analysis: CMLLAnalysis,
    pub lse_analysis: StepAnalysis,

    // Overall metrics
    pub efficiency_score: f32,  // 0.0 to 1.0
    pub suggestions: Vec<Suggestion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StepAnalysis {
    pub time: u32,              // milliseconds
    pub moves: Vec<String>,
    pub move_count: u32,
    pub optimal_move_count: u32,
    pub efficiency: f32,        // actual / optimal
    pub tps: f32,
    pub alternative_solutions: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CMLLAnalysis {
    pub time: u32,
    pub recognition_time: u32,  // Time before first CMLL move
    pub execution_time: u32,    // Time to execute algorithm
    pub case_name: String,      // e.g., "T Sune"
    pub algorithm_used: Vec<String>,
    pub optimal_algorithm: Vec<String>,
    pub recognition_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Suggestion {
    pub step: String,           // "FB", "SB", "CMLL", "LSE"
    pub severity: String,       // "info", "warning", "critical"
    pub title: String,
    pub description: String,
    pub example_moves: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CMLLInfo {
    pub case_name: String,
    pub recognition_features: Vec<String>,
    pub recommended_algorithms: Vec<Algorithm>,
    pub probability: f32,       // Confidence in recognition
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Algorithm {
    pub moves: Vec<String>,
    pub move_count: u32,
    pub difficulty: String,     // "easy", "medium", "hard"
    pub popularity: f32,        // 0.0 to 1.0
}
```

## Frontend Integration

### Worker Setup

```typescript
// apps/frontend/src/workers/analyzer.worker.ts
import init, {
  init_solver,
  solve_optimal,
  analyze_solve,
  find_alternatives,
  recognize_cmll
} from '../wasm/analyzer/rouxflow_wasm_analyzer'

let solverReady = false

async function ensureSolverReady() {
  if (!solverReady) {
    console.log('[Analyzer] Loading WASM module (~3-5 MB)...')
    const start = performance.now()

    await init()
    init_solver()

    const elapsed = performance.now() - start
    console.log(`[Analyzer] Ready in ${elapsed.toFixed(0)}ms`)
    solverReady = true
  }
}

self.onmessage = async ({ data }) => {
  await ensureSolverReady()

  const { type, payload, requestId } = data

  try {
    let result

    switch (type) {
      case 'SOLVE_OPTIMAL':
        result = solve_optimal(payload.scramble)
        break

      case 'ANALYZE_SOLVE':
        result = analyze_solve(
          payload.scramble,
          payload.moves,
          payload.timestamps
        )
        break

      case 'FIND_ALTERNATIVES':
        result = find_alternatives(payload.cubeState, payload.step)
        break

      case 'RECOGNIZE_CMLL':
        result = recognize_cmll(payload.cubeState)
        break

      default:
        throw new Error(`Unknown request type: ${type}`)
    }

    self.postMessage({
      requestId,
      success: true,
      result
    })

  } catch (error) {
    self.postMessage({
      requestId,
      success: false,
      error: error.message
    })
  }
}
```

### Service Layer

```typescript
// apps/frontend/src/services/analyzer.ts
import type {
  OptimalSolution,
  SolveAnalysis,
  CMLLInfo
} from '@/types/analysis'

class AnalyzerService {
  private worker: Worker | null = null
  private requestId = 0
  private pendingRequests = new Map<number, (value: any) => void>()

  private getWorker(): Worker {
    if (!this.worker) {
      this.worker = new Worker(
        new URL('../workers/analyzer.worker.ts', import.meta.url),
        { type: 'module' }
      )

      this.worker.onmessage = ({ data }) => {
        const { requestId, success, result, error } = data
        const resolve = this.pendingRequests.get(requestId)

        if (resolve) {
          this.pendingRequests.delete(requestId)
          if (success) {
            resolve(result)
          } else {
            console.error('[Analyzer] Error:', error)
            resolve(null)
          }
        }
      }
    }

    return this.worker
  }

  private async request<T>(type: string, payload: any): Promise<T | null> {
    const requestId = ++this.requestId
    const worker = this.getWorker()

    return new Promise((resolve) => {
      this.pendingRequests.set(requestId, resolve)
      worker.postMessage({ type, payload, requestId })
    })
  }

  async solveOptimal(scramble: string): Promise<OptimalSolution | null> {
    return this.request('SOLVE_OPTIMAL', { scramble })
  }

  async analyzeSolve(
    scramble: string,
    moves: string[],
    timestamps: number[]
  ): Promise<SolveAnalysis | null> {
    return this.request('ANALYZE_SOLVE', { scramble, moves, timestamps })
  }

  async findAlternatives(
    cubeState: string,
    step: 'FB' | 'SB' | 'CMLL' | 'LSE'
  ): Promise<string[][] | null> {
    return this.request('FIND_ALTERNATIVES', { cubeState, step })
  }

  async recognizeCMLL(cubeState: string): Promise<CMLLInfo | null> {
    return this.request('RECOGNIZE_CMLL', { cubeState })
  }
}

export const analyzerService = new AnalyzerService()
```

### Usage in Store

```typescript
// apps/frontend/src/stores/session.ts
import { analyzerService } from '@/services/analyzer'

export const useSessionStore = defineStore('session', () => {
  const currentSolveAnalysis = ref<SolveAnalysis | null>(null)
  const isAnalyzing = ref(false)

  async function finishSolve() {
    const moves = getCurrentMoves()
    const timestamps = getTimestamps()
    const scramble = getCurrentScramble()

    // Save solve immediately (main WASM, fast)
    await sessionManager.finish_solve(moves)

    // Analyze in background (analyzer WASM, lazy-loaded)
    isAnalyzing.value = true
    try {
      const analysis = await analyzerService.analyzeSolve(
        scramble,
        moves,
        timestamps
      )

      if (analysis) {
        currentSolveAnalysis.value = analysis
      }
    } finally {
      isAnalyzing.value = false
    }
  }

  return {
    currentSolveAnalysis,
    isAnalyzing,
    finishSolve
  }
})
```

## Build Configuration

### Package.json Scripts

```json
{
  "scripts": {
    "build:wasm": "pnpm run build:wasm:main && pnpm run build:wasm:analyzer",
    "build:wasm:main": "wasm-pack build ../../crates/rouxflow-wasm --target web --out-dir ../../apps/frontend/src/wasm/rouxflow",
    "build:wasm:analyzer": "wasm-pack build ../../crates/rouxflow-wasm-analyzer --target web --out-dir ../../apps/frontend/src/wasm/analyzer",
    "dev": "pnpm run build:wasm:main && vite",
    "build": "pnpm run build:wasm && vue-tsc -b && vite build"
  }
}
```

### .gitignore

```gitignore
# WASM outputs
apps/frontend/src/wasm/rouxflow/
apps/frontend/src/wasm/analyzer/

# Lookup tables (binary files, generated separately)
crates/rouxflow-ai/tables/*.bin
```

## Performance Expectations

### Initial Load (3G Connection)
- **Main WASM:** 500 KB → ~1-2 seconds
- **App ready:** Immediately after main WASM loads
- **Analyzer WASM:** Not loaded yet

### First Analysis (Lazy Load)
- **Analyzer WASM:** 4 MB → ~3-5 seconds download
- **Table initialization:** ~200 ms
- **First analysis:** ~500 ms
- **Total:** ~4-6 seconds

### Subsequent Analyses (Cached)
- **Analyzer WASM:** Instant (cached by service worker)
- **Analysis time:** ~100-500 ms (depending on solve length)

### Memory Usage
- **Main WASM:** ~2 MB (code + runtime)
- **Analyzer WASM:** ~8 MB (code + tables + runtime)
- **Total:** ~10 MB (acceptable for modern devices)

## Implementation Phases

### Phase 1: Core Solver (Current Priority)
- [ ] Implement basic Roux solver in `rouxflow-ai`
- [ ] Generate lookup tables offline
- [ ] Test solver accuracy and performance
- [ ] Benchmark solving time

### Phase 2: Analysis Algorithms
- [ ] FB/SB efficiency calculation
- [ ] CMLL recognition detection
- [ ] TPS calculation per step
- [ ] Alternative solution generation

### Phase 3: WASM Entry Point
- [ ] Create `rouxflow-wasm-analyzer` crate
- [ ] Implement WASM bindings
- [ ] Test with sample solves
- [ ] Optimize for size and speed

### Phase 4: Frontend Integration
- [ ] Create analyzer Worker
- [ ] Implement service layer
- [ ] Add UI for analysis results
- [ ] Test lazy loading behavior

### Phase 5: Polish
- [ ] Optimize lookup table size
- [ ] Add progress indicators
- [ ] Handle offline mode gracefully
- [ ] Performance profiling

## Open Questions

- [ ] Should we generate tables at build time or ship pre-built?
- [ ] How to handle analyzer WASM updates (versioning)?
- [ ] Should analysis be automatic or opt-in?
- [ ] Cache analysis results in IndexedDB?
- [ ] Show progress for long analyses (>1 second)?

## Notes

- **Priority:** Low (many core features needed first)
- **Size is critical:** Keep under 5 MB if possible
- **Lazy loading is essential:** Don't slow down initial app load
- **Worker isolation:** Keeps UI responsive during analysis
- **Cache everything:** Service worker must cache analyzer WASM

## See Also

- [Recognition Time Analysis](./recognition-time-analysis.md)
- [Roux Method Resources](./roux-resources.md)
- Main architecture: [CLAUDE.md](../CLAUDE.md)
