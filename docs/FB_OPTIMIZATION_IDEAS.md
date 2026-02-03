# Roux First Block (FB) Optimization Ideas

This document explores conceptual strategies for analyzing and proposing better First Block solutions.

## 1. The "Ghost Cube" Concept

Instead of just showing the final solved block, we want to visualize the *logic* of the blocks available after the scramble.

- **Selective Transparency**: Render the whole cube in semi-transparent grey, except for the specific 8 pieces (1 center, 3 edges, 4 corners) that form a potential FB.
- **Color Overlay**: Highlight potential "pairs" or "lines" already at the scramble stage.
- **Multiple Ghosts**: Allow the user to toggle between several "good" FB options (e.g., White-Blue, White-Green, Yellow-Blue, etc.).

## 2. FB Solution Search Algorithm

### 2.2 Brute-Force / IDA* Search
- **State Space**: Limit the search to the 8 pieces of the FB.
- **Moves**: Only use {L, U, B, D, F, r, M}.
- **Goal**: Find solutions under 8 moves.

### 2.3 Heuristic: "Roux Purity"
- An optimal solution (shortest move count) isn't always the best human solution.
- **Weighting**:
    - Favor moves that keep the right side (R layer) free for SB.
    - Penalize rotations (x, y, z).
    - Favor "fingertrick-friendly" sequences (e.g., `L U L'`, `U' B'`).

## 3. Comparative Analysis

When a user finishes a solve, we compare their FB with our "Better Solutions":

| Metric | User | AI Option A | AI Option B |
|--------|------|-------------|-------------|
| Moves | 14 | 8 | 9 |
| Rotations | 2 | 0 | 1 |
| Efficiency | Low | High | Medium |

## 4. Implementation Steps (Theory)

1. **Scramble Replay**: The core logic needs to be able to "time travel" back to the state right after the scramble.
2. **Pathfinding**: Run a bounded search to find the top 3 most efficient FBs for the given scramble.
3. **Step-by-Step UI**: In the walkthrough, show the AI's solution alongside the user's, highlighting specifically where the AI "saved" moves.
4. **Interactive "Ghost"**: In the 3D view, let users see the AI's proposed FB pieces glowing in the scrambled cube.

---
*Draft Version: 0.1*
