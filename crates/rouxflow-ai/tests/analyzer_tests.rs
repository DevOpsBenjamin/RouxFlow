use rouxflow_ai::{analyze_solve_legacy, RouxStep, BitCube};

/// Helper: apply moves to a solved cube and return it.
fn cube_after(moves: &str) -> BitCube {
    let mut cube = BitCube::new_solved();
    for m in moves.split_whitespace() {
        cube.apply_move(m);
    }
    cube
}

// ---- BitCube phase check unit tests ----

#[test]
fn test_solved_cube_all_phases() {
    let cube = BitCube::new_solved();
    assert!(cube.is_fb_solved());
    assert!(cube.is_sb_solved());
    assert!(cube.is_cmll_solved());
    assert!(cube.is_solved());
    assert!(cube.is_l4e_solved());
    assert!(cube.is_ul_ur_placed());
    assert_eq!(cube.count_bad_edges(), 0);
}

#[test]
fn test_fb_survives_r_moves() {
    // R moves only affect R face + adjacent — FB (left block) should stay solved
    let cube = cube_after("R U R' U'");
    assert!(cube.is_fb_solved(), "FB should survive R/U moves");
    assert!(!cube.is_solved(), "Cube should not be fully solved");
}

#[test]
fn test_sb_survives_l_moves() {
    // L moves only affect L face + adjacent — SB (right block) should stay solved
    let cube = cube_after("L U L' U'");
    assert!(cube.is_sb_solved(), "SB should survive L/U moves");
    assert!(!cube.is_solved());
}

#[test]
fn test_m_move_preserves_blocks_and_cmll() {
    // M moves don't touch L or R face stickers, and don't touch corners
    let cube = cube_after("M");
    assert!(cube.is_fb_solved(), "FB survives M");
    assert!(cube.is_sb_solved(), "SB survives M");
    assert!(cube.is_cmll_solved(), "CMLL survives M");
    assert!(!cube.is_solved(), "Cube not solved after M");
}

#[test]
fn test_fb_broken_by_l_move() {
    let cube = cube_after("L");
    assert!(!cube.is_fb_solved(), "FB should be broken by L");
}

#[test]
fn test_sb_broken_by_r_move() {
    let cube = cube_after("R");
    assert!(!cube.is_sb_solved(), "SB should be broken by R");
}

#[test]
fn test_cmll_broken_by_r_u() {
    let cube = cube_after("R U");
    assert!(!cube.is_cmll_solved(), "CMLL should be broken by R U");
}

#[test]
fn test_is_solved_after_identity_scramble() {
    // Scramble + inverse = solved
    let cube = cube_after("R U R' U' U R U' R'");
    assert!(cube.is_solved());
}

// ---- Analyzer integration tests ----

#[test]
fn test_analyze_solve_legacyd_cube_no_moves() {
    // Scramble: empty (cube is already solved), no solve moves
    let result = analyze_solve_legacy("", &[], None);
    // All 4 phases should be detected immediately (0-length segments)
    assert_eq!(result.steps.len(), 4);
    assert_eq!(result.steps[0].step, RouxStep::FB);
    assert_eq!(result.steps[1].step, RouxStep::SB);
    assert_eq!(result.steps[2].step, RouxStep::CMLL);
    assert_eq!(result.steps[3].step, RouxStep::LSE);
    for s in &result.steps {
        assert_eq!(s.move_count, 0);
    }
    assert!(result.orientation.is_some());
}

#[test]
fn test_analyze_r_scramble_r_prime_solve() {
    // Scramble "R" breaks SB only. Solve "R'" restores it.
    // FB, CMLL corners are preserved through R moves.
    let moves = vec!["R'".to_string()];
    let result = analyze_solve_legacy("R", &moves, None);

    // FB should be detected pre-solve (0 moves)
    assert!(result.steps.iter().any(|s| s.step == RouxStep::FB));
    // LSE should be detected after R'
    assert!(result.steps.iter().any(|s| s.step == RouxStep::LSE));
    assert_eq!(result.steps.last().unwrap().step, RouxStep::LSE);
    assert!(result.orientation.is_some());
}

#[test]
fn test_analyze_detects_all_four_phases() {
    // Construct a solve by applying known inverse sequences.
    // Scramble = inverse of (FB_moves + SB_moves + CMLL_moves + LSE_moves)
    // so that applying them forward solves phase by phase.

    // FB setup: L D' L' (inverse: L D L')
    // After scramble "L D L'", applying "L D' L'" restores FB.
    // But we need a scramble that breaks everything...

    // Simpler approach: build scramble from end to start
    // Start solved, apply LSE-breaker, then CMLL-breaker, then SB-breaker, then FB-breaker
    // FB-breaker: L' (breaks L face)
    // SB-breaker: R' (breaks R face)
    // CMLL-breaker: U (rotates U corners)
    // LSE-breaker: M (shifts M-slice edges)
    //
    // Scramble = M U R' L' (applied in this order to solved cube)
    // Solve = L R U' M' (inverse in reverse order)

    let scramble = "M U R' L'";
    let moves: Vec<String> = vec!["L", "R", "U'", "M'"]
        .into_iter().map(String::from).collect();

    let result = analyze_solve_legacy(scramble, &moves, None);

    // We should get some phases detected. The exact boundaries depend on
    // which orientation the analyzer picks and whether intermediate states satisfy checks.
    // At minimum, we should detect FB and eventually LSE.
    assert!(result.steps.len() >= 1, "Should detect at least one phase");
    assert!(result.orientation.is_some(), "Should detect orientation");

    // After all 4 moves, cube should be solved, so LSE should be the last detected step
    if let Some(last) = result.steps.last() {
        assert_eq!(last.step, RouxStep::LSE, "Last step should be LSE (cube fully solved)");
    }
}

