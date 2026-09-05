use crate::bitcube::BitCube;
use crate::move_indices::SliceMove;

impl BitCube {
    /// Applies a slice move (M, E, S) and its variations.
    pub fn apply_slice_move(&mut self, m: SliceMove) {
        match m {
            SliceMove::M => self.slice_m(),
            SliceMove::Mp => self.slice_m_prime(),
            SliceMove::M2 => self.slice_m2(),

            SliceMove::E => self.slice_e(),
            SliceMove::Ep => self.slice_e_prime(),
            SliceMove::E2 => self.slice_e2(),

            SliceMove::S => self.slice_s(),
            SliceMove::Sp => self.slice_s_prime(),
            SliceMove::S2 => self.slice_s2(),
        }
    }

    /// Rotates the Middle slice (between L and R) downwards (following L).
    /// Affects middle columns of U, F, D, and B.
    pub fn slice_m(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Clear middle columns: U(1,4,7), F(19,22,25), D(28,31,34), B(52,49,46)
            let mut next =
                old & !(0x92 | (0x92 << 18) | (0x92 << 27) | (1 << 52 | 1 << 49 | 1 << 46));

            // Cycle middle columns: U -> F -> D -> B (reversed) -> U
            next |= (old & 0x92) << 18; // U mid col -> F mid col
            next |= (old & (0x92 << 18)) << 9; // F mid col -> D mid col

            // D mid col -> B mid col (reversed)
            if (old & (1 << 28)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 34)) != 0 {
                next |= 1 << 46;
            }

            // B mid col -> U mid col (reversed)
            if (old & (1 << 52)) != 0 {
                next |= 1 << 1;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 7;
            }
            *b = next;
        }
    }

    /// Rotates the Standing slice (between F and B) clockwise (following F).
    /// Affects center stickers of U, R, D, L.
    pub fn slice_s(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Clear horizontal/vertical center stripes
            let mut next = old & !((0x7 << 3) | (0x49 << 10) | (0x7 << 30) | (0x49 << 37));

            // U mid row -> R mid col
            if (old & (1 << 3)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 16;
            }

            // R mid col -> D mid row (reversed)
            if (old & (1 << 10)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 30;
            }

            // D mid row -> L mid col (reversed)
            if (old & (1 << 32)) != 0 {
                next |= 1 << 43;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 37;
            }

            // L mid col -> U mid row
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

    /// Rotates the Equatorial slice (between U and D) clockwise (following D).
    /// Affects middle rows of F, R, B, L.
    pub fn slice_e(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            // Clear middle rows of side faces
            let mut next = old & !((0x7 << 21) | (0x7 << 12) | (0x7 << 48) | (0x7 << 39));

            // Cycle side middle rows: F -> R -> B -> L -> F
            next |= ((old >> 21) & 0x7) << 12; // F mid -> R mid
            next |= ((old >> 12) & 0x7) << 48; // R mid -> B mid
            next |= ((old >> 48) & 0x7) << 39; // B mid -> L mid
            next |= ((old >> 39) & 0x7) << 21; // L mid -> F mid
            *b = next;
        }
    }

    // --- Slice Primes ---

    pub fn slice_m_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !(0x92 | (0x92 << 18) | (0x92 << 27) | (1 << 52 | 1 << 49 | 1 << 46));

            // Cycle middle columns: U -> B (rev) -> D -> F -> U
            next |= (old & (0x92 << 18)) >> 18; // F -> U
            next |= (old & (0x92 << 27)) >> 9; // D -> F

            // U mid col -> B mid col (reversed)
            if (old & (1 << 1)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 7)) != 0 {
                next |= 1 << 46;
            }

            // B mid col -> D mid col (reversed)
            if (old & (1 << 52)) != 0 {
                next |= 1 << 28;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 34;
            }
            *b = next;
        }
    }

    pub fn slice_s_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 3) | (0x49 << 10) | (0x7 << 30) | (0x49 << 37));

            // R mid col -> U mid row
            if (old & (1 << 10)) != 0 {
                next |= 1 << 3;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 4;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 5;
            }

            // D mid row -> R mid col (reversed)
            if (old & (1 << 32)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 31)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 30)) != 0 {
                next |= 1 << 16;
            }

            // L mid col -> D mid row (reversed)
            if (old & (1 << 43)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 30;
            }

            // U mid row -> L mid col
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

    pub fn slice_e_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 21) | (0x7 << 12) | (0x7 << 48) | (0x7 << 39));
            next |= ((old >> 12) & 0x7) << 21; // R -> F
            next |= ((old >> 48) & 0x7) << 12; // B -> R
            next |= ((old >> 39) & 0x7) << 48; // L -> B
            next |= ((old >> 21) & 0x7) << 39; // F -> L
            *b = next;
        }
    }

    // --- Slice Doubles ---

    pub fn slice_m2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next =
                old & !(0x92 | (0x92 << 18) | (0x92 << 27) | (1 << 52 | 1 << 49 | 1 << 46));

            // U <-> D
            next |= (old & 0x92) << 27;
            next |= (old & (0x92 << 27)) >> 27;

            // F <-> B (reversed)
            if (old & (1 << 19)) != 0 {
                next |= 1 << 52;
            }
            if (old & (1 << 22)) != 0 {
                next |= 1 << 49;
            }
            if (old & (1 << 25)) != 0 {
                next |= 1 << 46;
            }
            if (old & (1 << 52)) != 0 {
                next |= 1 << 19;
            }
            if (old & (1 << 49)) != 0 {
                next |= 1 << 22;
            }
            if (old & (1 << 46)) != 0 {
                next |= 1 << 25;
            }
            *b = next;
        }
    }

    pub fn slice_s2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 3) | (0x49 << 10) | (0x7 << 30) | (0x49 << 37));

            // U <-> D (reversed row)
            if (old & (1 << 3)) != 0 {
                next |= 1 << 32;
            }
            if (old & (1 << 4)) != 0 {
                next |= 1 << 31;
            }
            if (old & (1 << 5)) != 0 {
                next |= 1 << 30;
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

            // R <-> L (reversed col)
            if (old & (1 << 10)) != 0 {
                next |= 1 << 43;
            }
            if (old & (1 << 13)) != 0 {
                next |= 1 << 40;
            }
            if (old & (1 << 16)) != 0 {
                next |= 1 << 37;
            }
            if (old & (1 << 43)) != 0 {
                next |= 1 << 10;
            }
            if (old & (1 << 40)) != 0 {
                next |= 1 << 13;
            }
            if (old & (1 << 37)) != 0 {
                next |= 1 << 16;
            }
            *b = next;
        }
    }

    pub fn slice_e2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = old & !((0x7 << 21) | (0x7 << 12) | (0x7 << 48) | (0x7 << 39));
            next |= ((old >> 21) & 0x7) << 48; // F -> B
            next |= ((old >> 48) & 0x7) << 21; // B -> F
            next |= ((old >> 12) & 0x7) << 39; // R -> L
            next |= ((old >> 39) & 0x7) << 12; // L -> R
            *b = next;
        }
    }
}
