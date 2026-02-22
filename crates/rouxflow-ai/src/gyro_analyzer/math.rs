pub use rouxflow_core::cube::facelet::Color;
use rouxflow_core::cube::Orientation;
use rouxflow_core::telemetry::GyroSample;

// ========== Color / Orientation types ==========

pub fn color_to_char(c: Color) -> char {
    match c {
        Color::White => 'W',
        Color::Yellow => 'Y',
        Color::Green => 'G',
        Color::Blue => 'B',
        Color::Red => 'R',
        Color::Orange => 'O',
    }
}

/// The 6 axis directions and their corresponding colors.
/// Aligned with RouxFlow standard: +Y=White, -Y=Yellow, +Z=Green, -Z=Blue, +X=Red, -X=Orange.
pub const AXIS_COLORS: [([f32; 3], Color); 6] = [
    ([0.0, 1.0, 0.0], Color::White),   // +Y (Up)
    ([0.0, -1.0, 0.0], Color::Yellow), // -Y (Down)
    ([0.0, 0.0, 1.0], Color::Green),   // +Z (Front)
    ([0.0, 0.0, -1.0], Color::Blue),   // -Z (Back)
    ([1.0, 0.0, 0.0], Color::Red),     // +X (Right)
    ([-1.0, 0.0, 0.0], Color::Orange), // -X (Left)
];

// ========== Quaternion math ==========

