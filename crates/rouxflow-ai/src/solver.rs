use crate::bitcube::BitCube;
use std::time::{Instant, Duration};

pub struct AISolver;

impl AISolver {
    /// Accelerated First Block (FB) search using BitCube
    pub fn find_fb_solutions(cube: &BitCube, count: usize) -> (Vec<Vec<String>>, Duration) {
        let start = Instant::now();
        let mut solutions = Vec::new();
        let mut work_cube = cube.clone();
        
        println!("      [AI Search] Thinking...");

        // Moveset for FB (Roux native style)
        let moves = [
            "U", "U'", "U2", "D", "D'", "D2", "L", "L'", "L2", 
            "F", "F'", "F2", "B", "B'", "B2", "M", "M'", "M2",
            "r", "r'", "r2"
        ];

        // IDA* style search (iterative deepening) - Limited to depth 5 for testing
        for depth in 0..=5 {
            let start_depth = Instant::now();
            let mut path = Vec::new();
            let mut nodes = 0;
            let prev_count = solutions.len();

            Self::dfs(&mut work_cube, &mut path, depth, &mut solutions, count, &moves, "", &mut nodes);
            
            let elapsed = start_depth.elapsed();
            if solutions.len() > prev_count {
                for i in prev_count..solutions.len() {
                    println!("      [AI Search] Solution {} found at depth {} (Nodes: {}, Time: {:?})", 
                        i + 1, depth, nodes, elapsed);
                }
            } else {
                println!("      [AI Search] Depth {} finished (Nodes: {}, Time: {:?})", depth, nodes, elapsed);
            }

            if solutions.len() >= count { break; }
        }
        
        (solutions, start.elapsed())
    }

    fn dfs(
        cube: &mut BitCube,
        path: &mut Vec<String>,
        limit: usize,
        solutions: &mut Vec<Vec<String>>,
        count: usize,
        moves: &[&'static str],
        last_face: &str,
        nodes: &mut usize
    ) {
        *nodes += 1;
        if solutions.len() >= count { return; }
        
        if cube.is_fb_solved() {
            solutions.push(path.clone());
            return;
        }
        
        if path.len() >= limit { return; }

        for &m in moves {
            let face = &m[0..1];
            if face == last_face { continue; }
            // Basic pruning: parallel moves
            if (face == "D" && last_face == "U") || (face == "B" && last_face == "F") || (face == "R" && last_face == "L") { continue; }

            // Move
            cube.apply_move(m);
            path.push(m.to_string());
            
            Self::dfs(cube, path, limit, solutions, count, moves, face, nodes);
            
            // Backtrack
            path.pop();
            cube.apply_move(&Self::invert_move(m));
            
            if solutions.len() >= count { return; }
        }
    }

    fn invert_move(m: &str) -> String {
        if m.ends_with('2') {
            m.to_string()
        } else if m.ends_with('\'') {
            m[0..m.len()-1].to_string()
        } else {
            format!("{}'", m)
        }
    }
}
