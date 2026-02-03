use rouxflow_core::cube::facelet::{FaceletCube, Color};
use crate::move_indices::Move;

/// A Rubik's Cube represented by bitboards for high-performance search.
/// Each color (White, Yellow, Green, Blue, Red, Orange) has a 64-bit integer.
/// Indices 0-53 are used (out of 64 bits).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitCube {
    pub boards: [u64; 6],
}

impl BitCube {
    pub fn new() -> Self {
        Self::from_facelet(&FaceletCube::new())
    }

    pub fn from_facelet(facelet_cube: &FaceletCube) -> Self {
        let mut boards = [0u64; 6];
        for (i, &color) in facelet_cube.facelets.iter().enumerate() {
            boards[color as usize] |= 1 << i;
        }
        BitCube { boards }
    }

    /// Optimized move application using numeric enum
    pub fn apply_move_enum(&mut self, m: Move) {
        match m {
            Move::U => self.rotate_u(), Move::Up => self.rotate_u_prime(), Move::U2 => { self.rotate_u(); self.rotate_u(); },
            Move::D => self.rotate_d(), Move::Dp => self.rotate_d_prime(), Move::D2 => { self.rotate_d(); self.rotate_d(); },
            Move::L => self.rotate_l(), Move::Lp => self.rotate_l_prime(), Move::L2 => { self.rotate_l(); self.rotate_l(); },
            Move::R => self.rotate_r(), Move::Rp => self.rotate_r_prime(), Move::R2 => { self.rotate_r(); self.rotate_r(); },
            Move::F => self.rotate_f(), Move::Fp => self.rotate_f_prime(), Move::F2 => { self.rotate_f(); self.rotate_f(); },
            Move::B => self.rotate_b(), Move::Bp => self.rotate_b_prime(), Move::B2 => { self.rotate_b(); self.rotate_b(); },
            Move::M => self.rotate_m(), Move::Mp => self.rotate_m_prime(), Move::M2 => { self.rotate_m(); self.rotate_m(); },
            Move::r => { self.rotate_r(); self.rotate_m_prime(); },
            Move::rp => { self.rotate_r_prime(); self.rotate_m(); },
            Move::r2 => { self.rotate_r(); self.rotate_r(); self.rotate_m(); self.rotate_m(); },
        }
    }

    pub fn apply_move(&mut self, move_str: &str) {
        let clean_move = move_str.trim();
        if clean_move.is_empty() { return; }

        let (m, count) = if clean_move.ends_with("2'") || clean_move.ends_with('2') {
            (&clean_move[0..clean_move.len()-1], 2)
        } else if clean_move.ends_with('\'') {
            (&clean_move[0..clean_move.len()-1], 3)
        } else {
            (clean_move, 1)
        };

        for _ in 0..count {
            match m {
                "U" => self.rotate_u(),
                "D" => self.rotate_d(),
                "L" => self.rotate_l(),
                "R" => self.rotate_r(),
                "F" => self.rotate_f(),
                "B" => self.rotate_b(),
                "M" => self.rotate_m(),
                "S" => self.rotate_s(),
                "E" => self.rotate_e(),
                "r" => { self.rotate_r(); self.rotate_m_prime(); },
                "l" => { self.rotate_l(); self.rotate_m(); },
                "x" => { self.rotate_r(); self.rotate_m_prime(); self.rotate_l_prime(); },
                "y" => { self.rotate_u(); self.rotate_e_prime(); self.rotate_d_prime(); },
                "z" => { self.rotate_f(); self.rotate_s(); self.rotate_b_prime(); },
                _ => {}
            }
        }
    }

    // Face rotation bit indices (unrolled for speed)
    // 012 -> 258, 345 -> 147, 678 -> 036
    #[inline(always)]
    fn rotate_face_bits(v: u64) -> u64 {
        // v bits: 0 1 2 / 3 4 5 / 6 7 8
        let corners = ((v & 0x01) << 2) | ((v & 0x04) << 6) | ((v & 0x100) >> 2) | ((v & 0x40) >> 6);
        let edges   = ((v & 0x02) << 4) | ((v & 0x20) << 2) | ((v & 0x80) >> 4) | ((v & 0x08) >> 2);
        let center  = v & 0x10;
        corners | edges | center
    }

