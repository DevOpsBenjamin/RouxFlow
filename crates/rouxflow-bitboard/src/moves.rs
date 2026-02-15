use crate::bitcube::BitCube;
use crate::move_indices::*;

impl BitCube {
    /// Optimized move application using numeric enum
    pub fn apply_move_enum(&mut self, m: Move) {
        match m {
            Move::Face(fm) => self.apply_face_move(fm),
            Move::Slice(sm) => self.apply_slice_move(sm),
            Move::Wide(wm) => self.apply_wide_move(wm),
            Move::Rotate(rm) => self.apply_rotation(rm),
        }
    }

    pub fn apply_face_move(&mut self, m: FaceMove) {
        match m {
            FaceMove::U => self.rotate_u(), FaceMove::Up => self.rotate_u_prime(), FaceMove::U2 => self.rotate_u2(),
            FaceMove::D => self.rotate_d(), FaceMove::Dp => self.rotate_d_prime(), FaceMove::D2 => self.rotate_d2(),
            FaceMove::L => self.rotate_l(), FaceMove::Lp => self.rotate_l_prime(), FaceMove::L2 => self.rotate_l2(),
            FaceMove::R => self.rotate_r(), FaceMove::Rp => self.rotate_r_prime(), FaceMove::R2 => self.rotate_r2(),
            FaceMove::F => self.rotate_f(), FaceMove::Fp => self.rotate_f_prime(), FaceMove::F2 => self.rotate_f2(),
            FaceMove::B => self.rotate_b(), FaceMove::Bp => self.rotate_b_prime(), FaceMove::B2 => self.rotate_b2(),
        }
    }

    pub fn apply_slice_move(&mut self, m: SliceMove) {
        match m {
            SliceMove::M => self.rotate_m(), SliceMove::Mp => self.rotate_m_prime(), SliceMove::M2 => self.rotate_m2(),
            SliceMove::E => self.rotate_e(), SliceMove::Ep => self.rotate_e_prime(), SliceMove::E2 => self.rotate_e2(),
            SliceMove::S => self.rotate_s(), SliceMove::Sp => self.rotate_s_prime(), SliceMove::S2 => self.rotate_s2(),
        }
    }

    pub fn apply_wide_move(&mut self, m: WideMove) {
        match m {
            WideMove::Uw => self.rotate_uw(), WideMove::Uwp => self.rotate_uw_prime(), WideMove::Uw2 => self.rotate_uw2(),
            WideMove::Dw => self.rotate_dw(), WideMove::Dwp => self.rotate_dw_prime(), WideMove::Dw2 => self.rotate_dw2(),
            WideMove::Lw => self.rotate_lw(), WideMove::Lwp => self.rotate_lw_prime(), WideMove::Lw2 => self.rotate_lw2(),
            WideMove::Rw => self.rotate_rw(), WideMove::Rwp => self.rotate_rw_prime(), WideMove::Rw2 => self.rotate_rw2(),
            WideMove::Fw => self.rotate_fw(), WideMove::Fwp => self.rotate_fw_prime(), WideMove::Fw2 => self.rotate_fw2(),
            WideMove::Bw => self.rotate_bw(), WideMove::Bwp => self.rotate_bw_prime(), WideMove::Bw2 => self.rotate_bw2(),
        }
    }

