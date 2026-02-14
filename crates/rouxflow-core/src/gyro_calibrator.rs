use crate::cube::{Face, Quaternion};

/// The 6 standard directions: +X, -X, +Y, -Y, +Z, -Z
const STANDARD_DIRS: [[f32; 3]; 6] = [
    [ 1.0,  0.0,  0.0], // 0: +X
    [-1.0,  0.0,  0.0], // 1: -X
    [ 0.0,  1.0,  0.0], // 2: +Y
    [ 0.0, -1.0,  0.0], // 3: -Y
    [ 0.0,  0.0,  1.0], // 4: +Z
    [ 0.0,  0.0, -1.0], // 5: -Z
];

const ZONE_NAMES: [&str; 6] = ["+X", "-X", "+Y", "-Y", "+Z", "-Z"];
const AXIS_NAMES: [&str; 3] = ["X(O↔R)", "Y(W↔Y)", "Z(G↔B)"];

/// Half-angle to enter a zone (must be within this to snap in).
const ZONE_ENTER_HALF_ANGLE_DEG: f32 = 30.0;
/// Half-angle to exit a zone (must drift past this to leave).
/// The gap (30°-40°) is the hysteresis dead band — no transitions happen there.
const ZONE_EXIT_HALF_ANGLE_DEG: f32 = 40.0;

/// Accumulates gyro quaternion samples during scrambling and computes
/// a "home" orientation with 3 axes. Tracks orientation zones during solving.
pub struct GyroCalibrator {
    /// All samples collected during calibration (sign-flipped to consistent hemisphere)
    samples: Vec<[f32; 4]>,
    /// First sample for sign-flip detection
    first: Option<[f32; 4]>,
    active: bool,

    // --- Calibration results ---
    home: Option<Quaternion>,
    /// The 3 home axes in world frame: [x_axis, y_axis, z_axis]
    home_axes: Option<[[f32; 3]; 3]>,

    // --- Zone tracking (active after calibration) ---
    /// Which standard direction each axis currently maps to. -1 = between zones.
    current_zones: [i8; 3],
    /// cos(enter_half_angle): dot must be >= this to enter a zone
    enter_cos: f32,
    /// cos(exit_half_angle): dot must drop below this to leave a zone
    exit_cos: f32,
}

