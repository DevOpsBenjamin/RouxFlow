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
fn slice_name(f1: &str, d1: i8, f2: &str, d2: i8) -> String {
    let (letter, ref_face) = match (f1, f2) {
        ("R", "L") | ("L", "R") => ("M", "L"),
        ("U", "D") | ("D", "U") => ("E", "D"),
        ("F", "B") | ("B", "F") => ("S", "F"),
        _ => return format!("?({}{}/{}{})", f1, if d1 < 0 { "'" } else { "" },
                            f2, if d2 < 0 { "'" } else { "" }),
    };
    // Negate: BLE reports inverse of core motion
    let ble_dir = if f1 == ref_face { d1 } else { d2 };
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
/// Used for orientation bookkeeping (merged_orient tracking), NOT for remap.
/// BitCube M: centers cycle U→F→D→B (same direction as x').
/// BitCube M': centers cycle U→B→D→F (same direction as x).
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

    // A consecutive run of the same orientation in gyro data.
    struct GyroRun {
        label: String,
        count: usize,
        t_start: f64, // timestamp of first sample in this run
    }

    // Collect consecutive runs of same-orientation samples in a time window.
    fn collect_orient_runs(
        gyro: &[GyroSample],
        home: &[f32; 4],
        t_start: f64,
        t_end: f64,
    ) -> Vec<GyroRun> {
        let mut runs: Vec<GyroRun> = Vec::new();
        let start_idx = gyro.partition_point(|s| s.t <= t_start);
        for s in &gyro[start_idx..] {
            if s.t >= t_end {
                break;
            }
            let q = [s.x, s.y, s.z, s.w];
            let rel = relative_quaternion(home, &q);
            let (top, front) = estimate_orientation(&rel);
            let label = orientation_label(top, front);
            if let Some(last) = runs.last_mut() {
                if last.label == label {
                    last.count += 1;
                    continue;
                }
            }
            runs.push(GyroRun {
                label,
                count: 1,
                t_start: s.t,
            });
        }
        runs
    }

    // Flag noise: a run with count <= noise_max surrounded by different orientations.
    // prev_ctx / next_ctx provide the adjacent window's boundary label so that
    // a single sample at a window edge isn't falsely flagged when it matches the neighbor window.
    fn is_noise(
        runs: &[GyroRun],
        idx: usize,
        noise_max: usize,
        prev_ctx: Option<&str>,
        next_ctx: Option<&str>,
    ) -> bool {
        if runs[idx].count > noise_max {
            return false;
        }
        let prev = if idx > 0 {
            Some(runs[idx - 1].label.as_str())
        } else {
            prev_ctx // use adjacent window's last label
        };
        let next = if idx + 1 < runs.len() {
            Some(runs[idx + 1].label.as_str())
        } else {
            next_ctx // use adjacent window's first label
        };
        // Noise if it differs from both neighbors
        match (prev, next) {
            (Some(p), Some(n)) => runs[idx].label != p && runs[idx].label != n,
            (Some(p), None) => runs[idx].label != p,
            (None, Some(n)) => runs[idx].label != n,
            (None, None) => false,
        }
    }

    // Get boundary labels for a window (first non-empty label, last non-empty label).
    fn window_boundary_labels(runs: &[GyroRun]) -> (Option<String>, Option<String>) {
        let first = runs.first().map(|r| r.label.clone());
        let last = runs.last().map(|r| r.label.clone());
        (first, last)
    }

    // Get the effective orientation of a window (ignoring noise runs, using last stable run).
    fn window_effective_orient(
        runs: &[GyroRun],
        prev_ctx: Option<&str>,
        next_ctx: Option<&str>,
    ) -> String {
        // Walk backwards to find last non-noise run
        for i in (0..runs.len()).rev() {
            if !is_noise(runs, i, 1, prev_ctx, next_ctx) {
                return runs[i].label.clone();
            }
        }
        // Fallback: first run
        runs.first()
            .map(|r| r.label.clone())
            .unwrap_or_else(|| "?/?".to_string())
    }

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
                            if let (Some(from), Some(to)) = (
                                parse_orient_label(prev),
                                parse_orient_label(&run.label),
                            ) {
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
            let boundary_is_slice = w >= 1
                && w - 1 < p1.len()
                && p1[w - 1].body_raw.len() == 2;
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

    // ========== Roux step detection ==========
    // Check if a set of sticker positions are all solved (match their face center).
    fn is_block_solved(cube: &BitCube, stickers: &[usize]) -> bool {
        stickers.iter().all(|&idx| {
            let face_center = (idx / 9) * 9 + 4;
            cube.get_color_at(idx) == cube.get_color_at(face_center)
        })
    }

    // 1x2x3 blocks = 5 pieces (2 corners + 3 edges) = 12 sticker positions each.
    // Face offsets: U=0, R=9, F=18, D=27, L=36, B=45

    // D-layer blocks (D on bottom)
    const DL_BLOCK: [usize; 12] = [
        27, 30, 33, 42, 43, 44, 24, 21, 53, 50, 41, 39,
    ];
    const DR_BLOCK: [usize; 12] = [
        29, 32, 35, 15, 16, 17, 26, 23, 51, 48, 12, 14,
    ];
    const DF_BLOCK: [usize; 12] = [
        27, 28, 29, 24, 25, 26, 44, 41, 15, 12, 21, 23,
    ];
    const DB_BLOCK: [usize; 12] = [
        33, 34, 35, 51, 52, 53, 42, 39, 17, 14, 50, 48,
    ];

    // U-layer blocks (U on bottom — user has cube flipped)
    // UL: UFL+UBL corners, UL+FL+BL edges
    const UL_BLOCK: [usize; 12] = [
        6, 18, 38, 0, 47, 36, 3, 37, 21, 41, 50, 39,
    ];
    // UR: UFR+UBR corners, UR+FR+BR edges
    const UR_BLOCK: [usize; 12] = [
        8, 20, 9, 2, 45, 11, 5, 10, 23, 12, 48, 14,
    ];
    // UF: UFL+UFR corners, UF+FL+FR edges
    const UF_BLOCK: [usize; 12] = [
        6, 18, 38, 8, 20, 9, 7, 19, 21, 41, 23, 12,
    ];
    // UB: UBL+UBR corners, UB+BL+BR edges
    const UB_BLOCK: [usize; 12] = [
        0, 47, 36, 2, 45, 11, 1, 46, 50, 39, 48, 14,
    ];

    // D-layer corners (for CMLL when D on bottom)
    const D_CORNERS: [usize; 12] = [
        27, 24, 44,   // DFL: D, F, L
        29, 26, 15,   // DFR: D, F, R
        33, 53, 42,   // DBL: D, B, L
        35, 51, 17,   // DBR: D, B, R
    ];
    // U-layer corners (for CMLL when U on bottom)
    const U_CORNERS: [usize; 12] = [
        6, 18, 38,    // UFL: U, F, L
        8, 20, 9,     // UFR: U, F, R
        0, 47, 36,    // UBL: U, B, L
        2, 45, 11,    // UBR: U, B, R
    ];

    const ALL_BLOCKS: [(&[usize; 12], &str, &[usize; 12], &str, &[usize; 12]); 8] = [
        // (fb_block, fb_name, sb_block, sb_name, cmll_corners)
        (&DL_BLOCK, "DL", &DR_BLOCK, "DR", &U_CORNERS),
        (&DR_BLOCK, "DR", &DL_BLOCK, "DL", &U_CORNERS),
        (&DF_BLOCK, "DF", &DB_BLOCK, "DB", &U_CORNERS),
        (&DB_BLOCK, "DB", &DF_BLOCK, "DF", &U_CORNERS),
        (&UL_BLOCK, "UL", &UR_BLOCK, "UR", &D_CORNERS),
        (&UR_BLOCK, "UR", &UL_BLOCK, "UL", &D_CORNERS),
        (&UF_BLOCK, "UF", &UB_BLOCK, "UB", &D_CORNERS),
        (&UB_BLOCK, "UB", &UF_BLOCK, "UF", &D_CORNERS),
    ];

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

    // Track Roux step completion
    // Store: (move_idx, fb_name, sb_block, sb_name, cmll_corners)
    let mut fb_done: Option<(usize, &str, &[usize; 12], &str, &[usize; 12])> = None;
    let mut sb_done: Option<usize> = None;
    let mut cmll_done: Option<usize> = None;

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
            let slice_move = m.body_label.split_whitespace().next().unwrap_or(&m.body_label);
            cube_detect.apply_move(slice_move);
        } else {
            cube_detect.apply_move(&m.body_raw[0]);
        }

        let rel_t = m.t - solve_start;
        let is_slice = m.body_raw.len() == 2;
        let move_marker = if is_slice {
            format!("{}S{}", YELLOW, RESET)
        } else {
            " ".to_string()
        };

        // Check Roux steps on detection cube
        let mut step_marker = String::new();

        // Flag any move where a 1x2x3 block is detected in the verified body state
        if cube_body.is_fb_block() {
            println!(" {}>> MY METHOD DETECTED A BLOCK (Move {})!{}", GREEN, idx + 1, RESET);
        }

        if fb_done.is_none() {
            for &(fb_block, fb_name, sb_block, sb_name, cmll_corners) in &ALL_BLOCKS {
                if is_block_solved(&cube_detect, fb_block.as_slice()) {
                    fb_done = Some((idx + 1, fb_name, sb_block, sb_name, cmll_corners));
                    step_marker = format!(
                        " {}>> FB DONE [{}]{}", GREEN, fb_name, RESET
                    );
                    break;
                }
            }
        } else if sb_done.is_none() {
            let (_, _, sb_block, sb_name, _) = fb_done.unwrap();
            if is_block_solved(&cube_detect, sb_block.as_slice()) {
                sb_done = Some(idx + 1);
                step_marker = format!(
                    " {}>> SB DONE [{}]{}", GREEN, sb_name, RESET
                );
            }
        }

        if sb_done.is_some() && cmll_done.is_none() {
            let (_, _, _, _, cmll_corners) = fb_done.unwrap();
            if is_block_solved(&cube_detect, cmll_corners.as_slice()) {
                cmll_done = Some(idx + 1);
                step_marker = format!(
                    " {}>> CMLL DONE{}", GREEN, RESET
                );
            }
        }

        let solved_marker = if cube_detect.is_solved() {
            format!(" {}>> SOLVED{}", GREEN, RESET)
        } else {
            String::new()
        };

        println!(
            "{:>4}  MOVE | {:<20} {:+7.2}s {}{}{}",
            idx + 1,
            m.body_label,
            rel_t,
            move_marker,
            step_marker,
            solved_marker,
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
        println!(
            "Body cube solved: {}",
            cube_body.is_solved()
        );
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
        let effective =
            window_effective_orient(runs, pc.as_deref(), nc.as_deref());

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
                    found = window_effective_orient(
                        fwd_runs,
                        fpc.as_deref(),
                        fnc.as_deref(),
                    ) == effective;
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
