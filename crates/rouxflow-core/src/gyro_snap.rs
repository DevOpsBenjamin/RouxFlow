use crate::cube::facelet::Color;
use crate::cube::Quaternion;

/// Thresholds for hysteresis in orientation matching based on dot product.
/// Math: dot = cos(theta / 2). A difference of 25 degrees means theta=25.
/// cos(25°/2) = cos(12.5°) = 0.9763
pub const DOT_THRESHOLD_ENTER: f32 = 0.9763; // ~25-degree cone to enter a new state

#[derive(Debug, Clone, Copy)]
pub struct CubePosture {
    pub top: Color,
    pub front: Color,
    /// Absolute quaternion representing this posture (Right-Handed)
    pub q: [f32; 4],
}

pub const POSTURES: [CubePosture; 24] = [
    // 1. W/G (HOME - Identity)
    CubePosture {
        top: Color::White,
        front: Color::Green,
        q: [1.0, 0.0, 0.0, 0.0],
    },
    // 2. W/O (+90° Y)
    CubePosture {
        top: Color::White,
        front: Color::Orange,
        q: [0.7071, 0.0, 0.7071, 0.0],
    },
    // 3. W/B (+180° Y)
    CubePosture {
        top: Color::White,
        front: Color::Blue,
        q: [0.0, 0.0, 1.0, 0.0],
    },
    // 4. W/R (-90° Y)
    CubePosture {
        top: Color::White,
        front: Color::Red,
        q: [0.7071, 0.0, -0.7071, 0.0],
    },
    // 5. Y/B (+180° X) -> Down is Up
    CubePosture {
        top: Color::Yellow,
        front: Color::Blue,
        q: [0.0, 1.0, 0.0, 0.0],
    },
    // 6. Y/O (+180° X, then -90° Y) -> Down Up, Orange Front
    CubePosture {
        top: Color::Yellow,
        front: Color::Orange,
        q: [0.0, 0.7071, 0.0, -0.7071],
    },
    // 7. Y/G (+180° X, then 180° Y) -> Down Up, Green Front
    CubePosture {
        top: Color::Yellow,
        front: Color::Green,
        q: [0.0, 0.0, 0.0, 1.0],
    },
    // 8. Y/R (+180° X, then +90° Y) -> Down Up, Red Front
    CubePosture {
        top: Color::Yellow,
        front: Color::Red,
        q: [0.0, 0.7071, 0.0, 0.7071],
    },
    // 9. G/Y (-90° X) -> Front is Up, Down is Front
    CubePosture {
        top: Color::Green,
        front: Color::Yellow,
        q: [0.7071, -0.7071, 0.0, 0.0],
    },
    // 10. G/O (-90° X, then +90° Y local rotation => -90° Z world)
    CubePosture {
        top: Color::Green,
        front: Color::Orange,
        q: [0.5, -0.5, 0.5, 0.5],
    },
    // 11. G/W (-90° X, 180° Y local)
    CubePosture {
        top: Color::Green,
        front: Color::White,
        q: [0.0, 0.0, 0.7071, 0.7071],
    },
    // 12. G/R (-90° X, -90° Y local => +90° Z world)
    CubePosture {
        top: Color::Green,
        front: Color::Red,
        q: [0.5, -0.5, -0.5, -0.5],
    },
    // 13. B/W (+90° X) -> Back is Up, Up is Front
    CubePosture {
        top: Color::Blue,
        front: Color::White,
        q: [0.7071, 0.7071, 0.0, 0.0],
    },
    // 14. B/O (+90° X, +90° Y local)
    CubePosture {
        top: Color::Blue,
        front: Color::Orange,
        q: [0.5, 0.5, 0.5, -0.5],
    },
    // 15. B/Y (+90° X, 180° Y local)
    CubePosture {
        top: Color::Blue,
        front: Color::Yellow,
        q: [0.0, 0.0, 0.7071, -0.7071],
    },
    // 16. B/R (+90° X, -90° Y local)
    CubePosture {
        top: Color::Blue,
        front: Color::Red,
        q: [0.5, 0.5, -0.5, 0.5],
    },
    // 17. R/G (+90° Z) -> Right is Up, Green is Front
    CubePosture {
        top: Color::Red,
        front: Color::Green,
        q: [0.7071, 0.0, 0.0, 0.7071],
    },
    // 18. R/W (+90° Z, +90° Y local)
    CubePosture {
        top: Color::Red,
        front: Color::White,
        q: [0.5, 0.5, 0.5, 0.5],
    },
    // 19. R/B (+90° Z, 180° Y local)
    CubePosture {
        top: Color::Red,
        front: Color::Blue,
        q: [0.0, 0.7071, 0.7071, 0.0],
    },
    // 20. R/Y (+90° Z, -90° Y local)
    CubePosture {
        top: Color::Red,
        front: Color::Yellow,
        q: [0.5, -0.5, -0.5, 0.5],
    },
    // 21. O/G (-90° Z) -> Left is Up, Green is Front
    CubePosture {
        top: Color::Orange,
        front: Color::Green,
        q: [0.7071, 0.0, 0.0, -0.7071],
    },
    // 22. O/Y (-90° Z, +90° Y local)
    CubePosture {
        top: Color::Orange,
        front: Color::Yellow,
        q: [0.5, -0.5, 0.5, -0.5],
    },
    // 23. O/B (-90° Z, 180° Y local)
    CubePosture {
        top: Color::Orange,
        front: Color::Blue,
        q: [0.0, 0.7071, -0.7071, 0.0],
    },
    // 24. O/W (-90° Z, -90° Y local)
    CubePosture {
        top: Color::Orange,
        front: Color::White,
        q: [0.5, 0.5, -0.5, -0.5],
    },
];

