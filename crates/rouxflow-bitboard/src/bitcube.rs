use crate::move_indices::Move;

/// A Rubik's Cube represented by bitboards for high-performance search.
/// Each color (White, Yellow, Green, Blue, Red, Orange) has a 64-bit integer.
/// Indices 0-53 are used (out of 64 bits).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitCube {
    pub boards: [u64; 6],
}

impl BitCube {
    /// Optimized constructor that sets bits directly.
    /// Useful for build scripts and tight loops.
    pub fn new_solved() -> Self {
        let mut boards = [0u64; 6];
        // U (White=0): 0-8
        boards[0] = 0x1FF;
        // D (Yellow=1): 27-35
        boards[1] = 0x1FF << 27;
        // F (Green=2): 18-26
        boards[2] = 0x1FF << 18;
        // B (Blue=3): 45-53
        boards[3] = 0x1FF << 45;
        // R (Red=4): 9-17
        boards[4] = 0x1FF << 9; 
        // L (Orange=5): 36-44
        boards[5] = 0x1FF << 36;
        
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
            Move::Rw => { self.rotate_r(); self.rotate_m_prime(); },
            Move::Rwp => { self.rotate_r_prime(); self.rotate_m(); },
            Move::Rw2 => { self.rotate_r(); self.rotate_r(); self.rotate_m(); self.rotate_m(); },
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
        // Face U (0-8) rotates: 0->2, 1->5, 2->8, 5->7, 8->6, 7->3, 6->0, 3->1
        // Side stickers: 
        // F: 18, 19, 20
        // R: 9, 10, 11
        // B: 45, 46, 47
        // L: 36, 37, 38
        // Cycle (Clockwise): F -> L, L -> B, B -> R, R -> F
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x1FF | (0x7 << 18) | (0x7 << 9) | (0x7 << 45) | (0x7 << 36));
            next |= Self::rotate_face_bits(old & 0x1FF);
            next |= ((old >> 9)  & 0x7) << 18; // R 9,10,11 -> F 18,19,20
            next |= ((old >> 18) & 0x7) << 36; // F 18,19,20 -> L 36,37,38
            next |= ((old >> 36) & 0x7) << 45; // L 36,37,38 -> B 45,46,47
            next |= ((old >> 45) & 0x7) << 9;  // B 45,46,47 -> R 9,10,11
            *b = next;
        }
    }

    fn rotate_d(&mut self) {
        // Face D (27-35)
        // Side stickers: F(24,25,26), R(15,16,17), B(51,52,53), L(42,43,44)
        // Cycle: F -> R, R -> B, B -> L, L -> F
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 27) | (0x7 << 24) | (0x7 << 15) | (0x7 << 51) | (0x7 << 42));
            next |= Self::rotate_face_bits((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 24) & 0x7) << 15; // F -> R
            next |= ((old >> 15) & 0x7) << 51; // R -> B
            next |= ((old >> 51) & 0x7) << 42; // B -> L
            next |= ((old >> 42) & 0x7) << 24; // L -> F
            *b = next;
        }
    }

    fn rotate_l(&mut self) {
        // Face L (36-44)
        // Sides: U(0,3,6), F(18,21,24), D(27,30,33), B(53,50,47)
        // Cycle: U -> F, F -> D, D -> B, B -> U
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 36) | 0x49 | (0x49 << 18) | (0x49 << 27) | (1<<53 | 1<<50 | 1<<47));
            next |= Self::rotate_face_bits((old >> 36) & 0x1FF) << 36;
            next |= (old & 0x49) << 18;      // U[0,3,6] -> F[18,21,24]
            next |= (old & (0x49 << 18)) << 9; // F[18,21,24] -> D[27,30,33]
            // D[27,30,33] -> B[53,50,47]
            if (old & (1 << 27)) != 0 { next |= 1 << 53; }
            if (old & (1 << 30)) != 0 { next |= 1 << 50; }
            if (old & (1 << 33)) != 0 { next |= 1 << 47; }
            // B[53,50,47] -> U[0,3,6]
            if (old & (1 << 53)) != 0 { next |= 1 << 0; }
            if (old & (1 << 50)) != 0 { next |= 1 << 3; }
            if (old & (1 << 47)) != 0 { next |= 1 << 6; }
            *b = next;
        }
    }

    fn rotate_r(&mut self) {
        // Face R (9-17)
        // Sides: U(2,5,8), B(51,48,45), D(29,32,35), F(20,23,26)
        // Cycle: U -> B, B -> D, D -> F, F -> U
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 9) | 0x124 | (0x124 << 18) | (0x124 << 27) | (1<<51 | 1<<48 | 1<<45));
            next |= Self::rotate_face_bits((old >> 9) & 0x1FF) << 9;
            // U[2,5,8] -> B[51,48,45]
            if (old & (1 << 2)) != 0 { next |= 1 << 51; }
            if (old & (1 << 5)) != 0 { next |= 1 << 48; }
            if (old & (1 << 8)) != 0 { next |= 1 << 45; }
            // B[51,48,45] -> D[29,32,35]
            if (old & (1 << 51)) != 0 { next |= 1 << 29; }
            if (old & (1 << 48)) != 0 { next |= 1 << 32; }
            if (old & (1 << 45)) != 0 { next |= 1 << 35; }
            next |= (old & (0x124 << 27)) >> 9; // D[29,32,35] -> F[20,23,26]
            next |= (old & (0x124 << 18)) >> 18; // F[20,23,26] -> U[2,5,8]
            *b = next;
        }
    }

    fn rotate_f(&mut self) {
        // Face F (18-26)
        // Sides: U(6,7,8), R(9,12,15), D(29,28,27), L(44,41,38)
        // Cycle: U -> R, R -> D, D -> L, L -> U
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 18) | (0x7 << 6) | (0x49 << 9) | (0x7 << 27) | (0x49 << 38));
            next |= Self::rotate_face_bits((old >> 18) & 0x1FF) << 18;
            // U[6,7,8] -> R[9,12,15]
            if (old & (1 << 6)) != 0 { next |= 1 << 9; }
            if (old & (1 << 7)) != 0 { next |= 1 << 12; }
            if (old & (1 << 8)) != 0 { next |= 1 << 15; }
            // R[9,12,15] -> D[29,28,27]
            if (old & (1 << 9)) != 0 { next |= 1 << 29; }
            if (old & (1 << 12)) != 0 { next |= 1 << 28; }
            if (old & (1 << 15)) != 0 { next |= 1 << 27; }
            // D[29,28,27] -> L[44,41,38]
            if (old & (1 << 29)) != 0 { next |= 1 << 44; }
            if (old & (1 << 28)) != 0 { next |= 1 << 41; }
            if (old & (1 << 27)) != 0 { next |= 1 << 38; }
            // L[44,41,38] -> U[6,7,8]
            if (old & (1 << 44)) != 0 { next |= 1 << 6; }
            if (old & (1 << 41)) != 0 { next |= 1 << 7; }
            if (old & (1 << 38)) != 0 { next |= 1 << 8; }
            *b = next;
        }
    }

    fn rotate_b(&mut self) {
        // Face B (45-53)
        // Sides: U(2,1,0), L(36,39,42), D(33,34,35), R(17,14,11)
        // Cycle: U -> L, L -> D, D -> R, R -> U
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 45) | 0x7 | (0x49 << 36) | (0x7 << 33) | (0x49 << 11));
            next |= Self::rotate_face_bits((old >> 45) & 0x1FF) << 45;
            // U[2,1,0] -> L[36,39,42]
            if (old & (1 << 2)) != 0 { next |= 1 << 36; }
            if (old & (1 << 1)) != 0 { next |= 1 << 39; }
            if (old & (1 << 0)) != 0 { next |= 1 << 42; }
            // L[36,39,42] -> D[33,34,35]
            if (old & (1 << 36)) != 0 { next |= 1 << 33; }
            if (old & (1 << 39)) != 0 { next |= 1 << 34; }
            if (old & (1 << 42)) != 0 { next |= 1 << 35; }
            // D[33,34,35] -> R[17,14,11]
            if (old & (1 << 33)) != 0 { next |= 1 << 17; }
            if (old & (1 << 34)) != 0 { next |= 1 << 14; }
            if (old & (1 << 35)) != 0 { next |= 1 << 11; }
            // R[17,14,11] -> U[2,1,0]
            if (old & (1 << 17)) != 0 { next |= 1 << 2; }
            if (old & (1 << 14)) != 0 { next |= 1 << 1; }
            if (old & (1 << 11)) != 0 { next |= 1 << 0; }
            *b = next;
        }
    }

    fn rotate_m(&mut self) {
        // Middle L-column (1,4,7)
        // Cycle: U -> F, F -> D, D -> B, B -> U
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x92 | (0x92 << 18) | (0x92 << 27) | (1<<52 | 1<<49 | 1<<46));
            next |= (old & 0x92) << 18; // U -> F
            next |= (old & (0x92 << 18)) << 9; // F -> D
            // D -> B (reversed)
            if (old & (1 << 28)) != 0 { next |= 1 << 52; }
            if (old & (1 << 31)) != 0 { next |= 1 << 49; }
            if (old & (1 << 34)) != 0 { next |= 1 << 46; }
            // B -> U (reversed)
            if (old & (1 << 52)) != 0 { next |= 1 << 1; }
            if (old & (1 << 49)) != 0 { next |= 1 << 4; }
            if (old & (1 << 46)) != 0 { next |= 1 << 7; }
            *b = next;
        }
    }

    fn rotate_s(&mut self) {
        // Middle Front-slice
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 3) | (0x49 << 10) | (0x7 << 30) | (0x49 << 37));
            // U[3,4,5] -> R[10,13,16]
            if (old & (1 << 3)) != 0 { next |= 1 << 10; }
            if (old & (1 << 4)) != 0 { next |= 1 << 13; }
            if (old & (1 << 5)) != 0 { next |= 1 << 16; }
            // R[10,13,16] -> D[32,31,30]
            if (old & (1 << 10)) != 0 { next |= 1 << 32; }
            if (old & (1 << 13)) != 0 { next |= 1 << 31; }
            if (old & (1 << 16)) != 0 { next |= 1 << 30; }
            // D[32,31,30] -> L[43,40,37]
            if (old & (1 << 32)) != 0 { next |= 1 << 43; }
            if (old & (1 << 31)) != 0 { next |= 1 << 40; }
            if (old & (1 << 30)) != 0 { next |= 1 << 37; }
            // L[43,40,37] -> U[3,4,5]
            if (old & (1 << 43)) != 0 { next |= 1 << 3; }
            if (old & (1 << 40)) != 0 { next |= 1 << 4; }
            if (old & (1 << 37)) != 0 { next |= 1 << 5; }
            *b = next;
        }
    }

    fn rotate_e(&mut self) {
        // Middle slice
        // Cycle: F -> R, R -> B, B -> L, L -> F (like D but middle)
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 21) | (0x7 << 12) | (0x7 << 48) | (0x7 << 39));
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
    pub fn get_color_at(&self, idx: usize) -> usize {
        for i in 0..6 {
            if (self.boards[i] & (1 << idx)) != 0 { return i; }
        }
        0
    }

    pub fn is_fb_solved(&self) -> bool {
        // Default Roux orientation: Left=Orange(5), Down=Yellow(1), Front=Green(2), Back=Blue(3)
        self.is_fb_solved_ext(5, 1, 2, 3)
    }

    pub fn is_fb_solved_ext(&self, l: usize, d: usize, f: usize, b: usize) -> bool {
        // Face L center check (40)
        if self.get_color_at(40) != l { return false; }
        
        // Face L: 39..=44 must match center
        for i in 39..=44 {
            if self.get_color_at(i) != l { return false; }
        }
        
        // D bar (27,30,33) match check
        // NOTE: We IGNORE the D center check to allow "Pseudo-Blocks" (misaligned M-slice).
        // The block is valid if the pieces are correct, even if centers are rotated (e.g. r2).
        // let d_center = self.get_color_at(31); 
        // if d_center != d { return false; } 
        if self.get_color_at(27) != d || self.get_color_at(30) != d || self.get_color_at(33) != d { return false; }
        
        // F bar (21,24) match check
        // let f_center = self.get_color_at(22);
        // if f_center != f { return false; }
        if self.get_color_at(21) != f || self.get_color_at(24) != f { return false; }
        
        // B bar (50,53) match check
        // let b_center = self.get_color_at(49);
        // if b_center != b { return false; }
        if self.get_color_at(50) != b || self.get_color_at(53) != b { return false; }
        
        true
    }

    // --- Global Rotation Helpers (Correct Bitwise) ---
    
    fn rot90_cw(face_bits: u64) -> u64 {
        let b = face_bits;
        let mut n = 0;
        if (b & (1<<0)) != 0 { n |= 1<<2; }
        if (b & (1<<1)) != 0 { n |= 1<<5; }
        if (b & (1<<2)) != 0 { n |= 1<<8; }
        if (b & (1<<5)) != 0 { n |= 1<<7; }
        if (b & (1<<8)) != 0 { n |= 1<<6; }
        if (b & (1<<7)) != 0 { n |= 1<<3; }
        if (b & (1<<6)) != 0 { n |= 1<<0; }
        if (b & (1<<3)) != 0 { n |= 1<<1; }
        if (b & (1<<4)) != 0 { n |= 1<<4; }
        n
    }

    fn rot90_ccw(face_bits: u64) -> u64 {
        let b = face_bits;
        let mut n = 0;
        if (b & (1<<0)) != 0 { n |= 1<<6; }
        if (b & (1<<1)) != 0 { n |= 1<<3; }
        if (b & (1<<2)) != 0 { n |= 1<<0; }
        if (b & (1<<5)) != 0 { n |= 1<<1; }
        if (b & (1<<8)) != 0 { n |= 1<<2; }
        if (b & (1<<7)) != 0 { n |= 1<<5; }
        if (b & (1<<6)) != 0 { n |= 1<<8; }
        if (b & (1<<3)) != 0 { n |= 1<<7; }
        if (b & (1<<4)) != 0 { n |= 1<<4; }
        n
    }
    
    fn rot180(face_bits: u64) -> u64 {
        let b = face_bits;
        let mut n = 0;
        if (b & (1<<0)) != 0 { n |= 1<<8; }
        if (b & (1<<1)) != 0 { n |= 1<<7; }
        if (b & (1<<2)) != 0 { n |= 1<<6; }
        if (b & (1<<5)) != 0 { n |= 1<<3; }
        if (b & (1<<8)) != 0 { n |= 1<<0; }
        if (b & (1<<7)) != 0 { n |= 1<<1; }
        if (b & (1<<6)) != 0 { n |= 1<<2; }
        if (b & (1<<3)) != 0 { n |= 1<<5; }
        if (b & (1<<4)) != 0 { n |= 1<<4; }
        n
    }

    pub fn rotate_y(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // U (0-8) -> CW
            next |= Self::rot90_cw(old & 0x1FF);
            // D (27-35) -> CCW
            next |= Self::rot90_ccw((old >> 27) & 0x1FF) << 27;
            // F(18-26) -> L(36-44)
            next |= ((old >> 18) & 0x1FF) << 36;
            // L(36-44) -> B(45-53)
            next |= ((old >> 36) & 0x1FF) << 45;
            // B(45-53) -> R(9-17)
            next |= ((old >> 45) & 0x1FF) << 9;
            // R(9-17) -> F(18-26)
            next |= ((old >> 9) & 0x1FF) << 18;
            *b = next;
        }
    }

    pub fn rotate_y_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // U (0-8) -> CCW
            next |= Self::rot90_ccw(old & 0x1FF);
            // D (27-35) -> CW
            next |= Self::rot90_cw((old >> 27) & 0x1FF) << 27;
            // F(18-26) -> R(9-17)
            next |= ((old >> 18) & 0x1FF) << 9;
            // R(9-17) -> B(45-53)
            next |= ((old >> 9) & 0x1FF) << 45;
            // B(45-53) -> L(36-44)
            next |= ((old >> 45) & 0x1FF) << 36;
            // L(36-44) -> F(18-26)
            next |= ((old >> 36) & 0x1FF) << 18;
            *b = next;
        }
    }
    
    pub fn rotate_y2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // U (0-8) -> 180
            next |= Self::rot180(old & 0x1FF);
            // D (27-35) -> 180
            next |= Self::rot180((old >> 27) & 0x1FF) << 27;
            // F(18-26) <-> B(45-53)
            next |= ((old >> 18) & 0x1FF) << 45;
            next |= ((old >> 45) & 0x1FF) << 18;
            // L(36-44) <-> R(9-17)
            next |= ((old >> 36) & 0x1FF) << 9;
            next |= ((old >> 9) & 0x1FF) << 36;
            *b = next;
        }
    }
    
    pub fn rotate_x2(&mut self) {
        // x2: F <-> B (flipped), U <-> D (flipped)
        // R and L (rotated 180)
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            
            // U(0-8) -> D(27-35) (180/Flipped? Yes, U becomes D upside down (relative to Front))
            // Standard x2 on U means: Back squares become Front squares of D. 
            // rot180 is correct.
            next |= Self::rot180(old & 0x1FF) << 27;
            
            // D(27-35) -> U(0-8) (180)
            next |= Self::rot180((old >> 27) & 0x1FF);
            
            // F(18-26) -> B(45-53) (180)
            next |= Self::rot180((old >> 18) & 0x1FF) << 45;
            
            // B(45-53) -> F(18-26) (180)
            next |= Self::rot180((old >> 45) & 0x1FF) << 18;
            
            // R(9-17) -> R(9-17) (180)
            next |= Self::rot180((old >> 9) & 0x1FF) << 9;
            
            // L(36-44) -> L(36-44) (180)
            next |= Self::rot180((old >> 36) & 0x1FF) << 36;
            
            *b = next;
        }
    }
}
