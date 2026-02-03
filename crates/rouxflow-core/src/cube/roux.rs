use super::facelet::{FaceletCube, Color};

pub struct RouxSolver;

impl RouxSolver {
    /// Detect if the First Block (FB) is solved (1x2x3 on the Left face)
    /// Independent of M-slice center alignment.
    pub fn is_fb_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        let l_color = f[40]; // L center (stable)
        
        // 1. L-face part (mid and bottom rows)
        let l_indices = [39, 41, 42, 43, 44];
        if !l_indices.iter().all(|&i| f[i] == l_color) {
            return false;
        }

        // 2. The block stickers on D, F, B must match each other in bars
        // D-face Left column (27, 30, 33)
        if f[27] != f[30] || f[30] != f[33] { return false; }
        // F-face Left-Bottom (21, 24)
        if f[21] != f[24] { return false; }
        // B-face Right-Bottom (50, 53)
        if f[50] != f[53] { return false; }

        // 3. Verify color consistency (e.g. if L is Orange, D-bar can't be Red)
        // For now, simple check: they shouldn't be the opposite of L
        if is_opposite(l_color, f[27]) { return false; }

        true
    }

    /// Detect if Second Block (SB) is solved (1x2x3 on the Right face)
    pub fn is_sb_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        let r_color = f[13]; // R center (stable)
        
        // 1. R-face part (mid and bottom rows)
        let r_indices = [12, 14, 15, 16, 17];
        if !r_indices.iter().all(|&i| f[i] == r_color) {
            return false;
        }

        // 2. Bars on D, F, B
        if f[29] != f[32] || f[32] != f[35] { return false; }
        if f[23] != f[26] { return false; }
        if f[48] != f[51] { return false; }

        if is_opposite(r_color, f[29]) { return false; }

        true
    }

    /// Detect if Top Corners (CMLL) are solved relative to each other
    pub fn is_cmll_solved(cube: &FaceletCube) -> bool {
        let f = &cube.facelets;
        let u_color = f[4]; // U center
        
        // 1. Orientation: All 4 top corners must have U-color on top
        let u_corners = [0, 2, 6, 8];
        if !u_corners.iter().all(|&i| f[i] == u_color) {
            return false;
        }

        // 2. Permutation: Check if side stickers of corners match each other
        if f[18] != f[20] { return false; } // F face top corners
        if f[9] != f[11] { return false; }   // R face top corners
        if f[36] != f[38] { return false; } // L face top corners
        if f[45] != f[47] { return false; } // B face top corners

        true
    }
}

fn is_opposite(c1: Color, c2: Color) -> bool {
    match (c1, c2) {
        (Color::White, Color::Yellow) | (Color::Yellow, Color::White) => true,
        (Color::Green, Color::Blue) | (Color::Blue, Color::Green) => true,
        (Color::Red, Color::Orange) | (Color::Orange, Color::Red) => true,
        _ => false,
    }
}
