use serde::{Serialize, Deserialize};
use crate::cube::{Face, Quaternion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveKind {
    Face,
    Wide,
    Slice,
    Rotation,
}

#[derive(Debug, Clone)]
pub struct InterpretedMove {
    pub notation: String,
    pub timestamp_ms: u32,
    pub raw_face_moves: Vec<(Face, i8)>,
    pub kind: MoveKind,
    /// Accumulated gyro rotation delta (x, y, z) in radians at time of emission.
    /// Some for Wide/Rotation/Slice moves, None for plain Face moves.
    pub gyro_delta: Option<[f32; 3]>,
}

pub struct InterpreterConfig {
    pub merge_window_ms: f64,
    pub rotation_threshold_rad: f32,
    pub wide_threshold_rad: f32,
    pub has_gyro: bool,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            merge_window_ms: 50.0,
            rotation_threshold_rad: 1.2,
            wide_threshold_rad: 0.5,
            has_gyro: false,
        }
    }
}

struct PendingFaceMove {
    face: Face,
    direction: i8,
    wall_ms: f64,
}

/// Suppress standalone gyro-only rotation detection for this long after emitting
/// a Slice/Wide/Rotation move, since those inherently cause body rotation.
const ROTATION_SUPPRESSION_MS: f64 = 200.0;

pub struct MoveInterpreter {
    config: InterpreterConfig,
    pending: Vec<PendingFaceMove>,
    last_gyro: Option<Quaternion>,
    anchor_gyro: Option<Quaternion>,
    /// Accumulated rotation delta (x, y, z) in radians since last anchor
    accum_rotation: [f32; 3],
    /// Wall-clock time of last emitted Slice/Wide/Rotation move (for suppression)
    last_body_move_ms: f64,
}

