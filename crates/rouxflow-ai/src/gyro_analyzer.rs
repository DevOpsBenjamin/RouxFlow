use rouxflow_bitboard::BitCube;
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

// ========== Color helpers (module-level) ==========

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

fn compute_right_color(top: Color, front: Color) -> Color {
    let up_dir = color_axis(top);
    let front_dir = color_axis(front);
    let right_dir = [
        up_dir[1] * front_dir[2] - up_dir[2] * front_dir[1],
        up_dir[2] * front_dir[0] - up_dir[0] * front_dir[2],
        up_dir[0] * front_dir[1] - up_dir[1] * front_dir[0],
    ];
    snap_to_axis(&right_dir)
}

fn char_to_color(c: char) -> Option<Color> {
    match c {
        'W' => Some(Color::White),
        'Y' => Some(Color::Yellow),
        'G' => Some(Color::Green),
        'B' => Some(Color::Blue),
        'R' => Some(Color::Red),
        'O' => Some(Color::Orange),
        _ => None,
    }
}

fn parse_orient_label(s: &str) -> Option<(Color, Color)> {
    let mut chars = s.chars();
    let top = char_to_color(chars.next()?)?;
    if chars.next() != Some('/') { return None; }
    let front = char_to_color(chars.next()?)?;
    Some((top, front))
}

// ========== Rotation detection ==========

/// Apply a cube rotation to an orientation, returning the new (top, front).
/// x = rotate like R (CW from right side).
/// y = rotate like U (CW from top).
/// z = rotate like F (CW from front).
fn apply_rotation(top: Color, front: Color, rot: &str) -> (Color, Color) {
    let right = compute_right_color(top, front);
    let bottom = opposite_color(top);
    let back = opposite_color(front);
    let left = opposite_color(right);

    match rot {
        "x"  => (front, bottom),
        "x'" => (back, top),
        "x2" => (bottom, back),
        "y"  => (top, right),
        "y'" => (top, left),
        "y2" => (top, back),
        "z"  => (left, front),
        "z'" => (right, front),
        "z2" => (bottom, front),
        _ => (top, front),
    }
}

