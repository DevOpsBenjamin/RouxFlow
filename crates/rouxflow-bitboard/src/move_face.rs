use crate::bitcube::BitCube;
use crate::move_indices::FaceMove;

impl BitCube {
    /// Applies a standard face move (U, D, L, R, F, B) and its variations (Prime, Double).
    pub fn apply_face_move(&mut self, m: FaceMove) {
        match m {
            // U
            FaceMove::U => self.face_u(),
            FaceMove::Up => self.face_u_prime(),
            FaceMove::U2 => self.face_u2(),
            // D
            FaceMove::D => self.face_d(),
            FaceMove::Dp => self.face_d_prime(),
            FaceMove::D2 => self.face_d2(),
            // L
            FaceMove::L => self.face_l(),
            FaceMove::Lp => self.face_l_prime(),
            FaceMove::L2 => self.face_l2(),
            // R
            FaceMove::R => self.face_r(),
            FaceMove::Rp => self.face_r_prime(),
            FaceMove::R2 => self.face_r2(),
            // F
            FaceMove::F => self.face_f(),
            FaceMove::Fp => self.face_f_prime(),
            FaceMove::F2 => self.face_f2(),
            // B
            FaceMove::B => self.face_b(),
            FaceMove::Bp => self.face_b_prime(),
            FaceMove::B2 => self.face_b2(),
        }
    }

