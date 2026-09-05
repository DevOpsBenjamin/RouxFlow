use crate::BitCube;
use serde::Serialize;

/// Roux method step labels.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum RouxStep {
    FB,
    SB,
    CMLL,
    LSE,
}

/// A single detected step segment within a solve.
#[derive(Debug, Clone, Serialize)]
pub struct StepSegment {
    pub step: RouxStep,
    pub start_move: usize,
    pub end_move: usize,
    pub move_count: usize,
    pub time_ms: Option<u32>,
}

/// Complete analysis of a Roux solve.
#[derive(Debug, Clone, Serialize)]
pub struct SolveAnalysis {
    pub steps: Vec<StepSegment>,
    pub orientation: Option<String>,
    pub total_moves: usize,
}

// Virtual rotations to normalize any FB orientation to the standard (Orange/Yellow) position.
#[derive(Debug, Clone, Copy)]
enum Orientation {
    Identity,
    Y,
    Yp,
    Y2,
    X2,
    X2Y,
    X2Yp,
    X2Y2,
}

impl Orientation {
    fn apply(&self, cube: &mut BitCube) {
        match self {
            Orientation::Identity => {}
            Orientation::Y => cube.rot_y(),
            Orientation::Yp => cube.rot_y_prime(),
            Orientation::Y2 => cube.rot_y2(),
            Orientation::X2 => cube.rot_x2(),
            Orientation::X2Y => {
                cube.rot_x2();
                cube.rot_y();
            }
            Orientation::X2Yp => {
                cube.rot_x2();
                cube.rot_y_prime();
            }
            Orientation::X2Y2 => {
                cube.rot_x2();
                cube.rot_y2();
            }
        }
    }
}

const ORIENTATIONS: [(Orientation, &str); 8] = [
    (Orientation::Identity, "Orange/Yellow"),
    (Orientation::Y, "Blue/Yellow"),
    (Orientation::Yp, "Green/Yellow"),
    (Orientation::Y2, "Red/Yellow"),
    (Orientation::X2, "Orange/White"),
    (Orientation::X2Y, "Blue/White"),
    (Orientation::X2Yp, "Green/White"),
    (Orientation::X2Y2, "Red/White"),
];

/// Analyze a Roux solve by detecting FB, SB, CMLL, and LSE step boundaries.
///
/// `scramble` — space-separated scramble string (e.g. "R U R' F2 D")
/// `moves` — ordered solve moves
/// `timed_moves` — optional (move_str, cumulative_ms) pairs for time-per-step calculation
pub fn analyze_solve_legacy(
    scramble: &str,
    moves: &[String],
    timed_moves: Option<&[(String, u32)]>,
) -> SolveAnalysis {
    let mut cube = BitCube::new_solved();

    // Apply scramble
    for token in scramble.split_whitespace() {
        cube.apply_move(token);
    }

    // Phase state machine
    let phases = [RouxStep::FB, RouxStep::SB, RouxStep::CMLL, RouxStep::LSE];
    let mut phase_idx: usize = 0;
    let mut detected_orientation: Option<(Orientation, String)> = None;
    let mut steps: Vec<StepSegment> = Vec::new();
    let mut phase_start: usize = 0;

    // Check if phases are already solved after scramble (before any solve move)
    loop {
        if phase_idx >= phases.len() {
            break;
        }
        let solved = check_phase(&cube, phases[phase_idx], &detected_orientation);
        if let Some(orient) = solved {
            if detected_orientation.is_none() {
                detected_orientation = Some(orient);
            }
            steps.push(StepSegment {
                step: phases[phase_idx],
                start_move: 0,
                end_move: 0,
                move_count: 0,
                time_ms: Some(0),
            });
            phase_idx += 1;
        } else {
            break;
        }
    }

    // Apply each solve move and check phase transitions
    for (i, move_str) in moves.iter().enumerate() {
        cube.apply_move(move_str);

        // Check current + subsequent phases (a single move can complete multiple phases)
        loop {
            if phase_idx >= phases.len() {
                break;
            }
            let solved = check_phase(&cube, phases[phase_idx], &detected_orientation);
            if let Some(orient) = solved {
                if detected_orientation.is_none() {
                    detected_orientation = Some(orient);
                }
                let end = i + 1;
                let time = timed_moves.map(|tm| {
                    let end_time = if end > 0 && end <= tm.len() {
                        tm[end - 1].1
                    } else {
                        0
                    };
                    let start_time = if phase_start > 0 && phase_start <= tm.len() {
                        tm[phase_start - 1].1
                    } else {
                        0
                    };
                    end_time.saturating_sub(start_time)
                });
                steps.push(StepSegment {
                    step: phases[phase_idx],
                    start_move: phase_start,
                    end_move: end,
                    move_count: end - phase_start,
                    time_ms: time,
                });
                phase_start = end;
                phase_idx += 1;
            } else {
                break;
            }
        }
    }

    SolveAnalysis {
        orientation: detected_orientation.map(|(_, name)| name),
        total_moves: moves.len(),
        steps,
    }
}

/// Check whether a specific phase is solved on the given cube state.
/// Returns Some((orientation, name)) if solved, None otherwise.
/// For FB, tries all 8 orientations. For later phases, uses the detected orientation.
fn check_phase(
    cube: &BitCube,
    phase: RouxStep,
    detected_orientation: &Option<(Orientation, String)>,
) -> Option<(Orientation, String)> {
    match phase {
        RouxStep::FB => {
            for &(orient, name) in &ORIENTATIONS {
                let mut test = cube.clone();
                orient.apply(&mut test);
                if test.is_fb_solved() {
                    return Some((orient, name.to_string()));
                }
            }
            None
        }
        RouxStep::SB => {
            let (orient, name) = detected_orientation.as_ref()?;
            let mut test = cube.clone();
            orient.apply(&mut test);
            if test.is_sb_solved() {
                Some((*orient, name.clone()))
            } else {
                None
            }
        }
        RouxStep::CMLL => {
            let (orient, name) = detected_orientation.as_ref()?;
            let mut test = cube.clone();
            orient.apply(&mut test);
            if test.is_cmll_solved() {
                Some((*orient, name.clone()))
            } else {
                None
            }
        }
        RouxStep::LSE => {
            let (orient, name) = detected_orientation.as_ref()?;
            let mut test = cube.clone();
            orient.apply(&mut test);
            if test.is_solved() {
                Some((*orient, name.clone()))
            } else {
                None
            }
        }
    }
}