pub struct AbsoluteStateTracker;

impl AbsoluteStateTracker {
    /// Returns the nearest CubePosture (and its index) if within the DOT_THRESHOLD_ENTER tolerance (~25 degrees).
    /// Otherwise, returns None.
    pub fn get_nearest_posture(q_rel_shell: &Quaternion) -> Option<(usize, CubePosture)> {
        let mut best_dot = -1.0;
        let mut best_idx = 0;

        for (i, posture) in POSTURES.iter().enumerate() {
            let dot = (posture.q[0] * q_rel_shell.w)
                + (posture.q[1] * q_rel_shell.x)
                + (posture.q[2] * q_rel_shell.y)
                + (posture.q[3] * q_rel_shell.z);
            let abs_dot = dot.abs();

            if abs_dot > best_dot {
                best_dot = abs_dot;
                best_idx = i;
            }
        }

        // 25 degrees (0.9763) recommended threshold for a very precise cone
        if best_dot >= DOT_THRESHOLD_ENTER {
            Some((best_idx, POSTURES[best_idx]))
        } else {
            None
        }
    }

    /// Computes the relative orientation of the shell (what the user actually sees)
    /// compared to the original home position.
    /// Formula: q_rel_shell = conjugate(home) * q_current
    pub fn compute_rel_shell(home: &Quaternion, q_current: &Quaternion) -> Quaternion {
        let conj = Quaternion {
            x: -home.x,
            y: -home.y,
            z: -home.z,
            w: home.w,
        };

        Quaternion {
            w: conj.w * q_current.w
                - conj.x * q_current.x
                - conj.y * q_current.y
                - conj.z * q_current.z,
            x: conj.w * q_current.x + conj.x * q_current.w + conj.y * q_current.z
                - conj.z * q_current.y,
            y: conj.w * q_current.y - conj.x * q_current.z
                + conj.y * q_current.w
                + conj.z * q_current.x,
            z: conj.w * q_current.z + conj.x * q_current.y - conj.y * q_current.x
                + conj.z * q_current.w,
        }
    }
}
