use rouxflow_bitboard::{BitCube, FaceMove, Rotation};

fn get_all_rotations() -> Vec<(Rotation, Rotation)> {
    vec![
        (Rotation::X, Rotation::Xp),
        (Rotation::Xp, Rotation::X),
        (Rotation::X2, Rotation::X2),
        (Rotation::Y, Rotation::Yp),
        (Rotation::Yp, Rotation::Y),
        (Rotation::Y2, Rotation::Y2),
        (Rotation::Z, Rotation::Zp),
        (Rotation::Zp, Rotation::Z),
        (Rotation::Z2, Rotation::Z2),
    ]
}

#[test]
fn rotation_fuzz_stress_test() {
    let mut cube = BitCube::new_solved();

    // 1. Initial scramble with face moves ONLY to create a messy state
    let scramble = vec![
        FaceMove::Bp,
        FaceMove::F2,
        FaceMove::U,
        FaceMove::L2,
        FaceMove::R2,
        FaceMove::Bp,
        FaceMove::Dp,
        FaceMove::B2,
        FaceMove::Fp,
        FaceMove::D2,
    ];
    for m in scramble {
        cube.apply_face_move(m);
    }

    let all_rotations = get_all_rotations();

    // 2. Stress test each rotation
    for (rot, inv) in all_rotations {
        let before = cube.clone();

        // Property 1: Rotation + Inverse = Identity
        let mut test_inv = before.clone();
        test_inv.apply_rotation(rot);
        test_inv.apply_rotation(inv);
        assert_eq!(test_inv, before, "Rotation + Inverse failed for {:?}", rot);

        // Property 2: Cycle Property (4x for 90deg, 2x for 180deg)
        let mut test_cycle = before.clone();
        let cycles = if format!("{:?}", rot).contains('2') {
            2
        } else {
            4
        };
        for _ in 0..cycles {
            test_cycle.apply_rotation(rot);
        }
        assert_eq!(test_cycle, before, "Cycle property failed for {:?}", rot);

        // Property 3: 180-degree equivalence (rot2 vs rot+rot)
        if !format!("{:?}", rot).contains('2') {
            let mut rot1 = before.clone();
            rot1.apply_rotation(rot);
            rot1.apply_rotation(rot);

            let rot2_enum = match rot {
                Rotation::X | Rotation::Xp => Rotation::X2,
                Rotation::Y | Rotation::Yp => Rotation::Y2,
                Rotation::Z | Rotation::Zp => Rotation::Z2,
                _ => unreachable!(),
            };
            let mut rot2 = before.clone();
            rot2.apply_rotation(rot2_enum);
            assert_eq!(rot1, rot2, "Double rotation mismatch for {:?}", rot);
        }

        // Advance state for next iteration
        cube.apply_rotation(rot);
    }
}