/// Detect which rotation transforms one orientation into another.
/// Tries single rotations first, then pairs if needed.
fn detect_rotation(from: (Color, Color), to: (Color, Color)) -> String {
    if from == to {
        return String::new();
    }

    const ROTS: [&str; 9] = ["x", "x'", "x2", "y", "y'", "y2", "z", "z'", "z2"];

    // Try single rotation
    for rot in &ROTS {
        if apply_rotation(from.0, from.1, rot) == to {
            return rot.to_string();
        }
    }

    // Try pair of rotations
    for r1 in &ROTS {
        let mid = apply_rotation(from.0, from.1, r1);
        for r2 in &ROTS {
            if apply_rotation(mid.0, mid.1, r2) == to {
                return format!("{} {}", r1, r2);
            }
        }
    }

    "?".to_string()
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

    let right_color = compute_right_color(top, front);
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

// ========== Slice detection ==========

/// Parse notation into (face_letter, direction). E.g. "F'" -> ("F", -1), "B" -> ("B", 1).
fn parse_face_dir(notation: &str) -> Option<(&str, i8)> {
    let (face, suffix) = parse_notation(notation);
    let dir = match suffix {
        "" => 1,
        "'" => -1,
        "2" => 2,
        _ => return None,
    };
    match face {
        "U" | "D" | "R" | "L" | "F" | "B" => Some((face, dir)),
        _ => None,
    }
}

/// Check if two faces are opposite.
fn are_opposite_faces(f1: &str, f2: &str) -> bool {
    matches!(
        (f1, f2),
        ("U", "D") | ("D", "U") | ("R", "L") | ("L", "R") | ("F", "B") | ("B", "F")
    )
}

/// Name a slice from two BLE-reported face moves.
/// BLE reports the APPARENT motion of outer faces (inverse of actual core motion).
/// So we NEGATE the direction: BLE F'(-1) + B(+1) → core went F direction → S (positive).
/// M follows L, E follows D, S follows F.
fn slice_name(f1: &str, d1: i8, f2: &str, _d2: i8) -> String {
    let (letter, ref_face) = match (f1, f2) {
        ("R", "L") | ("L", "R") => ("M", "L"),
        ("U", "D") | ("D", "U") => ("E", "D"),
        ("F", "B") | ("B", "F") => ("S", "F"),
        _ => return format!("?({}{}/{}{})", f1, if d1 < 0 { "'" } else { "" },
                            f2, if _d2 < 0 { "'" } else { "" }),
    };
    // Negate: BLE reports inverse of core motion
    let ble_dir = if f1 == ref_face { d1 } else { _d2 };
    let dir = -ble_dir;
    let suffix = if dir > 0 { "" } else { "'" };
    format!("{}{}", letter, suffix)
}

/// Check if two consecutive raw moves form a slice pair.
fn is_slice_pair(m1: &rouxflow_core::telemetry::RawMove, m2: &rouxflow_core::telemetry::RawMove) -> bool {
    // Same timestamp (BLE reports them together, up to ~2-3ms jitter)
    if (m1.t - m2.t).abs() > 0.005 {
        return false;
    }
    let Some((f1, d1)) = parse_face_dir(&m1.n) else { return false };
    let Some((f2, d2)) = parse_face_dir(&m2.n) else { return false };
    are_opposite_faces(f1, f2) && d1 == -d2
}

/// Remap a slice: remap both constituent faces, then name the home-frame slice.
fn remap_slice(n1: &str, n2: &str, top: Color, front: Color) -> String {
    let remapped1 = remap_move(n1, top, front);
    let remapped2 = remap_move(n2, top, front);
    let Some((rf1, rd1)) = parse_face_dir(&remapped1) else { return format!("{}+{}", remapped1, remapped2) };
    let Some((rf2, rd2)) = parse_face_dir(&remapped2) else { return format!("{}+{}", remapped1, remapped2) };
    if are_opposite_faces(rf1, rf2) && rd1 == -rd2 {
        slice_name(rf1, rd1, rf2, rd2)
    } else {
        format!("{}+{}", remapped1, remapped2)
    }
}

// ========== Intermediate representation ==========

/// A single analyzed move after slice detection + orientation remap.
struct AnalyzedMove {
    body_label: String,
    remapped: String,
    /// Original body-frame raw move notations (for BLE cube).
    /// Face move: `vec!["L"]`, Slice: `vec!["F'", "B"]`, Double: `vec!["L", "L"]`
    body_raw: Vec<String>,
    t: f64,
    rel_t: f64,
    orient: String,
    before_dt: f64,
    after_orient: Option<String>,
    after_dt: Option<f64>,
}

/// Strip direction suffix, returning the base face/slice name.
/// "M'" -> "M", "U" -> "U", "S'" -> "S", "R2" -> "R"
fn strip_suffix(notation: &str) -> &str {
    if notation.ends_with('\'') || notation.ends_with('2') {
        &notation[..notation.len() - 1]
    } else {
        notation
    }
}

/// Two consecutive same-direction moves merge into a double.
/// "U" + "U" -> "U2", "M'" + "M'" -> "M2", "R" + "R" -> "R2".
/// Half-turns are direction-agnostic so both CW+CW and CCW+CCW give X2.
fn can_merge_double(a: &str, b: &str) -> bool {
    a == b
}

/// Convert "M'" or "M" into "M2".
fn to_double(notation: &str) -> String {
    format!("{}2", strip_suffix(notation))
}

/// Mathematical core rotation caused by a home-frame slice move.
/// BitCube convention: M = U→F→D→B (x direction, opposite of standard).
/// So BitCube "M" = standard M' (follows R), BitCube "M'" = standard M (follows L).
/// Core rotation is what the BitCube move actually does:
///   BitCube M (=std M') → core x,  BitCube M' (=std M) → core x'
fn slice_core_rotation(notation: &str) -> Option<&'static str> {
    match notation {
        "M"  => Some("x'"),  // M centers: U→F (same as x')
        "M'" => Some("x"),   // M' centers: U→B (same as x)
        "M2" => Some("x2"),
        "S"  => Some("z"),
        "S'" => Some("z'"),
        "S2" => Some("z2"),
        "E"  => Some("y'"),
        "E'" => Some("y"),
        "E2" => Some("y2"),
        _ => None,
    }
}

// ========== Side-by-side cube display ==========

