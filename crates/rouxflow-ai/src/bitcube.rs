use rouxflow_core::cube::facelet::{FaceletCube, Color};

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

    /// Optimized move application using bitwise operations
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

    /// Helper to apply a permutation of bits to all boards
    fn apply_permutation(&mut self, p: &[usize; 54]) {
        let mut next_boards = [0u64; 6];
        for color_idx in 0..6 {
            let mut next_board = 0u64;
            let current_board = self.boards[color_idx];
            for (to, &from) in p.iter().enumerate() {
                if (current_board & (1 << from)) != 0 {
                    next_board |= 1 << to;
                }
            }
            next_boards[color_idx] = next_board;
        }
        self.boards = next_boards;
    }

    // Individual face rotations (Clockwise)
    // Indices: U(0-8), R(9-17), F(18-26), D(27-35), L(36-44), B(45-53)
    
    fn rotate_u(&mut self) {
        let f_mask: u64 = 0x7 << 18;
        let r_mask: u64 = 0x7 << 9;
        let b_mask: u64 = 0x7 << 45;
        let l_mask: u64 = 0x7 << 36;
        let u_face_mask: u64 = 0x1FF;

        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(f_mask | r_mask | b_mask | l_mask | u_face_mask);
            
            // Face rotation (012 -> 258, 345 -> 147, 678 -> 036)
            next |= ((old >> 0) & 1) << 2; next |= ((old >> 1) & 1) << 5; next |= ((old >> 2) & 1) << 8;
            next |= ((old >> 3) & 1) << 1; next |= ((old >> 4) & 1) << 4; next |= ((old >> 5) & 1) << 7;
            next |= ((old >> 6) & 1) << 0; next |= ((old >> 7) & 1) << 3; next |= ((old >> 8) & 1) << 6;

            // Sides: F -> L (18->36), L -> B (36->45), B -> R (45->9), R -> F (9->18)
            next |= (old & r_mask) << 9;   // R -> F
            next |= (old & f_mask) << 18;  // F -> L
            next |= (old & l_mask) << 9;   // L -> B
            next |= (old & b_mask) >> 36;  // B -> R
            *b = next;
        }
    }

    fn rotate_d(&mut self) {
        let f_mask: u64 = 0x7 << (18 + 6);
        let r_mask: u64 = 0x7 << (9 + 6);
        let b_mask: u64 = 0x7 << (45 + 6);
        let l_mask: u64 = 0x7 << (36 + 6);
        let d_face_mask: u64 = 0x1FF << 27;

        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(f_mask | r_mask | b_mask | l_mask | d_face_mask);
            let s = 27;
            next |= ((old >> (s+0)) & 1) << (s+2); next |= ((old >> (s+1)) & 1) << (s+5); next |= ((old >> (s+2)) & 1) << (s+8);
            next |= ((old >> (s+3)) & 1) << (s+1); next |= ((old >> (s+4)) & 1) << (s+4); next |= ((old >> (s+5)) & 1) << (s+7);
            next |= ((old >> (s+6)) & 1) << (s+0); next |= ((old >> (s+7)) & 1) << (s+3); next |= ((old >> (s+8)) & 1) << (s+6);

            // Sides: F -> R -> B -> L -> F
            next |= (old & l_mask) << 9; // L -> B
            next |= (old & b_mask) >> 36; // B -> R
            next |= (old & r_mask) << 9; // R -> F
            next |= (old & f_mask) << 18; // F -> L
            *b = next;
        }
    }

    // L Move: U -> F -> D -> B
    fn rotate_l(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            // Face simplified for now (L: 36..45)
            let s = 36; next &= !(0x1FF << s);
            next |= ((old >> (s+0)) & 1) << (s+2); next |= ((old >> (s+1)) & 1) << (s+5); next |= ((old >> (s+2)) & 1) << (s+8);
            next |= ((old >> (s+3)) & 1) << (s+1); next |= ((old >> (s+4)) & 1) << (s+4); next |= ((old >> (s+5)) & 1) << (s+7);
            next |= ((old >> (s+6)) & 1) << (s+0); next |= ((old >> (s+7)) & 1) << (s+3); next |= ((old >> (s+8)) & 1) << (s+6);
            
            // Sides (Indices: U0,3,6; F0,3,6; D0,3,6; B8,5,2)
            // U->F, F->D, D->B, B->U
            next &= !((1<<0)|(1<<3)|(1<<6) | (1<<18)|(1<<21)|(1<<24) | (1<<27)|(1<<30)|(1<<33) | (1<<53)|(1<<50)|(1<<47));
            next |= ((old >> 0) & 1) << 18; next |= ((old >> 3) & 1) << 21; next |= ((old >> 6) & 1) << 24; // U->F
            next |= ((old >> 18) & 1) << 27; next |= ((old >> 21) & 1) << 30; next |= ((old >> 24) & 1) << 33; // F->D
            next |= ((old >> 27) & 1) << 53; next |= ((old >> 30) & 1) << 50; next |= ((old >> 33) & 1) << 47; // D->B
            next |= ((old >> 53) & 1) << 0; next |= ((old >> 50) & 1) << 3; next |= ((old >> 47) & 1) << 6; // B->U
            *b = next;
        }
    }

    fn rotate_r(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            let s = 9; next &= !(0x1FF << s);
            next |= ((old >> (s+0)) & 1) << (s+2); next |= ((old >> (s+1)) & 1) << (s+5); next |= ((old >> (s+2)) & 1) << (s+8);
            next |= ((old >> (s+3)) & 1) << (s+1); next |= ((old >> (s+4)) & 1) << (s+4); next |= ((old >> (s+5)) & 1) << (s+7);
            next |= ((old >> (s+6)) & 1) << (s+0); next |= ((old >> (s+7)) & 1) << (s+3); next |= ((old >> (s+8)) & 1) << (s+6);
            
            // U->B, B->D, D->F, F->U
            next &= !((1<<2)|(1<<5)|(1<<8) | (1<<51)|(1<<48)|(1<<45) | (1<<29)|(1<<32)|(1<<35) | (1<<20)|(1<<23)|(1<<26));
            next |= ((old >> 2) & 1) << 51; next |= ((old >> 5) & 1) << 48; next |= ((old >> 8) & 1) << 45; // U->B
            next |= ((old >> 51) & 1) << 29; next |= ((old >> 48) & 1) << 32; next |= ((old >> 45) & 1) << 35; // B->D
            next |= ((old >> 29) & 1) << 20; next |= ((old >> 32) & 1) << 23; next |= ((old >> 35) & 1) << 26; // D->F
            next |= ((old >> 20) & 1) << 2; next |= ((old >> 23) & 1) << 5; next |= ((old >> 26) & 1) << 8; // F->U
            *b = next;
        }
    }

    fn rotate_f(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            let s = 18; next &= !(0x1FF << s);
            next |= ((old >> (s+0)) & 1) << (s+2); next |= ((old >> (s+1)) & 1) << (s+5); next |= ((old >> (s+2)) & 1) << (s+8);
            next |= ((old >> (s+3)) & 1) << (s+1); next |= ((old >> (s+4)) & 1) << (s+4); next |= ((old >> (s+5)) & 1) << (s+7);
            next |= ((old >> (s+6)) & 1) << (s+0); next |= ((old >> (s+7)) & 1) << (s+3); next |= ((old >> (s+8)) & 1) << (s+6);
            
            // U6,7,8 -> R0,3,6 -> D2,1,0 -> L8,5,2 -> U6,7,8
            next &= !((1<<6)|(1<<7)|(1<<8) | (1<<9)|(1<<12)|(1<<15) | (1<<29)|(1<<28)|(1<<27) | (1<<44)|(1<<41)|(1<<38));
            next |= ((old >> 6) & 1) << 9; next |= ((old >> 7) & 1) << 12; next |= ((old >> 8) & 1) << 15; // U->R
            next |= ((old >> 9) & 1) << 29; next |= ((old >> 12) & 1) << 28; next |= ((old >> 15) & 1) << 27; // R->D
            next |= ((old >> 29) & 1) << 44; next |= ((old >> 28) & 1) << 41; next |= ((old >> 27) & 1) << 38; // D->L
            next |= ((old >> 44) & 1) << 6; next |= ((old >> 41) & 1) << 7; next |= ((old >> 38) & 1) << 8; // L->U
            *b = next;
        }
    }

    fn rotate_b(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            let s = 45; next &= !(0x1FF << s);
            next |= ((old >> (s+0)) & 1) << (s+2); next |= ((old >> (s+1)) & 1) << (s+5); next |= ((old >> (s+2)) & 1) << (s+8);
            next |= ((old >> (s+3)) & 1) << (s+1); next |= ((old >> (s+4)) & 1) << (s+4); next |= ((old >> (s+5)) & 1) << (s+7);
            next |= ((old >> (s+6)) & 1) << (s+0); next |= ((old >> (s+7)) & 1) << (s+3); next |= ((old >> (s+8)) & 1) << (s+6);
            
            // U2,1,0 -> L0,3,6 -> D6,7,8 -> R8,5,2 -> U2,1,0
            next &= !((1<<2)|(1<<1)|(1<<0) | (1<<36)|(1<<39)|(1<<42) | (1<<33)|(1<<34)|(1<<35) | (1<<17)|(1<<14)|(1<<11));
            next |= ((old >> 2) & 1) << 36; next |= ((old >> 1) & 1) << 39; next |= ((old >> 0) & 1) << 42; // U->L
            next |= ((old >> 36) & 1) << 33; next |= ((old >> 39) & 1) << 34; next |= ((old >> 42) & 1) << 35; // L->D
            next |= ((old >> 33) & 1) << 17; next |= ((old >> 34) & 1) << 14; next |= ((old >> 35) & 1) << 11; // D->R
            next |= ((old >> 17) & 1) << 2; next |= ((old >> 14) & 1) << 1; next |= ((old >> 11) & 1) << 0; // R->U
            *b = next;
        }
    }

    fn rotate_m(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            // Center stickers: U1,4,7; F1,4,7; D1,4,7; B7,4,1
            next &= !((1<<1)|(1<<4)|(1<<7) | (1<<19)|(1<<22)|(1<<25) | (1<<28)|(1<<31)|(1<<34) | (1<<52)|(1<<49)|(1<<46));
            next |= ((old >> 1) & 1) << 19; next |= ((old >> 4) & 1) << 22; next |= ((old >> 7) & 1) << 25; // U->F
            next |= ((old >> 19) & 1) << 28; next |= ((old >> 22) & 1) << 31; next |= ((old >> 25) & 1) << 34; // F->D
            next |= ((old >> 28) & 1) << 52; next |= ((old >> 31) & 1) << 49; next |= ((old >> 34) & 1) << 46; // D->B
            next |= ((old >> 52) & 1) << 1; next |= ((old >> 49) & 1) << 4; next |= ((old >> 46) & 1) << 7; // B->U
            *b = next;
        }
    }

    fn rotate_s(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            // U3,4,5; R1,4,7; D5,4,3; L7,4,1
            next &= !((1<<3)|(1<<4)|(1<<5) | (1<<10)|(1<<13)|(1<<16) | (1<<32)|(1<<31)|(1<<30) | (1<<43)|(1<<40)|(1<<37));
            next |= ((old >> 3) & 1) << 10; next |= ((old >> 4) & 1) << 13; next |= ((old >> 5) & 1) << 16; // U->R
            next |= ((old >> 10) & 1) << 32; next |= ((old >> 13) & 1) << 31; next |= ((old >> 16) & 1) << 30; // R->D
            next |= ((old >> 32) & 1) << 43; next |= ((old >> 31) & 1) << 40; next |= ((old >> 30) & 1) << 37; // D->L
            next |= ((old >> 43) & 1) << 3; next |= ((old >> 40) & 1) << 4; next |= ((old >> 37) & 1) << 5; // L->U
            *b = next;
        }
    }

    fn rotate_e(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old;
            // F3,4,5; R3,4,5; B3,4,5; L3,4,5
            next &= !((0x7<<21) | (0x7<<12) | (0x7<<48) | (0x7<<39));
            next |= (old & (0x7<<21)) >> 9;  // F -> R
            next |= (old & (0x7<<12)) << 36; // R -> B
            next |= (old & (0x7<<48)) >> 9;  // B -> L
            next |= (old & (0x7<<39)) >> 18; // L -> F
            *b = next;
        }
    }

    // Helpers for full cube rotations or special moves
    fn rotate_m_prime(&mut self) { for _ in 0..3 { self.rotate_m(); } }
    fn rotate_l_prime(&mut self) { for _ in 0..3 { self.rotate_l(); } }
    fn rotate_e_prime(&mut self) { for _ in 0..3 { self.rotate_e(); } }
    fn rotate_d_prime(&mut self) { for _ in 0..3 { self.rotate_d(); } }
    fn rotate_b_prime(&mut self) { for _ in 0..3 { self.rotate_b(); } }

    fn get_identity_perm(&self) -> [usize; 54] {
        let mut p = [0usize; 54];
        for i in 0..54 { p[i] = i; }
        p
    }

    fn perm_rotate_face(&self, p: &mut [usize; 54], face_idx: usize) {
        let s = face_idx * 9;
        let old = *p;
        p[s + 0] = old[s + 6]; p[s + 1] = old[s + 3]; p[s + 2] = old[s + 0];
        p[s + 3] = old[s + 7]; p[s + 4] = old[s + 4]; p[s + 5] = old[s + 1];
        p[s + 6] = old[s + 8]; p[s + 7] = old[s + 5]; p[s + 8] = old[s + 2];
    }
    
    /// Check if First Block (FB) is solved on a specific color/orientation.
    /// Default: Left block on White/Yellow
    pub fn is_fb_solved(&self) -> bool {
        // DL edge: indices 30 (D facelet 3) and 43 (L facelet 7) -- Wait, let's check FaceletCube mapping again.
        // Actually, RouxSolver::is_fb_solved in core is what we want to clone.
        // Piece mapping for FaceletCube:
        // L Center: 40 (Always index 40 for L face)
        // DL edge: Facelets 31 (D) and 43 (L)
        // FL edge: Facelets 21 (F) and 39 (L)
        // BL edge: Facelets 49 (B) and 37 (L)
        // DFL corner: Facelets 24 (F), 33 (D), 42 (L)
        // DBL corner: Facelets 35 (D), 44 (L), 51 (B)
        
        // This check is bitwise: (boards[color] & (1 << index)) != 0
        // We need to know which color is which at identity.
        // Standard: U=White(0), R=Red(4), F=Green(2), D=Yellow(1), L=Orange(5), B=Blue(3)
        let orange = 5; let yellow = 1; let blue = 3; let white = 0; let green = 2;
        
        // Check L center
        if (self.boards[orange] & (1 << 40)) == 0 { return false; }
        
        // DL edge (Orange-Yellow)
        if (self.boards[yellow] & (1 << 31)) == 0 || (self.boards[orange] & (1 << 43)) == 0 { return false; }
        // FL edge (Orange-Green)
        if (self.boards[green] & (1 << 21)) == 0 || (self.boards[orange] & (1 << 39)) == 0 { return false; }
        // BL edge (Orange-Blue)
        if (self.boards[blue] & (1 << 49)) == 0 || (self.boards[orange] & (1 << 37)) == 0 { return false; }
        
        // DFL corner (Orange-Yellow-Green)
        if (self.boards[orange] & (1 << 42)) == 0 || (self.boards[yellow] & (1 << 33)) == 0 || (self.boards[green] & (1 << 24)) == 0 { return false; }
        // DBL corner (Orange-Yellow-Blue)
        if (self.boards[orange] & (1 << 44)) == 0 || (self.boards[yellow] & (1 << 35)) == 0 || (self.boards[blue] & (1 << 51)) == 0 { return false; }

        true
    }
}
