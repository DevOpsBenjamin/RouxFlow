use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::VecDeque;

use rouxflow_bitboard::{BitCube, Move};

// --- HELPER: Compute FB Hash directly (could act as indexer too) ---
// We copy the indexer logic here or import it if we move pruning to bitboard crate.
// For now, we COPY the indexer logic because it's specific to AI pruning.

const FB_SIZE: usize = 5_322_240;

pub fn get_fb_index(cube: &BitCube, target_l: usize, target_d: usize, target_f: usize, target_b: usize) -> usize {
    let edge_slots = [
        (7, 19), (3, 39), (1, 46), (5, 10), 
        (28, 25), (30, 43), (34, 52), (32, 16),
        (21, 41), (23, 12), (48, 37), (50, 14), 
    ];
    let mut edge_pos = [0usize; 3];
    let mut edge_ori = [0u8; 3];
    let pieces_e = [(target_l, target_d), (target_l, target_f), (target_l, target_b)];

    for (i, &(c1, c2)) in pieces_e.iter().enumerate() {
        for (slot, &(s1, s2)) in edge_slots.iter().enumerate() {
            if (cube.boards[c1] & (1 << s1)) != 0 && (cube.boards[c2] & (1 << s2)) != 0 {
                edge_pos[i] = slot; edge_ori[i] = 0; break;
            }
            if (cube.boards[c1] & (1 << s2)) != 0 && (cube.boards[c2] & (1 << s1)) != 0 {
                edge_pos[i] = slot; edge_ori[i] = 1; break;
            }
        }
    }

    let corner_slots = [
        (6, 18, 38), (8, 9, 20), (2, 11, 45), (0, 47, 36), 
        (27, 44, 24), (29, 15, 26), (35, 17, 51), (33, 42, 53), 
    ];
    let pieces_c = [
        (target_l, target_d, target_f), 
        (target_l, target_d, target_b)
    ];
    let mut corn_pos = [0usize; 2];
    let mut corn_ori = [0u8; 2];

    for (i, &(c1, c2, c3)) in pieces_c.iter().enumerate() {
        for (slot, &(s1, s2, s3)) in corner_slots.iter().enumerate() {
            if (cube.boards[c1] & (1 << s1)) != 0 && (cube.boards[c2] & (1 << s2)) != 0 && (cube.boards[c3] & (1 << s3)) != 0 {
                corn_pos[i] = slot; corn_ori[i] = 0; break;
            }
            if (cube.boards[c1] & (1 << s2)) != 0 && (cube.boards[c2] & (1 << s3)) != 0 && (cube.boards[c3] & (1 << s1)) != 0 {
                corn_pos[i] = slot; corn_ori[i] = 1; break;
            }
            if (cube.boards[c1] & (1 << s3)) != 0 && (cube.boards[c2] & (1 << s1)) != 0 && (cube.boards[c3] & (1 << s2)) != 0 {
                corn_pos[i] = slot; corn_ori[i] = 2; break;
            }
        }
    }

    let mut e_idx = (edge_pos[0] * 11 * 10 + edge_pos[1] * 10 + edge_pos[2]) * 8;
    e_idx += (edge_ori[0] as usize * 4 + edge_ori[1] as usize * 2 + edge_ori[2] as usize);
    let mut c_idx = (corn_pos[0] * 7 + corn_pos[1]) * 9;
    c_idx += (corn_ori[0] as usize * 3 + corn_ori[1] as usize);
    e_idx * (56 * 9) + c_idx
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("fb_table.bin");

    println!("cargo:rerun-if-changed=build.rs");
    let start_time = std::time::Instant::now(); // Start timer
    
    // Use new_solved from bitboard crate directly!
    let mut table = vec![255u8; FB_SIZE];
    let mut queue = VecDeque::new();
    
    // Add ALL "Pseudo-Solved" states to queue with dist 0
    // This ensures the heuristic matches our relaxed goal (Misaligned M-slice is OK)
    let solved_base = BitCube::new_solved();
    let m_moves = [None, Some(Move::M), Some(Move::Mp), Some(Move::M2)]; 

    for m_opt in m_moves {
        let mut c = solved_base.clone();
        if let Some(m) = m_opt {
            c.apply_move_enum(m);
        }
        let idx = get_fb_index(&c, 5, 1, 2, 3);
        if table[idx] == 255 {
            table[idx] = 0;
            queue.push_back((c, 0u8));
        }
    }
    
    println!("cargo:warning=[AI Build] Table initialized with Pseudo-Blocks.");

    let moves = Move::ALL;
    let mut visited = 1;

    while let Some((cube, dist)) = queue.pop_front() {
        if dist >= 9 { continue; }

        for &m in &moves {
            let mut next_cube = cube.clone();
            next_cube.apply_move_enum(m);
            let idx = get_fb_index(&next_cube, 5, 1, 2, 3);

            if table[idx] == 255 {
                table[idx] = dist + 1;
                queue.push_back((next_cube, dist + 1));
                visited += 1;
            }
        }
    }

    let mut f = File::create(&dest_path).unwrap();
    f.write_all(&table).unwrap();
    
    println!("cargo:warning=[AI Build] Table generated in {:.2?}", start_time.elapsed());
}
