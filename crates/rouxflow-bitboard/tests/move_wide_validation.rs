use rouxflow_bitboard::{BitCube, FaceMove, SliceMove, WideMove};

#[test]
fn wide_move_equivalence_stress_test() {
    let mut base_cube = BitCube::new_solved();
    // Scramble it to make it messy
    let scramble = "B' F2 U L2 R2 B' D' B2 F' D2 R2 B' U2 D' L2 U' B' D' B2 L";
    for m in scramble.split_whitespace() {
        base_cube.apply_move(m);
    }

    let moves_to_test = [
        ("Uw", WideMove::Uw, FaceMove::U, Some(SliceMove::Ep)),
        ("Uwp", WideMove::Uwp, FaceMove::Up, Some(SliceMove::E)),
        ("Uw2", WideMove::Uw2, FaceMove::U2, Some(SliceMove::E2)),
        ("Dw", WideMove::Dw, FaceMove::D, Some(SliceMove::E)),
        ("Dwp", WideMove::Dwp, FaceMove::Dp, Some(SliceMove::Ep)),
        ("Dw2", WideMove::Dw2, FaceMove::D2, Some(SliceMove::E2)),
        ("Lw", WideMove::Lw, FaceMove::L, Some(SliceMove::M)),
        ("Lwp", WideMove::Lwp, FaceMove::Lp, Some(SliceMove::Mp)),
        ("Lw2", WideMove::Lw2, FaceMove::L2, Some(SliceMove::M2)),
        ("Rw", WideMove::Rw, FaceMove::R, Some(SliceMove::Mp)),
        ("Rwp", WideMove::Rwp, FaceMove::Rp, Some(SliceMove::M)),
        ("Rw2", WideMove::Rw2, FaceMove::R2, Some(SliceMove::M2)),
        ("Fw", WideMove::Fw, FaceMove::F, Some(SliceMove::S)),
        ("Fwp", WideMove::Fwp, FaceMove::Fp, Some(SliceMove::Sp)),
        ("Fw2", WideMove::Fw2, FaceMove::F2, Some(SliceMove::S2)),
        ("Bw", WideMove::Bw, FaceMove::B, Some(SliceMove::Sp)),
        ("Bwp", WideMove::Bwp, FaceMove::Bp, Some(SliceMove::S)),
        ("Bw2", WideMove::Bw2, FaceMove::B2, Some(SliceMove::S2)),
    ];

    let mut failures = 0;
    for (name, wide, face, slice) in moves_to_test {
        let mut cube_a = base_cube.clone();
        let mut cube_b = base_cube.clone();

        cube_a.apply_wide_move(wide);
        cube_b.apply_face_move(face);
        if let Some(s) = slice {
            cube_b.apply_slice_move(s);
        }

        if cube_a != cube_b {
            println!("Mismatch found for Wide Move: {}", name);
            failures += 1;
        }
    }
    assert_eq!(
        failures, 0,
        "Found {} wide move equivalence failures!",
        failures
    );
}

#[test]
fn wide_move_cycle_property() {
    let mut base_cube = BitCube::new_solved();
    let scramble = "L2 U B2 D' F2 L2 R2 U' B2 U2 R2 B L' D' B' R' U2 B2 R' D";
    for m in scramble.split_whitespace() {
        base_cube.apply_move(m);
    }

    let wide_moves = [
        WideMove::Uw,
        WideMove::Dw,
        WideMove::Lw,
        WideMove::Rw,
        WideMove::Fw,
        WideMove::Bw,
    ];

    for m in wide_moves {
        let mut cube = base_cube.clone();
        for _ in 0..4 {
            cube.apply_wide_move(m);
        }
        assert_eq!(
            cube, base_cube,
            "Cycle property failed for wide move: {:?}",
            m
        );
    }
}
