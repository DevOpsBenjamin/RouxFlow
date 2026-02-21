use super::math::{color_to_home_face, compute_right_color, opposite_color};
use rouxflow_core::cube::Orientation;
use rouxflow_core::telemetry::RawMove;

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
pub fn remap_move(notation: &str, orient: Orientation) -> String {
    let (face_str, suffix) = parse_notation(notation);
    let face_map = build_face_map(orient);

    if let Some(&home_face) = face_map.get(face_str) {
        format!("{}{}", home_face, suffix)
    } else {
        // Unknown face (e.g., M, S, E, x, y, z) -- pass through
        notation.to_string()
    }
}

pub fn parse_notation(notation: &str) -> (&str, &str) {
    if notation.ends_with('2') {
        (&notation[..notation.len() - 1], "2")
    } else if notation.ends_with('\'') {
        (&notation[..notation.len() - 1], "'")
    } else {
        (notation, "")
    }
}

/// For a given orientation, map each body-frame face
/// to a home face name. Direction is always preserved (no flip).
pub fn build_face_map(
    orient: Orientation,
) -> std::collections::HashMap<&'static str, &'static str> {
    let mut map = std::collections::HashMap::new();

    let right_color = compute_right_color(orient.top, orient.front);
    let bottom = opposite_color(orient.top);
    let back = opposite_color(orient.front);
    let left = opposite_color(right_color);

    map.insert("U", color_to_home_face(orient.top));
    map.insert("D", color_to_home_face(bottom));
    map.insert("F", color_to_home_face(orient.front));
    map.insert("B", color_to_home_face(back));
    map.insert("R", color_to_home_face(right_color));
    map.insert("L", color_to_home_face(left));

    map
}

// ========== Slice detection ==========

/// Parse notation into (face_letter, direction). E.g. "F'" -> ("F", -1), "B" -> ("B", 1).
pub fn parse_face_dir(notation: &str) -> Option<(&str, i8)> {
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
pub fn are_opposite_faces(f1: &str, f2: &str) -> bool {
    matches!(
        (f1, f2),
        ("U", "D") | ("D", "U") | ("R", "L") | ("L", "R") | ("F", "B") | ("B", "F")
    )
}

/// Name a slice from two BLE-reported face moves.
/// BLE reports the APPARENT motion of outer faces (inverse of actual core motion).
/// So we NEGATE the direction: BLE F'(-1) + B(+1) → core went F direction → S (positive).
/// M follows L, E follows D, S follows F.
pub fn slice_name(f1: &str, d1: i8, f2: &str, d2: i8) -> String {
    let (letter, ref_face) = match (f1, f2) {
        ("R", "L") | ("L", "R") => ("M", "L"),
        ("U", "D") | ("D", "U") => ("E", "D"),
        ("F", "B") | ("B", "F") => ("S", "F"),
        _ => {
            return format!(
                "?({}{}/{}{})",
                f1,
                if d1 < 0 { "'" } else { "" },
                f2,
                if d2 < 0 { "'" } else { "" }
            )
        }
    };
    // Negate: BLE reports inverse of core motion
    let ble_dir = if f1 == ref_face { d1 } else { d2 };
    let dir = -ble_dir;
    let suffix = if dir > 0 { "" } else { "'" };
    format!("{}{}", letter, suffix)
}

/// Check if two consecutive raw moves form a slice pair.
pub fn is_slice_pair(m1: &RawMove, m2: &RawMove) -> bool {
    // Same timestamp (BLE reports them together, up to ~2-3ms jitter)
    if (m1.t - m2.t).abs() > 0.005 {
        return false;
    }
    let Some((f1, d1)) = parse_face_dir(&m1.n) else {
        return false;
    };
    let Some((f2, d2)) = parse_face_dir(&m2.n) else {
        return false;
    };
    are_opposite_faces(f1, f2) && d1 == -d2
}

/// Remap a slice: remap both constituent faces, then name the home-frame slice.
pub fn remap_slice(n1: &str, n2: &str, orient: Orientation) -> String {
    let remapped1 = remap_move(n1, orient);
    let remapped2 = remap_move(n2, orient);
    let Some((rf1, rd1)) = parse_face_dir(&remapped1) else {
        return format!("{}+{}", remapped1, remapped2);
    };
    let Some((rf2, rd2)) = parse_face_dir(&remapped2) else {
        return format!("{}+{}", remapped1, remapped2);
    };
    if are_opposite_faces(rf1, rf2) && rd1 == -rd2 {
        slice_name(rf1, rd1, rf2, rd2)
    } else {
        format!("{}+{}", remapped1, remapped2)
    }
}

// ========== Intermediate representation ==========

/// A single analyzed move after slice detection + orientation remap.
#[allow(dead_code)]
pub struct AnalyzedMove {
    pub body_label: String,
    pub remapped: String,
    /// Original body-frame raw move notations (for BLE cube).
    /// Face move: `vec!["L"]`, Slice: `vec!["F'", "B"]`, Double: `vec!["L", "L"]`
    pub body_raw: Vec<String>,
    pub t: f64,
    pub rel_t: f64,
    pub orient: String,
    pub before_dt: f64,
    pub after_orient: Option<String>,
    pub after_dt: Option<f64>,
}

/// Strip direction suffix, returning the base face/slice name.
/// "M'" -> "M", "U" -> "U", "S'" -> "S", "R2" -> "R"
pub fn strip_suffix(notation: &str) -> &str {
    if notation.ends_with('\'') || notation.ends_with('2') {
        &notation[..notation.len() - 1]
    } else {
        notation
    }
}

/// Two consecutive same-direction moves merge into a double.
/// "U" + "U" -> "U2", "M'" + "M'" -> "M2", "R" + "R" -> "R2".
/// Half-turns are direction-agnostic so both CW+CW and CCW+CCW give X2.
pub fn can_merge_double(a: &str, b: &str) -> bool {
    a == b
}

/// Convert "M'" or "M" into "M2".
pub fn to_double(notation: &str) -> String {
    format!("{}2", strip_suffix(notation))
}

/// Mathematical core rotation caused by a home-frame slice move.
/// Used for orientation bookkeeping (merged_orient tracking), NOT for remap.
/// BitCube M: centers cycle U→F→D→B (same direction as x').
/// BitCube M': centers cycle U→B→D→F (same direction as x).
pub fn slice_core_rotation(notation: &str) -> Option<&'static str> {
    match notation {
        "M" => Some("x'"), // M centers: U→F (same as x')
        "M'" => Some("x"), // M' centers: U→B (same as x)
        "M2" => Some("x2"),
        "S" => Some("z"),
        "S'" => Some("z'"),
        "S2" => Some("z2"),
        "E" => Some("y'"),
        "E'" => Some("y"),
        "E2" => Some("y2"),
        _ => None,
    }
}
