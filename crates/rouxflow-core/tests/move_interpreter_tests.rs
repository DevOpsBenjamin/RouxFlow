use rouxflow_core::move_interpreter::{MoveInterpreter, InterpreterConfig, MoveKind};
use rouxflow_core::cube::{Face, Quaternion};

fn default_config() -> InterpreterConfig {
    InterpreterConfig {
        merge_window_ms: 50.0,
        rotation_threshold_rad: 1.2,
        wide_threshold_rad: 0.5,
        has_gyro: false,
    }
}

#[test]
fn single_move_passthrough() {
    let mut interp = MoveInterpreter::new(default_config());
    interp.feed_face_move(Face::R, 1, 100.0);
    // Not expired yet
    let moves = interp.flush(120.0, 0.0);
    assert!(moves.is_empty());
    // Now expired
    let moves = interp.flush(160.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "R");
    assert_eq!(moves[0].kind, MoveKind::Face);
    assert_eq!(moves[0].raw_face_moves.len(), 1);
}

#[test]
fn single_move_prime() {
    let mut interp = MoveInterpreter::new(default_config());
    interp.feed_face_move(Face::U, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "U'");
}

#[test]
fn slice_m_merge() {
    let mut interp = MoveInterpreter::new(default_config());
    // M = L + R' (same direction as L)
    interp.feed_face_move(Face::L, 1, 100.0);
    interp.feed_face_move(Face::R, -1, 100.0);
    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "M");
    assert_eq!(moves[0].kind, MoveKind::Slice);
    assert_eq!(moves[0].raw_face_moves.len(), 2);
}

#[test]
fn slice_m_prime_merge() {
    let mut interp = MoveInterpreter::new(default_config());
    // M' = L' + R
    interp.feed_face_move(Face::R, 1, 100.0);
    interp.feed_face_move(Face::L, -1, 100.0);
    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "M'");
    assert_eq!(moves[0].kind, MoveKind::Slice);
}

#[test]
fn slice_e_merge() {
    let mut interp = MoveInterpreter::new(default_config());
    // E = D + U' (same direction as D)
    interp.feed_face_move(Face::D, 1, 100.0);
    interp.feed_face_move(Face::U, -1, 100.0);
    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "E");
    assert_eq!(moves[0].kind, MoveKind::Slice);
}

#[test]
fn slice_s_merge() {
    let mut interp = MoveInterpreter::new(default_config());
    // S = F + B' (same direction as F)
    interp.feed_face_move(Face::F, 1, 100.0);
    interp.feed_face_move(Face::B, -1, 100.0);
    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "S");
    assert_eq!(moves[0].kind, MoveKind::Slice);
}

#[test]
fn non_slice_same_direction() {
    let mut interp = MoveInterpreter::new(default_config());
    // R + L (same direction, not a slice)
    interp.feed_face_move(Face::R, 1, 100.0);
    interp.feed_face_move(Face::L, 1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].notation, "R");
    assert_eq!(moves[1].notation, "L");
}

#[test]
fn non_slice_different_axis() {
    let mut interp = MoveInterpreter::new(default_config());
    // R + U' → not opposite faces
    interp.feed_face_move(Face::R, 1, 100.0);
    interp.feed_face_move(Face::U, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].notation, "R");
    assert_eq!(moves[1].notation, "U'");
}

#[test]
fn window_expiry_emits_first() {
    let mut interp = MoveInterpreter::new(default_config());
    interp.feed_face_move(Face::R, 1, 100.0);
    // Still in window
    let moves = interp.flush(140.0, 0.0);
    assert!(moves.is_empty());
    // Expired
    let moves = interp.flush(160.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "R");
}

#[test]
fn multi_move_batch() {
    let mut interp = MoveInterpreter::new(default_config());
    // M slice + standalone F
    interp.feed_face_move(Face::L, 1, 100.0);
    interp.feed_face_move(Face::R, -1, 100.0);
    interp.feed_face_move(Face::F, 1, 110.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].notation, "M");
    assert_eq!(moves[1].notation, "F");
}

#[test]
fn timestamp_relative_to_solve_start() {
    let mut interp = MoveInterpreter::new(default_config());
    interp.feed_face_move(Face::R, 1, 1500.0);
    let moves = interp.flush(1600.0, 1000.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].timestamp_ms, 500); // 1500 - 1000
}