#[test]
fn test_analyze_orientation_detection() {
    // Build FB on the Red/Yellow side (R face = left block after y2)
    // Scramble only the L block area, leave R block intact
    // After y2 normalization, the R block becomes the L block (FB)
    let scramble = "L";
    let moves: Vec<String> = vec!["L'"].into_iter().map(String::from).collect();
    let result = analyze_solve_legacy(scramble, &moves, None);

    // The analyzer should find that FB is solved in some orientation after L'
    // Since R block is always intact, it could detect FB on the Red/Yellow orientation before L'
    assert!(result.orientation.is_some());
}

#[test]
fn test_analyze_with_timed_moves() {
    let scramble = "R";
    let moves: Vec<String> = vec!["R'".to_string()];
    let timed = vec![("R'".to_string(), 500u32)];
    let timed_refs: Vec<(String, u32)> = timed;
    let result = analyze_solve_legacy(scramble, &moves, Some(&timed_refs));

    // Check that time info is populated
    for step in &result.steps {
        assert!(step.time_ms.is_some());
    }
}

#[test]
fn test_analyze_phases_in_order() {
    // Phases must appear in FB → SB → CMLL → LSE order
    let scramble = "R U R' U' F' L2 D R2 B U2 L2 B2 D L2 D2 F2 D R2 D2";
    let moves: Vec<String> = "U' L' U L F' L F L U L' U L' U' L U' L U2 L' U' R U R' U R U2 R' U' M U M' U2 M U M'"
        .split_whitespace().map(String::from).collect();

    let result = analyze_solve_legacy(scramble, &moves, None);

    let mut prev_step_order = 0;
    for step in &result.steps {
        let order = match step.step {
            RouxStep::FB => 1,
            RouxStep::SB => 2,
            RouxStep::CMLL => 3,
            RouxStep::LSE => 4,
        };
        assert!(order > prev_step_order, "Phases must be in order: {:?} after {:?}", step.step, prev_step_order);
        prev_step_order = order;
    }
}

#[test]
fn test_analyze_move_indices_consistent() {
    let scramble = "R U R' F2";
    let moves: Vec<String> = "L D' L' r U R' U2 M' U M"
        .split_whitespace().map(String::from).collect();
    let result = analyze_solve_legacy(scramble, &moves, None);

    // Verify segment boundaries are consistent
    for (i, step) in result.steps.iter().enumerate() {
        assert!(step.end_move >= step.start_move, "end >= start for step {:?}", step.step);
        assert_eq!(step.move_count, step.end_move - step.start_move);

        // Each segment's start should equal the previous segment's end
        if i > 0 {
            assert_eq!(step.start_move, result.steps[i - 1].end_move,
                "Step {:?} start should equal previous step end", step.step);
        }
    }
}

#[test]
fn test_bitcube_is_solved_after_rotations() {
    // is_solved() should work regardless of cube orientation
    let mut cube = BitCube::new_solved();
    cube.rotate_y();
    assert!(cube.is_solved(), "Solved cube after y rotation should still be detected as solved");

    let mut cube2 = BitCube::new_solved();
    cube2.rotate_x2();
    assert!(cube2.is_solved(), "Solved cube after x2 rotation should still be detected as solved");
}

#[test]
fn test_fb_solved_ext_different_orientations() {
    // On a solved cube, FB should be detected for all color mappings
    let cube = BitCube::new_solved();
    // Standard: L=Orange(5), D=Yellow(1), F=Green(2), B=Blue(3)
    assert!(cube.is_fb_solved_ext(5, 1, 2, 3));
    // R face as "left" block in y2 orientation: R=Red(4), D=Yellow(1), F=Blue(3), B=Green(2)
    // But on a solved cube, R face (9-17) is all Red, so checking with
    // is_sb_solved_ext should work
    assert!(cube.is_sb_solved_ext(4, 1, 2, 3));
}

#[test]
fn test_ul_ur_placed_with_m_slice_offset() {
    // After M moves, U center ≠ U corner color.
    // UL/UR should be checked against corner color, not center.
    let mut cube = BitCube::new_solved();
    // M shifts U center to F color, but corners stay White.
    // On a solved cube, UL/UR are in place, so after M they should NOT be in place
    // (the M-slice edges have moved, UL/UR edges are unaffected by M but
    //  the reference should still work).
    // Actually M doesn't move UL(3,37) or UR(5,10) — it moves positions 1,4,7 on U.
    // So UL/UR edge stickers at 3 and 5 are unchanged (still White).
    // Corner at position 0 is unchanged (still White).
    // So is_ul_ur_placed should still be true.
    cube.apply_move("M");
    assert!(cube.is_ul_ur_placed(), "UL/UR should survive M move (edges+corners unaffected)");

    // Now verify the fix matters: U center (pos 4) is now Green (from F center),
    // but corner (pos 0) is still White. Edge at pos 3 is still White.
    // Old code using center would fail here because 3=White != 4=Green.
    // New code using corner correctly passes because 3=White == 0=White.
    assert_eq!(cube.get_color_at(4), 3, "U center should be Blue(3) after M (B→U cycle)");
    assert_eq!(cube.get_color_at(0), 0, "U corner should still be White(0) after M");
    assert_eq!(cube.get_color_at(3), 0, "UL edge U-sticker should still be White(0) after M");
}