fn colored_sticker(cube: &BitCube, bit_idx: usize) -> String {
    const RESET: &str = "\x1b[0m";
    const COLORS: [(&str, char); 6] = [
        ("\x1b[97;1m", 'W'),       // White (bright bold)
        ("\x1b[93;1m", 'Y'),       // Yellow (bright bold)
        ("\x1b[32;1m", 'G'),       // Green (bold)
        ("\x1b[34;1m", 'B'),       // Blue (bold)
        ("\x1b[31;1m", 'R'),       // Red (bold)
        ("\x1b[38;5;208m", 'O'),   // Orange (256-color)
    ];
    let c = cube.get_color_at(bit_idx);
    format!("{}{}{}", COLORS[c].0, COLORS[c].1, RESET)
}

fn cube_face_row(cube: &BitCube, face_offset: usize, row: usize) -> String {
    let mut s = String::new();
    for col in 0..3 {
        s.push_str(&colored_sticker(cube, face_offset + row * 3 + col));
        s.push(' ');
    }
    s
}

/// Render a cube to 9 lines (with ANSI colors). Each line has the same visual width (24 chars).
fn cube_to_lines(cube: &BitCube) -> Vec<String> {
    let pad = "      "; // 6 spaces = L face width (3 stickers × 2 chars)
    let trail = "            "; // 12 spaces (pads U/D rows to 24 visual width)
    let mut lines = Vec::with_capacity(9);

    // U face (rows 0-2)
    for row in 0..3 {
        lines.push(format!("{}{}{}", pad, cube_face_row(cube, 0, row), trail));
    }
    // Middle band: L F R B (rows 0-2)
    for row in 0..3 {
        lines.push(format!(
            "{}{}{}{}",
            cube_face_row(cube, 36, row), // L
            cube_face_row(cube, 18, row), // F
            cube_face_row(cube, 9, row),  // R
            cube_face_row(cube, 45, row), // B
        ));
    }
    // D face (rows 0-2)
    for row in 0..3 {
        lines.push(format!("{}{}{}", pad, cube_face_row(cube, 27, row), trail));
    }
    lines
}

fn print_cubes_side_by_side(cubes: &[(&BitCube, &str)]) {
    let all_lines: Vec<Vec<String>> = cubes.iter().map(|(c, _)| cube_to_lines(c)).collect();

    // Header
    let header: Vec<String> = cubes.iter().map(|(_, label)| format!("{:^24}", label)).collect();
    println!("{}", header.join("  |  "));

    // Rows
    for row in 0..9 {
        let parts: Vec<&str> = all_lines.iter().map(|lines| lines[row].as_str()).collect();
        println!("{}", parts.join("  |  "));
    }
    println!();
}

// ========== Main analysis entry point ==========