pub fn quat_conjugate(q: &[f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

pub fn quat_multiply(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    let (ax, ay, az, aw) = (a[0], a[1], a[2], a[3]);
    let (bx, by, bz, bw) = (b[0], b[1], b[2], b[3]);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

pub fn quat_rotate_vec(q: &[f32; 4], v: &[f32; 3]) -> [f32; 3] {
    // q * (0,v) * conj(q)
    let v_quat = [v[0], v[1], v[2], 0.0];
    let conj = quat_conjugate(q);
    let tmp = quat_multiply(q, &v_quat);
    let result = quat_multiply(&tmp, &conj);
    [result[0], result[1], result[2]]
}

pub fn quat_normalize(q: &[f32; 4]) -> [f32; 4] {
    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    [q[0] / len, q[1] / len, q[2] / len, q[3] / len]
}

pub fn quat_dot(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// conj(home) * current -- orientation of current relative to home.
pub fn relative_quaternion(home: &[f32; 4], current: &[f32; 4]) -> [f32; 4] {
    quat_multiply(&quat_conjugate(home), current)
}

// ========== Gyro lookup ==========

/// Binary search: last sample with sample.t <= t.
pub fn find_gyro_before<'a>(gyro: &'a [GyroSample], t: f64) -> Option<&'a GyroSample> {
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
pub fn find_gyro_after<'a>(gyro: &'a [GyroSample], t: f64) -> Option<&'a GyroSample> {
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
pub fn compute_home(samples: &[GyroSample]) -> [f32; 4] {
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
pub fn snap_to_axis(v: &[f32; 3]) -> Color {
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
pub fn estimate_orientation(rel: &[f32; 4]) -> Orientation {
    let up = quat_rotate_vec(rel, &[0.0, 1.0, 0.0]);
    let front = quat_rotate_vec(rel, &[0.0, 0.0, 1.0]);
    Orientation {
        top: snap_to_axis(&up),
        front: snap_to_axis(&front),
    }
}

pub fn orientation_label(orient: Orientation) -> String {
    format!(
        "{}/{}",
        color_to_char(orient.top),
        color_to_char(orient.front)
    )
}

// ========== Color helpers (module-level) ==========

pub fn opposite_color(c: Color) -> Color {
    match c {
        Color::White => Color::Yellow,
        Color::Yellow => Color::White,
        Color::Green => Color::Blue,
        Color::Blue => Color::Green,
        Color::Red => Color::Orange,
        Color::Orange => Color::Red,
    }
}

pub fn color_axis(c: Color) -> [f32; 3] {
    match c {
        Color::White => [0.0, 1.0, 0.0],
        Color::Yellow => [0.0, -1.0, 0.0],
        Color::Green => [0.0, 0.0, 1.0],
        Color::Blue => [0.0, 0.0, -1.0],
        Color::Red => [1.0, 0.0, 0.0],
        Color::Orange => [-1.0, 0.0, 0.0],
    }
}

pub fn color_to_home_face(c: Color) -> &'static str {
    match c {
        Color::White => "U",
        Color::Yellow => "D",
        Color::Green => "F",
        Color::Blue => "B",
        Color::Red => "R",
        Color::Orange => "L",
    }
}

pub fn face_to_color(face: &str) -> Option<Color> {
    match face {
        "U" => Some(Color::White),
        "D" => Some(Color::Yellow),
        "F" => Some(Color::Green),
        "B" => Some(Color::Blue),
        "R" => Some(Color::Red),
        "L" => Some(Color::Orange),
        _ => None,
    }
}

pub fn color_to_perspectival_face(c: Color, orient: Orientation) -> Option<&'static str> {
    if c == orient.top {
        return Some("U");
    }
    if c == opposite_color(orient.top) {
        return Some("D");
    }
    if c == orient.front {
        return Some("F");
    }
    if c == opposite_color(orient.front) {
        return Some("B");
    }
    let right = compute_right_color(orient.top, orient.front);
    if c == right {
        return Some("R");
    }
    if c == opposite_color(right) {
        return Some("L");
    }
    None
}

use rouxflow_bitboard::move_indices::Move;

pub fn map_move_to_orientation(m: Move, orient: Orientation) -> Move {
    let face_str = match m {
        Move::Face(f) => &f.as_str()[0..1],
        Move::Wide(w) => &w.as_str()[0..1].to_uppercase(),
        Move::Slice(s) => match s {
            rouxflow_bitboard::move_indices::SliceMove::M
            | rouxflow_bitboard::move_indices::SliceMove::Mp
            | rouxflow_bitboard::move_indices::SliceMove::M2 => "L",
            rouxflow_bitboard::move_indices::SliceMove::E
            | rouxflow_bitboard::move_indices::SliceMove::Ep
            | rouxflow_bitboard::move_indices::SliceMove::E2 => "D",
            rouxflow_bitboard::move_indices::SliceMove::S
            | rouxflow_bitboard::move_indices::SliceMove::Sp
            | rouxflow_bitboard::move_indices::SliceMove::S2 => "F",
        },
        _ => return m,
    };

    if let Some(hardware_color) = face_to_color(face_str) {
        if let Some(perspectival_face) = color_to_perspectival_face(hardware_color, orient) {
            return m.with_face(perspectival_face).unwrap_or(m);
        }
    }
    m
}

pub fn compute_right_color(top: Color, front: Color) -> Color {
    let up_dir = color_axis(top);
    let front_dir = color_axis(front);
    let right_dir = [
        up_dir[1] * front_dir[2] - up_dir[2] * front_dir[1],
        up_dir[2] * front_dir[0] - up_dir[0] * front_dir[2],
        up_dir[0] * front_dir[1] - up_dir[1] * front_dir[0],
    ];
    snap_to_axis(&right_dir)
}

pub fn char_to_color(c: char) -> Option<Color> {
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

pub fn parse_orient_label(s: &str) -> Option<Orientation> {
    let mut chars = s.chars();
    let top = char_to_color(chars.next()?)?;
    if chars.next() != Some('/') {
        return None;
    }
    let front = char_to_color(chars.next()?)?;
    Some(Orientation { top, front })
}

// ========== Rotation detection ==========

/// Apply a cube rotation to an orientation, returning the new Orientation.
/// x = rotate like R (CW from right side).
/// y = rotate like U (CW from top).
/// z = rotate like F (CW from front).
pub fn apply_rotation(orient: Orientation, rot: &str) -> Orientation {
    let right = compute_right_color(orient.top, orient.front);
    let bottom = opposite_color(orient.top);
    let back = opposite_color(orient.front);
    let left = opposite_color(right);

    let (new_top, new_front) = match rot {
        "x" => (orient.front, bottom),
        "x'" => (back, orient.top),
        "x2" => (bottom, back),
        "y" => (orient.top, right),
        "y'" => (orient.top, left),
        "y2" => (orient.top, back),
        "z" => (left, orient.front),
        "z'" => (right, orient.front),
        "z2" => (bottom, orient.front),
        _ => (orient.top, orient.front),
    };
    Orientation {
        top: new_top,
        front: new_front,
    }
}

/// Detect which rotation transforms one orientation into another.
/// Tries single rotations first, then pairs if needed.
pub fn detect_rotation(from: Orientation, to: Orientation) -> String {
    if from == to {
        return String::new();
    }

    const ROTS: [&str; 9] = ["x", "x'", "x2", "y", "y'", "y2", "z", "z'", "z2"];

    // Try single rotation
    for rot in &ROTS {
        if apply_rotation(from, rot) == to {
            return rot.to_string();
        }
    }

    // Try pair of rotations
    for r1 in &ROTS {
        let mid = apply_rotation(from, r1);
        for r2 in &ROTS {
            if apply_rotation(mid, r2) == to {
                return format!("{} {}", r1, r2);
            }
        }
    }

    "?".to_string()
}

/// Combine shell orientation (from gyro) and centers orientation (from slices).
pub fn combine_orientations(shell: Orientation, centers: Orientation) -> Orientation {
    let home = Orientation {
        top: Color::White,
        front: Color::Green,
    };
    if centers == home {
        return shell;
    }
    let rot = detect_rotation(home, centers);
    let mut result = shell;
    for part in rot.split_whitespace() {
        result = apply_rotation(result, part);
    }
    result
}

// ========== Gyro Timeline Analysis ==========

pub struct GyroRun {
    pub label: String,
    pub count: usize,
    pub t_start: f64, // timestamp of first sample in this run
}

/// Collect consecutive runs of same-orientation samples in a time window.
pub fn collect_orient_runs(
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
        let orient = estimate_orientation(&rel);
        let label = orientation_label(orient);
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

/// Flag noise: a run with count <= noise_max surrounded by different orientations.
/// prev_ctx / next_ctx provide the adjacent window's boundary label so that
/// a single sample at a window edge isn't falsely flagged when it matches the neighbor window.
pub fn is_noise(
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

/// Get boundary labels for a window (first non-empty label, last non-empty label).
pub fn window_boundary_labels(runs: &[GyroRun]) -> (Option<String>, Option<String>) {
    let first = runs.first().map(|r| r.label.clone());
    let last = runs.last().map(|r| r.label.clone());
    (first, last)
}

/// Get the effective orientation of a window (ignoring noise runs, using last stable run).
pub fn window_effective_orient(
    runs: &[GyroRun],
    prev_ctx: Option<&str>,
    next_ctx: Option<&str>,
) -> String {
    // Walk forward to find first non-noise run
    for i in 0..runs.len() {
        if !is_noise(runs, i, 1, prev_ctx, next_ctx) {
            return runs[i].label.clone();
        }
    }
    // Fallback: first run
    runs.first()
        .map(|r| r.label.clone())
        .unwrap_or_else(|| "?/?".to_string())
}
