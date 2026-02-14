use rouxflow_core::telemetry::{GyroSample, SolveTelemetry};

// ========== Color / Orientation types ==========

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    White,
    Yellow,
    Green,
    Blue,
    Red,
    Orange,
}

impl Color {
    fn label(&self) -> char {
        match self {
            Color::White => 'W',
            Color::Yellow => 'Y',
            Color::Green => 'G',
            Color::Blue => 'B',
            Color::Red => 'R',
            Color::Orange => 'O',
        }
    }
}

/// The 6 axis directions and their corresponding colors.
/// MoYu V10: +Y=White, -Y=Yellow, +Z=Green, -Z=Blue, +X=Red, -X=Orange.
const AXIS_COLORS: [([f32; 3], Color); 6] = [
    ([0.0,  1.0,  0.0], Color::White),   // +Y
    ([0.0, -1.0,  0.0], Color::Yellow),  // -Y
    ([0.0,  0.0,  1.0], Color::Green),   // +Z
    ([0.0,  0.0, -1.0], Color::Blue),    // -Z
    ([1.0,  0.0,  0.0], Color::Red),     // +X
    ([-1.0, 0.0,  0.0], Color::Orange),  // -X
];

// ========== Quaternion math ==========

fn quat_conjugate(q: &[f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

fn quat_multiply(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    let (ax, ay, az, aw) = (a[0], a[1], a[2], a[3]);
    let (bx, by, bz, bw) = (b[0], b[1], b[2], b[3]);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

fn quat_rotate_vec(q: &[f32; 4], v: &[f32; 3]) -> [f32; 3] {
    // q * (0,v) * conj(q)
    let v_quat = [v[0], v[1], v[2], 0.0];
    let conj = quat_conjugate(q);
    let tmp = quat_multiply(q, &v_quat);
    let result = quat_multiply(&tmp, &conj);
    [result[0], result[1], result[2]]
}

fn quat_normalize(q: &[f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

fn quat_dot(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// conj(home) * current -- orientation of current relative to home.
fn relative_quaternion(home: &[f32; 4], current: &[f32; 4]) -> [f32; 4] {
    quat_multiply(&quat_conjugate(home), current)
}

// ========== Gyro lookup ==========

/// Binary search: last sample with sample.t <= t.
fn find_gyro_before<'a>(gyro: &'a [GyroSample], t: f64) -> Option<&'a GyroSample> {
    if gyro.is_empty() {
        return None;
    }
    let idx = gyro.partition_point(|s| s.t <= t);
    if idx == 0 {
        None
    } else {
        Some(&gyro[idx - 1])
    }
}

/// Binary search: first sample with sample.t >= t.
fn find_gyro_after<'a>(gyro: &'a [GyroSample], t: f64) -> Option<&'a GyroSample> {
    if gyro.is_empty() {
        return None;
    }
    let idx = gyro.partition_point(|s| s.t < t);
    if idx < gyro.len() {
        Some(&gyro[idx])
    } else {
        None
    }
}

// ========== Home quaternion computation ==========

/// Compute the "home" orientation from scramble gyro data.
/// 1. Sign-flip all samples to same hemisphere (if dot(sample, first) < 0, negate)
/// 2. Compute val = x^2 + y^2 + z^2 for each
/// 3. Discard bottom 10% and top 10% by val
/// 4. Average remaining 80% component-wise
/// 5. Normalize to unit quaternion
fn compute_home(samples: &[GyroSample]) -> [f32; 4] {
    if samples.is_empty() {
        return [0.0, 0.0, 0.0, 1.0];
    }

    let first = [samples[0].x, samples[0].y, samples[0].z, samples[0].w];

    // Sign-flip to same hemisphere + compute val
    let mut flipped: Vec<([f32; 4], f32)> = samples
        .iter()
        .map(|s| {
            let mut q = [s.x, s.y, s.z, s.w];
            if quat_dot(&q, &first) < 0.0 {
                q = [-q[0], -q[1], -q[2], -q[3]];
            }
            let val = q[0] * q[0] + q[1] * q[1] + q[2] * q[2];
            (q, val)
        })
        .collect();

    // Sort by val
    flipped.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // Discard bottom 10% and top 10%
    let n = flipped.len();
    let lo = n / 10;
    let hi = n - n / 10;
    let trimmed = &flipped[lo..hi];

    if trimmed.is_empty() {
        return [0.0, 0.0, 0.0, 1.0];
    }

    // Average component-wise
    let mut sum = [0.0f64; 4];
    for (q, _) in trimmed {
        sum[0] += q[0] as f64;
        sum[1] += q[1] as f64;
        sum[2] += q[2] as f64;
        sum[3] += q[3] as f64;
    }
    let count = trimmed.len() as f64;
    let avg = [
        (sum[0] / count) as f32,
        (sum[1] / count) as f32,
        (sum[2] / count) as f32,
        (sum[3] / count) as f32,
    ];

    quat_normalize(&avg)
}

// ========== Orientation estimation ==========

fn dot3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Snap a vector to the nearest axis direction and return the corresponding color.
fn snap_to_axis(v: &[f32; 3]) -> Color {
    let mut best_color = AXIS_COLORS[0].1;
    let mut best_dot = f32::NEG_INFINITY;
    for &(axis, color) in &AXIS_COLORS {
        let d = dot3(v, &axis);
        if d > best_dot {
            best_dot = d;
            best_color = color;
        }
    }
    best_color
}

/// Estimate (top_color, front_color) from a relative quaternion.
fn estimate_orientation(rel: &[f32; 4]) -> (Color, Color) {
    let up = quat_rotate_vec(rel, &[0.0, 1.0, 0.0]);
    let front = quat_rotate_vec(rel, &[0.0, 0.0, 1.0]);
    (snap_to_axis(&up), snap_to_axis(&front))
}

fn orientation_label(top: Color, front: Color) -> String {
    format!("{}/{}", top.label(), front.label())
}

// ========== Move remapping ==========

/// Remap a raw body-frame move to home-frame notation given current orientation.
///
/// The cube hardware reports moves in body frame (U/D/R/L/F/B relative to
/// the physical cube faces). When the cube is rotated from home position,
/// these body-frame moves correspond to different home-frame layers.
///
/// Direction is always preserved (no flip needed) because the standard face
/// convention (CW viewed from outside) maps directly: body face CW from body
/// axis direction = home face CW from home axis direction.
fn remap_move(notation: &str, top: Color, front: Color) -> String {
    let (face_str, suffix) = parse_notation(notation);
    let face_map = build_face_map(top, front);

    if let Some(&home_face) = face_map.get(face_str) {
        format!("{}{}", home_face, suffix)
    } else {
        // Unknown face (e.g., M, S, E, x, y, z) -- pass through
        notation.to_string()
    }
}

fn parse_notation(notation: &str) -> (&str, &str) {
    if notation.ends_with('2') {
        (&notation[..notation.len() - 1], "2")
    } else if notation.ends_with('\'') {
        (&notation[..notation.len() - 1], "'")
    } else {
        (notation, "")
    }
}

/// For a given (top_color, front_color) orientation, map each body-frame face
/// to a home face name. Direction is always preserved (no flip).
fn build_face_map(top: Color, front: Color) -> std::collections::HashMap<&'static str, &'static str> {
    let mut map = std::collections::HashMap::new();

    fn color_to_home_face(c: Color) -> &'static str {
        match c {
            Color::White => "U",
            Color::Yellow => "D",
            Color::Green => "F",
            Color::Blue => "B",
            Color::Red => "R",
            Color::Orange => "L",
        }
    }

    fn opposite_color(c: Color) -> Color {
        match c {
            Color::White => Color::Yellow,
            Color::Yellow => Color::White,
            Color::Green => Color::Blue,
            Color::Blue => Color::Green,
            Color::Red => Color::Orange,
            Color::Orange => Color::Red,
        }
    }

    fn color_axis(c: Color) -> [f32; 3] {
        match c {
            Color::White => [0.0, 1.0, 0.0],
            Color::Yellow => [0.0, -1.0, 0.0],
            Color::Green => [0.0, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, -1.0],
            Color::Red => [1.0, 0.0, 0.0],
            Color::Orange => [-1.0, 0.0, 0.0],
        }
    }

    let up_dir = color_axis(top);
    let front_dir = color_axis(front);
    // right = cross(up, front): in right-hand coords cross(+Y, +Z) = +X
    let right_dir = [
        up_dir[1] * front_dir[2] - up_dir[2] * front_dir[1],
        up_dir[2] * front_dir[0] - up_dir[0] * front_dir[2],
        up_dir[0] * front_dir[1] - up_dir[1] * front_dir[0],
    ];

    let right_color = snap_to_axis(&right_dir);
    let bottom = opposite_color(top);
    let back = opposite_color(front);
    let left = opposite_color(right_color);

    map.insert("U", color_to_home_face(top));
    map.insert("D", color_to_home_face(bottom));
    map.insert("F", color_to_home_face(front));
    map.insert("B", color_to_home_face(back));
    map.insert("R", color_to_home_face(right_color));
    map.insert("L", color_to_home_face(left));

    map
}

// ========== Main analysis entry point ==========

/// Analyze a solve from raw telemetry data, printing debug output.
///
/// This is the debug-first version -- it prints everything to stdout via `println!`
/// and returns nothing structured yet.
pub fn analyze_solve(telemetry: &SolveTelemetry) {
    let duration = telemetry.solve_end_t - telemetry.solve_start_t;

    println!("=== SOLVE ANALYSIS (debug) ===");
    println!(
        "Scramble: {}",
        if telemetry.scramble.is_empty() { "(not recorded)" } else { &telemetry.scramble }
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
        home[0], home[1], home[2], home[3],
        orientation_label(home_top, home_front)
    );
    println!();

    // Combine scramble + solve gyro for lookups (moves may occur slightly before solve_start_t)
    let mut all_gyro: Vec<&GyroSample> = Vec::with_capacity(
        telemetry.scramble_gyro.len() + telemetry.solve_gyro.len(),
    );
    for s in &telemetry.scramble_gyro {
        all_gyro.push(s);
    }
    for s in &telemetry.solve_gyro {
        all_gyro.push(s);
    }
    // Should already be sorted by time, but ensure it
    all_gyro.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));

    let combined_gyro: Vec<GyroSample> = all_gyro.iter().map(|s| (*s).clone()).collect();

    for (i, raw_move) in telemetry.solve_moves.iter().enumerate() {
        let rel_t = raw_move.t - telemetry.solve_start_t;

        let before = find_gyro_before(&combined_gyro, raw_move.t);
        let after = find_gyro_after(&combined_gyro, raw_move.t);

        print!(
            "[move #{:>3}] {:>3} at {:+.2}s (t={:.3})",
            i + 1,
            raw_move.n,
            rel_t,
            raw_move.t,
        );

        if let Some(b) = before {
            let bq = [b.x, b.y, b.z, b.w];
            let rel_b = relative_quaternion(&home, &bq);
            let (top_b, front_b) = estimate_orientation(&rel_b);
            let dt_before = raw_move.t - b.t;
            print!("  |  before: ({:.3},{:.3},{:.3},{:.3}) t-{:.3}s -> {}",
                b.x, b.y, b.z, b.w, dt_before, orientation_label(top_b, front_b));

            // Use 'before' orientation for remapping
            let remapped = remap_move(&raw_move.n, top_b, front_b);
            if let Some(a) = after {
                let aq = [a.x, a.y, a.z, a.w];
                let rel_a = relative_quaternion(&home, &aq);
                let (top_a, front_a) = estimate_orientation(&rel_a);
                let dt_after = a.t - raw_move.t;
                print!("  |  after: ({:.3},{:.3},{:.3},{:.3}) t+{:.3}s -> {}",
                    a.x, a.y, a.z, a.w, dt_after, orientation_label(top_a, front_a));
            }

            if remapped != raw_move.n {
                println!("  |  remap: {} -> {}", raw_move.n, remapped);
            } else {
                println!("  |  remap: {} (identity)", raw_move.n);
            }
        } else {
            println!("  |  (no gyro before)");
        }
    }

    println!();
    println!("=== END ANALYSIS ===");
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quat_conjugate() {
        let q = [0.1, 0.2, 0.3, 0.9];
        let c = quat_conjugate(&q);
        assert_eq!(c, [-0.1, -0.2, -0.3, 0.9]);
    }

    #[test]
    fn test_quat_multiply_identity() {
        let id = [0.0, 0.0, 0.0, 1.0];
        let q = [0.1, 0.2, 0.3, 0.9];
        let result = quat_multiply(&id, &q);
        for i in 0..4 {
            assert!((result[i] - q[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_relative_quaternion_identity() {
        let home = quat_normalize(&[0.01, -0.03, -0.02, 0.99]);
        let rel = relative_quaternion(&home, &home);
        // Should be approximately identity
        assert!((rel[3] - 1.0).abs() < 0.01, "w should be ~1.0, got {}", rel[3]);
        assert!(rel[0].abs() < 0.01);
        assert!(rel[1].abs() < 0.01);
        assert!(rel[2].abs() < 0.01);
    }

    #[test]
    fn test_estimate_orientation_identity() {
        let id = [0.0, 0.0, 0.0, 1.0];
        let (top, front) = estimate_orientation(&id);
        assert_eq!(top, Color::White);
        assert_eq!(front, Color::Green);
    }

    #[test]
    fn test_remap_identity() {
        // W/G = home -> all moves pass through unchanged
        assert_eq!(remap_move("U", Color::White, Color::Green), "U");
        assert_eq!(remap_move("U'", Color::White, Color::Green), "U'");
        assert_eq!(remap_move("R", Color::White, Color::Green), "R");
        assert_eq!(remap_move("F2", Color::White, Color::Green), "F2");
    }

    #[test]
    fn test_find_gyro_before() {
        let samples = vec![
            GyroSample { t: 1.0, x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            GyroSample { t: 2.0, x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            GyroSample { t: 3.0, x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
        ];
        assert_eq!(find_gyro_before(&samples, 2.5).unwrap().t, 2.0);
        assert_eq!(find_gyro_before(&samples, 2.0).unwrap().t, 2.0);
        assert!(find_gyro_before(&samples, 0.5).is_none());
    }

    #[test]
    fn test_find_gyro_after() {
        let samples = vec![
            GyroSample { t: 1.0, x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            GyroSample { t: 2.0, x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            GyroSample { t: 3.0, x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
        ];
        assert_eq!(find_gyro_after(&samples, 1.5).unwrap().t, 2.0);
        assert_eq!(find_gyro_after(&samples, 2.0).unwrap().t, 2.0);
        assert!(find_gyro_after(&samples, 3.5).is_none());
    }

    #[test]
    fn test_orientation_label() {
        assert_eq!(orientation_label(Color::White, Color::Green), "W/G");
        assert_eq!(orientation_label(Color::Yellow, Color::Red), "Y/R");
    }

    #[test]
    fn test_parse_notation() {
        assert_eq!(parse_notation("U"), ("U", ""));
        assert_eq!(parse_notation("U'"), ("U", "'"));
        assert_eq!(parse_notation("R2"), ("R", "2"));
        assert_eq!(parse_notation("F"), ("F", ""));
    }

    #[test]
    fn test_compute_home_basic() {
        // All samples near identity -> home should be near identity
        let samples: Vec<GyroSample> = (0..20)
            .map(|i| GyroSample {
                t: i as f64,
                x: 0.01,
                y: 0.02,
                z: -0.01,
                w: 0.999,
            })
            .collect();
        let home = compute_home(&samples);
        assert!(home[3] > 0.99, "w should be near 1.0, got {}", home[3]);
    }

    #[test]
    fn test_remap_y_rotation() {
        // After y rotation (CW from top): White top, Red front (W/R)
        // Body +Z (F) now points toward Red (+X in home) -> body F = home R
        // Body +X (R) now points toward Blue (-Z in home) -> body R = home B
        // Direction is always preserved (no flip).
        let remapped = remap_move("F", Color::White, Color::Red);
        assert_eq!(remapped, "R", "Body F in W/R should map to home R");

        let remapped = remap_move("R", Color::White, Color::Red);
        assert_eq!(remapped, "B", "Body R in W/R should map to home B");
    }

    #[test]
    fn test_remap_x2_rotation() {
        // After x2: Yellow top, Blue front (Y/B)
        // Body U -> home D, Body F -> home B
        let remapped = remap_move("U", Color::Yellow, Color::Blue);
        assert_eq!(remapped, "D", "Body U in Y/B should map to home D");

        let remapped = remap_move("F", Color::Yellow, Color::Blue);
        assert_eq!(remapped, "B", "Body F in Y/B should map to home B");

        // R stays R (x2 doesn't change left/right axis)
        let remapped = remap_move("R", Color::Yellow, Color::Blue);
        assert_eq!(remapped, "R", "Body R in Y/B should map to home R");
    }

    #[test]
    fn test_remap_z_prime_rotation() {
        // After z' (CCW from front): Red top, Green front (R/G)
        // Body U -> home R, Body D -> home L
        let remapped = remap_move("U", Color::Red, Color::Green);
        assert_eq!(remapped, "R", "Body U in R/G should map to home R");

        let remapped = remap_move("D", Color::Red, Color::Green);
        assert_eq!(remapped, "L", "Body D in R/G should map to home L");

        // Front stays front
        let remapped = remap_move("F", Color::Red, Color::Green);
        assert_eq!(remapped, "F", "Body F in R/G should stay home F");
    }
}
