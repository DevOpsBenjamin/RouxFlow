use super::facelet::{FaceletCube, Color};

pub struct RouxSolver;

impl RouxSolver {
    pub fn is_fb_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        let l_color = f[40];
        if !f[39..=44].iter().all(|&c| c == l_color) { return false; }
        if f[27] != f[30] || f[30] != f[33] { return false; } // D bar
        if f[21] != f[24] { return false; } // F bar
        if f[50] != f[53] { return false; } // B bar
        !is_opposite(l_color, f[27])
    }

    pub fn is_sb_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        let r_color = f[13];
        if !f[12..=17].iter().all(|&c| c == r_color) { return false; }
        if f[29] != f[32] || f[32] != f[35] { return false; } // D bar
        if f[23] != f[26] { return false; } // F bar
        if f[48] != f[51] { return false; } // B bar
        !is_opposite(r_color, f[29])
    }

    pub fn is_cmll_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        // Use the front-left corner as reference for top color
        let top_color = f[6]; 
        if ![0, 2, 8].iter().all(|&i| f[i] == top_color) { return false; }

        // Check if side labels match (permutation)
        if f[18] != f[20] { return false; } // F
        if f[9] != f[11] { return false; }   // R
        if f[36] != f[38] { return false; } // L
        if f[45] != f[47] { return false; } // B
        true
    }

    /// Check if UL and UR edges are in place (relative to centers L/R)
    pub fn is_ul_ur_placed(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        let l_color = f[40];
        let r_color = f[13];
        
        // UL: sticker on U face index 3, sticker on L face index 1
        // UR: sticker on U face index 5, sticker on R face index 1
        f[3] == f[4] && f[37] == l_color && f[5] == f[4] && f[10] == r_color
    }

    /// Check if the Last 4 Edges (L4E) are solved
    pub fn is_l4e_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        
        // First check: if the whole cube is uniform (solved), then L4E is solved
        let mut all_uniform = true;
        for face_idx in 0..6 {
            let start = face_idx * 9;
            let c = f[start];
            for i in 1..9 {
                if f[start + i] != c {
                    all_uniform = false;
                    break;
                }
            }
            if !all_uniform { break; }
        }
        if all_uniform { return true; }

        // Fallback: Check M-slice edges relative to centers
        // For now, if the cube is not fully finished, we check the standard orientation
        // (This will be improved later for full color neutrality)
        let d_color = f[31];
        let f_color = f[22];
        
        f[7] == f[22] && f[19] == f[4] &&  // UF
        f[31] == d_color && f[25] == f_color && // DF
        f[34] == d_color && // DB
        // UB check
        f[1] == f[46] && f[46] == f[4] 
    }

    /// Count misoriented edges for LSE EO phase (Bad edges)
    pub fn count_bad_edges(cube: &FaceletCube) -> usize {
        let f = &cube.facelets;
        let u_ref = f[4];
        let f_ref = f[22];
        let mut bad = 0;

        let edges = [
            (7, 19),   // UF
            (1, 46),   // UB
            (31, 25),  // DF
            (34, 52),  // DB
            (3, 37),   // UL
            (5, 10),   // UR
        ];

        for (st1, st2) in edges {
            let (c1, c2) = (f[st1], f[st2]);
            if is_bad_edge(c1, c2, u_ref, f_ref) {
                bad += 1;
            }
        }
        bad
    }
}

fn is_bad_edge(top_front: Color, side: Color, u_ref: Color, f_ref: Color) -> bool {
    // Standard EO rule: side color cannot be U/D color
    if side == u_ref || side == get_opposite(u_ref) { return true; }
    // If side is F/B, it depends on top/front
    if (side == f_ref || side == get_opposite(f_ref)) && (top_front == u_ref || top_front == get_opposite(u_ref)) {
        return false;
    }
    false
}

fn is_opposite(c1: Color, c2: Color) -> bool {
    c1 == get_opposite(c2)
}

fn get_opposite(c: Color) -> Color {
    match c {
        Color::White => Color::Yellow,
        Color::Yellow => Color::White,
        Color::Green => Color::Blue,
        Color::Blue => Color::Green,
        Color::Red => Color::Orange,
        Color::Orange => Color::Red,
    }
}
