// RECONSTRUCTED FROM BACKGROUND MEMORY (TRUNCATED)
// This file contains the fragments I was able to recover for you to use in your manual repair.

// --- Fragment 1 (Lines 1-100) ---
use rouxflow_bitboard::move_indices::Rotation;
use rouxflow_bitboard::BitCube;
use rouxflow_core::telemetry::{
    DebugPass1Move, DebugPass2State, DebugPass3Rotation, DebugTrace, GyroSample, ParsedSolve,
    SolveEvent, SolveTelemetry,
};

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
pub fn analyze_solve(
    telemetry: &SolveTelemetry,
    mut debug: Option<&mut DebugTrace>,
) -> ParsedSolve {
    let _start_instant = std::time::Instant::now();

    // Stage 0: Correct solve start if first move happened earlier (latency/BLE buffer)
    let first_move_t = telemetry.solve_moves.first().map(|m| m.t);
    let effective_start = match first_move_t {
        Some(t) if t < telemetry.solve_start_t => t,
        _ => telemetry.solve_start_t,
    };

    let duration = telemetry.solve_end_t - effective_start;
    let solve_start = effective_start; // for relative printing

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

    if let Some(ref mut d) = debug {
        d.scramble = telemetry.scramble.clone();
    }

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

    if let Some(ref mut d) = debug {
        for m in &p1 {
            d.pass1_moves.push(DebugPass1Move {
                t: m.t - solve_start,
                original_moves: m.body_raw.clone(),
                merged_move: m.body_label.clone(),
            });
        }
    }

    // ========== PASS 2: Gyro orientation table ==========
    // For each move, collect ALL gyro samples in the window before and after.
    // Majority vote determines the most reliable orientation.

    // Compute N+1 windows between consecutive moves.
    let mut boundaries: Vec<f64> = Vec::with_capacity(p1.len() + 2);
    boundaries.push(solve_start);
    for m in &p1 {
        boundaries.push(m.t);
    }
    boundaries.push(telemetry.solve_end_t);

    let window_runs: Vec<Vec<GyroRun>> = (0..boundaries.len() - 1)
        .map(|w| collect_orient_runs(&combined_gyro, &home, boundaries[w], boundaries[w + 1]))
        .collect();

    // Print interleaved MOVE / GYRO timeline
    // (print_gyro_runs closure removed in favor of structured trace)

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
    let mut cube_detect = rouxflow_core::cube::CubeState::new();
    for token in telemetry.scramble.split_whitespace() {
        cube_detect.apply_move(token);
    }

    let (pc0, nc0) = window_ctx(0);
    // Initial state before move 1 could be recorded here if desired
    let initial_raw = window_effective_orient(&window_runs[0], pc0.as_deref(), nc0.as_deref());

    // Track centers for compensated orientation in Pass 2
    let mut centers_orient = home_orient;

    let mut debug_runs0: Vec<rouxflow_core::telemetry::DebugGyroRun> = Vec::new();
    for (i, run) in window_runs[0].iter().enumerate() {
        let noise = is_noise(&window_runs[0], i, 1, pc0.as_deref(), nc0.as_deref());
        let label = if noise {
            format!("{} (x{}) << noise", run.label, run.count)
        } else {
            format!("{} (x{})", run.label, run.count)
        };
        debug_runs0.push(rouxflow_core::telemetry::DebugGyroRun {
            t: run.t_start - solve_start,
            label,
        });
    }

    if let Some(ref mut d) = debug {
        d.pass2_states.push(DebugPass2State {
            t: 0.0,
            body_move: "START".to_string(),
            active_gyro_window: initial_raw, // Initial doesn't have center shifts yet
            gyro_runs: debug_runs0,
            cube_state: cube_body.to_html_string(),
        });
    }

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

            // Track center shifts in Pass 2 just for debug display
            if let Some(rot) = slice_core_rotation(slice_move) {
                centers_orient = apply_rotation(centers_orient, rot);
            }
        } else {
            let m_str = &m.body_raw[0];
            cube_detect.apply_move(m_str);

            // Track center shifts for wide moves in Pass 2
            if let Some(rot) = wide_core_rotation(m_str) {
                centers_orient = apply_rotation(centers_orient, rot);
            }
        }

        let mut f_stat = "X".to_string();
        if cube_detect.is_fb_solved() {
            f_stat = "V".to_string();
        }
        let mut s_stat = "X".to_string();
        if cube_detect.is_sb_solved() {
            s_stat = "V".to_string();
        }
        let mut c_stat = "X".to_string();
        if cube_detect.is_cmll_solved() {
            c_stat = "V".to_string();
        }
        let mut u_stat = "X".to_string();
        if cube_detect.is_ul_ur_placed() {
            u_stat = "V".to_string();
        }

        let mut aligned_cube: Option<rouxflow_core::cube::CubeState> = None;

        if cube_detect.is_sb_solved() {
            aligned_cube = Some(cube_detect.clone());
        }

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

        let relative_move = body_move;

        // Step logic Tracking
        let is_fb = f_stat == "V";
        let is_sb = s_stat == "V";
        let is_cmll = c_stat == "V";
        let is_ur_lr = u_stat == "V";
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
            t: m.t - solve_start,
            original: m.body_raw.clone(),
            body_move,
            relative_move,
        });

        let w = idx + 1;
        let (pc, nc) = window_ctx(w);
        let raw_eff = window_effective_orient(&window_runs[w], pc.as_deref(), nc.as_deref());

        // Compensate the debug display so it shows PERSPECTIVAL orientation
        let eff = if let Some(shell) = parse_orient_label(&raw_eff) {
            orientation_label(combine_orientations(shell, centers_orient))
        } else {
            raw_eff
        };

        let rel_t = m.t - solve_start;

        let mut debug_runs: Vec<rouxflow_core::telemetry::DebugGyroRun> = Vec::new();
        for (i, run) in window_runs[w].iter().enumerate() {
            let noise = is_noise(&window_runs[w], i, 1, pc.as_deref(), nc.as_deref());
            let label = if noise {
                format!("{} (x{}) << noise", run.label, run.count)
            } else {
                format!("{} (x{})", run.label, run.count)
            };
            debug_runs.push(rouxflow_core::telemetry::DebugGyroRun {
                t: run.t_start - solve_start,
                label,
            });
        }

        if let Some(ref mut d) = debug {
            d.pass2_states.push(DebugPass2State {
                t: rel_t,
                body_move: m.body_label.clone(),
                active_gyro_window: eff,
                gyro_runs: debug_runs,
                cube_state: cube_body.to_html_string(),
            });
        }
    }

    println!();

    // ========== PASS 3: Rotation detection ==========
    // We use a "flattened stable runs" approach.
    // 1. Collect all non-noise runs across all windows.
    // 2. Identify transitions between different stable orientations.
    // 3. Mark transitions as "baseline shifts" if they cross a slice move boundary.
    // 4. Everything else is a user rotation (persistent or inspection peek).

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

    struct StableRun {
        t: f64,
        label: String,
        window_idx: usize,
    }

    let mut stable_sequence: Vec<StableRun> = Vec::new();
    for (w, runs) in window_runs.iter().enumerate() {
        let (pc, nc) = window_ctx(w);
        for (i, run) in runs.iter().enumerate() {
            if !is_noise(runs, i, 1, pc.as_deref(), nc.as_deref()) {
                stable_sequence.push(StableRun {
                    t: run.t_start,
                    label: run.label.clone(),
                    window_idx: w,
                });
            }
        }
    }

    let mut current_orient = orientation_label(home_orient);

    if !stable_sequence.is_empty() {
        // Handle initial orientation if it differs from home
        let first_stable = &stable_sequence[0];
        if first_stable.label != current_orient && first_stable.window_idx <= 2 {
            if let (Some(from), Some(to)) = (
                parse_orient_label(&current_orient),
                parse_orient_label(&first_stable.label),
            ) {
                let rot_str = detect_rotation(from, to);
                let parts: Vec<&str> = rot_str.split_whitespace().collect();
                let mut temp_orient = from;
                let mut t_offset = 0.0;

                for part in parts {
                    if let Some(r) = str_to_rot_enum(part) {
                        // Intermediate orientation for this step
                        let step_to_orient = apply_rotation(temp_orient, part);

                        // Force initial rotation to -0.01s relative to solve_start
                        let t_rot = solve_start - 0.01 + t_offset;
                        parsed_solve.timeline.push(SolveEvent::Rotation {
                            t: t_rot - solve_start,
                            axis: r,
                            from_orientation: temp_orient,
                            to_orientation: step_to_orient,
                            is_inspection: false,
                        });
                        if let Some(ref mut d) = debug {
                            d.pass3_rotations.push(DebugPass3Rotation {
                                t: t_rot - solve_start,
                                before_move_idx: 1, // Start of the solve
                                rotation_label: part.to_string(),
                                from_orient: orientation_label(temp_orient),
                                to_orient: orientation_label(step_to_orient),
                                is_inspection: false,
                            });
                        }
                        temp_orient = step_to_orient;
                        t_offset += 0.001; // Spaced by 1ms to avoid HTML grouping collisions
                    }
                }
                current_orient = first_stable.label.clone();
            }
        }

        // Process transitions in the stable sequence
        let mut i = 0;
        while i + 1 < stable_sequence.len() {
            let cur = &stable_sequence[i];
            let next = &stable_sequence[i + 1];

            if cur.label != next.label {
                // Determine if this is a baseline shift (slice move between them)
                let mut has_slice = false;
                for w_idx in cur.window_idx..next.window_idx {
                    if w_idx < p1.len() && p1[w_idx].body_raw.len() == 2 {
                        has_slice = true;
                        break;
                    }
                }

                if has_slice {
                    // Silently update baseline
                    current_orient = next.label.clone();
                } else {
                    // This is a user rotation.
                    // Is it an inspection peek? Check if we eventually return to cur.label
                    // before any other persistent rotation or slice move.
                    let mut is_peek = false;
                    for j in (i + 2)..stable_sequence.len() {
                        let fwd = &stable_sequence[j];
                        // If we see a slice move before returning, it's not a simple peek.
                        let mut slice_interrupt = false;
                        for w_idx in next.window_idx..fwd.window_idx {
                            if w_idx < p1.len() && p1[w_idx].body_raw.len() == 2 {
                                slice_interrupt = true;
                                break;
                            }
                        }
                        if slice_interrupt {
                            break;
                        }
                        if fwd.label == cur.label {
                            is_peek = true;
                            break;
                        }
                        if fwd.label != next.label {
                            // Another rotation happened
                            break;
                        }
                    }

                    if let (Some(from), Some(to)) = (
                        parse_orient_label(&current_orient),
                        parse_orient_label(&next.label),
                    ) {
                        let rot_str = detect_rotation(from, to);
                        let parts: Vec<&str> = rot_str.split_whitespace().collect();
                        let mut temp_orient = from;
                        let mut t_offset = 0.0;

                        for part in parts {
                            if let Some(r) = str_to_rot_enum(part) {
                                let step_to_orient = apply_rotation(temp_orient, part);
                                let t_rot = next.t + t_offset;
                                parsed_solve.timeline.push(SolveEvent::Rotation {
                                    t: t_rot - solve_start,
                                    axis: r,
                                    from_orientation: temp_orient,
                                    to_orientation: step_to_orient,
                                    is_inspection: is_peek,
                                });
                                if let Some(ref mut d) = debug {
                                    d.pass3_rotations.push(DebugPass3Rotation {
                                        t: t_rot - solve_start,
                                        before_move_idx: next.window_idx.max(1),
                                        rotation_label: part.to_string(),
                                        from_orient: orientation_label(temp_orient),
                                        to_orient: orientation_label(step_to_orient),
                                        is_inspection: is_peek,
                                    });
                                }
                                temp_orient = step_to_orient;
                                t_offset += 0.001;
                            }
                        }
                        current_orient = next.label.clone();
                    }
                }
            }
            i += 1;
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
        let mut centers_orient = home_orient;
        let mut last_gyro_shell_orient = home_orient;

        for event in &mut parsed_solve.timeline {
            match event {
                SolveEvent::Rotation {
                    to_orientation,
                    is_inspection,
                    ..
                } => {
                    // Update physical shell orientation from gyro
                    if !*is_inspection {
                        last_gyro_shell_orient = *to_orientation;
                    }
                }
                SolveEvent::Move {
                    body_move,
                    relative_move,
                    ..
                } => {
                    // Combine shell orientation and center shifts for net perspectival mapping
                    let active_orient =
                        combine_orientations(last_gyro_shell_orient, centers_orient);
                    *relative_move = map_move_to_orientation(*body_move, active_orient);

                    // Track center shifts caused by core-rotating moves
                    let rel_str = relative_move.as_str();
                    if let Some(rot) = slice_core_rotation(rel_str) {
                        centers_orient = apply_rotation(centers_orient, rot);
                    } else if let Some(rot) = wide_core_rotation(rel_str) {
                        centers_orient = apply_rotation(centers_orient, rot);
                    }
                }
            }
        }
    }

    if let Some(ref mut d) = debug {
        d.clean_replay = Some(parsed_solve.to_clean());
    }

    parsed_solve
}