impl GyroCalibrator {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            first: None,
            active: false,
            home: None,
            home_axes: None,
            current_zones: [-1; 3],
            enter_cos: (ZONE_ENTER_HALF_ANGLE_DEG.to_radians()).cos(),
            exit_cos: (ZONE_EXIT_HALF_ANGLE_DEG.to_radians()).cos(),
        }
    }

    /// Begin accumulating samples. Clears previous data.
    pub fn start(&mut self) {
        self.samples.clear();
        self.first = None;
        self.active = true;
        self.home = None;
        self.home_axes = None;
        self.current_zones = [-1; 3];
    }

    /// Feed a quaternion sample. Only accumulates while active.
    /// Handles sign-flip: if dot(sample, first) < 0, negates sample before storing.
    pub fn feed(&mut self, q: &Quaternion) {
        if !self.active {
            return;
        }

        let (mut x, mut y, mut z, mut w) = (q.x, q.y, q.z, q.w);

        match self.first {
            None => {
                self.first = Some([x, y, z, w]);
            }
            Some(f) => {
                let dot = x * f[0] + y * f[1] + z * f[2] + w * f[3];
                if dot < 0.0 {
                    x = -x;
                    y = -y;
                    z = -z;
                    w = -w;
                }
            }
        }

        self.samples.push([x, y, z, w]);
    }

    /// Finalize calibration. Averages samples, removes P90 outliers, re-averages.
    /// Extracts 3 home axes. Requires >= 10 samples.
    pub fn finalize(&mut self) -> Option<Quaternion> {
        self.active = false;

        if self.samples.len() < 10 {
            return None;
        }

        // Step 1: compute initial average
        let initial_avg = Self::average_quaternions(&self.samples)?;

        // Step 2: compute angular distance of each sample to the average
        let mut distances: Vec<(usize, f32)> = self.samples.iter().enumerate()
            .map(|(i, s)| {
                let dot = (s[0] * initial_avg[0] + s[1] * initial_avg[1]
                    + s[2] * initial_avg[2] + s[3] * initial_avg[3]).abs();
                let angle = dot.min(1.0).acos() * 2.0; // angular distance in radians
                (i, angle)
            })
            .collect();

        // Step 3: sort by distance, keep closest 90% (discard top 10% outliers)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let keep_count = (distances.len() as f32 * 0.9) as usize;

        // Step 4: take only the closest samples
        let filtered: Vec<[f32; 4]> = distances[..keep_count]
            .iter()
            .map(|(i, _)| self.samples[*i])
            .collect();

        if filtered.len() < 10 {
            return None;
        }

        // Step 5: recompute average from filtered samples
        let avg = Self::average_quaternions(&filtered)?;

        let home_q = Quaternion {
            x: avg[0],
            y: avg[1],
            z: avg[2],
            w: avg[3],
        };

        // Step 6: extract 3 home axes by rotating standard basis by home quaternion
        let axes = [
            rotate_vec_by_quat([1.0, 0.0, 0.0], &home_q),
            rotate_vec_by_quat([0.0, 1.0, 0.0], &home_q),
            rotate_vec_by_quat([0.0, 0.0, 1.0], &home_q),
        ];

        self.home = Some(home_q);
        self.home_axes = Some(axes);

        // Initialize current zones to home position
        // At home, the relative rotation is identity, so axes map to their standard dirs
        self.current_zones = [0, 2, 4]; // +X, +Y, +Z

        Some(home_q)
    }

    /// Average a set of quaternions (assumed sign-consistent). Returns normalized [x,y,z,w].
    fn average_quaternions(samples: &[[f32; 4]]) -> Option<[f32; 4]> {
        let n = samples.len() as f64;
        if n < 1.0 { return None; }

        let mut sum = [0.0f64; 4];
        for s in samples {
            sum[0] += s[0] as f64;
            sum[1] += s[1] as f64;
            sum[2] += s[2] as f64;
            sum[3] += s[3] as f64;
        }

        let avg = [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n];
        let len = (avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2] + avg[3] * avg[3]).sqrt();
        if len < 1e-10 { return None; }

        Some([
            (avg[0] / len) as f32,
            (avg[1] / len) as f32,
            (avg[2] / len) as f32,
            (avg[3] / len) as f32,
        ])
    }

    // === Calibration results ===

    pub fn home(&self) -> Option<&Quaternion> {
        self.home.as_ref()
    }

    pub fn home_axes(&self) -> Option<&[[f32; 3]; 3]> {
        self.home_axes.as_ref()
    }

    /// Compute the renderer offset: conjugate(home) as (x, y, z, w).
    pub fn compute_render_offset(&self) -> Option<(f32, f32, f32, f32)> {
        self.home.map(|h| (-h.x, -h.y, -h.z, h.w))
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    // === Zone tracking ===

    /// Format home axes as debug string.
    pub fn debug_home_axes(&self) -> Option<String> {
        let axes = self.home_axes.as_ref()?;
        Some(format!(
            "X(O↔R)=[{:.3},{:.3},{:.3}] Y(W↔Y)=[{:.3},{:.3},{:.3}] Z(G↔B)=[{:.3},{:.3},{:.3}]",
            axes[0][0], axes[0][1], axes[0][2],
            axes[1][0], axes[1][1], axes[1][2],
            axes[2][0], axes[2][1], axes[2][2],
        ))
    }

    /// Update zone tracking with a new gyro quaternion.
    /// Uses hysteresis: enter_cos (tight) to snap into a zone, exit_cos (loose) to leave.
    /// Returns a list of debug log messages (zone exits, entries, rotations).
    pub fn track_orientation(&mut self, q: &Quaternion) -> Vec<String> {
        let home = match &self.home {
            Some(h) => *h,
            None => return Vec::new(),
        };

        let mut logs = Vec::new();

        // Compute relative rotation: Q_rel = conjugate(home) * q_current
        let q_rel = quat_mul(&quat_conjugate(&home), q);

        // Rotate standard basis by Q_rel to get where each axis currently points
        let current_axes = [
            rotate_vec_by_quat([1.0, 0.0, 0.0], &q_rel),
            rotate_vec_by_quat([0.0, 1.0, 0.0], &q_rel),
            rotate_vec_by_quat([0.0, 0.0, 1.0], &q_rel),
        ];

        for axis_idx in 0..3 {
            let old_zone = self.current_zones[axis_idx];
            let v = current_axes[axis_idx];

            if old_zone >= 0 {
                // Currently in a zone — check if we should EXIT (need to drift past exit_cos)
                let dir = STANDARD_DIRS[old_zone as usize];
                let dot = v[0] * dir[0] + v[1] * dir[1] + v[2] * dir[2];

                if dot < self.exit_cos {
                    // Left the zone — check if we immediately enter another
                    let axis_name = AXIS_NAMES[axis_idx];
                    let new_zone = nearest_zone(v, self.enter_cos);

                    if new_zone >= 0 && new_zone != old_zone {
                        // Direct zone-to-zone transition = rotation detected
                        logs.push(format!("[zone] ROTATION: {} {} -> {}",
                            axis_name,
                            ZONE_NAMES[old_zone as usize],
                            ZONE_NAMES[new_zone as usize]));
                        self.current_zones[axis_idx] = new_zone;
                    } else if new_zone >= 0 {
                        // Re-entered same zone (shouldn't happen, but handle it)
                        self.current_zones[axis_idx] = new_zone;
                    } else {
                        logs.push(format!("[zone] {} EXIT {} (angle: {:.1}°)",
                            axis_name, ZONE_NAMES[old_zone as usize],
                            dot.min(1.0).max(-1.0).acos().to_degrees()));
                        self.current_zones[axis_idx] = -1;
                    }
                }
            } else {
                // Currently between zones — check if we should ENTER (need enter_cos)
                let new_zone = nearest_zone(v, self.enter_cos);

                if new_zone >= 0 {
                    let axis_name = AXIS_NAMES[axis_idx];
                    logs.push(format!("[zone] {} ENTER {}",
                        axis_name, ZONE_NAMES[new_zone as usize]));
                    self.current_zones[axis_idx] = new_zone;
                }
            }
        }

        logs
    }

    /// Get current zone state as a debug string.
    pub fn debug_zones(&self) -> String {
        format!("zones=[{}, {}, {}]",
            zone_label(self.current_zones[0]),
            zone_label(self.current_zones[1]),
            zone_label(self.current_zones[2]))
    }

    /// Get current zone state for external use.
    pub fn current_zones(&self) -> &[i8; 3] {
        &self.current_zones
    }

    /// Remap a move notation from cube body frame to home frame based on current zones.
    /// Returns the original notation if calibration isn't available or zones aren't resolved.
    pub fn remap_notation(&self, notation: &str) -> String {
        if self.home.is_none() {
            return notation.to_string();
        }
        remap_notation_by_zones(notation, &self.current_zones)
    }
}

