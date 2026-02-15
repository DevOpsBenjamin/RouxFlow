use rouxflow_bitboard::BitCube;
use rouxflow_core::telemetry::{GyroSample, SolveTelemetry};

pub mod math;
pub mod moves;
pub mod visualizer;

// Re-export everything for tests and convenience
pub use math::*;
pub use moves::*;
pub use visualizer::*;

#[cfg(test)]
mod tests;

// ========== Main analysis entry point ==========

/// Analyze a solve from raw telemetry data, printing debug output.
///
/// Multi-pass approach:
/// - Pass 1: Slice detection (body frame, no orientation)
/// - Pass 2: Gyro orientation table (all samples, majority vote)
pub fn analyze_solve(telemetry: &SolveTelemetry, idx_print: usize) {
    let t_start = std::time::Instant::now();
    let duration = telemetry.solve_end_t - telemetry.solve_start_t;

    println!("=== SOLVE ANALYSIS (multi-pass) ===");
    println!(
        "Scramble: {}",
        if telemetry.scramble.is_empty() {
            "(not recorded)"
        } else {
            &telemetry.scramble
        }
    );
    println!(
        "Duration: {:.2}s (solve_start={:.3}, solve_end={:.3})",
        duration, telemetry.solve_start_t, telemetry.solve_end_t
    );
    println!(
        "Scramble gyro: {} samples, Solve gyro: {} samples, Raw moves: {}",
        telemetry.scramble_gyro.len(),
        telemetry.solve_gyro.len(),
        telemetry.solve_moves.len()
    );
    println!();

    // Compute home orientation from scramble gyro
    let home = compute_home(&telemetry.scramble_gyro);
    let home_rel = relative_quaternion(&home, &home);
    let (home_top, home_front) = estimate_orientation(&home_rel);
    println!(
        "[home] q=({:.4}, {:.4}, {:.4}, {:.4}) -> {}",
        home[0],
        home[1],
        home[2],
        home[3],
        orientation_label(home_top, home_front)
    );
    println!();

    // Combine scramble + solve gyro for lookups
    let mut all_gyro: Vec<&GyroSample> =
        Vec::with_capacity(telemetry.scramble_gyro.len() + telemetry.solve_gyro.len());
    for s in &telemetry.scramble_gyro {
        all_gyro.push(s);
    }
    for s in &telemetry.solve_gyro {
        all_gyro.push(s);
    }
    all_gyro.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    let combined_gyro: Vec<GyroSample> = all_gyro.iter().map(|s| (*s).clone()).collect();

    // BLE sanity check
    {
        let mut cube_ble = BitCube::new_solved();
        for token in telemetry.scramble.split_whitespace() {
            cube_ble.apply_move(token);
        }
        for m in &telemetry.solve_moves {
            cube_ble.apply_move(&m.n);
        }
        println!(
            "BLE sanity check (scramble + all raw moves): solved = {}",
            cube_ble.is_solved()
        );
        println!();
    }

    // ========== PASS 1: Slice detection ==========
    // Merge simultaneous opposite-face move pairs into slice notation.
    // Body frame only — no orientation or remap.

    struct P1Move {
        body_label: String,
        body_raw: Vec<String>,
        t: f64,
    }

    let raw = &telemetry.solve_moves;
    let mut p1: Vec<P1Move> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if i + 1 < raw.len() && is_slice_pair(&raw[i], &raw[i + 1]) {
            let (f1, d1) = parse_face_dir(&raw[i].n).unwrap();
            let (f2, d2) = parse_face_dir(&raw[i + 1].n).unwrap();
            let body_slice = slice_name(f1, d1, f2, d2);
            p1.push(P1Move {
                body_label: format!("{} ({}+{})", body_slice, raw[i].n, raw[i + 1].n),
                body_raw: vec![raw[i].n.clone(), raw[i + 1].n.clone()],
                t: raw[i].t,
            });
            i += 2;
        } else {
            p1.push(P1Move {
                body_label: raw[i].n.clone(),
                body_raw: vec![raw[i].n.clone()],
                t: raw[i].t,
            });
            i += 1;
        }
    }

    println!(
        "=== PASS 1: Slice detection ({} raw -> {} moves) ===",
        raw.len(),
        p1.len()
    );
    println!();

    // ========== PASS 2: Gyro orientation table ==========
    // For each move, collect ALL gyro samples in the window before and after.
    // Majority vote determines the most reliable orientation.

    // Compute N+1 windows between consecutive moves.
    let mut boundaries: Vec<f64> = Vec::with_capacity(p1.len() + 2);
    boundaries.push(telemetry.solve_start_t);
    for m in &p1 {
        boundaries.push(m.t);
    }
    boundaries.push(telemetry.solve_end_t);

    let window_runs: Vec<Vec<GyroRun>> = (0..boundaries.len() - 1)
        .map(|w| collect_orient_runs(&combined_gyro, &home, boundaries[w], boundaries[w + 1]))
        .collect();

    // Print interleaved MOVE / GYRO timeline
    const YELLOW: &str = "\x1b[33m";
    const CYAN: &str = "\x1b[36m";
    const DIM: &str = "\x1b[2m";
    const RESET: &str = "\x1b[0m";

    println!("=== PASS 2: Gyro / Move Timeline ===");
    println!();

    let solve_start = telemetry.solve_start_t;

    let print_gyro_runs =
        |runs: &[GyroRun], solve_start: f64, prev_ctx: Option<&str>, next_ctx: Option<&str>| {
            if runs.is_empty() {
                return;
            }
            let mut last_stable: Option<&str> = None;
            for (i, run) in runs.iter().enumerate() {
                let noise = is_noise(runs, i, 1, prev_ctx, next_ctx);
                if noise {
                    println!(
                        "       GYRO | {}({} x{}) << noise{}",
                        DIM, run.label, run.count, RESET
                    );
                } else {
                    // Detect rotation: non-noise run differs from previous non-noise run
                    if let Some(prev) = last_stable {
                        if prev != run.label {
                            if let (Some(from), Some(to)) =
                                (parse_orient_label(prev), parse_orient_label(&run.label))
                            {
                                let rot = detect_rotation(from, to);
                                let rel_t = run.t_start - solve_start;
                                println!(
                                    "       {}~~~~ {} ({} -> {}) at {:+.2}s ~~~~{}",
                                    CYAN, rot, prev, run.label, rel_t, RESET
                                );
                            }
                        }
                    }
                    println!("       GYRO | {} (x{})", run.label, run.count);
                    last_stable = Some(&run.label);
                }
            }
        };

    // Compute context for noise detection at window w, respecting slice boundaries.
    // window_runs[w] is between move w (p1[w-1]) and move w+1 (p1[w]).
    // Don't cross a slice boundary — the gyro shifts at slices.
    let window_ctx = |w: usize| -> (Option<String>, Option<String>) {
        // prev context: window_runs[w-1]. Boundary move = p1[w-1] (the move that starts window w).
        let prev = if w > 0 {
            let boundary_is_slice = w >= 1 && w - 1 < p1.len() && p1[w - 1].body_raw.len() == 2;
            if boundary_is_slice {
                None
            } else {
                window_runs[w - 1].last().map(|r| r.label.clone())
            }
        } else {
            None
        };
        // next context: window_runs[w+1]. Boundary move = p1[w] (the move that ends window w).
        let next = if w + 1 < window_runs.len() {
            let boundary_is_slice = w < p1.len() && p1[w].body_raw.len() == 2;
            if boundary_is_slice {
                None
            } else {
                window_runs[w + 1].first().map(|r| r.label.clone())
            }
        } else {
            None
        };
        (prev, next)
    };

    const GREEN: &str = "\x1b[32;1m";

    // BitCube for visual verification (body frame, raw moves as BLE reports)
    let mut cube_body = BitCube::new_solved();
    for token in telemetry.scramble.split_whitespace() {
        cube_body.apply_move(token);
    }

    // BitCube for block detection — uses slice notation (S/M/E) for slice pairs
    let mut cube_detect = BitCube::new_solved();
    for token in telemetry.scramble.split_whitespace() {
        cube_detect.apply_move(token);
    }

    // Window before first move
    let (pc0, nc0) = window_ctx(0);
    print_gyro_runs(&window_runs[0], solve_start, pc0.as_deref(), nc0.as_deref());

    for (idx, m) in p1.iter().enumerate() {
        // Apply raw moves to display cube
        for raw in &m.body_raw {
            cube_body.apply_move(raw);
        }

        // Apply to detection cube: use slice notation for slice pairs
        if m.body_raw.len() == 2 {
            // Extract slice name from body_label (e.g. "S (F'+B)" → "S")
            let slice_move = m
                .body_label
                .split_whitespace()
                .next()
                .unwrap_or(&m.body_label);
            cube_detect.apply_move(slice_move);
        } else {
            cube_detect.apply_move(&m.body_raw[0]);
        }

        let rel_t = m.t - solve_start;
        let is_slice = m.body_raw.len() == 2;
        let _move_marker = if is_slice {
            format!("{}S{}", YELLOW, RESET)
        } else {
            " ".to_string()
        };

        // Check Roux steps on detection cube
        let green_v = format!("{}V{}", GREEN, RESET);
        let red_x = format!("{}X{}", "\x1b[31m", RESET);

        let mut f_stat = red_x.clone();
        let mut s_stat = red_x.clone();
        let mut c_stat = red_x.clone();
        let mut u_stat = red_x.clone();
        let mut e_stat = "?".to_string();

        let opp = |c: usize| -> usize {
            match c {
                0 => 3,
                3 => 0,
                1 => 4,
                4 => 1,
                2 => 5,
                5 => 2,
                _ => c,
            }
        };

        // Search for an orientation where FB (L-block) is formed
        let mut aligned_cube = None;
        let mut temp = cube_body.clone();

        // 1. Ring y scan
        'search: for _ in 0..4 {
            for _ in 0..4 {
                if temp.is_l_block_formed() {
                    aligned_cube = Some(temp.clone());
                    break 'search;
                }
                temp.rot_x();
            }
            temp.rot_y();
        }

        if aligned_cube.is_none() {
            // 2. U to L
            temp = cube_body.clone();
            temp.rot_z_prime();
            'search2: for _ in 0..4 {
                if temp.is_l_block_formed() {
                    aligned_cube = Some(temp.clone());
                    break 'search2;
                }
                temp.rot_x();
            }
        }

        if aligned_cube.is_none() {
            // 3. D to L
            temp = cube_body.clone();
            temp.rot_z();
            'search3: for _ in 0..4 {
                if temp.is_l_block_formed() {
                    aligned_cube = Some(temp.clone());
                    break 'search3;
                }
                temp.rot_x();
            }
        }

        if let Some(aligned) = aligned_cube {
            f_stat = green_v.clone();

            // Extract colors from the aligned block
            let find_color = |mask: u64| -> Option<usize> {
                aligned.boards.iter().position(|&b| (b & mask) == mask)
            };

            if let (Some(l), Some(d), Some(f), Some(b)) = (
                find_color(BitCube::L_BLOCK),
                find_color(BitCube::D_BAR_L),
                find_color(BitCube::F_BAR_L),
                find_color(BitCube::B_BAR_L),
            ) {
                // Check SB (opposite to FB)
                let r = opp(l);

                // SB Check: Use BitCube constants but ignore centers and U-layer stickers (lines 20, 45, 47)
                // This ensures SB is detected even if CMLL is not solved.

                // R strict (no center, no top row - R_BLOCK already excludes top row)
                let sb_r_ok = (aligned.boards[r] & BitCube::R_BLOCK) == BitCube::R_BLOCK;

                // D strict (Right bar of D - D_BAR_R is bottom layer only)
                let sb_d_ok = (aligned.boards[d] & BitCube::D_BAR_R) == BitCube::D_BAR_R;

                // F strict (Right bar of F - F_BAR_R includes bit 20 (U-layer), mask it out)
                let mask_f = BitCube::F_BAR_R & !(1 << 20);
                let sb_f_ok = (aligned.boards[f] & mask_f) == mask_f;

                // B strict (Left bar of B (from B persp)? No, B_BAR_R is Left Bar of B touching R)
                // B_BAR_R matches col 45,48,51. 45 is Top Row. Mask it out.
                let mask_b = BitCube::B_BAR_R & !(1 << 45);
                let sb_b_ok = (aligned.boards[b] & mask_b) == mask_b;

                if sb_r_ok && sb_d_ok && sb_f_ok && sb_b_ok {
                    s_stat = green_v.clone();
                } else {
                    /*
                    println!("DEBUG SB FAIL: r{} d{} f{} b{} | L_col={} R_col={}",
                        sb_r_ok, sb_d_ok, sb_f_ok, sb_b_ok, l, r);
                    */
                }

                // CMLL (Corners on U, permutation)
                if aligned.is_cmll_solved() {
                    c_stat = green_v.clone();
                }

                // UL/UR
                if aligned.is_ul_ur_placed() {
                    u_stat = green_v.clone();
                }

                e_stat = format!("{}", aligned.count_bad_edges());
            }
        }

        println!(
            "{:>4}  MOVE | {:<20} {:+7.2}s  FB: {} SB: {} CMLL: {} UL/UR: {} EdgeCount: {}",
            idx + 1,
            m.body_label,
            rel_t,
            f_stat,
            s_stat,
            c_stat,
            u_stat,
            e_stat
        );

        // Print cube state if idx_print is set and we're past it
        if idx_print > 0 && idx + 1 >= idx_print {
            let label = format!("#{} {}", idx + 1, m.body_label);
            print_cubes_side_by_side(&[(&cube_body, &label)]);
        }

        let w = idx + 1;
        let (pc, nc) = window_ctx(w);
        print_gyro_runs(&window_runs[w], solve_start, pc.as_deref(), nc.as_deref());
    }

    if idx_print > 0 {
        println!("Body cube solved: {}", cube_body.is_solved());
        println!();
    }

    println!();

    // ========== PASS 3: Rotation detection ==========
    // Walk through effective orientations (last stable run per window).
    // A rotation requires 2 consecutive windows to agree on the new orientation.
    // Also detects round-trip rotations (inspection: rotate → peek → rotate back).

    const MIN_ROTATION_SAMPLES: usize = 3;

    struct DetectedRotation {
        before_move: usize, // 1-indexed
        rotation: String,
        from: String,
        to: String,
    }

    let mut current_orient = "?/?".to_string();
    let mut detected_rotations: Vec<DetectedRotation> = Vec::new();
    let mut move_orients: Vec<String> = Vec::with_capacity(p1.len());

    const SLICE_LOOKBACK: usize = 2; // skip rotation detection if a slice is within this many moves

    // Reuse window_ctx for Pass 3 context (same slice-boundary awareness)

    for (idx, _m) in p1.iter().enumerate() {
        let runs = &window_runs[idx];
        let total: usize = runs.iter().map(|r| r.count).sum();
        let (pc, nc) = window_ctx(idx);
        let effective = window_effective_orient(runs, pc.as_deref(), nc.as_deref());

        // Check if any of the PREVIOUS SLICE_LOOKBACK moves is a slice.
        // Note: d starts at 1 — the current move's BEFORE window is pre-slice, still clean.
        let near_slice = (1..=SLICE_LOOKBACK).any(|d| {
            if d > idx {
                return false;
            }
            p1[idx - d].body_raw.len() == 2
        });

        if total < MIN_ROTATION_SAMPLES || near_slice {
            // Not enough samples or near a slice — carry forward
            if near_slice && total > 0 && current_orient != "?/?" {
                // Silently update baseline after slice (gyro shifted, not a user rotation).
                current_orient = effective.clone();
            }
            move_orients.push(current_orient.clone());
            continue;
        }

        if current_orient == "?/?" {
            current_orient = effective.clone();
        } else if effective != current_orient {
            // Potential rotation — require NEXT reliable window to confirm.
            let confirmed = {
                let mut found = false;
                for fwd in (idx + 1)..p1.len() {
                    let fwd_runs = &window_runs[fwd];
                    let fwd_total: usize = fwd_runs.iter().map(|r| r.count).sum();
                    let fwd_near_slice = (1..=SLICE_LOOKBACK).any(|d| {
                        if d > fwd {
                            return false;
                        }
                        p1[fwd - d].body_raw.len() == 2
                    });
                    if fwd_total < MIN_ROTATION_SAMPLES || fwd_near_slice {
                        continue;
                    }
                    let (fpc, fnc) = window_ctx(fwd);
                    found = window_effective_orient(fwd_runs, fpc.as_deref(), fnc.as_deref())
                        == effective;
                    break;
                }
                found
            };

            if confirmed {
                if let (Some(from), Some(to)) = (
                    parse_orient_label(&current_orient),
                    parse_orient_label(&effective),
                ) {
                    let rot = detect_rotation(from, to);
                    detected_rotations.push(DetectedRotation {
                        before_move: idx + 1,
                        rotation: rot,
                        from: current_orient.clone(),
                        to: effective.clone(),
                    });
                }
                current_orient = effective.clone();
            }
        }

        move_orients.push(current_orient.clone());
    }

    println!("=== PASS 3: Rotation Detection ===");
    if detected_rotations.is_empty() {
        println!("  No rotations detected.");
    } else {
        for r in &detected_rotations {
            println!(
                "  Before move {:>3}: {:>4}  ({} -> {})",
                r.before_move, r.rotation, r.from, r.to
            );
        }
    }
    println!();

    // Orientation history from persistent rotations
    let mut orient_history: Vec<&str> = Vec::new();
    if let Some(first) = detected_rotations.first() {
        orient_history.push(&first.from);
    }
    for r in &detected_rotations {
        orient_history.push(&r.to);
    }

    println!(
        "Orientation history: {}",
        if orient_history.is_empty() {
            current_orient.clone()
        } else {
            orient_history.join(" -> ")
        }
    );
    println!("Final: {}", current_orient);
    println!();

    // Detect inspections: within-window round-trips where non-noise runs
    // start and end at the same orientation with different ones in between.
    println!("Inspections (in-window round-trips):");
    let mut inspections_found = false;
    for (w, runs) in window_runs.iter().enumerate() {
        let (pc, nc) = window_ctx(w);
        // Collect non-noise run labels
        let stable: Vec<&str> = runs
            .iter()
            .enumerate()
            .filter(|(i, _)| !is_noise(runs, *i, 1, pc.as_deref(), nc.as_deref()))
            .map(|(_, r)| r.label.as_str())
            .collect();
        if stable.len() < 3 {
            continue; // need at least: start, different, back
        }
        let first = stable[0];
        let last = stable[stable.len() - 1];
        if first != last {
            continue; // not a round-trip
        }
        // Check there's at least one different orientation in between
        let has_different = stable[1..stable.len() - 1].iter().any(|s| *s != first);
        if !has_different {
            continue;
        }
        // Collect the visited orientations (deduplicated sequence)
        let mut visited: Vec<&str> = vec![first];
        for s in &stable[1..] {
            if *s != *visited.last().unwrap() {
                visited.push(s);
            }
        }

        // Which move is this between?
        // window w is between move w (boundary start) and move w+1 (boundary end)
        let after_move = if w > 0 { w } else { 0 }; // 1-indexed
        let before_move = if w < p1.len() { w + 1 } else { w }; // 1-indexed
        let duration_ms = (boundaries[w + 1] - boundaries[w]) * 1000.0;

        inspections_found = true;
        println!(
            "  Between moves {:>3}-{:>3} ({:.0}ms): {}",
            after_move,
            before_move,
            duration_ms,
            visited.join(" -> "),
        );
    }
    if !inspections_found {
        println!("  None detected.");
    }

    println!();
    println!(
        "=== END ANALYSIS ({:.2}ms) ===",
        t_start.elapsed().as_secs_f64() * 1000.0
    );
}
