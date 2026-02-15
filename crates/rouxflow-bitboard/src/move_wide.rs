use crate::bitcube::BitCube;
use crate::move_indices::WideMove;

impl BitCube {
    /// Applies a wide move (move face + adjacent slice).
    pub fn apply_wide_move(&mut self, m: WideMove) {
        match m {
            WideMove::Uw => self.wide_uw(),
            WideMove::Uwp => self.wide_uw_prime(),
            WideMove::Uw2 => self.wide_uw2(),

            WideMove::Dw => self.wide_dw(),
            WideMove::Dwp => self.wide_dw_prime(),
            WideMove::Dw2 => self.wide_dw2(),

            WideMove::Lw => self.wide_lw(),
            WideMove::Lwp => self.wide_lw_prime(),
            WideMove::Lw2 => self.wide_lw2(),

            WideMove::Rw => self.wide_rw(),
            WideMove::Rwp => self.wide_rw_prime(),
            WideMove::Rw2 => self.wide_rw2(),

            WideMove::Fw => self.wide_fw(),
            WideMove::Fwp => self.wide_fw_prime(),
            WideMove::Fw2 => self.wide_fw2(),

            WideMove::Bw => self.wide_bw(),
            WideMove::Bwp => self.wide_bw_prime(),
            WideMove::Bw2 => self.wide_bw2(),
        }
    }

    // --- Optimized Single-Pass Wide Moves ---