    pub fn apply_rotation(&mut self, m: Rotation) {
        match m {
            Rotation::X => self.rotate_x(), Rotation::Xp => self.rotate_x_prime(), Rotation::X2 => self.rotate_x2(),
            Rotation::Y => self.rotate_y(), Rotation::Yp => self.rotate_y_prime(), Rotation::Y2 => self.rotate_y2(),
            Rotation::Z => self.rotate_z(), Rotation::Zp => self.rotate_z_prime(), Rotation::Z2 => self.rotate_z2(),
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
                "r" => self.rotate_rw(),
                "l" => self.rotate_lw(),
                "f" => self.rotate_fw(),
                "b" => self.rotate_bw(),
                "u" => self.rotate_uw(),
                "d" => self.rotate_dw(),
                "x" => self.rotate_x(),
                "y" => self.rotate_y(),
                "z" => self.rotate_z(),
                _ => {}
            }
        }
    }

    #[inline(always)]
    fn rotate_face_bits(v: u64) -> u64 {
        let corners = ((v & 0x01) << 2) | ((v & 0x04) << 6) | ((v & 0x100) >> 2) | ((v & 0x40) >> 6);
        let edges   = ((v & 0x02) << 4) | ((v & 0x20) << 2) | ((v & 0x80) >> 4) | ((v & 0x08) >> 2);
        let center  = v & 0x10;
        corners | edges | center
    }

    pub fn rotate_u(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x1FF | (0x7 << 18) | (0x7 << 9) | (0x7 << 45) | (0x7 << 36));
            next |= Self::rotate_face_bits(old & 0x1FF);
            next |= ((old >> 9)  & 0x7) << 18; 
            next |= ((old >> 18) & 0x7) << 36; 
            next |= ((old >> 36) & 0x7) << 45; 
            next |= ((old >> 45) & 0x7) << 9;  
            *b = next;
        }
    }

    pub fn rotate_d(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 27) | (0x7 << 24) | (0x7 << 15) | (0x7 << 51) | (0x7 << 42));
            next |= Self::rotate_face_bits((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 24) & 0x7) << 15; 
            next |= ((old >> 15) & 0x7) << 51; 
            next |= ((old >> 51) & 0x7) << 42; 
            next |= ((old >> 42) & 0x7) << 24; 
            *b = next;
        }
    }

    pub fn rotate_l(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 36) | 0x49 | (0x49 << 18) | (0x49 << 27) | (1<<53 | 1<<50 | 1<<47));
            next |= Self::rotate_face_bits((old >> 36) & 0x1FF) << 36;
            next |= (old & 0x49) << 18;      
            next |= (old & (0x49 << 18)) << 9; 
            if (old & (1 << 27)) != 0 { next |= 1 << 53; }
            if (old & (1 << 30)) != 0 { next |= 1 << 50; }
            if (old & (1 << 33)) != 0 { next |= 1 << 47; }
            if (old & (1 << 53)) != 0 { next |= 1 << 0; }
            if (old & (1 << 50)) != 0 { next |= 1 << 3; }
            if (old & (1 << 47)) != 0 { next |= 1 << 6; }
            *b = next;
        }
    }

    pub fn rotate_r(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 9) | 0x124 | (0x124 << 18) | (0x124 << 27) | (1<<51 | 1<<48 | 1<<45));
            next |= Self::rotate_face_bits((old >> 9) & 0x1FF) << 9;
            if (old & (1 << 2)) != 0 { next |= 1 << 51; }
            if (old & (1 << 5)) != 0 { next |= 1 << 48; }
            if (old & (1 << 8)) != 0 { next |= 1 << 45; }
            if (old & (1 << 51)) != 0 { next |= 1 << 29; }
            if (old & (1 << 48)) != 0 { next |= 1 << 32; }
            if (old & (1 << 45)) != 0 { next |= 1 << 35; }
            next |= (old & (0x124 << 27)) >> 9; 
            next |= (old & (0x124 << 18)) >> 18; 
            *b = next;
        }
    }

    pub fn rotate_f(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 18) | (0x7 << 6) | (0x49 << 9) | (0x7 << 27) | (0x49 << 38));
            next |= Self::rotate_face_bits((old >> 18) & 0x1FF) << 18;
            if (old & (1 << 6)) != 0 { next |= 1 << 9; }
            if (old & (1 << 7)) != 0 { next |= 1 << 12; }
            if (old & (1 << 8)) != 0 { next |= 1 << 15; }
            if (old & (1 << 9)) != 0 { next |= 1 << 29; }
            if (old & (1 << 12)) != 0 { next |= 1 << 28; }
            if (old & (1 << 15)) != 0 { next |= 1 << 27; }
            if (old & (1 << 29)) != 0 { next |= 1 << 44; }
            if (old & (1 << 28)) != 0 { next |= 1 << 41; }
            if (old & (1 << 27)) != 0 { next |= 1 << 38; }
            if (old & (1 << 44)) != 0 { next |= 1 << 6; }
            if (old & (1 << 41)) != 0 { next |= 1 << 7; }
            if (old & (1 << 38)) != 0 { next |= 1 << 8; }
            *b = next;
        }
    }

    pub fn rotate_b(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 45) | 0x7 | (0x49 << 36) | (0x7 << 33) | (0x49 << 11));
            next |= Self::rotate_face_bits((old >> 45) & 0x1FF) << 45;
            if (old & (1 << 2)) != 0 { next |= 1 << 36; }
            if (old & (1 << 1)) != 0 { next |= 1 << 39; }
            if (old & (1 << 0)) != 0 { next |= 1 << 42; }
            if (old & (1 << 36)) != 0 { next |= 1 << 33; }
            if (old & (1 << 39)) != 0 { next |= 1 << 34; }
            if (old & (1 << 42)) != 0 { next |= 1 << 35; }
            if (old & (1 << 33)) != 0 { next |= 1 << 17; }
            if (old & (1 << 34)) != 0 { next |= 1 << 14; }
            if (old & (1 << 35)) != 0 { next |= 1 << 11; }
            if (old & (1 << 17)) != 0 { next |= 1 << 2; }
            if (old & (1 << 14)) != 0 { next |= 1 << 1; }
            if (old & (1 << 11)) != 0 { next |= 1 << 0; }
            *b = next;
        }
    }

    pub fn rotate_m(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x92 | (0x92 << 18) | (0x92 << 27) | (1<<52 | 1<<49 | 1<<46));
            next |= (old & 0x92) << 18; 
            next |= (old & (0x92 << 18)) << 9; 
            if (old & (1 << 28)) != 0 { next |= 1 << 52; }
            if (old & (1 << 31)) != 0 { next |= 1 << 49; }
            if (old & (1 << 34)) != 0 { next |= 1 << 46; }
            if (old & (1 << 52)) != 0 { next |= 1 << 1; }
            if (old & (1 << 49)) != 0 { next |= 1 << 4; }
            if (old & (1 << 46)) != 0 { next |= 1 << 7; }
            *b = next;
        }
    }

    pub fn rotate_s(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 3) | (0x49 << 10) | (0x7 << 30) | (0x49 << 37));
            if (old & (1 << 3)) != 0 { next |= 1 << 10; }
            if (old & (1 << 4)) != 0 { next |= 1 << 13; }
            if (old & (1 << 5)) != 0 { next |= 1 << 16; }
            if (old & (1 << 10)) != 0 { next |= 1 << 32; }
            if (old & (1 << 13)) != 0 { next |= 1 << 31; }
            if (old & (1 << 16)) != 0 { next |= 1 << 30; }
            if (old & (1 << 32)) != 0 { next |= 1 << 43; }
            if (old & (1 << 31)) != 0 { next |= 1 << 40; }
            if (old & (1 << 30)) != 0 { next |= 1 << 37; }
            if (old & (1 << 43)) != 0 { next |= 1 << 3; }
            if (old & (1 << 40)) != 0 { next |= 1 << 4; }
            if (old & (1 << 37)) != 0 { next |= 1 << 5; }
            *b = next;
        }
    }

    pub fn rotate_e(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 21) | (0x7 << 12) | (0x7 << 48) | (0x7 << 39));
            next |= ((old >> 21) & 0x7) << 12; 
            next |= ((old >> 12) & 0x7) << 48; 
            next |= ((old >> 48) & 0x7) << 39; 
            next |= ((old >> 39) & 0x7) << 21; 
            *b = next;
        }
    }

    #[inline(always)] pub fn rotate_u_prime(&mut self) { for _ in 0..3 { self.rotate_u(); } }
    #[inline(always)] pub fn rotate_d_prime(&mut self) { for _ in 0..3 { self.rotate_d(); } }
    #[inline(always)] pub fn rotate_l_prime(&mut self) { for _ in 0..3 { self.rotate_l(); } }
    #[inline(always)] pub fn rotate_r_prime(&mut self) { for _ in 0..3 { self.rotate_r(); } }
    #[inline(always)] pub fn rotate_f_prime(&mut self) { for _ in 0..3 { self.rotate_f(); } }
    #[inline(always)] pub fn rotate_b_prime(&mut self) { for _ in 0..3 { self.rotate_b(); } }
    #[inline(always)] pub fn rotate_m_prime(&mut self) { for _ in 0..3 { self.rotate_m(); } }
    #[inline(always)] pub fn rotate_s_prime(&mut self) { for _ in 0..3 { self.rotate_s(); } }
    #[inline(always)] pub fn rotate_e_prime(&mut self) { for _ in 0..3 { self.rotate_e(); } }

    #[inline(always)] pub fn rotate_u2(&mut self) { self.rotate_u(); self.rotate_u(); }
    #[inline(always)] pub fn rotate_d2(&mut self) { self.rotate_d(); self.rotate_d(); }
    #[inline(always)] pub fn rotate_l2(&mut self) { self.rotate_l(); self.rotate_l(); }
    #[inline(always)] pub fn rotate_r2(&mut self) { self.rotate_r(); self.rotate_r(); }
    #[inline(always)] pub fn rotate_f2(&mut self) { self.rotate_f(); self.rotate_f(); }
    #[inline(always)] pub fn rotate_b2(&mut self) { self.rotate_b(); self.rotate_b(); }
    #[inline(always)] pub fn rotate_m2(&mut self) { self.rotate_m(); self.rotate_m(); }
    #[inline(always)] pub fn rotate_s2(&mut self) { self.rotate_s(); self.rotate_s(); }
    #[inline(always)] pub fn rotate_e2(&mut self) { self.rotate_e(); self.rotate_e(); }

    // --- Wide Moves ---
    pub fn rotate_uw(&mut self) { self.rotate_u(); self.rotate_e_prime(); }
    pub fn rotate_dw(&mut self) { self.rotate_d(); self.rotate_e(); }
    pub fn rotate_lw(&mut self) { self.rotate_l(); self.rotate_m(); }
    pub fn rotate_rw(&mut self) { self.rotate_r(); self.rotate_m_prime(); }
    pub fn rotate_fw(&mut self) { self.rotate_f(); self.rotate_s(); }
    pub fn rotate_bw(&mut self) { self.rotate_b(); self.rotate_s_prime(); }

    pub fn rotate_uw_prime(&mut self) { self.rotate_u_prime(); self.rotate_e(); }
    pub fn rotate_dw_prime(&mut self) { self.rotate_d_prime(); self.rotate_e_prime(); }
    pub fn rotate_lw_prime(&mut self) { self.rotate_l_prime(); self.rotate_m_prime(); }
    pub fn rotate_rw_prime(&mut self) { self.rotate_r_prime(); self.rotate_m(); }
    pub fn rotate_fw_prime(&mut self) { self.rotate_f_prime(); self.rotate_s_prime(); }
    pub fn rotate_bw_prime(&mut self) { self.rotate_b_prime(); self.rotate_s(); }
    
    pub fn rotate_uw2(&mut self) { self.rotate_uw(); self.rotate_uw(); }
    pub fn rotate_dw2(&mut self) { self.rotate_dw(); self.rotate_dw(); }
    pub fn rotate_lw2(&mut self) { self.rotate_lw(); self.rotate_lw(); }
    pub fn rotate_rw2(&mut self) { self.rotate_rw(); self.rotate_rw(); }
    pub fn rotate_fw2(&mut self) { self.rotate_fw(); self.rotate_fw(); }
    pub fn rotate_bw2(&mut self) { self.rotate_bw(); self.rotate_bw(); }

    // --- Global Rotations ---
    pub fn rotate_x(&mut self) { self.rotate_r(); self.rotate_m_prime(); self.rotate_l_prime(); }
    pub fn rotate_x_prime(&mut self) { self.rotate_r_prime(); self.rotate_m(); self.rotate_l(); }
    pub fn rotate_z(&mut self) { self.rotate_f(); self.rotate_s(); self.rotate_b_prime(); }
    pub fn rotate_z_prime(&mut self) { self.rotate_f_prime(); self.rotate_s_prime(); self.rotate_b(); }
    pub fn rotate_z2(&mut self) { self.rotate_z(); self.rotate_z(); }

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
            next |= Self::rot90_cw(old & 0x1FF);
            next |= Self::rot90_ccw((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 18) & 0x1FF) << 36;
            next |= ((old >> 36) & 0x1FF) << 45;
            next |= ((old >> 45) & 0x1FF) << 9;
            next |= ((old >> 9) & 0x1FF) << 18;
            *b = next;
        }
    }

    pub fn rotate_y_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            next |= Self::rot90_ccw(old & 0x1FF);
            next |= Self::rot90_cw((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 18) & 0x1FF) << 9;
            next |= ((old >> 9) & 0x1FF) << 45;
            next |= ((old >> 45) & 0x1FF) << 36;
            next |= ((old >> 36) & 0x1FF) << 18;
            *b = next;
        }
    }
    
    pub fn rotate_y2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            next |= Self::rot180(old & 0x1FF);
            next |= Self::rot180((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 18) & 0x1FF) << 45;
            next |= ((old >> 45) & 0x1FF) << 18;
            next |= ((old >> 36) & 0x1FF) << 9;
            next |= ((old >> 9) & 0x1FF) << 36;
            *b = next;
        }
    }
    
    pub fn rotate_x2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            next |= Self::rot180(old & 0x1FF) << 27;
            next |= Self::rot180((old >> 27) & 0x1FF);
            next |= Self::rot180((old >> 18) & 0x1FF) << 45;
            next |= Self::rot180((old >> 45) & 0x1FF) << 18;
            next |= Self::rot180((old >> 9) & 0x1FF) << 9;
            next |= Self::rot180((old >> 36) & 0x1FF) << 36;
            *b = next;
        }
    }
}