impl Default for GyroCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

// === Helpers ===

fn zone_label(z: i8) -> &'static str {
    if z >= 0 && (z as usize) < ZONE_NAMES.len() {
        ZONE_NAMES[z as usize]
    } else {
        "??"
    }
}

/// Find which of the 6 standard directions is closest to `v`.
/// Returns the zone index (0-5) if within the cone, or -1 if between zones.
fn nearest_zone(v: [f32; 3], threshold_cos: f32) -> i8 {
    let mut best_dot = -2.0f32;
    let mut best_idx = -1i8;
    for (i, dir) in STANDARD_DIRS.iter().enumerate() {
        let dot = v[0] * dir[0] + v[1] * dir[1] + v[2] * dir[2];
        if dot > best_dot {
            best_dot = dot;
            best_idx = i as i8;
        }
    }
    if best_dot >= threshold_cos {
        best_idx
    } else {
        -1
    }
}

/// Rotate a 3D vector by a quaternion: q * v * q^-1
fn rotate_vec_by_quat(v: [f32; 3], q: &Quaternion) -> [f32; 3] {
    // Using the formula: rotated = q * (0,v) * conjugate(q)
    let vq = Quaternion { x: v[0], y: v[1], z: v[2], w: 0.0 };
    let conj = quat_conjugate(q);
    let tmp = quat_mul(q, &vq);
    let result = quat_mul(&tmp, &conj);
    [result.x, result.y, result.z]
}