    pub fn wide_uw(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !(0x1FF | (0x3F << 9) | (0x3F << 18) | (0x3F << 36) | (0x3F << 45));
            next |= Self::rotate_face_bits(old & 0x1FF);
            next |= ((old >> 9) & 0x3F) << 18; // R -> F
            next |= ((old >> 18) & 0x3F) << 36; // F -> L
            next |= ((old >> 36) & 0x3F) << 45; // L -> B
            next |= ((old >> 45) & 0x3F) << 9; // B -> R
            *b = next;
        }
    }

    pub fn wide_uw_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !(0x1FF | (0x3F << 9) | (0x3F << 18) | (0x3F << 36) | (0x3F << 45));
            next |= Self::rotate_face_bits_prime(old & 0x1FF);
            next |= ((old >> 18) & 0x3F) << 9; // F -> R
            next |= ((old >> 9) & 0x3F) << 45; // R -> B
            next |= ((old >> 45) & 0x3F) << 36; // B -> L
            next |= ((old >> 36) & 0x3F) << 18; // L -> F
            *b = next;
        }
    }

    pub fn wide_uw2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !(0x1FF | (0x3F << 9) | (0x3F << 18) | (0x3F << 36) | (0x3F << 45));
            next |= Self::rotate_face_bits_180(old & 0x1FF);
            next |= ((old >> 18) & 0x3F) << 45; // F <-> B
            next |= ((old >> 45) & 0x3F) << 18;
            next |= ((old >> 9) & 0x3F) << 36; // R <-> L
            next |= ((old >> 36) & 0x3F) << 9;
            *b = next;
        }
    }

    pub fn wide_dw(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Layers: R:12..18, F:21..27, L:39..45, B:48..54
            let mut next =
                old & !((0x1FF << 27) | (0x3F << 12) | (0x3F << 21) | (0x3F << 39) | (0x3F << 48));
            next |= Self::rotate_face_bits((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 21) & 0x3F) << 12; // F -> R
            next |= ((old >> 12) & 0x3F) << 48; // R -> B
            next |= ((old >> 48) & 0x3F) << 39; // B -> L
            next |= ((old >> 39) & 0x3F) << 21; // L -> F
            *b = next;
        }
    }

    pub fn wide_dw_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 27) | (0x3F << 12) | (0x3F << 21) | (0x3F << 39) | (0x3F << 48));
            next |= Self::rotate_face_bits_prime((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 12) & 0x3F) << 21; // R -> F
            next |= ((old >> 21) & 0x3F) << 39; // F -> L
            next |= ((old >> 39) & 0x3F) << 48; // L -> B
            next |= ((old >> 48) & 0x3F) << 12; // B -> R
            *b = next;
        }
    }

    pub fn wide_dw2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 27) | (0x3F << 12) | (0x3F << 21) | (0x3F << 39) | (0x3F << 48));
            next |= Self::rotate_face_bits_180((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 21) & 0x3F) << 48; // F <-> B
            next |= ((old >> 48) & 0x3F) << 21;
            next |= ((old >> 12) & 0x3F) << 39; // R <-> L
            next |= ((old >> 39) & 0x3F) << 12;
            *b = next;
        }
    }

    pub fn wide_lw(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Masks: U/F/D col 0,1 (0xDB), B col 1,2 (0x1B6)
            let mut next =
                old & !((0x1FF << 36) | 0xDB | (0xDB << 18) | (0xDB << 27) | (0x1B6 << 45));
            next |= Self::rotate_face_bits((old >> 36) & 0x1FF) << 36;
            next |= (old & 0xDB) << 18; // U -> F
            next |= (old & (0xDB << 18)) << 9; // F -> D
                                               // D bits 27..34 -> B bits 53..46 (rev)
            if (old & (1 << 27)) != 0 {
                next |= 1 << 53;
            }
            if (old & (1 << 28)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 50;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 33)) != 0 {
                next |= 1 << 47;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 46;
            }
            // B bits 53..46 -> U bits 0..7 (rev)
            if (old & (1 << 53)) != 0 {
                next |= 1 << 0;
            }
            if (old & (1 << 52)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 50)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 47)) != 0 {
                next |= 1 << 6;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 7;
            }
            *b = next;
        }
    }

    pub fn wide_lw_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 36) | 0xDB | (0xDB << 18) | (0xDB << 27) | (0x1B6 << 45));
            next |= Self::rotate_face_bits_prime((old >> 36) & 0x1FF) << 36;
            next |= (old & (0xDB << 18)) >> 18; // F -> U
            next |= (old & (0xDB << 27)) >> 9; // D -> F
                                               // U -> B (rev)
            if (old & (1 << 0)) != 0 {
                next |= 1 << 53;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 50;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 6)) != 0 {
                next |= 1 << 47;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 46;
            }
            // B -> D (rev)
            if (old & (1 << 53)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 52)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 50)) != 0 {
                next |= 1 << 30;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 47)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 34;
            }
            *b = next;
        }
    }

    pub fn wide_lw2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 36) | 0xDB | (0xDB << 18) | (0xDB << 27) | (0x1B6 << 45));
            next |= Self::rotate_face_bits_180((old >> 36) & 0x1FF) << 36;
            next |= (old & 0xDB) << 27; // U <-> D
            next |= (old & (0xDB << 27)) >> 27;
            // F <-> B (rev)
            if (old & (1 << 18)) != 0 {
                next |= 1 << 53;
            }
            if (old & (1 << 19)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 21)) != 0 {
                next |= 1 << 50;
            }
            if (old & (1 << 22)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 24)) != 0 {
                next |= 1 << 47;
            }
            if (old & (1 << 25)) != 0 {
                next |= 1 << 46;
            }
            if (old & (1 << 53)) != 0 {
                next |= 1 << 18;
            }
            if (old & (1 << 52)) != 0 {
                next |= 1 << 19;
            }
            if (old & (1 << 50)) != 0 {
                next |= 1 << 21;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 22;
            }
            if (old & (1 << 47)) != 0 {
                next |= 1 << 24;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 25;
            }
            *b = next;
        }
    }

    pub fn wide_rw(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Masks: U/F/D col 1,2 (0x1B6), B col 0,1 (0xDB)
            let mut next =
                old & !((0x1FF << 9) | 0x1B6 | (0x1B6 << 18) | (0x1B6 << 27) | (0xDB << 45));
            next |= Self::rotate_face_bits((old >> 9) & 0x1FF) << 9;
            next |= (old & (0x1B6 << 27)) >> 9; // D -> F
            next |= (old & (0x1B6 << 18)) >> 18; // F -> U
                                                 // U -> B (rev)
            if (old & (1 << 1)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 2)) != 0 {
                next |= 1 << 51;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 48;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 46;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 45;
            }
            // B -> D (rev)
            if (old & (1 << 52)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 51)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 48)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 45)) != 0 {
                next |= 1 << 35;
            }
            *b = next;
        }
    }

    pub fn wide_rw_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 9) | 0x1B6 | (0x1B6 << 18) | (0x1B6 << 27) | (0xDB << 45));
            next |= Self::rotate_face_bits_prime((old >> 9) & 0x1FF) << 9;
            next |= (old & 0x1B6) << 18; // U -> F
            next |= (old & (0x1B6 << 18)) << 9; // F -> D
                                                // D -> B (rev)
            if (old & (1 << 28)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 29)) != 0 {
                next |= 1 << 51;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 48;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 46;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 45;
            }
            // B -> U (rev)
            if (old & (1 << 52)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 51)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 48)) != 0 {
                next |= 1 << 5;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 7;
            }
            if (old & (1 << 45)) != 0 {
                next |= 1 << 8;
            }
            *b = next;
        }
    }

    pub fn wide_rw2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 9) | 0x1B6 | (0x1B6 << 18) | (0x1B6 << 27) | (0xDB << 45));
            next |= Self::rotate_face_bits_180((old >> 9) & 0x1FF) << 9;
            next |= (old & 0x1B6) << 27; // U <-> D
            next |= (old & (0x1B6 << 27)) >> 27;
            // F <-> B (rev)
            if (old & (1 << 19)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 20)) != 0 {
                next |= 1 << 51;
            }
            if (old & (1 << 22)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 23)) != 0 {
                next |= 1 << 48;
            }
            if (old & (1 << 25)) != 0 {
                next |= 1 << 46;
            }
            if (old & (1 << 26)) != 0 {
                next |= 1 << 45;
            }
            if (old & (1 << 52)) != 0 {
                next |= 1 << 19;
            }
            if (old & (1 << 51)) != 0 {
                next |= 1 << 20;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 22;
            }
            if (old & (1 << 48)) != 0 {
                next |= 1 << 23;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 25;
            }
            if (old & (1 << 45)) != 0 {
                next |= 1 << 26;
            }
            *b = next;
        }
    }

    pub fn wide_fw(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Mask: U[3..9](0x3F<<3), R[9,10, 12,13, 15,16](0xDB<<9), F[all](0x1FF<<18), D[27..33](0x3F<<27), L[37,38, 40,41, 43,44](0xDB<<37)
            let mut next =
                old & !((0x1FF << 18) | (0x3F << 3) | (0xDB << 9) | (0x3F << 27) | (0xDB << 37));
            next |= Self::rotate_face_bits((old >> 18) & 0x1FF) << 18;

            // U -> R
            if (old & (1 << 6)) != 0 {
                next |= 1 << 9;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 12;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 15;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 16;
            }
            // R -> D
            if (old & (1 << 9)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 12)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 15)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 10)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 30;
            }
            // D -> L
            if (old & (1 << 29)) != 0 {
                next |= 1 << 44;
            }
            if (old & (1 << 28)) != 0 {
                next |= 1 << 41;
            }
            if (old & (1 << 27)) != 0 {
                next |= 1 << 38;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 43;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 37;
            }
            // L -> U
            if (old & (1 << 44)) != 0 {
                next |= 1 << 6;
            }
            if (old & (1 << 41)) != 0 {
                next |= 1 << 7;
            }
            if (old & (1 << 38)) != 0 {
                next |= 1 << 8;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 5;
            }
            *b = next;
        }
    }

    pub fn wide_fw_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 18) | (0x3F << 3) | (0xDB << 9) | (0x3F << 27) | (0xDB << 37));
            next |= Self::rotate_face_bits_prime((old >> 18) & 0x1FF) << 18;
            // R -> U
            if (old & (1 << 9)) != 0 {
                next |= 1 << 6;
            }
            if (old & (1 << 12)) != 0 {
                next |= 1 << 7;
            }
            if (old & (1 << 15)) != 0 {
                next |= 1 << 8;
            }
            if (old & (1 << 10)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 5;
            }
            // D -> R
            if (old & (1 << 29)) != 0 {
                next |= 1 << 9;
            }
            if (old & (1 << 28)) != 0 {
                next |= 1 << 12;
            }
            if (old & (1 << 27)) != 0 {
                next |= 1 << 15;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 16;
            }
            // L -> D
            if (old & (1 << 44)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 41)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 38)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 30;
            }
            // U -> L
            if (old & (1 << 6)) != 0 {
                next |= 1 << 44;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 41;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 38;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 43;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 37;
            }
            *b = next;
        }
    }

    pub fn wide_fw2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 18) | (0x3F << 3) | (0xDB << 9) | (0x3F << 27) | (0xDB << 37));
            next |= Self::rotate_face_bits_180((old >> 18) & 0x1FF) << 18;
            // U <-> D (bits 3,4,5,6,7,8 <-> 32,31,30,29,28,27)
            if (old & (1 << 3)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 30;
            }
            if (old & (1 << 6)) != 0 {
                next |= 1 << 29;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 8)) != 0 {
                next |= 1 << 27;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 5;
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
            // R <-> L (bits 9,10, 12,13, 15,16 <-> 44,43, 41,40, 38,37)
            if (old & (1 << 9)) != 0 {
                next |= 1 << 44;
            }
            if (old & (1 << 10)) != 0 {
                next |= 1 << 43;
            }
            if (old & (1 << 12)) != 0 {
                next |= 1 << 41;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 15)) != 0 {
                next |= 1 << 38;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 37;
            }
            if (old & (1 << 44)) != 0 {
                next |= 1 << 9;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 41)) != 0 {
                next |= 1 << 12;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 38)) != 0 {
                next |= 1 << 15;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 16;
            }
            *b = next;
        }
    }

    pub fn wide_bw(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Mask: U[0..6](0x3F), R[10,11, 13,14, 16,17](0xDB<<10), B[all](0x1FF<<45), D[30..36](0x3F<<30), L[36,37, 39,40, 42,43](0xDB<<36)
            let mut next =
                old & !((0x1FF << 45) | 0x3F | (0xDB << 10) | (0x3F << 30) | (0xDB << 36));
            next |= Self::rotate_face_bits((old >> 45) & 0x1FF) << 45;
            // U -> L
            if (old & (1 << 2)) != 0 {
                next |= 1 << 36;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 39;
            }
            if (old & (1 << 0)) != 0 {
                next |= 1 << 42;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 37;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 43;
            }
            // L -> D
            if (old & (1 << 36)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 39)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 42)) != 0 {
                next |= 1 << 35;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 30;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 32;
            }
            // D -> R
            if (old & (1 << 33)) != 0 {
                next |= 1 << 17;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 14;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 11;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 16;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 10;
            }
            // R -> U
            if (old & (1 << 17)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 14)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 11)) != 0 {
                next |= 1 << 0;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 5;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 10)) != 0 {
                next |= 1 << 3;
            }
            *b = next;
        }
    }

    pub fn wide_bw_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 45) | 0x3F | (0xDB << 10) | (0x3F << 30) | (0xDB << 36));
            next |= Self::rotate_face_bits_prime((old >> 45) & 0x1FF) << 45;
            // L -> U
            if (old & (1 << 36)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 39)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 42)) != 0 {
                next |= 1 << 0;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 5;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 3;
            }
            // D -> L
            if (old & (1 << 33)) != 0 {
                next |= 1 << 36;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 39;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 42;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 37;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 43;
            }
            // R -> D
            if (old & (1 << 17)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 14)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 11)) != 0 {
                next |= 1 << 35;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 30;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 10)) != 0 {
                next |= 1 << 32;
            }
            // U -> R
            if (old & (1 << 2)) != 0 {
                next |= 1 << 17;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 14;
            }
            if (old & (1 << 0)) != 0 {
                next |= 1 << 11;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 16;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 10;
            }
            *b = next;
        }
    }

    pub fn wide_bw2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !((0x1FF << 45) | 0x3F | (0xDB << 10) | (0x3F << 30) | (0xDB << 36));
            next |= Self::rotate_face_bits_180((old >> 45) & 0x1FF) << 45;
            // U <-> D (bits 0,1,2,3,4,5 <-> 35,34,33,32,31,30)
            if (old & (1 << 0)) != 0 {
                next |= 1 << 35;
            }
            if (old & (1 << 1)) != 0 {
                next |= 1 << 34;
            }
            if (old & (1 << 2)) != 0 {
                next |= 1 << 33;
            }
            if (old & (1 << 3)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 30;
            }
            if (old & (1 << 35)) != 0 {
                next |= 1 << 0;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 33)) != 0 {
                next |= 1 << 2;
            }
            if (old & (1 << 32)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 5;
            }
            // L <-> R (bits 36,37, 39,40, 42,43 <-> 17,16, 14,13, 11,10)
            if (old & (1 << 36)) != 0 {
                next |= 1 << 17;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 16;
            }
            if (old & (1 << 39)) != 0 {
                next |= 1 << 14;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 42)) != 0 {
                next |= 1 << 11;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 17)) != 0 {
                next |= 1 << 36;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 37;
            }
            if (old & (1 << 14)) != 0 {
                next |= 1 << 39;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 11)) != 0 {
                next |= 1 << 42;
            }
            if (old & (1 << 10)) != 0 {
                next |= 1 << 43;
            }
            *b = next;
        }
    }
}