#[test]
fn timestamp_zero_when_not_solving() {
    let mut interp = MoveInterpreter::new(default_config());
    interp.feed_face_move(Face::R, 1, 1500.0);
    let moves = interp.flush(1600.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].timestamp_ms, 0);
}

#[test]
fn reset_clears_pending() {
    let mut interp = MoveInterpreter::new(default_config());
    interp.feed_face_move(Face::R, 1, 100.0);
    interp.reset();
    let moves = interp.flush(200.0, 0.0);
    assert!(moves.is_empty());
}

#[test]
fn gyro_only_rotation_x() {
    let mut config = default_config();
    config.has_gyro = true;
    let mut interp = MoveInterpreter::new(config);

    // Anchor at identity
    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 100.0);

    // Rotate ~90° around x axis
    let angle = std::f32::consts::FRAC_PI_2;
    let half = angle / 2.0;
    let rotated = Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() };
    interp.feed_gyro(&rotated, 200.0);

    // Zone-based gating: hint must be set (normally done by WASM from calibrator)
    interp.set_zone_rotation_hint(true);
    let moves = interp.flush(250.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "x");
    assert_eq!(moves[0].kind, MoveKind::Rotation);
    assert!(moves[0].raw_face_moves.is_empty());
}

#[test]
fn no_gyro_rotation_without_zone_hint() {
    let mut config = default_config();
    config.has_gyro = true;
    let mut interp = MoveInterpreter::new(config);

    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 100.0);

    let angle = std::f32::consts::FRAC_PI_2;
    let half = angle / 2.0;
    let rotated = Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() };
    interp.feed_gyro(&rotated, 200.0);

    // No zone hint set → no rotation emitted (prevents false M-wobble rotations)
    let moves = interp.flush(250.0, 0.0);
    assert!(moves.is_empty());
}

#[test]
fn no_gyro_rotation_without_flag() {
    let mut interp = MoveInterpreter::new(default_config()); // has_gyro = false

    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 100.0);

    let angle = std::f32::consts::FRAC_PI_2;
    let half = angle / 2.0;
    let rotated = Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() };
    interp.feed_gyro(&rotated, 200.0);

    let moves = interp.flush(250.0, 0.0);
    assert!(moves.is_empty()); // No rotation emitted when gyro flag is off
}

#[test]
fn pair_always_slice_even_with_high_gyro() {
    let mut config = default_config();
    config.has_gyro = true;
    let mut interp = MoveInterpreter::new(config);

    // Set anchor gyro at identity
    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 90.0);

    // Simulate 90° rotation around x axis
    let angle = std::f32::consts::FRAC_PI_2;
    let half = angle / 2.0;
    let rotated = Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() };
    interp.feed_gyro(&rotated, 95.0);

    // Feed R + L' → always classified as slice M' (never upgraded to rotation)
    interp.feed_face_move(Face::R, 1, 100.0);
    interp.feed_face_move(Face::L, -1, 100.0);

    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "M'");
    assert_eq!(moves[0].kind, MoveKind::Slice);
    assert_eq!(moves[0].raw_face_moves.len(), 2);
}

#[test]
fn pair_always_slice_with_small_gyro() {
    let mut config = default_config();
    config.has_gyro = true;
    let mut interp = MoveInterpreter::new(config);

    // Set anchor gyro at identity
    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 90.0);

    // Small rotation (10°)
    let angle = 10.0_f32.to_radians();
    let half = angle / 2.0;
    let rotated = Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() };
    interp.feed_gyro(&rotated, 95.0);

    // Feed R + L' → slice M'
    interp.feed_face_move(Face::R, 1, 100.0);
    interp.feed_face_move(Face::L, -1, 100.0);

    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "M'");
    assert_eq!(moves[0].kind, MoveKind::Slice);
}

// ========== Wide Move Tests ==========

/// Helper: create a gyro-enabled interpreter with identity anchor and a rotation applied.
fn gyro_interp_with_rotation(axis: usize, angle_rad: f32) -> MoveInterpreter {
    let mut config = default_config();
    config.has_gyro = true;
    let mut interp = MoveInterpreter::new(config);

    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 90.0);

    let half = angle_rad / 2.0;
    let rotated = match axis {
        0 => Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() },
        1 => Quaternion { x: 0.0, y: half.sin(), z: 0.0, w: half.cos() },
        2 => Quaternion { x: 0.0, y: 0.0, z: half.sin(), w: half.cos() },
        _ => unreachable!(),
    };
    interp.feed_gyro(&rotated, 95.0);
    interp
}