fn quat_conjugate(q: &Quaternion) -> Quaternion {
    Quaternion { x: -q.x, y: -q.y, z: -q.z, w: q.w }
}

fn quat_mul(a: &Quaternion, b: &Quaternion) -> Quaternion {
    Quaternion {
        w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
        x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    }
}

// === Notation Remapping ===

/// Remap a move notation from cube body frame to home frame based on zone state.
/// Returns the original notation unchanged if remapping isn't possible.
fn remap_notation_by_zones(notation: &str, zones: &[i8; 3]) -> String {
    remap_notation_inner(notation, zones).unwrap_or_else(|| notation.to_string())
}

fn remap_notation_inner(notation: &str, zones: &[i8; 3]) -> Option<String> {
    let trimmed = notation.trim();
    if trimmed.is_empty() { return None; }

    let first_char = trimmed.chars().next()?;
    let suffix = &trimmed[first_char.len_utf8()..]; // "'" or "2" or ""

    match first_char {
        // Face moves (uppercase)
        'R' | 'L' | 'U' | 'D' | 'F' | 'B' => {
            let face = char_to_face(first_char)?;
            let new_face = remap_face(face, zones)?;
            Some(format!("{}{}", face_to_char(new_face), suffix))
        }
        // Wide moves (lowercase)
        'r' | 'l' | 'u' | 'd' | 'f' | 'b' => {
            let face = char_to_face(first_char.to_ascii_uppercase())?;
            let new_face = remap_face(face, zones)?;
            Some(format!("{}{}", face_to_char(new_face).to_ascii_lowercase(), suffix))
        }
        // Slices
        'M' | 'E' | 'S' => remap_axis_move(first_char, suffix, zones, false),
        // Rotations
        'x' | 'y' | 'z' => remap_axis_move(first_char, suffix, zones, true),
        _ => None,
    }
}

fn char_to_face(c: char) -> Option<Face> {
    match c {
        'R' => Some(Face::R),
        'L' => Some(Face::L),
        'U' => Some(Face::U),
        'D' => Some(Face::D),
        'F' => Some(Face::F),
        'B' => Some(Face::B),
        _ => None,
    }
}

fn face_to_char(f: Face) -> char {
    match f {
        Face::R => 'R',
        Face::L => 'L',
        Face::U => 'U',
        Face::D => 'D',
        Face::F => 'F',
        Face::B => 'B',
    }
}

/// Remap a face from body frame to home frame using zone state.
/// Returns None if the relevant zone is between (-1).
fn remap_face(face: Face, zones: &[i8; 3]) -> Option<Face> {
    let (axis_idx, is_positive) = match face {
        Face::R => (0, true),
        Face::L => (0, false),
        Face::U => (1, true),
        Face::D => (1, false),
        Face::F => (2, true),
        Face::B => (2, false),
    };

    let zone = zones[axis_idx];
    if zone < 0 { return None; }

    // For positive faces, use the zone directly.
    // For negative faces, use the opposite zone (e.g., if +X→+Z, then -X→-Z).
    let std_dir = if is_positive { zone } else { zone ^ 1 };

    zone_to_face(std_dir)
}

fn zone_to_face(z: i8) -> Option<Face> {
    match z {
        0 => Some(Face::R),  // +X
        1 => Some(Face::L),  // -X
        2 => Some(Face::U),  // +Y
        3 => Some(Face::D),  // -Y
        4 => Some(Face::F),  // +Z
        5 => Some(Face::B),  // -Z
        _ => None,
    }
}

fn opposite_face(f: Face) -> Face {
    match f {
        Face::R => Face::L,
        Face::L => Face::R,
        Face::U => Face::D,
        Face::D => Face::U,
        Face::F => Face::B,
        Face::B => Face::F,
    }
}

