use crate::BitCube;
use std::time::{Instant, Duration};
use crate::Move;
use rayon::prelude::*;

pub struct AISolver;

impl AISolver {
    pub fn find_fb_solutions(cube: &BitCube, count: usize) -> (Vec<Vec<String>>, Duration) {
        let start = Instant::now();
        let mut solutions = Vec::new();
        println!("      [AI Search] Parallel Thinking (Rayon Enabled)...");

        let moves = Move::ALL;

        // IDA* style search
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
                        &mut nodes,
                        None 
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
            
            if solutions.len() >= count { break; }
        }
        
        (solutions, start.elapsed())
    }

    pub fn find_fb_solutions_optimized(cube: &BitCube, count: usize, table: &crate::pruning::PruningTable) -> (Vec<Vec<String>>, Duration) {
        let start = Instant::now();
        let mut solutions = Vec::new();
        println!("      [AI Search] Optimized Thinking (Pruning Table Enabled)...");

        let moves = Move::ALL;

        for depth in 2..=10 {
            let start_depth = Instant::now();
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
                        &mut nodes,
                        Some(table)
                    );
                    (local_solutions, nodes)
                })
                .unzip();

            let depth_nodes: usize = total_nodes.into_iter().sum();
            let flattened: Vec<Vec<Move>> = depth_solutions.into_iter().flatten().collect();
            for sol in flattened {
                let sol_str = sol.iter().map(|m| m.as_str().to_string()).collect();
                if !solutions.contains(&sol_str) { solutions.push(sol_str); }
            }

            println!("      [AI Search] Depth {} finished (Nodes: {}, Solutions Found: {}, Time: {:?})", 
                depth, depth_nodes, solutions.len(), start_depth.elapsed());
            
            if solutions.len() >= count { break; }
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
        nodes: &mut usize,
        table: Option<&crate::pruning::PruningTable>
    ) {
        *nodes += 1;
        
        if let Some(tbl) = table {
            let dist = tbl.get_dist(cube);
            if path.len() + dist as usize > limit { return; }
        }
        
        if cube.is_fb_solved() {
            solutions.push(path.clone());
            return;
        }
        
        if path.len() >= limit { return; }

        for &m in moves {
            let face = m.face();
            if face == last_face { continue; }
            
            // Pruning for commuting faces (e.g., allow U-then-D, prune D-then-U)
            // Convention: always play the lower index face first when they commute.
            // Faces: U:0, D:1 | L:2, R:3 | F:4, B:5
            if (face ^ 1 == last_face) && (face < last_face) { continue; }
            
            cube.apply_move_enum(m);
            path.push(m);
            
            Self::dfs_numerical(cube, path, limit, solutions, count, moves, face, nodes, table);
            
            path.pop();
            cube.apply_move_enum(m.inverse());
            
            if solutions.len() >= count { return; }
        }
    }
}