#[test]
fn wide_r() {
    // Wide r: cube reports L' + gyro x+
    let mut interp = gyro_interp_with_rotation(0, 1.5); // ~86° x+
    interp.feed_face_move(Face::L, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "r");
    assert_eq!(moves[0].kind, MoveKind::Wide);
    assert_eq!(moves[0].raw_face_moves.len(), 1);
    assert_eq!(moves[0].raw_face_moves[0], (Face::L, -1));
}

#[test]
fn wide_r_prime() {
    // Wide r': cube reports L + gyro x-
    let mut interp = gyro_interp_with_rotation(0, -1.5); // x-
    interp.feed_face_move(Face::L, 1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "r'");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn wide_l() {
    // Wide l: cube reports R + gyro x-
    let mut interp = gyro_interp_with_rotation(0, -1.5);
    interp.feed_face_move(Face::R, 1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "l");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn wide_l_prime() {
    // Wide l': cube reports R' + gyro x+
    let mut interp = gyro_interp_with_rotation(0, 1.5);
    interp.feed_face_move(Face::R, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "l'");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn wide_u() {
    // Wide u: cube reports D' + gyro y+
    let mut interp = gyro_interp_with_rotation(1, 1.5);
    interp.feed_face_move(Face::D, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "u");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn wide_u_prime() {
    // Wide u': cube reports D + gyro y-
    let mut interp = gyro_interp_with_rotation(1, -1.5);
    interp.feed_face_move(Face::D, 1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "u'");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn wide_f() {
    // Wide f: cube reports B' + gyro z+
    let mut interp = gyro_interp_with_rotation(2, 1.5);
    interp.feed_face_move(Face::B, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "f");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn wide_b() {
    // Wide b: cube reports F + gyro z-
    let mut interp = gyro_interp_with_rotation(2, -1.5);
    interp.feed_face_move(Face::F, 1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "b");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}

#[test]
fn no_wide_without_gyro_flag() {
    // Without has_gyro, even with accumulated rotation, should be a plain face move
    let mut interp = MoveInterpreter::new(default_config()); // has_gyro = false
    let identity = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    interp.feed_gyro(&identity, 90.0);
    let half = 0.75_f32;
    let rotated = Quaternion { x: half.sin(), y: 0.0, z: 0.0, w: half.cos() };
    interp.feed_gyro(&rotated, 95.0);

    interp.feed_face_move(Face::L, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "L'");
    assert_eq!(moves[0].kind, MoveKind::Face);
}

#[test]
fn no_wide_with_inconsistent_direction() {
    // L + gyro x+ → direction consistency fails (dir=1, gyro_sign=1, 1*1 >= 0)
    let mut interp = gyro_interp_with_rotation(0, 1.5); // x+
    interp.feed_face_move(Face::L, 1, 100.0); // L (not L')
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "L");
    assert_eq!(moves[0].kind, MoveKind::Face);
}

#[test]
fn no_wide_below_threshold() {
    // Small gyro rotation (0.3 rad) — below wide_threshold (0.5)
    let mut interp = gyro_interp_with_rotation(0, 0.3);
    interp.feed_face_move(Face::L, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "L'");
    assert_eq!(moves[0].kind, MoveKind::Face);
}

#[test]
fn pair_wins_over_wide() {
    // L' + R within window → always slice pair, NOT wide, NOT rotation
    let mut interp = gyro_interp_with_rotation(0, 1.5);
    interp.feed_face_move(Face::L, -1, 100.0);
    interp.feed_face_move(Face::R, 1, 100.0);
    let moves = interp.flush(100.0, 0.0);
    assert_eq!(moves.len(), 1);
    // Pairs are always slices — real rotations come from standalone gyro only
    assert_eq!(moves[0].notation, "M'");
    assert_eq!(moves[0].kind, MoveKind::Slice);
    assert_eq!(moves[0].raw_face_moves.len(), 2);
}

#[test]
fn wide_move_has_one_raw_face_move() {
    let mut interp = gyro_interp_with_rotation(0, 1.5);
    interp.feed_face_move(Face::L, -1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].raw_face_moves.len(), 1);
}

#[test]
fn wide_d() {
    // Wide d: cube reports U + gyro y-
    let mut interp = gyro_interp_with_rotation(1, -1.5);
    interp.feed_face_move(Face::U, 1, 100.0);
    let moves = interp.flush(200.0, 0.0);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].notation, "d");
    assert_eq!(moves[0].kind, MoveKind::Wide);
}