/// Remap a slice (M/E/S) or rotation (x/y/z) move.
///
/// Each such move has a "positive direction" face:
///   Slices:    M→L, E→D, S→F
///   Rotations: x→R, y→U, z→F
///
/// For prime moves, the actual direction is the opposite face.
/// We remap that direction face, then convert back to the target notation.
fn remap_axis_move(base_char: char, suffix: &str, zones: &[i8; 3], is_rotation: bool) -> Option<String> {
    let is_double = suffix == "2";
    let is_prime = suffix == "'";

    // The "positive direction" face for this move
    let pos_face = match base_char {
        'M' => Face::L,  // M follows L (-X)
        'E' => Face::D,  // E follows D (-Y)
        'S' => Face::F,  // S follows F (+Z)
        'x' => Face::R,  // x follows R (+X)
        'y' => Face::U,  // y follows U (+Y)
        'z' => Face::F,  // z follows F (+Z)
        _ => return None,
    };

    if is_double {
        // For doubles, direction doesn't matter (180° is symmetric).
        // Just determine which axis the move maps to.
        let new_pos_face = remap_face(pos_face, zones)?;
        let axis_char = if is_rotation {
            match new_pos_face {
                Face::R | Face::L => 'x',
                Face::U | Face::D => 'y',
                Face::F | Face::B => 'z',
            }
        } else {
            match new_pos_face {
                Face::R | Face::L => 'M',
                Face::U | Face::D => 'E',
                Face::F | Face::B => 'S',
            }
        };
        return Some(format!("{}2", axis_char));
    }

    // For non-doubles: determine actual direction face (prime flips it)
    let dir_face = if is_prime { opposite_face(pos_face) } else { pos_face };

    // Remap to home frame
    let new_dir_face = remap_face(dir_face, zones)?;

    // Convert remapped direction face to notation
    if is_rotation {
        Some(match new_dir_face {
            Face::R => "x".to_string(),
            Face::L => "x'".to_string(),
            Face::U => "y".to_string(),
            Face::D => "y'".to_string(),
            Face::F => "z".to_string(),
            Face::B => "z'".to_string(),
        })
    } else {
        Some(match new_dir_face {
            Face::L => "M".to_string(),
            Face::R => "M'".to_string(),
            Face::D => "E".to_string(),
            Face::U => "E'".to_string(),
            Face::F => "S".to_string(),
            Face::B => "S'".to_string(),
        })
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    fn q(x: f32, y: f32, z: f32, w: f32) -> Quaternion {
        Quaternion { x, y, z, w }
    }

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn test_basic_calibration() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.0, 0.0, 0.3827, 0.9239); // ~45° around z
        for _ in 0..20 {
            cal.feed(&sample);
        }

        let home = cal.finalize().unwrap();
        assert!(approx_eq(home.x, sample.x));
        assert!(approx_eq(home.y, sample.y));
        assert!(approx_eq(home.z, sample.z));
        assert!(approx_eq(home.w, sample.w));

        // Home axes should exist
        assert!(cal.home_axes().is_some());
    }

    #[test]
    fn test_sign_flip_handled() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.0, 0.0, 0.3827, 0.9239);
        let negated = q(-sample.x, -sample.y, -sample.z, -sample.w);

        for _ in 0..10 {
            cal.feed(&sample);
        }
        for _ in 0..10 {
            cal.feed(&negated);
        }

        let home = cal.finalize().unwrap();
        assert!(approx_eq(home.x, sample.x));
        assert!(approx_eq(home.y, sample.y));
        assert!(approx_eq(home.z, sample.z));
        assert!(approx_eq(home.w, sample.w));
    }

    #[test]
    fn test_min_samples() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..9 {
            cal.feed(&sample);
        }

        assert!(cal.finalize().is_none());
    }

    #[test]
    fn test_not_active_ignores_feed() {
        let mut cal = GyroCalibrator::new();
        let sample = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 {
            cal.feed(&sample);
        }

        assert_eq!(cal.sample_count(), 0);
        assert!(cal.finalize().is_none());
    }

    #[test]
    fn test_render_offset_is_conjugate() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.1, 0.2, 0.3, 0.9274);
        for _ in 0..20 {
            cal.feed(&sample);
        }

        cal.finalize().unwrap();
        let (ox, oy, oz, ow) = cal.compute_render_offset().unwrap();
        let home = cal.home().unwrap();

        assert!(approx_eq(ox, -home.x));
        assert!(approx_eq(oy, -home.y));
        assert!(approx_eq(oz, -home.z));
        assert!(approx_eq(ow, home.w));

        // offset * home should be ~identity
        let rw = ow * home.w - ox * home.x - oy * home.y - oz * home.z;
        let rx = ow * home.x + ox * home.w + oy * home.z - oz * home.y;
        let ry = ow * home.y - ox * home.z + oy * home.w + oz * home.x;
        let rz = ow * home.z + ox * home.y - oy * home.x + oz * home.w;

        assert!(approx_eq(rx, 0.0));
        assert!(approx_eq(ry, 0.0));
        assert!(approx_eq(rz, 0.0));
        assert!(approx_eq(rw, 1.0));
    }

    #[test]
    fn test_start_clears_previous() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let sample = q(0.5, 0.5, 0.5, 0.5);
        for _ in 0..20 {
            cal.feed(&sample);
        }

        cal.start();
        assert_eq!(cal.sample_count(), 0);
        assert!(cal.home.is_none());

        for _ in 0..5 {
            cal.feed(&sample);
        }
        assert!(cal.finalize().is_none());
    }

    #[test]
    fn test_p90_outlier_filtering() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        // 90 good samples near identity
        let good = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..90 {
            cal.feed(&good);
        }

        // 10 outliers (way off — 90° rotation)
        let bad = q(0.7071, 0.0, 0.0, 0.7071);
        for _ in 0..10 {
            cal.feed(&bad);
        }

        let home = cal.finalize().unwrap();
        // Should be very close to identity despite outliers
        assert!(approx_eq(home.w, 1.0));
        assert!(home.x.abs() < 0.05);
    }

    #[test]
    fn test_zone_tracking_identity() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let identity = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 {
            cal.feed(&identity);
        }
        cal.finalize().unwrap();

        // At home, zones should be +X, +Y, +Z
        assert_eq!(cal.current_zones, [0, 2, 4]);

        // Feed identity again — no zone changes
        let logs = cal.track_orientation(&identity);
        assert!(logs.is_empty());
    }

    #[test]
    fn test_hysteresis_no_chatter() {
        let mut cal = GyroCalibrator::new();
        cal.start();
        let identity = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 { cal.feed(&identity); }
        cal.finalize().unwrap();

        // Tilt 35° around X — past enter (30°) but under exit (40°)
        // This is the dead band: once in a zone, we should NOT exit at 35°
        let angle = 35.0f32.to_radians();
        let half = angle / 2.0;
        let tilt = q(half.sin(), 0.0, 0.0, half.cos());

        // We're in zone (+X,+Y,+Z) and tilting 35° shouldn't trigger exit (need > 40°)
        let logs = cal.track_orientation(&tilt);
        assert!(logs.is_empty(), "Should not exit at 35° tilt (within exit threshold): {:?}", logs);
        assert_eq!(cal.current_zones, [0, 2, 4]); // Still in home zones

        // Tilt back to identity — no change since we never left
        let logs2 = cal.track_orientation(&identity);
        assert!(logs2.is_empty());

        // Tilt 45° — exits zone (past 40° exit) but doesn't enter neighbor
        // (nearest neighbor at 90°-45°=45° away, need <30° to enter)
        let angle2 = 45.0f32.to_radians();
        let half2 = angle2 / 2.0;
        let tilt2 = q(half2.sin(), 0.0, 0.0, half2.cos());
        let logs3 = cal.track_orientation(&tilt2);
        assert!(!logs3.is_empty(), "Should exit at 45° tilt");
        assert_eq!(cal.current_zones[0], 0); // X still +X (rotation around X)
        assert_eq!(cal.current_zones[1], -1); // Y between
        assert_eq!(cal.current_zones[2], -1); // Z between

        // Return to identity (0° = well within 30° enter)
        let logs4 = cal.track_orientation(&identity);
        assert!(!logs4.is_empty(), "Should re-enter at 0°");
        assert_eq!(cal.current_zones, [0, 2, 4]); // Back to home zones
    }

    #[test]
    fn test_zone_tracking_x_rotation() {
        let mut cal = GyroCalibrator::new();
        cal.start();

        let identity = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 {
            cal.feed(&identity);
        }
        cal.finalize().unwrap();

        // Apply 90° rotation around x axis
        let half = std::f32::consts::FRAC_PI_4;
        let x_rot = q(half.sin(), 0.0, 0.0, half.cos());
        let logs = cal.track_orientation(&x_rot);

        // X axis should stay at +X, Y and Z should change
        assert!(!logs.is_empty());
        assert_eq!(cal.current_zones[0], 0); // X still +X
        // Y moved to +Z, Z moved to -Y (for x rotation)
        assert_eq!(cal.current_zones[1], 4); // Y → +Z
        assert_eq!(cal.current_zones[2], 3); // Z → -Y
    }

    // ========== Notation Remapping Tests ==========

    /// Helper: create a calibrated GyroCalibrator at identity, then rotate to set zones.
    fn calibrated_with_zones(zones: [i8; 3]) -> GyroCalibrator {
        let mut cal = GyroCalibrator::new();
        cal.start();
        let identity = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 { cal.feed(&identity); }
        cal.finalize().unwrap();
        cal.current_zones = zones;
        cal
    }

    #[test]
    fn remap_identity_no_change() {
        let cal = calibrated_with_zones([0, 2, 4]); // +X, +Y, +Z = identity
        assert_eq!(cal.remap_notation("R"), "R");
        assert_eq!(cal.remap_notation("R'"), "R'");
        assert_eq!(cal.remap_notation("R2"), "R2");
        assert_eq!(cal.remap_notation("L"), "L");
        assert_eq!(cal.remap_notation("U'"), "U'");
        assert_eq!(cal.remap_notation("M"), "M");
        assert_eq!(cal.remap_notation("M'"), "M'");
        assert_eq!(cal.remap_notation("x"), "x");
        assert_eq!(cal.remap_notation("y'"), "y'");
        assert_eq!(cal.remap_notation("r"), "r");
        assert_eq!(cal.remap_notation("S2"), "S2");
    }

    #[test]
    fn remap_after_x_rotation() {
        // After x rotation: +X→+X, +Y→+Z, +Z→-Y
        let cal = calibrated_with_zones([0, 4, 3]);

        // Face moves
        assert_eq!(cal.remap_notation("R"), "R");   // R stays R (X axis unchanged)
        assert_eq!(cal.remap_notation("L"), "L");   // L stays L
        assert_eq!(cal.remap_notation("U"), "F");   // U (+Y) → +Z → F
        assert_eq!(cal.remap_notation("U'"), "F'");
        assert_eq!(cal.remap_notation("D"), "B");   // D (-Y) → -Z → B
        assert_eq!(cal.remap_notation("F"), "D");   // F (+Z) → -Y → D
        assert_eq!(cal.remap_notation("B"), "U");   // B (-Z) → +Y → U
        assert_eq!(cal.remap_notation("B'"), "U'");

        // Wide moves
        assert_eq!(cal.remap_notation("u"), "f");
        assert_eq!(cal.remap_notation("d'"), "b'");

        // Slices: M follows L (-X), after x rotation -X→-X → M stays M
        assert_eq!(cal.remap_notation("M"), "M");
        // E follows D (-Y), after x rotation -Y→-Z → S' (since S follows F=+Z, -Z is opposite)
        assert_eq!(cal.remap_notation("E"), "S'");
        // S follows F (+Z), after x rotation +Z→-Y → E (since E follows D=-Y)
        assert_eq!(cal.remap_notation("S"), "E");

        // Rotations: x follows R (+X→+X → x stays x
        assert_eq!(cal.remap_notation("x"), "x");
        // y follows U (+Y→+Z → z
        assert_eq!(cal.remap_notation("y"), "z");
        // z follows F (+Z→-Y → y' (since y follows U=+Y, -Y is opposite)
        assert_eq!(cal.remap_notation("z"), "y'");
        assert_eq!(cal.remap_notation("z'"), "y");
    }

    #[test]
    fn remap_after_y_rotation() {
        // After y rotation (CW from top): +X→+Z, +Y→+Y, +Z→-X
        let cal = calibrated_with_zones([4, 2, 1]);

        assert_eq!(cal.remap_notation("R"), "F");   // R (+X) → +Z → F
        assert_eq!(cal.remap_notation("L"), "B");   // L (-X) → -Z → B
        assert_eq!(cal.remap_notation("U"), "U");   // unchanged
        assert_eq!(cal.remap_notation("F"), "L");   // F (+Z) → -X → L
        assert_eq!(cal.remap_notation("B"), "R");   // B (-Z) → +X → R

        // M follows L (-X) → after y: -X→-Z → B → S'
        assert_eq!(cal.remap_notation("M"), "S'");
        assert_eq!(cal.remap_notation("M'"), "S");

        // x follows R (+X→+Z) → z
        assert_eq!(cal.remap_notation("x"), "z");
        // y follows U (+Y→+Y) → y
        assert_eq!(cal.remap_notation("y"), "y");

        // Doubles
        assert_eq!(cal.remap_notation("R2"), "F2");
        assert_eq!(cal.remap_notation("M2"), "S2");
        assert_eq!(cal.remap_notation("x2"), "z2");
    }

    #[test]
    fn remap_after_z_rotation() {
        // After z rotation (CW from front): +X→+Y, +Y→-X, +Z→+Z
        let cal = calibrated_with_zones([2, 1, 4]);

        assert_eq!(cal.remap_notation("R"), "U");   // R (+X) → +Y → U
        assert_eq!(cal.remap_notation("L"), "D");   // L (-X) → -Y → D
        assert_eq!(cal.remap_notation("U"), "L");   // U (+Y) → -X → L
        assert_eq!(cal.remap_notation("D"), "R");   // D (-Y) → +X → R
        assert_eq!(cal.remap_notation("F"), "F");   // unchanged
        assert_eq!(cal.remap_notation("B"), "B");   // unchanged

        // M follows L (-X→-Y) → E (E follows D=-Y)
        assert_eq!(cal.remap_notation("M"), "E");
        // x follows R (+X→+Y) → y
        assert_eq!(cal.remap_notation("x"), "y");
    }

    #[test]
    fn remap_between_zones_returns_original() {
        // Zone 0 is between (-1)
        let cal = calibrated_with_zones([-1, 2, 4]);
        assert_eq!(cal.remap_notation("R"), "R");   // can't remap (X zone unknown)
        assert_eq!(cal.remap_notation("L"), "L");   // same axis
        assert_eq!(cal.remap_notation("U"), "U");   // Y zone is resolved, so this works
        assert_eq!(cal.remap_notation("M"), "M");   // M needs X axis → can't remap
        assert_eq!(cal.remap_notation("x"), "x");   // x needs X axis → can't remap
    }

    #[test]
    fn remap_no_home_returns_original() {
        let cal = GyroCalibrator::new(); // no calibration
        assert_eq!(cal.remap_notation("R"), "R");
        assert_eq!(cal.remap_notation("M'"), "M'");
        assert_eq!(cal.remap_notation("x"), "x");
    }

    #[test]
    fn remap_double_moves() {
        // After x rotation
        let cal = calibrated_with_zones([0, 4, 3]);
        assert_eq!(cal.remap_notation("U2"), "F2");
        assert_eq!(cal.remap_notation("D2"), "B2");
        assert_eq!(cal.remap_notation("E2"), "S2");
        assert_eq!(cal.remap_notation("y2"), "z2");
        // X axis unchanged
        assert_eq!(cal.remap_notation("R2"), "R2");
        assert_eq!(cal.remap_notation("M2"), "M2");
        assert_eq!(cal.remap_notation("x2"), "x2");
    }

    #[test]
    fn remap_integrated_with_zone_tracking() {
        // Full lifecycle: calibrate at identity, track x rotation, verify remapping
        let mut cal = GyroCalibrator::new();
        cal.start();
        let identity = q(0.0, 0.0, 0.0, 1.0);
        for _ in 0..20 { cal.feed(&identity); }
        cal.finalize().unwrap();

        // At identity: no remapping
        assert_eq!(cal.remap_notation("R"), "R");
        assert_eq!(cal.remap_notation("U"), "U");

        // Apply 90° x rotation via zone tracking
        let half = std::f32::consts::FRAC_PI_4;
        let x_rot = q(half.sin(), 0.0, 0.0, half.cos());
        cal.track_orientation(&x_rot);

        // Now zones should be [0, 4, 3] (after x: Y→+Z, Z→-Y)
        assert_eq!(cal.current_zones, [0, 4, 3]);

        // Remapping should now convert body→home
        assert_eq!(cal.remap_notation("U"), "F");
        assert_eq!(cal.remap_notation("F"), "D");
        assert_eq!(cal.remap_notation("B"), "U");
        assert_eq!(cal.remap_notation("R"), "R");
    }
}
