# Recognition Time vs Turn Time Analysis

## Concept

During a Roux solve, the time between moves can be split into two components:

- **Turn Time**: The physical execution of a move (muscle memory, finger tricks)
- **Recognition Time**: The pause before a move where the solver is looking at the cube, planning the next step

A long gap between moves indicates the solver **stopped to think** — poor lookahead. Good solvers maintain continuous turning because they plan ahead while executing.

## Detection Method

Given timestamps from BLE cube protocols (each move has a `cube_timestamp`), compute the **inter-move delay**:

```
delay = timestamp[i] - timestamp[i-1]
```

If `delay > threshold`, classify it as **recognition time** (pause/inspection).
If `delay <= threshold`, classify it as **turn time** (continuous execution).

## Adaptive Thresholds by Solver Level

A Sub-15 solver turns much faster than a beginner. The threshold must adapt:

| Solver Level | Threshold | Rationale |
|-------------|-----------|-----------|
| Sub-15      | 250ms     | Elite solvers average ~8 TPS, gaps >250ms are clearly pauses |
| Sub-25      | 400ms     | Intermediate solvers, ~5-6 TPS average |
| Sub-45      | 600ms     | Casual solvers, still learning finger tricks |
| Sub-60+     | 800ms     | Beginners, slower turning speed overall |

### Auto-detection

The solver level can be inferred from:
- User's average solve time (stored in session history)
- Current solve's rolling TPS (turns per second)
- Manual setting in user preferences

## Per-Phase Breakdown (Roux Method)

Recognition time matters differently in each Roux phase:

### FB (First Block)
- **Expected**: High recognition time — FB requires the most planning
- **Good sign**: Long initial inspection, then continuous block-building
- **Bad sign**: Frequent pauses mid-block (lost track of pieces)

### SB (Second Block)
- **Expected**: Moderate recognition — similar to FB but with restricted moves (preserve FB)
- **Good sign**: Smooth pair insertions with minimal pauses
- **Bad sign**: Long pauses between pairs (can't find pieces while preserving FB)

### CMLL (Corners of Last Layer)
- **Expected**: Very low recognition time — this is an algorithm set
- **Good sign**: One short recognition pause, then full algorithm execution
- **Bad sign**: Multiple pauses mid-algorithm (not fully memorized), or long initial recognition (can't identify the case)

### LSE (Last Six Edges)
- **Expected**: Low recognition time for experienced solvers
- **Good sign**: Quick EO recognition → continuous M/U moves
- **Bad sign**: Pauses during M-slice moves (poor edge tracking)

## Example Solve Report

```
Solve #142 — 18.34s (Sub-25 threshold: 400ms)

Phase     | Time   | Moves | TPS  | Recognition | Turn Time
----------|--------|-------|------|-------------|----------
FB        | 5.2s   | 9     | 1.7  | 2.8s (54%) | 2.4s (46%)
SB        | 4.8s   | 11    | 2.3  | 1.9s (40%) | 2.9s (60%)
CMLL      | 2.1s   | 9     | 4.3  | 0.6s (29%) | 1.5s (71%)
LSE       | 6.2s   | 15    | 2.4  | 3.1s (50%) | 3.1s (50%)

Biggest pauses:
  1. FB move 4→5: 1.2s (looking for DL edge?)
  2. LSE move 2→3: 0.9s (EO recognition)
  3. SB move 6→7: 0.8s (pair search)

Recommendation: FB lookahead needs work — 54% recognition time.
                LSE EO recognition is slow — practice arrow cases.
```

## Timestamp Sources by Protocol

| Protocol   | Timestamp Field                        | Resolution |
|-----------|----------------------------------------|------------|
| GAN v2    | `cube_timestamp` from move packet      | ~1ms       |
| GAN v3    | `cube_timestamp` from move packet      | ~1ms       |
| GAN v4    | `cube_timestamp` from move packet      | ~1ms       |
| MoYu V3   | 16-bit timestamps in move packet (0xA5)| ~1ms       |
| Giiker    | None — use host-side `performance.now()`| ~1ms (host)|
| GoCube    | None — use host-side timestamps        | ~1ms (host)|
| QiYi      | None — use host-side timestamps        | ~1ms (host)|

For protocols without cube-side timestamps, the host-side timestamp is less accurate due to BLE latency (~10-30ms jitter), but still usable for recognition time analysis since we're looking at gaps >250ms.

## Implementation Notes

### Where to implement
- `rouxflow-core/src/session.rs` — SessionManager already tracks moves
- Add `recognition_time` and `turn_time` accumulators per phase
- Threshold config in user preferences (or auto-detect from solve history)

### Data needed
- Move timestamps (from CubeEvent::Move)
- Phase boundaries (from RouxSolver phase detection)
- Solver level (from solve history or user setting)

### Future enhancements
- Rolling average of recognition time across solves (trend tracking)
- Heat map visualization: which move transitions cause the most pauses
- Comparison mode: overlay recognition patterns between PB and average solves
- AI suggestions: "You consistently pause after inserting the first SB pair"