/// Analyze a solve from raw telemetry data, printing debug output.
///
/// This is the debug-first version -- it prints everything to stdout via `println!`
/// and returns nothing structured yet.
pub fn analyze_solve(telemetry: &SolveTelemetry, idx_print: usize) {
    let t_start = std::time::Instant::now();
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

    // Combine scramble + solve gyro for lookups
    let mut all_gyro: Vec<&GyroSample> = Vec::with_capacity(
        telemetry.scramble_gyro.len() + telemetry.solve_gyro.len(),
    );
    for s in &telemetry.scramble_gyro {
        all_gyro.push(s);
    }
    for s in &telemetry.solve_gyro {
        all_gyro.push(s);
    }
    all_gyro.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    let combined_gyro: Vec<GyroSample> = all_gyro.iter().map(|s| (*s).clone()).collect();

    // -- Pass 1: slice detection + orientation remap --
    // Uses MATHEMATICAL orientation tracking for remap (not raw gyro).
    // Gyro is still read for diagnostics and rotation detection between moves.
    let raw_moves: Vec<_> = telemetry.solve_moves.iter().collect();
    let mut analyzed: Vec<AnalyzedMove> = Vec::new();

    // Detect initial orientation from first move's gyro (reliable — during stable hold)
    let initial_orient = if let Some(first) = raw_moves.first() {
        if let Some(b) = find_gyro_before(&combined_gyro, first.t) {
            let bq = [b.x, b.y, b.z, b.w];
            let rel_b = relative_quaternion(&home, &bq);
            estimate_orientation(&rel_b)
        } else {
            (home_top, home_front)
        }
    } else {
        (home_top, home_front)
    };
    let mut math_orient = initial_orient;
    let mut expected_orient_p1: (Color, Color) = initial_orient;

    let mut i = 0;
    while i < raw_moves.len() {
        let t = raw_moves[i].t;
        let rel_t = t - telemetry.solve_start_t;
        let before = find_gyro_before(&combined_gyro, t);
        let after = find_gyro_after(&combined_gyro, t);

        // Gyro orientation (for diagnostics + rotation detection)
        let (gyro_orient_label, before_dt, gyro_top, gyro_front) = if let Some(b) = before {
            let bq = [b.x, b.y, b.z, b.w];
            let rel_b = relative_quaternion(&home, &bq);
            let (tb, fb) = estimate_orientation(&rel_b);
            (orientation_label(tb, fb), t - b.t, tb, fb)
        } else {
            ("?/?".to_string(), 0.0, home_top, home_front)
        };

        let (after_orient, after_dt) = if let Some(a) = after {
            let aq = [a.x, a.y, a.z, a.w];
            let rel_a = relative_quaternion(&home, &aq);
            let (ta, fa) = estimate_orientation(&rel_a);
            (Some(orientation_label(ta, fa)), Some(a.t - t))
        } else {
            (None, None)
        };

        // Rotation detection: if gyro says different from expected AND not transient,
        // it's a real user rotation → update math_orient
        let gyro_orient = (gyro_top, gyro_front);
        if gyro_orient != expected_orient_p1 {
            let is_transient = after.is_some() && {
                let aq = [after.unwrap().x, after.unwrap().y, after.unwrap().z, after.unwrap().w];
                let rel_a = relative_quaternion(&home, &aq);
                let (ta, fa) = estimate_orientation(&rel_a);
                (ta, fa) == expected_orient_p1
            };
            if !is_transient {
                // Real rotation detected — update math_orient to match gyro
                math_orient = gyro_orient;
            }
        }

        // Remap using math_orient (not gyro)
        let orient = orientation_label(math_orient.0, math_orient.1);

        // Check for slice pair
        if i + 1 < raw_moves.len() && is_slice_pair(&raw_moves[i], &raw_moves[i + 1]) {
            let m1 = &raw_moves[i];
            let m2 = &raw_moves[i + 1];
            let (f1, d1) = parse_face_dir(&m1.n).unwrap();
            let (f2, d2) = parse_face_dir(&m2.n).unwrap();
            let body_slice = slice_name(f1, d1, f2, d2);
            let remapped = remap_slice(&m1.n, &m2.n, math_orient.0, math_orient.1);

            // Update math_orient for slice core rotation
            if let Some(core_rot) = slice_core_rotation(&remapped) {
                math_orient = apply_rotation(math_orient.0, math_orient.1, core_rot);
            }

            analyzed.push(AnalyzedMove {
                body_label: format!("{} ({}+{})", body_slice, m1.n, m2.n),
                remapped: remapped.clone(),
                body_raw: vec![m1.n.clone(), m2.n.clone()],
                t, rel_t, orient, before_dt, after_orient, after_dt,
            });
            i += 2;
        } else {
            let raw = &raw_moves[i];
            let remapped = remap_move(&raw.n, math_orient.0, math_orient.1);

            analyzed.push(AnalyzedMove {
                body_label: raw.n.clone(),
                remapped: remapped.clone(),
                body_raw: vec![raw.n.clone()],
                t, rel_t, orient, before_dt, after_orient, after_dt,
            });
            i += 1;
        }

        // Update expected orientation for next move
        // Face moves: orient stays. Slices: already updated math_orient above.
        expected_orient_p1 = math_orient;
    }

    // -- Pass 2: merge consecutive same-remapped moves into doubles --
    let mut merged: Vec<AnalyzedMove> = Vec::new();
    let mut j = 0;
    while j < analyzed.len() {
        if j + 1 < analyzed.len() && can_merge_double(&analyzed[j].remapped, &analyzed[j + 1].remapped) {
            let a = &analyzed[j];
            let b = &analyzed[j + 1];
            let double_remapped = to_double(&a.remapped);
            let mut combined_body_raw = a.body_raw.clone();
            combined_body_raw.extend(b.body_raw.iter().cloned());
            merged.push(AnalyzedMove {
                body_label: format!("{} + {}", a.body_label, b.body_label),
                remapped: double_remapped,
                body_raw: combined_body_raw,
                t: a.t,
                rel_t: a.rel_t,
                orient: a.orient.clone(),
                before_dt: a.before_dt,
                after_orient: b.after_orient.clone(),
                after_dt: b.after_dt,
            });
            j += 2;
        } else {
            let a = &analyzed[j];
            merged.push(AnalyzedMove {
                body_label: a.body_label.clone(),
                remapped: a.remapped.clone(),
                body_raw: a.body_raw.clone(),
                t: a.t,
                rel_t: a.rel_t,
                orient: a.orient.clone(),
                before_dt: a.before_dt,
                after_orient: a.after_orient.clone(),
                after_dt: a.after_dt,
            });
            j += 1;
        }
    }

    // -- BitCube validation --
    // MERGED: scramble + rotation + remapped (home-frame) moves
    // BLE:    scramble + raw body-frame moves
    // COPY:   clone MERGED, apply rotation_to_wg() → should match BLE
    const ORANGE: &str = "\x1b[38;5;208m";
    const CYAN: &str = "\x1b[36m";
    const RED_ANSI: &str = "\x1b[31m";
    const GREEN_ANSI: &str = "\x1b[32m";
    const RESET: &str = "\x1b[0m";

    let mut cube_merged = BitCube::new_solved();
    let mut cube_ble = BitCube::new_solved();
    for token in telemetry.scramble.split_whitespace() {
        cube_merged.apply_move(token);
        cube_ble.apply_move(token);
    }

    /// Given the current orientation (top, front), return the rotation
    /// to apply to get back to W/G. Just detect_rotation(current, W/G).
    fn rotation_to_wg(top: Color, front: Color) -> String {
        detect_rotation((top, front), (Color::White, Color::Green))
    }

    /// Clone cube, apply rotation to W/G, return the copy.
    fn copy_to_wg(cube: &BitCube, top: Color, front: Color) -> BitCube {
        let mut copy = cube.clone();
        let rot = rotation_to_wg(top, front);
        for part in rot.split_whitespace() {
            copy.apply_move(part);
        }
        copy
    }

    // Detect initial rotation (e.g., W/G -> Y/R = x2 y)
    let home_label = orientation_label(home_top, home_front);
    let mut merged_orient: (Color, Color) = (home_top, home_front);

    if !merged.is_empty() {
        let first_orient = &merged[0].orient;
        if *first_orient != home_label {
            if let (Some(from), Some(to)) = (parse_orient_label(&home_label), parse_orient_label(first_orient)) {
                let rot = detect_rotation(from, to);
                if rot != "?" {
                    println!("Initial rotation: '{}' applied to MERGED ({} -> {})", rot, home_label, first_orient);
                    for rot_part in rot.split_whitespace() {
                        cube_merged.apply_move(rot_part);
                    }
                    merged_orient = to;

                    let copy = copy_to_wg(&cube_merged, merged_orient.0, merged_orient.1);
                    let matches = copy == cube_ble;
                    println!("--- After scramble + {} ---", rot);
                    print_cubes_side_by_side(&[
                        (&cube_merged, &format!("MERGED ({})", first_orient)),
                        (&copy, "COPY (→W/G)"),
                        (&cube_ble, "BLE"),
                    ]);
                    println!("COPY == BLE: {}{}{}", if matches { GREEN_ANSI } else { RED_ANSI }, matches, RESET);
                    println!();
                }
            }
        }
    }

    // Full BLE sanity check
    {
        let mut cube_ble_full = BitCube::new_solved();
        for token in telemetry.scramble.split_whitespace() {
            cube_ble_full.apply_move(token);
        }
        for m in &telemetry.solve_moves {
            cube_ble_full.apply_move(&m.n);
        }
        println!("RAW BLE full check (scramble + ALL raw moves): solved = {}", cube_ble_full.is_solved());
        println!();
    }

    // Track expected orientation — starts at merged_orient (after initial rotation), not home
    let mut expected_orient: Option<String> = Some(orientation_label(merged_orient.0, merged_orient.1));
    let mut any_mismatch = false;
    let mut post_slice = false; // Skip rotation detection on the move right after a slice

    for (idx, m) in merged.iter().enumerate() {
        let is_slice = matches!(strip_suffix(&m.remapped), "M" | "S" | "E");

        // After a slice, gyro is noisy — skip rotation detection and resync
        let skip_rotation = post_slice;
        if post_slice {
            expected_orient = Some(m.orient.clone());
            post_slice = false;
        }

        let orient_differs = !skip_rotation && expected_orient.as_ref().is_some_and(|exp| *exp != m.orient);
        let is_transient = orient_differs
            && m.after_orient.as_ref().is_some_and(|ao| {
                expected_orient.as_ref().is_some_and(|exp| ao == exp)
            });

        // Real rotation: orient changed AND didn't revert back
        if orient_differs && !is_transient {
            if let (Some(from), Some(to)) = (
                parse_orient_label(expected_orient.as_deref().unwrap_or("")),
                parse_orient_label(&m.orient),
            ) {
                let rot = detect_rotation(from, to);
                println!(
                    "{}       >>>  {}  ({} -> {}){}",
                    CYAN, rot, orientation_label(from.0, from.1), m.orient, RESET
                );
                // Apply delta rotation to MERGED to keep frames in sync
                for part in rot.split_whitespace() {
                    cube_merged.apply_move(part);
                }
                merged_orient = to;
            }
        }

        // Wide move detection
        let within_move_shift = !is_slice && m.after_orient.as_ref().is_some_and(|ao| *ao != m.orient);
        let is_wide = !is_slice && (is_transient || within_move_shift);

        // Apply remapped (home-frame) move to MERGED
        cube_merged.apply_move(&m.remapped);

        // Apply body-frame raw moves to BLE
        for raw in &m.body_raw {
            cube_ble.apply_move(raw);
        }

        // After slice: update frame tracking mathematically (NOT applied to cube —
        // M2 already moved the correct stickers, this is just orientation bookkeeping)
        if let Some(core_rot) = slice_core_rotation(&m.remapped) {
            let old_label = orientation_label(merged_orient.0, merged_orient.1);
            merged_orient = apply_rotation(merged_orient.0, merged_orient.1, core_rot);
            let new_label = orientation_label(merged_orient.0, merged_orient.1);
            println!(
                "{}       [frame]  {}  ({} -> {}){}",
                CYAN, core_rot, old_label, new_label, RESET
            );
        }

        // Make COPY = clone MERGED, apply rotation to W/G
        let copy = copy_to_wg(&cube_merged, merged_orient.0, merged_orient.1);
        let cubes_match = copy == cube_ble;
        if !cubes_match {
            any_mismatch = true;
        }

        let orient_label = orientation_label(merged_orient.0, merged_orient.1);
        if is_wide {
            print!("{}", ORANGE);
        }

        print!(
            "[move #{:>3}] {:>4} at {:+.2}s  |  {} t-{:.3}s",
            idx + 1, m.remapped, m.rel_t, m.orient, m.before_dt,
        );
        if let (Some(ao), Some(ad)) = (&m.after_orient, m.after_dt) {
            print!("  |  {} t+{:.3}s", ao, ad);
        }
        if m.remapped != m.body_label {
            print!("  |  body: {}", m.body_label);
        }
        print!("  [frame:{}]", orient_label);
        if is_wide {
            print!("  << WIDE?");
        }
        if !cubes_match {
            print!("{}  << MISMATCH{}", RED_ANSI, RESET);
        }

        if is_wide {
            println!("{}", RESET);
        } else {
            println!();
        }

        // Print cubes (idx_print .. idx_print+5)
        if idx + 1 >= idx_print && idx + 1 < idx_print + 5 {
            print_cubes_side_by_side(&[
                (&cube_merged, &format!("MERGED ({})", orient_label)),
                (&copy, "COPY (→W/G)"),
                (&cube_ble, "BLE"),
            ]);
        }

        // Update expected orientation
        if is_slice {
            // Gyro is noisy after slices — don't trust after_orient.
            // Flag to resync from the next move's before gyro instead.
            post_slice = true;
        } else if is_transient {
            // Wide move / gyro noise — keep expected unchanged
        } else {
            expected_orient = Some(m.orient.clone());
        }
    }

    // Summary
    println!();
    print!("Remapped sequence: ");
    for m in &merged {
        print!("{} ", m.remapped);
    }
    println!();

    let copy_final = copy_to_wg(&cube_merged, merged_orient.0, merged_orient.1);
    println!();
    println!("MERGED solved:      {}", cube_merged.is_solved());
    println!("BLE solved:         {}", cube_ble.is_solved());
    println!("COPY == BLE:        {}{}{}", if copy_final == cube_ble { GREEN_ANSI } else { RED_ANSI }, copy_final == cube_ble, RESET);
    println!("Any mismatch:       {}{}{}", if any_mismatch { RED_ANSI } else { GREEN_ANSI }, any_mismatch, RESET);
    if !cube_ble.is_solved() {
        println!("\nBLE final state:");
        println!("{}", cube_ble);
    }
    if copy_final != cube_ble {
        println!("\nCOPY (→W/G) final:");
        println!("{}", copy_final);
        println!("BLE final:");
        println!("{}", cube_ble);
    }
    println!();
    println!("=== END ANALYSIS ({:.2}ms) ===", t_start.elapsed().as_secs_f64() * 1000.0);
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

    #[test]
    fn test_slice_name_body_frame() {
        // BLE directions are negated to get core motion.
        // BLE L(+1) R(-1) = core M' (follows R, opposite of L)
        assert_eq!(slice_name("L", 1, "R", -1), "M'");
        // BLE R(+1) L(-1) = core M (follows L)
        assert_eq!(slice_name("R", 1, "L", -1), "M");
        // BLE F(+1) B(-1) = core S' (opposite of F)
        assert_eq!(slice_name("F", 1, "B", -1), "S'");
        // BLE F(-1) B(+1) = core S (follows F)
        assert_eq!(slice_name("F", -1, "B", 1), "S");
        assert_eq!(slice_name("B", 1, "F", -1), "S");
        // BLE D(+1) U(-1) = core E' (opposite of D)
        assert_eq!(slice_name("D", 1, "U", -1), "E'");
        // BLE U(+1) D(-1) = core E (follows D)
        assert_eq!(slice_name("U", 1, "D", -1), "E");
    }

    #[test]
    fn test_remap_slice_yr() {
        // In Y/R: BLE F'(-1)+B(+1) = core S in body frame → home M'
        // (body S follows body F = home R → slice follows R = home M')
        let home_slice = remap_slice("F'", "B", Color::Yellow, Color::Red);
        assert_eq!(home_slice, "M'", "Body core-S in Y/R should remap to home M'");

        // In Y/R: BLE B(+1)+F'(-1) same pair, still = home M'
        let home_slice = remap_slice("B", "F'", Color::Yellow, Color::Red);
        assert_eq!(home_slice, "M'", "Body B+F' in Y/R should remap to home M'");
    }

    #[test]
    fn test_remap_slice_identity() {
        // In W/G (home): BLE L(+1)+R'(-1) = core M' stays M'
        let home_slice = remap_slice("L", "R'", Color::White, Color::Green);
        assert_eq!(home_slice, "M'");
    }

    #[test]
    fn test_is_slice_pair_detection() {
        use rouxflow_core::telemetry::RawMove;
        use rouxflow_core::move_interpreter::MoveKind;

        let m1 = RawMove { n: "F'".to_string(), t: 86.101, k: MoveKind::Face };
        let m2 = RawMove { n: "B".to_string(), t: 86.101, k: MoveKind::Face };
        assert!(is_slice_pair(&m1, &m2));

        // Same direction = not a slice
        let m3 = RawMove { n: "F".to_string(), t: 86.101, k: MoveKind::Face };
        let m4 = RawMove { n: "B".to_string(), t: 86.101, k: MoveKind::Face };
        assert!(!is_slice_pair(&m3, &m4));

        // 2ms jitter = still a slice
        let m5 = RawMove { n: "F'".to_string(), t: 86.101, k: MoveKind::Face };
        let m6 = RawMove { n: "B".to_string(), t: 86.103, k: MoveKind::Face };
        assert!(is_slice_pair(&m5, &m6));

        // Different timestamps (>5ms) = not a slice
        let m7 = RawMove { n: "F'".to_string(), t: 86.101, k: MoveKind::Face };
        let m8 = RawMove { n: "B".to_string(), t: 86.500, k: MoveKind::Face };
        assert!(!is_slice_pair(&m7, &m8));
    }
}