impl MoveInterpreter {
    pub fn new(config: InterpreterConfig) -> Self {
        Self {
            config,
            pending: Vec::new(),
            last_gyro: None,
            anchor_gyro: None,
            accum_rotation: [0.0; 3],
            last_body_move_ms: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.pending.clear();
        self.last_gyro = None;
        self.anchor_gyro = None;
        self.accum_rotation = [0.0; 3];
        self.last_body_move_ms = 0.0;
    }

    pub fn set_has_gyro(&mut self, has_gyro: bool) {
        self.config.has_gyro = has_gyro;
    }

    pub fn current_accum_rotation(&self) -> [f32; 3] {
        self.accum_rotation
    }

    pub fn feed_face_move(&mut self, face: Face, direction: i8, wall_ms: f64) {
        self.pending.push(PendingFaceMove { face, direction, wall_ms });
    }

    pub fn feed_gyro(&mut self, q: &Quaternion, _wall_ms: f64) {
        if let Some(anchor) = &self.anchor_gyro {
            let delta = quat_mul(&quat_conjugate(anchor), q);
            let (ax, ay, az, angle) = quat_to_axis_angle(&delta);
            self.accum_rotation[0] = ax * angle;
            self.accum_rotation[1] = ay * angle;
            self.accum_rotation[2] = az * angle;
        }
        self.last_gyro = Some(*q);
        if self.anchor_gyro.is_none() {
            self.anchor_gyro = Some(*q);
        }
    }

    pub fn flush(&mut self, wall_ms: f64, solve_start_ms: f64) -> Vec<InterpretedMove> {
        let mut result = Vec::new();

        // Process pending face moves
        loop {
            if self.pending.len() >= 2 {
                // Check if first two are opposite-face, opposite-direction → slice
                let f1 = self.pending[0].face;
                let d1 = self.pending[0].direction;
                let f2 = self.pending[1].face;
                let d2 = self.pending[1].direction;

                if d1 == -d2 {
                    if let Some((notation, kind)) = self.classify_pair(f1, d1, f2, d2) {
                        let ts = self.compute_timestamp(self.pending[0].wall_ms, solve_start_ms);
                        let gyro_delta = self.capture_gyro_delta(&kind);
                        // Track body-movement moves for suppression
                        if matches!(kind, MoveKind::Slice | MoveKind::Wide | MoveKind::Rotation) {
                            self.last_body_move_ms = wall_ms;
                        }
                        result.push(InterpretedMove {
                            notation,
                            timestamp_ms: ts,
                            raw_face_moves: vec![(f1, d1), (f2, d2)],
                            kind,
                            gyro_delta,
                        });
                        // Update anchor after emitting a face move pair
                        self.update_anchor();
                        self.pending.drain(..2);
                        continue;
                    }
                }

                // First two aren't a slice pair. If first is expired, emit it.
                if wall_ms - self.pending[0].wall_ms > self.config.merge_window_ms {
                    let p = self.pending.remove(0);
                    let ts = self.compute_timestamp(p.wall_ms, solve_start_ms);
                    let emitted = self.emit_single_or_wide(p.face, p.direction, ts);
                    if matches!(emitted.kind, MoveKind::Wide) {
                        self.last_body_move_ms = wall_ms;
                    }
                    result.push(emitted);
                    self.update_anchor();
                    continue;
                }

                // First hasn't expired yet, stop processing
                break;
            } else if self.pending.len() == 1 {
                if wall_ms - self.pending[0].wall_ms > self.config.merge_window_ms {
                    let p = self.pending.remove(0);
                    let ts = self.compute_timestamp(p.wall_ms, solve_start_ms);
                    let emitted = self.emit_single_or_wide(p.face, p.direction, ts);
                    if matches!(emitted.kind, MoveKind::Wide) {
                        self.last_body_move_ms = wall_ms;
                    }
                    result.push(emitted);
                    self.update_anchor();
                    continue;
                }
                break;
            } else {
                break;
            }
        }

        // Gyro-only rotation check (no pending face moves, has gyro, accumulated rotation is large)
        // Suppressed for a short window after Slice/Wide/Rotation moves, since those
        // inherently cause body rotation that would trigger false standalone rotations.
        let suppressed = wall_ms - self.last_body_move_ms < ROTATION_SUPPRESSION_MS;
        if self.pending.is_empty() && self.config.has_gyro && !suppressed {
            if let Some(rotation) = self.check_gyro_rotation() {
                let ts = self.compute_timestamp(wall_ms, solve_start_ms);
                let gyro_delta = Some(self.accum_rotation);
                self.last_body_move_ms = wall_ms;
                result.push(InterpretedMove {
                    notation: rotation,
                    timestamp_ms: ts,
                    raw_face_moves: vec![],
                    kind: MoveKind::Rotation,
                    gyro_delta,
                });
                self.update_anchor();
            }
        }

        result
    }

    /// Classify a pair of opposite-face moves as a slice (M/E/S).
    /// Opposite-face pairs are always slices — real whole-cube rotations (x/y/z)
    /// are detected by standalone gyro-only rotation, not from face move pairs.
    /// The gyro/zone tracking handles orientation updates separately.
    fn classify_pair(&self, f1: Face, d1: i8, f2: Face, _d2: i8) -> Option<(String, MoveKind)> {
        let (slice_face_1, axis_index) = match (f1, f2) {
            (Face::R, Face::L) | (Face::L, Face::R) => (Face::L, 0), // M axis
            (Face::F, Face::B) | (Face::B, Face::F) => (Face::F, 2), // S axis
            (Face::U, Face::D) | (Face::D, Face::U) => (Face::D, 1), // E axis
            _ => return None,
        };

        // Determine slice direction: M follows L, E follows D, S follows F
        let dir = if f1 == slice_face_1 { d1 } else { _d2 };
        let suffix = if dir > 0 { "" } else { "'" };

        let slice_names = ["M", "E", "S"];
        let notation = format!("{}{}", slice_names[axis_index], suffix);
        Some((notation, MoveKind::Slice))
    }

    /// Try to classify a single face move + gyro as a wide move.
    /// Returns the wide move notation if conditions are met.
    fn classify_wide(&self, face: Face, direction: i8) -> Option<String> {
        if !self.config.has_gyro {
            return None;
        }

        let axis = match face {
            Face::R | Face::L => 0,
            Face::U | Face::D => 1,
            Face::F | Face::B => 2,
        };

        let gyro = self.accum_rotation[axis];
        if gyro.abs() < self.config.wide_threshold_rad {
            return None;
        }

        let gyro_sign = if gyro > 0.0 { 1 } else { -1 };

        // Direction consistency: detected face STAYED in place,
        // so it moved opposite to body rotation in core frame.
        // Condition: dir * gyro_sign < 0
        if direction * gyro_sign >= 0 {
            return None;
        }

        // Explicit wide move mapping table
        let notation = match (face, direction, gyro_sign) {
            // x axis: R/L
            (Face::L, -1,  1) => "r",
            (Face::L,  1, -1) => "r'",
            (Face::R,  1, -1) => "l",
            (Face::R, -1,  1) => "l'",
            // y axis: U/D
            (Face::D, -1,  1) => "u",
            (Face::D,  1, -1) => "u'",
            (Face::U,  1, -1) => "d",
            (Face::U, -1,  1) => "d'",
            // z axis: F/B
            (Face::B, -1,  1) => "f",
            (Face::B,  1, -1) => "f'",
            (Face::F,  1, -1) => "b",
            (Face::F, -1,  1) => "b'",
            _ => return None,
        };

        Some(notation.to_string())
    }

    /// Emit a single face move, checking for wide move upgrade first.
    fn emit_single_or_wide(&self, face: Face, direction: i8, timestamp_ms: u32) -> InterpretedMove {
        if let Some(notation) = self.classify_wide(face, direction) {
            return InterpretedMove {
                notation,
                timestamp_ms,
                raw_face_moves: vec![(face, direction)],
                kind: MoveKind::Wide,
                gyro_delta: Some(self.accum_rotation),
            };
        }
        self.single_face_move(face, direction, timestamp_ms)
    }

    fn single_face_move(&self, face: Face, direction: i8, timestamp_ms: u32) -> InterpretedMove {
        let face_names = ["U", "R", "F", "D", "L", "B"];
        let suffix = if direction == 1 { "" } else if direction == -1 { "'" } else { "2" };
        let notation = format!("{}{}", face_names[face as usize], suffix);
        InterpretedMove {
            notation,
            timestamp_ms,
            raw_face_moves: vec![(face, direction)],
            kind: MoveKind::Face,
            gyro_delta: None,
        }
    }

    fn compute_timestamp(&self, wall_ms: f64, solve_start_ms: f64) -> u32 {
        if solve_start_ms > 0.0 {
            ((wall_ms - solve_start_ms).max(0.0)) as u32
        } else {
            0
        }
    }

    /// Capture gyro_delta: Some for Slice/Rotation/Wide, None for Face.
    fn capture_gyro_delta(&self, kind: &MoveKind) -> Option<[f32; 3]> {
        match kind {
            MoveKind::Face => None,
            _ => Some(self.accum_rotation),
        }
    }

    fn update_anchor(&mut self) {
        self.anchor_gyro = self.last_gyro;
        self.accum_rotation = [0.0; 3];
    }

    fn check_gyro_rotation(&self) -> Option<String> {
        let threshold = self.config.rotation_threshold_rad;
        let ax = self.accum_rotation[0].abs();
        let ay = self.accum_rotation[1].abs();
        let az = self.accum_rotation[2].abs();

        // Find dominant axis — must be at least 2x the other two axes
        // to avoid false positives from diagonal tilts where all components
        // are roughly equal.
        let (dominant_val, dominant_idx) = if ax >= ay && ax >= az {
            (ax, 0)
        } else if ay >= ax && ay >= az {
            (ay, 1)
        } else {
            (az, 2)
        };

        if dominant_val < threshold {
            return None;
        }

        // Dominance check: the dominant axis must be at least 2x the others
        let others_max = match dominant_idx {
            0 => ay.max(az),
            1 => ax.max(az),
            _ => ax.max(ay),
        };
        if others_max > 0.0 && dominant_val / others_max < 2.0 {
            return None;
        }

        let names = ["x", "y", "z"];
        let suffix = if self.accum_rotation[dominant_idx] > 0.0 { "" } else { "'" };
        Some(format!("{}{}", names[dominant_idx], suffix))
    }
}

// ========== Quaternion math helpers ==========

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

fn quat_to_axis_angle(q: &Quaternion) -> (f32, f32, f32, f32) {
    let norm = (q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
    if norm < 1e-6 {
        return (0.0, 0.0, 1.0, 0.0);
    }
    let angle = 2.0 * norm.atan2(q.w);
    let inv_norm = 1.0 / norm;
    (q.x * inv_norm, q.y * inv_norm, q.z * inv_norm, angle)
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

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

        let moves = interp.flush(250.0, 0.0);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].notation, "x");
        assert_eq!(moves[0].kind, MoveKind::Rotation);
        assert!(moves[0].raw_face_moves.is_empty());
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
        // Real rotations come from standalone gyro detection, not face move pairs.
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
}
