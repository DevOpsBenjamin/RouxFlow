use crate::bitcube::BitCube;
use std::time::{Instant, Duration};
use crate::move_indices::Move;
use rayon::prelude::*;

pub struct AISolver;

impl AISolver {
    pub fn find_fb_solutions(cube: &BitCube, count: usize) -> (Vec<Vec<String>>, Duration) {
        let start = Instant::now();
        let mut solutions = Vec::new();
        
        println!("      [AI Search] Parallel Thinking (Rayon Enabled)...");

        let moves = Move::ALL;

        // IDA* style search
        // We stop at depth 8 for benchmark
        for depth in 2..=8 {
            let start_depth = Instant::now();
            
            // Parallelize top level and aggregate solutions AND nodes
            let (depth_solutions, total_nodes): (Vec<Vec<Vec<Move>>>, Vec<usize>) = moves.into_par_iter()
                .map(|first_move| {
                    let mut local_solutions = Vec::new();
                    let mut work_cube = cube.clone();
                    let mut path = vec![first_move];
                    let mut nodes = 0;
                    
                    work_cube.apply_move_enum(first_move);
                    Self::dfs_numerical(
                        &mut work_cube, 
                        &mut path, 
                        depth, 
                        &mut local_solutions, 
                        count, 
                        &moves, 
                        first_move.face(), 
                        &mut nodes
                    );
                    (local_solutions, nodes)
                })
                .unzip();

            let depth_nodes: usize = total_nodes.into_iter().sum();
            let elapsed = start_depth.elapsed();
            
            // Add unique solutions
            let flattened: Vec<Vec<Move>> = depth_solutions.into_iter().flatten().collect();
            for sol in flattened {
                let sol_str = sol.iter().map(|m| m.as_str().to_string()).collect();
                if !solutions.contains(&sol_str) {
                    solutions.push(sol_str);
                }
            }

            println!("      [AI Search] Depth {} finished (Nodes: {}, Solutions Found: {}, Time: {:?})", 
                depth, depth_nodes, solutions.len(), elapsed);
            
            // Benchmarking usually ignores count limits to see full time
        }
        
        (solutions, start.elapsed())
    }

    fn dfs_numerical(
        cube: &mut BitCube,
        path: &mut Vec<Move>,
        limit: usize,
        solutions: &mut Vec<Vec<Move>>,
        count: usize,
        moves: &[Move],
        last_face: u8,
        nodes: &mut usize
    ) {
        *nodes += 1;
        
        // --- PRUNING PLACEHOLDER ---
        // If we had a table, we would check: 
        // if path.len() + table.estimate(cube) > limit { return; }
        
        if cube.is_fb_solved() {
            solutions.push(path.clone());
            return;
        }
        
        if path.len() >= limit { return; }

        for &m in moves {
            let face = m.face();
            if face == last_face { continue; }
            
            // Basic pruning: parallel moves (U after D, etc)
            if (face == 1 && last_face == 0) || (face == 5 && last_face == 4) { continue; }

            cube.apply_move_enum(m);
            path.push(m);
            
            Self::dfs_numerical(cube, path, limit, solutions, count, moves, face, nodes);
            
            path.pop();
            cube.apply_move_enum(m.inverse());
            
            if solutions.len() >= count { return; }
        }
    }
}
