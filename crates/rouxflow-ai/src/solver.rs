use crate::pruning::PruningTable;
use crate::BitCube;
use crate::Move;
use rayon::prelude::*;
use std::time::{Duration, Instant};

pub struct Solution {
    pub moves: Vec<String>,
    pub orientation_name: String, // e.g. "Orange/Yellow"
}

// Colors: U=0, D=1, F=2, B=3, R=4, L=5
#[allow(dead_code)]
const COLORS: [&str; 6] = ["White", "Yellow", "Green", "Blue", "Red", "Orange"];

// Local definition of normalization operations since Move doesn't have rotations
#[derive(Debug, Clone, Copy)]
enum NormalizationOps {
    Identity,
    Y,
    Yp,
    Y2,
    X2,
    X2Y,
    X2Yp,
    X2Y2,
}

// Helper to get transformations that bring various Left/Down pairs to the Standard (Orange, Yellow) position.
fn get_normalization_transforms() -> Vec<(NormalizationOps, String)> {
    vec![
        // --- Yellow Bottom ---
        (NormalizationOps::Identity, "Orange/Yellow".to_string()),
        (NormalizationOps::Y, "Blue/Yellow".to_string()),
        (NormalizationOps::Yp, "Green/Yellow".to_string()),
        (NormalizationOps::Y2, "Red/Yellow".to_string()),
        // --- White Bottom ---
        (NormalizationOps::X2, "Orange/White".to_string()),
        (NormalizationOps::X2Y, "Blue/White".to_string()),
        (NormalizationOps::X2Yp, "Green/White".to_string()),
        (NormalizationOps::X2Y2, "Red/White".to_string()),
    ]
}

impl NormalizationOps {
    fn apply(&self, cube: &mut BitCube) {
        match self {
            NormalizationOps::Identity => {}
            NormalizationOps::Y => cube.rot_y(),
            NormalizationOps::Yp => cube.rot_y_prime(),
            NormalizationOps::Y2 => cube.rot_y2(),
            NormalizationOps::X2 => cube.rot_x2(),
            NormalizationOps::X2Y => {
                cube.rot_x2();
                cube.rot_y();
            }
            NormalizationOps::X2Yp => {
                cube.rot_x2();
                cube.rot_y_prime();
            }
            NormalizationOps::X2Y2 => {
                cube.rot_x2();
                cube.rot_y2();
            }
        }
    }
}

pub struct AISolver;

impl AISolver {
    pub fn find_fb_solutions_optimized(
        cube: &BitCube,
        max_depth: u8,
        table: &PruningTable,
    ) -> (Vec<Solution>, Duration) {
        let start = Instant::now();
        let transforms = get_normalization_transforms();

        let _all_solutions: Vec<Solution> = Vec::new();
        let moves = Move::ALL;

        for depth in 1..=max_depth {
            let found_sols: Vec<Solution> = moves
                .par_iter()
                .flat_map(|&first_move| {
                    let mut local_solutions = Vec::new();
                    let mut next = cube.clone();
                    next.apply_move_enum(first_move);

                    // --- Pruning Check (Virtual Rotations) ---
                    let mut best_potential = 99;

                    for (trans, _) in &transforms {
                        let mut virt = next.clone();
                        trans.apply(&mut virt);

                        let h = table.get_dist(&virt);
                        if h < best_potential {
                            best_potential = h;
                        }
                    }

                    if best_potential > (depth - 1) {
                        return local_solutions;
                    }

                    let mut path = Vec::with_capacity(depth as usize);
                    path.push(first_move.as_str().to_string());

                    Self::dfs_multi_goal(
                        &next,
                        depth - 1,
                        &mut path,
                        &mut local_solutions,
                        table,
                        &transforms,
                    );

                    local_solutions
                })
                .collect();

            if !found_sols.is_empty() {
                return (found_sols, start.elapsed());
            }
        }

        (vec![], start.elapsed())
    }

    fn dfs_multi_goal(
        cube: &BitCube,
        remaining_depth: u8,
        path: &mut Vec<String>,
        solutions: &mut Vec<Solution>,
        table: &PruningTable,
        transforms: &Vec<(NormalizationOps, String)>,
    ) {
        let mut best_potential = 99;
        let mut solved_targets = Vec::new();

        for (trans, name) in transforms {
            let mut virt = cube.clone();
            trans.apply(&mut virt);

            let h = table.get_dist(&virt);
            if h == 0 {
                solved_targets.push(name.clone());
            }
            if h < best_potential {
                best_potential = h;
            }
        }

        if !solved_targets.is_empty() {
            for name in solved_targets {
                solutions.push(Solution {
                    moves: path.clone(),
                    orientation_name: name,
                });
            }
            return;
        }

        if remaining_depth == 0 {
            return;
        }
        if best_potential > remaining_depth {
            return;
        }

        let last_move_str = path.last().unwrap();
        let last_face = Self::get_face(last_move_str);

        let moves = Move::ALL;
        for &m in &moves {
            let face = m.face() as usize;
            if face == last_face {
                continue;
            }
            if (face ^ 1 == last_face) && (face < last_face) {
                continue;
            }

            let mut next = cube.clone();
            next.apply_move_enum(m);

            path.push(m.as_str().to_string());
            Self::dfs_multi_goal(
                &next,
                remaining_depth - 1,
                path,
                solutions,
                table,
                transforms,
            );
            path.pop();
        }
    }

    fn get_face(m: &str) -> usize {
        match m.chars().next().unwrap() {
            'U' => 0,
            'D' => 1,
            'F' => 2,
            'B' => 3,
            'R' => 4,
            'L' => 5,
            _ => 99,
        }
    }

    #[allow(dead_code)]
    fn dfs_numerical(
        cube: &mut BitCube,
        path: &mut Vec<Move>,
        limit: usize,
        solutions: &mut Vec<Vec<Move>>,
        count: usize,
        moves: &[Move],
        last_face: u8,
        nodes: &mut usize,
        table: Option<&crate::pruning::PruningTable>,
    ) {
        *nodes += 1;

        if let Some(tbl) = table {
            let dist = tbl.get_dist(cube);
            if path.len() + dist as usize > limit {
                return;
            }
        }

        if cube.is_fb_solved() {
            solutions.push(path.clone());
            return;
        }

        if path.len() >= limit {
            return;
        }

        for &m in moves {
            let face = m.face();
            if face == last_face {
                continue;
            }

            // Pruning for commuting faces (e.g., allow U-then-D, prune D-then-U)
            // Convention: always play the lower index face first when they commute.
            // Faces: U:0, D:1 | L:2, R:3 | F:4, B:5
            if (face ^ 1 == last_face) && (face < last_face) {
                continue;
            }

            cube.apply_move_enum(m);
            path.push(m);

            Self::dfs_numerical(
                cube, path, limit, solutions, count, moves, face, nodes, table,
            );

            path.pop();
            cube.apply_move_enum(m.inverse());

            if solutions.len() >= count {
                return;
            }
        }
    }
}
