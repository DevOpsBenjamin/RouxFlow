use rouxflow_bitboard::move_indices::Rotation;
use rouxflow_bitboard::BitCube;
use rouxflow_core::telemetry::{GyroSample, ParsedSolve, SolveEvent, SolveTelemetry};

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

/// Analyze a solve from raw telemetry data, returning a ParsedSolve result.
///
/// Multi-pass approach:
/// - Pass 1: Slice detection (body frame, no orientation)
/// - Pass 2: Gyro orientation table (all samples, majority vote)
pub fn analyze_solve(telemetry: &SolveTelemetry, print_output: bool) -> ParsedSolve {
    let duration = telemetry.solve_end_t - telemetry.solve_start_t;

    if print_output {
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
    }

    // Compute home orientation from scramble gyro
    let home = compute_home(&telemetry.scramble_gyro);
    let home_rel = relative_quaternion(&home, &home);
    let home_orient = estimate_orientation(&home_rel);

    let mut parsed_solve = ParsedSolve {
        solve_duration_ms: duration * 1000.0,
        is_solved: false,
        move_count: Default::default(),
        tps: Default::default(),
        step_details: rouxflow_core::telemetry::StepDetails::default(),
        initial_orientation: home_orient,
        timeline: Vec::new(),
    };

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
        if print_output {
            println!(
                "BLE sanity check (scramble + all raw moves): solved = {}",
                cube_ble.is_solved()
            );
            println!();
        }
    }

    // ========== PASS 1: Move consolidation ==========
    // Stage 1: Slice detection (opposite-face simultaneous moves)
    struct P1Move {
        body_label: String,
        body_raw: Vec<String>,
        t: f64,
    }

    let raw = &telemetry.solve_moves;
    let mut p0: Vec<P1Move> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if i + 1 < raw.len() && is_slice_pair(&raw[i], &raw[i + 1]) {
            let (f1, d1) = parse_face_dir(&raw[i].n).unwrap();
            let (f2, d2) = parse_face_dir(&raw[i + 1].n).unwrap();
            let body_slice = slice_name(f1, d1, f2, d2);
            p0.push(P1Move {
                body_label: format!("{} ({}+{})", body_slice, raw[i].n, raw[i + 1].n),
                body_raw: vec![raw[i].n.clone(), raw[i + 1].n.clone()],
                t: raw[i].t,
            });
            i += 2;
        } else {
            p0.push(P1Move {
                body_label: raw[i].n.clone(),
                body_raw: vec![raw[i].n.clone()],
                t: raw[i].t,
            });
            i += 1;
        }
    }

    // Stage 2: Generalized double-turn merging (consecutive identical turns within 400ms)
    let mut p1: Vec<P1Move> = Vec::new();
    let mut j = 0;
    while j < p0.len() {
        if j + 1 < p0.len()
            && can_merge_labels(
                &p0[j].body_label,
                &p0[j + 1].body_label,
                p0[j + 1].t - p0[j].t,
            )
        {
            let merged_label = merge_labels(&p0[j].body_label, &p0[j + 1].body_label);
            let mut body_raw = p0[j].body_raw.clone();
            body_raw.extend(p0[j + 1].body_raw.clone());

            p1.push(P1Move {
                body_label: merged_label,
                body_raw,
                t: p0[j].t,
            });
            j += 2;
        } else {
            p1.push(P1Move {
                body_label: p0[j].body_label.clone(),
                body_raw: p0[j].body_raw.clone(),
                t: p0[j].t,
            });
            j += 1;
        }
    }

    if print_output {
        println!(
            "=== PASS 1: Slice detection ({} raw -> {} moves) ===",
            raw.len(),
            p1.len()
        );
        println!();
    }

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

    if print_output {
        println!("=== PASS 2: Gyro / Move Timeline ===");
        println!();
    }

    let solve_start = telemetry.solve_start_t;

    let print_gyro_runs =
        |runs: &[GyroRun], solve_start: f64, prev_ctx: Option<&str>, next_ctx: Option<&str>| {
            if !print_output {
                return;
            }
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
                0 => 1, // W <-> Y
                1 => 0,
                2 => 3, // G <-> B
                3 => 2,
                4 => 5, // R <-> O
                5 => 4,
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

        if let Some(ref aligned) = aligned_cube {
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

                // SB Check: Use BitCube constants.
                // R strict (no center, no top row - R_BLOCK already excludes them)
                let sb_r_ok = (aligned.boards[r] & BitCube::R_BLOCK) == BitCube::R_BLOCK;

                // D strict (Right bar of D)
                let sb_d_ok = (aligned.boards[d] & BitCube::D_BAR_R) == BitCube::D_BAR_R;

                // F strict (Right bar of F)
                // Note: F_BAR_R in BitCube (23|26) already excludes top row (20).
                let sb_f_ok = (aligned.boards[f] & BitCube::F_BAR_R) == BitCube::F_BAR_R;

                // B strict (Left bar of B)
                // Note: B_BAR_R in BitCube (48|51) already excludes top row (45).
                let sb_b_ok = (aligned.boards[b] & BitCube::B_BAR_R) == BitCube::B_BAR_R;

                if sb_r_ok && sb_d_ok && sb_f_ok && sb_b_ok {
                    s_stat = green_v.clone();
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

        // Map the string moves into `Move` enum to save in ParsedSolve
        let parse_move_str = |s: &str| -> Option<rouxflow_bitboard::move_indices::Move> {
            for m in &rouxflow_bitboard::move_indices::Move::ALL {
                if m.as_str() == s {
                    return Some(*m);
                }
            }
            None
        };

        let body_move_str = canonical_label(&m.body_label);

        let body_move =
            parse_move_str(body_move_str).unwrap_or(rouxflow_bitboard::move_indices::Move::Face(
                rouxflow_bitboard::move_indices::FaceMove::U,
            )); // fallback

        // Note: relative move remapping based on orientation is tricky here due to Pass 3
        // happening AFTER Pass 1 & 2. We will just compute it if we can, or fallback to body_move
        // For a full implementation, we need the `current_orient` from Pass 3 here.
        // We will just do a placeholder for now since the original code didn't do it inline.
        let relative_move = body_move;

        // Step logic Tracking
        let is_fb = f_stat == green_v;
        let is_sb = s_stat == green_v;
        let is_cmll = c_stat == green_v;
        let is_ur_lr = u_stat == green_v;
        let bad_edges_count = aligned_cube
            .as_ref()
            .map(|c| c.count_bad_edges())
            .unwrap_or(0);

        let move_idx = (idx + 1) as isize;
        if is_fb && parsed_solve.step_details.fb == -1 {
            parsed_solve.step_details.fb = move_idx;
        }
        if is_sb && parsed_solve.step_details.sb == -1 {
            parsed_solve.step_details.sb = move_idx;
        }
        if is_cmll && parsed_solve.step_details.cmll == -1 {
            parsed_solve.step_details.cmll = move_idx;
        }
        // Count bad_edges == 0 as EO being solved, ONLY if FB+SB are solved
        if is_fb && is_sb && bad_edges_count == 0 && parsed_solve.step_details.eo == -1 {
            parsed_solve.step_details.eo = move_idx;
        }
        if is_ur_lr && parsed_solve.step_details.ur_lr == -1 {
            parsed_solve.step_details.ur_lr = move_idx;
        }

        parsed_solve.timeline.push(SolveEvent::Move {
            t: m.t,
            original: m.body_raw.clone(),
            body_move,
            relative_move,
        });

        // Print cube state if logging is enabled
        if print_output {
            let label = format!("#{} {}", idx + 1, m.body_label);
            print_cubes_side_by_side(&[(&cube_body, &label)]);
        }

        let w = idx + 1;
        let (pc, nc) = window_ctx(w);
        print_gyro_runs(&window_runs[w], solve_start, pc.as_deref(), nc.as_deref());
    }

    if print_output {
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

    let mut current_orient = orientation_label(home_orient);
    let mut detected_rotations: Vec<DetectedRotation> = Vec::new();
    let mut move_orients: Vec<String> = Vec::with_capacity(p1.len());

    // Emit an initial Rotation at t = first_move - 0.001 if the solver is already
    // holding a different orientation than home when the solve begins.
    // Scan window_runs starting from w=1 (after first move) to find the first
    // reliable gyro window — that gives the true starting orientation.
    {
        let mut initial_orient: Option<String> = None;
        'find_init: for w in 1..window_runs.len().min(6) {
            let runs = &window_runs[w];
            let total: usize = runs.iter().map(|r| r.count).sum();
            if total < 1 {
                continue;
            }
            let (pc, nc) = window_ctx(w);
            let eff = window_effective_orient(runs, pc.as_deref(), nc.as_deref());
            if eff != "?/?" && eff != current_orient {
                initial_orient = Some(eff);
                break 'find_init;
            }
        }

        if let Some(solve_start_orient) = initial_orient {
            if let (Some(from), Some(to)) = (
                parse_orient_label(&current_orient),
                parse_orient_label(&solve_start_orient),
            ) {
                let rot_str = detect_rotation(from, to);
                let parts: Vec<&str> = rot_str.split_whitespace().collect();
                let mut t_offset = 0.0;

                for part in parts {
                    let rot_enum = match part {
                        "x" => Some(Rotation::X),
                        "x'" => Some(Rotation::Xp),
                        "x2" => Some(Rotation::X2),
                        "y" => Some(Rotation::Y),
                        "y'" => Some(Rotation::Yp),
                        "y2" => Some(Rotation::Y2),
                        "z" => Some(Rotation::Z),
                        "z'" => Some(Rotation::Zp),
                        "z2" => Some(Rotation::Z2),
                        _ => None,
                    };

                    if let Some(r) = rot_enum {
                        let t_initial = p1
                            .first()
                            .map(|m| m.t - 0.001 + t_offset)
                            .unwrap_or(telemetry.solve_start_t + t_offset);
                        parsed_solve.timeline.push(SolveEvent::Rotation {
                            t: t_initial,
                            axis: r,
                            from_orientation: from, // Note: intermediate orientations not tracked here
                            to_orientation: to,     // but this is the final goal
                            is_inspection: false,
                        });
                        t_offset += 0.0001;
                    }
                }

                detected_rotations.push(DetectedRotation {
                    before_move: 1,
                    rotation: rot_str,
                    from: current_orient.clone(),
                    to: solve_start_orient.clone(),
                });
            }
            current_orient = solve_start_orient;
        }
    }

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
            // Potential rotation — immediately accept if window is reliable
            if let (Some(from), Some(to)) = (
                parse_orient_label(&current_orient),
                parse_orient_label(&effective),
            ) {
                let rot_str = detect_rotation(from, to);
                let parts: Vec<&str> = rot_str.split_whitespace().collect();
                let mut t_offset = 0.0;

                for part in parts {
                    let rot_enum = match part {
                        "x" => Some(Rotation::X),
                        "x'" => Some(Rotation::Xp),
                        "x2" => Some(Rotation::X2),
                        "y" => Some(Rotation::Y),
                        "y'" => Some(Rotation::Yp),
                        "y2" => Some(Rotation::Y2),
                        "z" => Some(Rotation::Z),
                        "z'" => Some(Rotation::Zp),
                        "z2" => Some(Rotation::Z2),
                        _ => None,
                    };

                    if let Some(r) = rot_enum {
                        let t_rot =
                            if (idx + 1) < window_runs.len() && !window_runs[idx + 1].is_empty() {
                                window_runs[idx + 1][0].t_start
                            } else if !runs.is_empty() {
                                runs[0].t_start
                            } else {
                                telemetry.solve_start_t
                            };

                        parsed_solve.timeline.push(SolveEvent::Rotation {
                            t: t_rot + t_offset,
                            axis: r,
                            from_orientation: from,
                            to_orientation: to,
                            is_inspection: false,
                        });
                        t_offset += 0.0001;
                    }
                }

                detected_rotations.push(DetectedRotation {
                    before_move: idx + 1,
                    rotation: rot_str,
                    from: current_orient.clone(),
                    to: effective.clone(),
                });
                current_orient = effective.clone();
            }
        }

        move_orients.push(current_orient.clone());
    }

    if print_output {
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
    }

    // ========== Inspection detection ==========
    // A window where stable gyro labels form a round-trip (A -> B -> A) is an inspection:
    // the solver peeked at another face and rotated back. Each transition is stored as a
    // SolveEvent::Rotation with is_inspection = true. These never affect move_count / tps.
    {
        let str_to_rot_enum = |s: &str| -> Option<Rotation> {
            match s {
                "x" => Some(Rotation::X),
                "x'" => Some(Rotation::Xp),
                "x2" => Some(Rotation::X2),
                "y" => Some(Rotation::Y),
                "y'" => Some(Rotation::Yp),
                "y2" => Some(Rotation::Y2),
                "z" => Some(Rotation::Z),
                "z'" => Some(Rotation::Zp),
                "z2" => Some(Rotation::Z2),
                _ => None,
            }
        };

        let mut inspections_found = false;
        for (w, runs) in window_runs.iter().enumerate() {
            let (pc, nc) = window_ctx(w);
            // Collect non-noise run labels for this window
            let stable: Vec<&str> = runs
                .iter()
                .enumerate()
                .filter(|(i, _)| !is_noise(runs, *i, 1, pc.as_deref(), nc.as_deref()))
                .map(|(_, r)| r.label.as_str())
                .collect();

            if stable.len() < 3 {
                continue; // need at least: start -> peek -> back
            }
            let first = stable[0];
            let last = stable[stable.len() - 1];
            if first != last {
                continue; // not a round-trip
            }
            let has_different = stable[1..stable.len() - 1].iter().any(|s| *s != first);
            if !has_different {
                continue;
            }

            // Build deduplicated sequence of visited orientations
            let mut visited: Vec<&str> = vec![first];
            for s in &stable[1..] {
                if *s != *visited.last().unwrap() {
                    visited.push(s);
                }
            }

            // Distribute the window's time equally across each rotation step
            let t_win_start = boundaries[w];
            let t_win_end = boundaries[w + 1];
            let n_steps = visited.len().saturating_sub(1).max(1);
            let step_dt = (t_win_end - t_win_start) / n_steps as f64;

            // Emit one SolveEvent::Rotation(is_inspection=true) per consecutive pair
            for pair_idx in 0..visited.len().saturating_sub(1) {
                let from_str = visited[pair_idx];
                let to_str = visited[pair_idx + 1];
                if let (Some(from_o), Some(to_o)) =
                    (parse_orient_label(from_str), parse_orient_label(to_str))
                {
                    let rot_str = detect_rotation(from_o, to_o);
                    if let Some(r) = str_to_rot_enum(&rot_str) {
                        let t_step = t_win_start + step_dt * pair_idx as f64;
                        parsed_solve.timeline.push(SolveEvent::Rotation {
                            t: t_step,
                            axis: r,
                            from_orientation: from_o,
                            to_orientation: to_o,
                            is_inspection: true,
                        });
                    }
                }
            }

            if print_output {
                let after_move = if w > 0 { w } else { 0 };
                let before_move = if w < p1.len() { w + 1 } else { w };
                let duration_ms = (t_win_end - t_win_start) * 1000.0;
                println!(
                    "  Between moves {:>3}-{:>3} ({:.0}ms): {}",
                    after_move,
                    before_move,
                    duration_ms,
                    visited.join(" -> "),
                );
                inspections_found = true;
            }
        }

        if print_output {
            if !inspections_found {
                println!("Inspections (in-window round-trips):");
                println!("  None detected.");
            }
            println!();
        }
    }

    parsed_solve.is_solved = cube_body.is_solved();
    parsed_solve.move_count = p1.len();
    if duration > 0.0 {
        parsed_solve.tps = p1.len() as f64 / duration;
    }

    if parsed_solve.is_solved {
        parsed_solve.step_details.l4e = p1.len() as isize;
    }

    // Sort timeline by time so Rotations interleave with Moves correctly
    parsed_solve.timeline.sort_by(|a, b| {
        a.t()
            .partial_cmp(&b.t())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // ========== Perspectival Move Mapping ==========
    // Re-map hardware moves to relative moves based on the current orientation at the time of the move.
    {
        let mut active_orient = home_orient;
        for event in &mut parsed_solve.timeline {
            match event {
                SolveEvent::Rotation { to_orientation, .. } => {
                    active_orient = *to_orientation;
                }
                SolveEvent::Move {
                    body_move,
                    relative_move,
                    ..
                } => {
                    *relative_move = map_move_to_orientation(*body_move, active_orient);
                }
            }
        }
    }

    parsed_solve
}