    fn rotate_u(&mut self) {
        const SIDE_MASK: u64 = (0x7 << 18) | (0x7 << 9) | (0x7 << 45) | (0x7 << 36);
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(SIDE_MASK | 0x1FF);
            next |= Self::rotate_face_bits(old & 0x1FF);
            next |= ((old >> 9) & 0x7) << 18; // R -> F
            next |= ((old >> 18) & 0x7) << 36; // F -> L
            next |= ((old >> 36) & 0x7) << 45; // L -> B
            next |= ((old >> 45) & 0x7) << 9;  // B -> R
            *b = next;
        }
    }

    fn rotate_d(&mut self) {
        const SIDE_MASK: u64 = (0x7 << 24) | (0x7 << 15) | (0x7 << 51) | (0x7 << 42);
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(SIDE_MASK | (0x1FF << 27));
            next |= Self::rotate_face_bits((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 42) & 0x7) << 51; // L -> B
            next |= ((old >> 51) & 0x7) << 15; // B -> R
            next |= ((old >> 15) & 0x7) << 24; // R -> F
            next |= ((old >> 24) & 0x7) << 42; // F -> L
            *b = next;
        }
    }

    fn rotate_l(&mut self) {
        const U_COL: u64 = 0x49; // 0,3,6
        const F_COL: u64 = U_COL << 18;
        const D_COL: u64 = U_COL << 27;
        const B_COL: u64 = (1 << 47) | (1 << 50) | (1 << 53);
        const L_FACE_MASK: u64 = 0x1FF << 36;
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(U_COL | F_COL | D_COL | B_COL | L_FACE_MASK);
            next |= Self::rotate_face_bits((old >> 36) & 0x1FF) << 36;
            next |= (old & U_COL) << 18; // U -> F
            next |= (old & F_COL) << 9;  // F -> D
            // D -> B reversed
            next |= ((old >> 27) & 1) << 53; next |= ((old >> 30) & 1) << 50; next |= ((old >> 33) & 1) << 47;
            // B -> U reversed
            next |= ((old >> 53) & 1) << 0; next |= ((old >> 50) & 1) << 3; next |= ((old >> 47) & 1) << 6;
            *b = next;
        }
    }

    fn rotate_r(&mut self) {
        const U_COL: u64 = 0x49 << 2; // 2,5,8
        const F_COL: u64 = U_COL << 18;
        const D_COL: u64 = U_COL << 27;
        const B_COL: u64 = (1 << 45) | (1 << 48) | (1 << 51);
        const R_FACE_MASK: u64 = 0x1FF << 9;
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(U_COL | F_COL | D_COL | B_COL | R_FACE_MASK);
            next |= Self::rotate_face_bits((old >> 9) & 0x1FF) << 9;
            // Cycle: U -> B, B -> D, D -> F, F -> U
            next |= ((old >> 2) & 1) << 51; next |= ((old >> 5) & 1) << 48; next |= ((old >> 8) & 1) << 45;
            next |= ((old >> 51) & 1) << 29; next |= ((old >> 48) & 1) << 32; next |= ((old >> 45) & 1) << 35;
            next |= (old & D_COL) >> 9;  // D -> F
            next |= (old & F_COL) >> 18; // F -> U
            *b = next;
        }
    }

    fn rotate_f(&mut self) {
        const U_ROW: u64 = 0x7 << 6;   // 6,7,8
        const R_COL: u64 = 0x49 << 9;  // 9,12,15
        const D_ROW: u64 = (1<<27)|(1<<28)|(1<<29); // 27,28,29
        const L_COL: u64 = 0x49 << 38; // 38,41,44
        const F_FACE_MASK: u64 = 0x1FF << 18;
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(U_ROW | R_COL | D_ROW | L_COL | F_FACE_MASK);
            next |= Self::rotate_face_bits((old >> 18) & 0x1FF) << 18;
            // Cycle: U678->R91215, R91215->D292827, D292827->L444138, L444138->U678
            next |= ((old >> 6) & 1) << 9; next |= ((old >> 7) & 1) << 12; next |= ((old >> 8) & 1) << 15;
            next |= ((old >> 9) & 1) << 29; next |= ((old >> 12) & 1) << 28; next |= ((old >> 15) & 1) << 27;
            next |= ((old >> 29) & 1) << 44; next |= ((old >> 28) & 1) << 41; next |= ((old >> 27) & 1) << 38;
            next |= ((old >> 44) & 1) << 6; next |= ((old >> 41) & 1) << 7; next |= ((old >> 38) & 1) << 8;
            *b = next;
        }
    }

    fn rotate_b(&mut self) {
        const U_ROW: u64 = 0x7; // 0,1,2
        const L_COL: u64 = 0x49 << 36; // 36,39,42
        const D_ROW: u64 = (1<<33)|(1<<34)|(1<<35); // 33,34,35
        const R_COL: u64 = 0x49 << 11; // 11,14,17
        const B_FACE_MASK: u64 = 0x1FF << 45;
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(U_ROW | L_COL | D_ROW | R_COL | B_FACE_MASK);
            next |= Self::rotate_face_bits((old >> 45) & 0x1FF) << 45;
            // Cycle: U012->L363942, L363942->D353433, D353433->R171411, R171411->U012
            next |= ((old >> 2) & 1) << 36; next |= ((old >> 1) & 1) << 39; next |= ((old >> 0) & 1) << 42;
            next |= ((old >> 36) & 1) << 33; next |= ((old >> 39) & 1) << 34; next |= ((old >> 42) & 1) << 35;
            next |= ((old >> 33) & 1) << 17; next |= ((old >> 34) & 1) << 14; next |= ((old >> 35) & 1) << 11;
            next |= ((old >> 17) & 1) << 2; next |= ((old >> 14) & 1) << 1; next |= ((old >> 11) & 1) << 0;
            *b = next;
        }
    }

    fn rotate_m(&mut self) {
        const U_COL: u64 = 0x49 << 1; 
        const F_COL: u64 = U_COL << 18;
        const D_COL: u64 = U_COL << 27;
        const B_COL: u64 = (1 << 46) | (1 << 49) | (1 << 52);
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(U_COL | F_COL | D_COL | B_COL);
            next |= (old & U_COL) << 18;
            next |= (old & F_COL) << 9;
            next |= ((old >> 28) & 1) << 52; next |= ((old >> 31) & 1) << 49; next |= ((old >> 34) & 1) << 46;
            next |= ((old >> 52) & 1) << 1; next |= ((old >> 49) & 1) << 4; next |= ((old >> 46) & 1) << 7;
            *b = next;
        }
    }

    fn rotate_s(&mut self) {
        const U_ROW: u64 = 0x7 << 3; 
        const R_COL: u64 = 0x49 << 10;
        const D_ROW: u64 = 0x7 << 30;
        const L_COL: u64 = 0x49 << 37;
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(U_ROW | R_COL | D_ROW | L_COL);
            next |= ((old >> 3) & 1) << 10; next |= ((old >> 4) & 1) << 13; next |= ((old >> 5) & 1) << 16;
            next |= ((old >> 10) & 1) << 32; next |= ((old >> 13) & 1) << 31; next |= ((old >> 16) & 1) << 30;
            next |= ((old >> 32) & 1) << 43; next |= ((old >> 31) & 1) << 40; next |= ((old >> 30) & 1) << 37;
            next |= ((old >> 43) & 1) << 3; next |= ((old >> 40) & 1) << 4; next |= ((old >> 37) & 1) << 5;
            *b = next;
        }
    }

    fn rotate_e(&mut self) {
        const F_ROW: u64 = 0x7 << 21;
        const R_ROW: u64 = 0x7 << 12;
        const B_ROW: u64 = 0x7 << 48;
        const L_ROW: u64 = 0x7 << 39;
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(F_ROW | R_ROW | B_ROW | L_ROW);
            next |= ((old >> 21) & 0x7) << 12; // F -> R
            next |= ((old >> 12) & 0x7) << 48; // R -> B
            next |= ((old >> 48) & 0x7) << 39; // B -> L
            next |= ((old >> 39) & 0x7) << 21; // L -> F
            *b = next;
        }
    }

    // Helpers for full cube rotations or special moves
    #[inline(always)] pub fn rotate_u_prime(&mut self) { for _ in 0..3 { self.rotate_u(); } }
    #[inline(always)] pub fn rotate_d_prime(&mut self) { for _ in 0..3 { self.rotate_d(); } }
    #[inline(always)] pub fn rotate_l_prime(&mut self) { for _ in 0..3 { self.rotate_l(); } }
    #[inline(always)] pub fn rotate_r_prime(&mut self) { for _ in 0..3 { self.rotate_r(); } }
    #[inline(always)] pub fn rotate_f_prime(&mut self) { for _ in 0..3 { self.rotate_f(); } }
    #[inline(always)] pub fn rotate_b_prime(&mut self) { for _ in 0..3 { self.rotate_b(); } }
    #[inline(always)] pub fn rotate_m_prime(&mut self) { for _ in 0..3 { self.rotate_m(); } }
    #[inline(always)] pub fn rotate_e_prime(&mut self) { for _ in 0..3 { self.rotate_e(); } }
    
    /// Check if First Block (FB) is solved on a specific color/orientation.
    /// Default: Left block on Orange face (Face 4)
    pub fn is_fb_solved(&self) -> bool {
        // Colors from rouxflow_core::cube::facelet::Color:
        // White=0, Yellow=1, Green=2, Blue=3, Red=4, Orange=5
        let orange = 5; let yellow = 1; let blue = 3; let green = 2;
        
        // Piece Mapping for Face 4 (Left / Orange):
        // L Center: 40
        // DL edge: D3 (30) + L7 (43)
        // FL edge: F3 (21) + L5 (41)
        // BL edge: B3 (48) + L1 (37)
        // DFL corner: D0 (27) + L8 (44) + F6 (24)
        // DBL corner: D6 (33) + L6 (42) + B8 (53)
        
        // 1. Center
        if (self.boards[orange] & (1 << 40)) == 0 { return false; }
        
        // 2. Edges
        if (self.boards[yellow] & (1 << 30)) == 0 || (self.boards[orange] & (1 << 43)) == 0 { return false; } // DL
        if (self.boards[green]  & (1 << 21)) == 0 || (self.boards[orange] & (1 << 41)) == 0 { return false; } // FL
        if (self.boards[blue]   & (1 << 48)) == 0 || (self.boards[orange] & (1 << 37)) == 0 { return false; } // BL
        
        // 3. Corners
        if (self.boards[orange] & (1 << 44)) == 0 || (self.boards[yellow] & (1 << 27)) == 0 || (self.boards[green] & (1 << 24)) == 0 { return false; } // DFL
        if (self.boards[orange] & (1 << 42)) == 0 || (self.boards[yellow] & (1 << 33)) == 0 || (self.boards[blue]  & (1 << 53)) == 0 { return false; } // DBL

        true
    }
}
