use rouxflow_bitboard::{BitCube, FaceMove, SliceMove, WideMove};

fn get_all_wide_moves() -> Vec<(WideMove, FaceMove, Option<SliceMove>, WideMove)> {
    vec![
        (
            WideMove::Uw,
            FaceMove::U,
            Some(SliceMove::Ep),
            WideMove::Uwp,
        ),
        (
            WideMove::Uwp,
            FaceMove::Up,
            Some(SliceMove::E),
            WideMove::Uw,
        ),
        (
            WideMove::Uw2,
            FaceMove::U2,
            Some(SliceMove::E2),
            WideMove::Uw2,
        ),
        (WideMove::Dw, FaceMove::D, Some(SliceMove::E), WideMove::Dwp),
        (
            WideMove::Dwp,
            FaceMove::Dp,
            Some(SliceMove::Ep),
            WideMove::Dw,
        ),
        (
            WideMove::Dw2,
            FaceMove::D2,
            Some(SliceMove::E2),
            WideMove::Dw2,
        ),
        (WideMove::Lw, FaceMove::L, Some(SliceMove::M), WideMove::Lwp),
        (
            WideMove::Lwp,
            FaceMove::Lp,
            Some(SliceMove::Mp),
            WideMove::Lw,
        ),
        (
            WideMove::Lw2,
            FaceMove::L2,
            Some(SliceMove::M2),
            WideMove::Lw2,
        ),
        (
            WideMove::Rw,
            FaceMove::R,
            Some(SliceMove::Mp),
            WideMove::Rwp,
        ),
        (
            WideMove::Rwp,
            FaceMove::Rp,
            Some(SliceMove::M),
            WideMove::Rw,
        ),
        (
            WideMove::Rw2,
            FaceMove::R2,
            Some(SliceMove::M2),
            WideMove::Rw2,
        ),
        (WideMove::Fw, FaceMove::F, Some(SliceMove::S), WideMove::Fwp),
        (
            WideMove::Fwp,
            FaceMove::Fp,
            Some(SliceMove::Sp),
            WideMove::Fw,
        ),
        (
            WideMove::Fw2,
            FaceMove::F2,
            Some(SliceMove::S2),
            WideMove::Fw2,
        ),
        (
            WideMove::Bw,
            FaceMove::B,
            Some(SliceMove::Sp),
            WideMove::Bwp,
        ),
        (
            WideMove::Bwp,
            FaceMove::Bp,
            Some(SliceMove::S),
            WideMove::Bw,
        ),
        (
            WideMove::Bw2,
            FaceMove::B2,
            Some(SliceMove::S2),
            WideMove::Bw2,
        ),
    ]
}

#[test]
fn wide_move_fuzz_stress_test() {
    let mut cube = BitCube::new_solved();
    let all_moves = get_all_wide_moves();

    // 1. Initial scramble to get away from solid colors
    for i in 0..200 {
        let idx = (i * 7 + 3) % all_moves.len();
        cube.apply_wide_move(all_moves[idx].0);
    }

    // 2. Fuzzing loop
    for i in 0..1000 {
        let idx = (i * 13 + 7) % all_moves.len();
        let (m, face, slice, inv) = all_moves[idx];

        let before = cube.clone();

        // Property 1: Move + Inverse = Identity
        cube.apply_wide_move(m);
        cube.apply_wide_move(inv);
        assert_eq!(
            cube, before,
            "Move + Inverse failed for {:?} at iteration {}",
            m, i
        );

        // Property 2: Wide == Face + Slice
        let mut cube_val = before.clone();
        cube_val.apply_face_move(face);
        if let Some(s) = slice {
            cube_val.apply_slice_move(s);
        }

        let mut cube_actual = before.clone();
        cube_actual.apply_wide_move(m);
        assert_eq!(
            cube_actual, cube_val,
            "Equivalence failed for {:?} at iteration {}",
            m, i
        );

        // Property 3: Apply move 4 times (or 2 for double) returns to start
        let mut cube_cycle = before.clone();
        let cycles = if format!("{:?}", m).ends_with('2') {
            2
        } else {
            4
        };
        for _ in 0..cycles {
            cube_cycle.apply_wide_move(m);
        }
        assert_eq!(
            cube_cycle, before,
            "Cycle property failed for {:?} at iteration {}",
            m, i
        );

        // Advance the state for the next iteration
        cube.apply_wide_move(m);
    }
}