    /// 90° clockwise rotation of 3x3 face bits.
    #[inline(always)]
    pub(crate) fn rotate_face_bits(v: u64) -> u64 {
        let corners =
            ((v & 0x01) << 2) | ((v & 0x04) << 6) | ((v & 0x100) >> 2) | ((v & 0x40) >> 6);
        let edges = ((v & 0x02) << 4) | ((v & 0x20) << 2) | ((v & 0x80) >> 4) | ((v & 0x08) >> 2);
        corners | edges | (v & 0x10)
    }
    /// 90° counter-clockwise rotation of 3x3 face bits.
    #[inline(always)]
    pub(crate) fn rotate_face_bits_prime(v: u64) -> u64 {
        let corners =
            ((v & 0x01) << 6) | ((v & 0x40) << 2) | ((v & 0x100) >> 6) | ((v & 0x04) >> 2);
        let edges = ((v & 0x02) << 2) | ((v & 0x08) << 4) | ((v & 0x80) >> 2) | ((v & 0x20) >> 4);
        corners | edges | (v & 0x10)
    }
    /// 180° rotation of 3x3 face bits.
    #[inline(always)]
    pub(crate) fn rotate_face_bits_180(v: u64) -> u64 {
        let corners =
            ((v & 0x01) << 8) | ((v & 0x100) >> 8) | ((v & 0x04) << 4) | ((v & 0x40) >> 4);
        let edges = ((v & 0x02) << 6) | ((v & 0x80) >> 6) | ((v & 0x08) << 2) | ((v & 0x20) >> 2);
        corners | edges | (v & 0x10)
    }
    // --- U Face ---
    pub fn face_u(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x1FF | (0x7 << 18) | (0x7 << 9) | (0x7 << 45) | (0x7 << 36));
            next |= Self::rotate_face_bits(old & 0x1FF);
            next |= ((old >> 9) & 0x7) << 18; // R -> F
            next |= ((old >> 18) & 0x7) << 36; // F -> L
            next |= ((old >> 36) & 0x7) << 45; // L -> B
            next |= ((old >> 45) & 0x7) << 9; // B -> R
            *b = next;
        }
    }
    pub fn face_u_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x1FF | (0x7 << 18) | (0x7 << 9) | (0x7 << 45) | (0x7 << 36));
            next |= Self::rotate_face_bits_prime(old & 0x1FF);
            next |= ((old >> 18) & 0x7) << 9; // F -> R
            next |= ((old >> 9) & 0x7) << 45; // R -> B
            next |= ((old >> 45) & 0x7) << 36; // B -> L
            next |= ((old >> 36) & 0x7) << 18; // L -> F
            *b = next;
        }
    }
    pub fn face_u2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !(0x1FF | (0x7 << 18) | (0x7 << 9) | (0x7 << 45) | (0x7 << 36));
            next |= Self::rotate_face_bits_180(old & 0x1FF);
            next |= ((old >> 18) & 0x7) << 45; // F -> B
            next |= ((old >> 45) & 0x7) << 18; // B -> F
            next |= ((old >> 9) & 0x7) << 36; // R -> L
            next |= ((old >> 36) & 0x7) << 9; // L -> R
            *b = next;
        }
    }
    // --- D Face ---
    pub fn face_d(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 27) | (0x7 << 24) | (0x7 << 15) | (0x7 << 51) | (0x7 << 42));
            next |= Self::rotate_face_bits((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 24) & 0x7) << 15; // F -> R
            next |= ((old >> 15) & 0x7) << 51; // R -> B
            next |= ((old >> 51) & 0x7) << 42; // B -> L
            next |= ((old >> 42) & 0x7) << 24; // L -> F
            *b = next;
        }
    }
    pub fn face_d_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 27) | (0x7 << 24) | (0x7 << 15) | (0x7 << 51) | (0x7 << 42));
            next |= Self::rotate_face_bits_prime((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 15) & 0x7) << 24; // R -> F
            next |= ((old >> 24) & 0x7) << 42; // F -> L
            next |= ((old >> 42) & 0x7) << 51; // L -> B
            next |= ((old >> 51) & 0x7) << 15; // B -> R
            *b = next;
        }
    }
    pub fn face_d2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 27) | (0x7 << 24) | (0x7 << 15) | (0x7 << 51) | (0x7 << 42));
            next |= Self::rotate_face_bits_180((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 24) & 0x7) << 51; // F <-> B
            next |= ((old >> 51) & 0x7) << 24;
            next |= ((old >> 15) & 0x7) << 42; // R <-> L
            next |= ((old >> 42) & 0x7) << 15;
            *b = next;
        }
    }
    // --- L Face ---
    pub fn face_l(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old
                & !((0x1FF << 36)
                    | 0x49
                    | (0x49 << 18)
                    | (0x49 << 27)
                    | (1 << 53 | 1 << 50 | 1 << 47));
            next |= Self::rotate_face_bits((old >> 36) & 0x1FF) << 36;
            next |= (old & 0x49) << 18; // U -> F
            next |= (old & (0x49 << 18)) << 9; // F -> D
            if (old & (1 << 27)) != 0 {
                next |= 1 << 53;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 50;
            }
            if (old & (1 << 33)) != 0 {
                next |= 1 << 47;
            }
            if (old & (1 << 53)) != 0 {
                next |= 1 << 0;
            }
            if (old & (1 << 50)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 47)) != 0 {
                next |= 1 << 6;
            }
            *b = next;
        }
    }
    pub fn face_l_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old
                & !((0x1FF << 36)
                    | 0x49
                    | (0x49 << 18)
                    | (0x49 << 27)
                    | (1 << 53 | 1 << 50 | 1 << 47));
            next |= Self::rotate_face_bits_prime((old >> 36) & 0x1FF) << 36;
            next |= (old & (0x49 << 18)) >> 18; // F -> U
            next |= (old & (0x49 << 27)) >> 9; // D -> F
            if (old & (1 << 53)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 50)) != 0 {
                next |= 1 << 30;
            }
            if (old & (1 << 47)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 0)) != 0 {
                next |= 1 << 53;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 50;
            }
            if (old & (1 << 6)) != 0 {
                next |= 1 << 47;
            }
            *b = next;
        }
    }
    pub fn face_l2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old
                & !((0x1FF << 36)
                    | 0x49
                    | (0x49 << 18)
                    | (0x49 << 27)
                    | (1 << 53 | 1 << 50 | 1 << 47));
            next |= Self::rotate_face_bits_180((old >> 36) & 0x1FF) << 36;
            next |= (old & 0x49) << 27; // U <-> D
            next |= (old & (0x49 << 27)) >> 27;
            if (old & (1 << 18)) != 0 {
                next |= 1 << 53;
            } // F <-> B (rev)
            if (old & (1 << 21)) != 0 {
                next |= 1 << 50;
            }
            if (old & (1 << 24)) != 0 {
                next |= 1 << 47;
            }
            if (old & (1 << 53)) != 0 {
                next |= 1 << 18;
            }
            if (old & (1 << 50)) != 0 {
                next |= 1 << 21;
            }
            if (old & (1 << 47)) != 0 {
                next |= 1 << 24;
            }
            *b = next;
        }
    }
    // --- R Face ---
    pub fn face_r(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old
                & !((0x1FF << 9)
                    | 0x124
                    | (0x124 << 18)
                    | (0x124 << 27)
                    | (1 << 51 | 1 << 48 | 1 << 45));
            next |= Self::rotate_face_bits((old >> 9) & 0x1FF) << 9;
            if (old & (1 << 2)) != 0 {
                next |= 1 << 51;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 48;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 45;
            }
            if (old & (1 << 51)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 48)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 45)) != 0 {
                next |= 1 << 35;
            }
            next |= (old & (0x124 << 27)) >> 9; // D -> F
            next |= (old & (0x124 << 18)) >> 18; // F -> U
            *b = next;
        }
    }
    pub fn face_r_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old
                & !((0x1FF << 9)
                    | 0x124
                    | (0x124 << 18)
                    | (0x124 << 27)
                    | (1 << 51 | 1 << 48 | 1 << 45));
            next |= Self::rotate_face_bits_prime((old >> 9) & 0x1FF) << 9;
            next |= (old & 0x124) << 18; // U -> F
            next |= (old & (0x124 << 18)) << 9; // F -> D
            if (old & (1 << 29)) != 0 {
                next |= 1 << 51;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 48;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 45;
            }
            if (old & (1 << 51)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 48)) != 0 {
                next |= 1 << 5;
            }
            if (old & (1 << 45)) != 0 {
                next |= 1 << 8;
            }
            *b = next;
        }
    }
    pub fn face_r2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old
                & !((0x1FF << 9)
                    | 0x124
                    | (0x124 << 18)
                    | (0x124 << 27)
                    | (1 << 51 | 1 << 48 | 1 << 45));
            next |= Self::rotate_face_bits_180((old >> 9) & 0x1FF) << 9;
            next |= (old & 0x124) << 27; // U <-> D
            next |= (old & (0x124 << 27)) >> 27;
            if (old & (1 << 20)) != 0 {
                next |= 1 << 51;
            } // F <-> B (rev)
            if (old & (1 << 23)) != 0 {
                next |= 1 << 48;
            }
            if (old & (1 << 26)) != 0 {
                next |= 1 << 45;
            }
            if (old & (1 << 51)) != 0 {
                next |= 1 << 20;
            }
            if (old & (1 << 48)) != 0 {
                next |= 1 << 23;
            }
            if (old & (1 << 45)) != 0 {
                next |= 1 << 26;
            }
            *b = next;
        }
    }
    // --- F Face ---
    pub fn face_f(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 18) | (0x7 << 6) | (0x49 << 9) | (0x7 << 27) | (0x49 << 38));
            next |= Self::rotate_face_bits((old >> 18) & 0x1FF) << 18;
            if (old & (1 << 6)) != 0 {
                next |= 1 << 9;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 12;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 15;
            }
            if (old & (1 << 9)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 12)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 15)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 29)) != 0 {
                next |= 1 << 44;
            }
            if (old & (1 << 28)) != 0 {
                next |= 1 << 41;
            }
            if (old & (1 << 27)) != 0 {
                next |= 1 << 38;
            }
            if (old & (1 << 44)) != 0 {
                next |= 1 << 6;
            }
            if (old & (1 << 41)) != 0 {
                next |= 1 << 7;
            }
            if (old & (1 << 38)) != 0 {
                next |= 1 << 8;
            }
            *b = next;
        }
    }
    pub fn face_f_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 18) | (0x7 << 6) | (0x49 << 9) | (0x7 << 27) | (0x49 << 38));
            next |= Self::rotate_face_bits_prime((old >> 18) & 0x1FF) << 18;
            if (old & (1 << 6)) != 0 {
                next |= 1 << 44;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 41;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 38;
            }
            if (old & (1 << 44)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 41)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 38)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 29)) != 0 {
                next |= 1 << 9;
            }
            if (old & (1 << 28)) != 0 {
                next |= 1 << 12;
            }
            if (old & (1 << 27)) != 0 {
                next |= 1 << 15;
            }
            if (old & (1 << 9)) != 0 {
                next |= 1 << 6;
            }
            if (old & (1 << 12)) != 0 {
                next |= 1 << 7;
            }
            if (old & (1 << 15)) != 0 {
                next |= 1 << 8;
            }
            *b = next;
        }
    }
    pub fn face_f2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 18) | (0x7 << 6) | (0x49 << 9) | (0x7 << 27) | (0x49 << 38));
            next |= Self::rotate_face_bits_180((old >> 18) & 0x1FF) << 18;
            // U <-> D (bits 6,7,8 <-> 29,28,27)
            if (old & (1 << 6)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 29)) != 0 {
                next |= 1 << 6;
            }
            if (old & (1 << 28)) != 0 {
                next |= 1 << 7;
            }
            if (old & (1 << 27)) != 0 {
                next |= 1 << 8;
            }

            // R <-> L (bits 9,12,15 <-> 44,41,38)
            if (old & (1 << 9)) != 0 {
                next |= 1 << 44;
            }
            if (old & (1 << 12)) != 0 {
                next |= 1 << 41;
            }
            if (old & (1 << 15)) != 0 {
                next |= 1 << 38;
            }
            if (old & (1 << 44)) != 0 {
                next |= 1 << 9;
            }
            if (old & (1 << 41)) != 0 {
                next |= 1 << 12;
            }
            if (old & (1 << 38)) != 0 {
                next |= 1 << 15;
            }
            *b = next;
        }
    }
    // --- B Face ---
    pub fn face_b(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 45) | 0x7 | (0x49 << 36) | (0x7 << 33) | (0x49 << 11));
            next |= Self::rotate_face_bits((old >> 45) & 0x1FF) << 45;
            if (old & (1 << 2)) != 0 {
                next |= 1 << 36;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 39;
            }
            if (old & (1 << 0)) != 0 {
                next |= 1 << 42;
            }
            if (old & (1 << 36)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 39)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 42)) != 0 {
                next |= 1 << 35;
            }
            if (old & (1 << 33)) != 0 {
                next |= 1 << 17;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 14;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 11;
            }
            if (old & (1 << 17)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 14)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 11)) != 0 {
                next |= 1 << 0;
            }
            *b = next;
        }
    }
    pub fn face_b_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 45) | 0x7 | (0x49 << 36) | (0x7 << 33) | (0x49 << 11));
            next |= Self::rotate_face_bits_prime((old >> 45) & 0x1FF) << 45;
            if (old & (1 << 2)) != 0 {
                next |= 1 << 17;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 14;
            }
            if (old & (1 << 0)) != 0 {
                next |= 1 << 11;
            }
            if (old & (1 << 17)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 14)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 11)) != 0 {
                next |= 1 << 35;
            }
            if (old & (1 << 33)) != 0 {
                next |= 1 << 36;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 39;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 42;
            }
            if (old & (1 << 36)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 39)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 42)) != 0 {
                next |= 1 << 0;
            }
            *b = next;
        }
    }
    pub fn face_b2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x1FF << 45) | 0x7 | (0x49 << 36) | (0x7 << 33) | (0x49 << 11));
            next |= Self::rotate_face_bits_180((old >> 45) & 0x1FF) << 45;
            // U <-> D (bits 2,1,0 <-> 33,34,35)
            if (old & (1 << 2)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 0)) != 0 {
                next |= 1 << 35;
            }
            if (old & (1 << 33)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 0;
            }

            // L <-> R (bits 36,39,42 <-> 17,14,11)
            if (old & (1 << 36)) != 0 {
                next |= 1 << 17;
            }
            if (old & (1 << 39)) != 0 {
                next |= 1 << 14;
            }
            if (old & (1 << 42)) != 0 {
                next |= 1 << 11;
            }
            if (old & (1 << 17)) != 0 {
                next |= 1 << 36;
            }
            if (old & (1 << 14)) != 0 {
                next |= 1 << 39;
            }
            if (old & (1 << 11)) != 0 {
                next |= 1 << 42;
            }
            *b = next;
        }
    }
}
