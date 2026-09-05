use crate::bitcube::BitCube;
use crate::move_indices::Rotation;

impl BitCube {
    /// Applies a global cube rotation along the X, Y, or Z axis.
    pub fn apply_rotation(&mut self, m: Rotation) {
        match m {
            Rotation::X => self.rot_x(),
            Rotation::Xp => self.rot_x_prime(),
            Rotation::X2 => self.rot_x2(),

            Rotation::Y => self.rot_y(),
            Rotation::Yp => self.rot_y_prime(),
            Rotation::Y2 => self.rot_y2(),

            Rotation::Z => self.rot_z(),
            Rotation::Zp => self.rot_z_prime(),
            Rotation::Z2 => self.rot_z2(),
        }
    }

    /// Rotates the whole cube on the X axis (following R).
    pub fn rot_x(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // R, L faces
            next |= Self::rot90_cw((old >> 9) & 0x1FF) << 9;
            next |= Self::rot90_ccw((old >> 36) & 0x1FF) << 36;
            // Side faces cycle: U <- F, F <- D, D <- B(rev), B <- U(rev)
            next |= (old >> 18) & 0x1FF; // F -> U
            next |= ((old >> 27) & 0x1FF) << 18; // D -> F
            next |= Self::rot180((old >> 45) & 0x1FF) << 27; // B -> D (rev)
            next |= Self::rot180(old & 0x1FF) << 45; // U -> B (rev)
            *b = next;
        }
    }

    /// Rotates the whole cube on the X axis counter-clockwise (following R').
    pub fn rot_x_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // R, L faces
            next |= Self::rot90_ccw((old >> 9) & 0x1FF) << 9;
            next |= Self::rot90_cw((old >> 36) & 0x1FF) << 36;
            // Side faces cycle: F <- U, D <- F, B <- D(rev), U <- B(rev)
            next |= (old & 0x1FF) << 18; // U -> F
            next |= ((old >> 18) & 0x1FF) << 27; // F -> D
            next |= Self::rot180((old >> 27) & 0x1FF) << 45; // D -> B (rev)
            next |= Self::rot180((old >> 45) & 0x1FF); // B -> U (rev)
            *b = next;
        }
    }

    /// Rotates the whole cube on the Z axis (following F).
    pub fn rot_z(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // F, B faces
            next |= Self::rot90_cw((old >> 18) & 0x1FF) << 18;
            next |= Self::rot90_ccw((old >> 45) & 0x1FF) << 45;
            // Side faces cycle (all 90 CW): U <- L, L <- D, D <- R, R <- U
            next |= Self::rot90_cw((old >> 36) & 0x1FF); // L -> U
            next |= Self::rot90_cw((old >> 27) & 0x1FF) << 36; // D -> L
            next |= Self::rot90_cw((old >> 9) & 0x1FF) << 27; // R -> D
            next |= Self::rot90_cw(old & 0x1FF) << 9; // U -> R
            *b = next;
        }
    }

    /// Rotates the whole cube on the Z axis counter-clockwise (following F').
    pub fn rot_z_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // F, B faces
            next |= Self::rot90_ccw((old >> 18) & 0x1FF) << 18;
            next |= Self::rot90_cw((old >> 45) & 0x1FF) << 45;
            // Side faces cycle (all 90 CCW): U <- R, R <- D, D <- L, L <- U
            next |= Self::rot90_ccw((old >> 9) & 0x1FF); // R -> U
            next |= Self::rot90_ccw((old >> 27) & 0x1FF) << 9; // D -> R
            next |= Self::rot90_ccw((old >> 36) & 0x1FF) << 27; // L -> D
            next |= Self::rot90_ccw(old & 0x1FF) << 36; // U -> L
            *b = next;
        }
    }

    /// 180-degree rotation on Z axis.
    pub fn rot_z2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // F, B faces
            next |= Self::rot180((old >> 18) & 0x1FF) << 18;
            next |= Self::rot180((old >> 45) & 0x1FF) << 45;
            // Side faces cycle (all 180): U <-> D, L <-> R
            next |= Self::rot180((old >> 27) & 0x1FF); // D -> U
            next |= Self::rot180(old & 0x1FF) << 27; // U -> D
            next |= Self::rot180((old >> 9) & 0x1FF) << 36; // R -> L
            next |= Self::rot180((old >> 36) & 0x1FF) << 9; // L -> R
            *b = next;
        }
    }

    /// Helper: 90-degree clockwise rotation of a single face's bits.
    pub(crate) fn rot90_cw(face_bits: u64) -> u64 {
        let b = face_bits;
        let mut n = 0;
        if (b & (1 << 0)) != 0 {
            n |= 1 << 2;
        }
        if (b & (1 << 1)) != 0 {
            n |= 1 << 5;
        }
        if (b & (1 << 2)) != 0 {
            n |= 1 << 8;
        }
        if (b & (1 << 5)) != 0 {
            n |= 1 << 7;
        }
        if (b & (1 << 8)) != 0 {
            n |= 1 << 6;
        }
        if (b & (1 << 7)) != 0 {
            n |= 1 << 3;
        }
        if (b & (1 << 6)) != 0 {
            n |= 1 << 0;
        }
        if (b & (1 << 3)) != 0 {
            n |= 1 << 1;
        }
        if (b & (1 << 4)) != 0 {
            n |= 1 << 4;
        }
        n
    }

    /// Helper: 90-degree counter-clockwise rotation of a single face's bits.
    pub(crate) fn rot90_ccw(face_bits: u64) -> u64 {
        let b = face_bits;
        let mut n = 0;
        if (b & (1 << 0)) != 0 {
            n |= 1 << 6;
        }
        if (b & (1 << 1)) != 0 {
            n |= 1 << 3;
        }
        if (b & (1 << 2)) != 0 {
            n |= 1 << 0;
        }
        if (b & (1 << 5)) != 0 {
            n |= 1 << 1;
        }
        if (b & (1 << 8)) != 0 {
            n |= 1 << 2;
        }
        if (b & (1 << 7)) != 0 {
            n |= 1 << 5;
        }
        if (b & (1 << 6)) != 0 {
            n |= 1 << 8;
        }
        if (b & (1 << 3)) != 0 {
            n |= 1 << 7;
        }
        if (b & (1 << 4)) != 0 {
            n |= 1 << 4;
        }
        n
    }

    /// Helper: 180-degree rotation of a single face's bits.
    pub(crate) fn rot180(face_bits: u64) -> u64 {
        let b = face_bits;
        let mut n = 0;
        if (b & (1 << 0)) != 0 {
            n |= 1 << 8;
        }
        if (b & (1 << 1)) != 0 {
            n |= 1 << 7;
        }
        if (b & (1 << 2)) != 0 {
            n |= 1 << 6;
        }
        if (b & (1 << 5)) != 0 {
            n |= 1 << 3;
        }
        if (b & (1 << 8)) != 0 {
            n |= 1 << 0;
        }
        if (b & (1 << 7)) != 0 {
            n |= 1 << 1;
        }
        if (b & (1 << 6)) != 0 {
            n |= 1 << 2;
        }
        if (b & (1 << 3)) != 0 {
            n |= 1 << 5;
        }
        if (b & (1 << 4)) != 0 {
            n |= 1 << 4;
        }
        n
    }

    /// Rotates the whole cube on the Y axis (following U).
    pub fn rot_y(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            next |= Self::rot90_cw(old & 0x1FF); // Top face
            next |= Self::rot90_ccw((old >> 27) & 0x1FF) << 27; // Down face
                                                                // side face cycle (middle stripes skip top/bottom layers)
            next |= ((old >> 18) & 0x1FF) << 36; // F -> L
            next |= ((old >> 36) & 0x1FF) << 45; // L -> B
            next |= ((old >> 45) & 0x1FF) << 9; // B -> R
            next |= ((old >> 9) & 0x1FF) << 18; // R -> F
            *b = next;
        }
    }

    /// Rotates the whole cube on the Y axis counter-clockwise (following U').
    pub fn rot_y_prime(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            next |= Self::rot90_ccw(old & 0x1FF);
            next |= Self::rot90_cw((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 18) & 0x1FF) << 9; // F -> R
            next |= ((old >> 9) & 0x1FF) << 45; // R -> B
            next |= ((old >> 45) & 0x1FF) << 36; // B -> L
            next |= ((old >> 36) & 0x1FF) << 18; // L -> F
            *b = next;
        }
    }

    /// 180-degree rotation on Y axis.
    pub fn rot_y2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            next |= Self::rot180(old & 0x1FF);
            next |= Self::rot180((old >> 27) & 0x1FF) << 27;
            next |= ((old >> 18) & 0x1FF) << 45; // F <-> B
            next |= ((old >> 45) & 0x1FF) << 18;
            next |= ((old >> 36) & 0x1FF) << 9; // L <-> R
            next |= ((old >> 9) & 0x1FF) << 36;
            *b = next;
        }
    }

    /// 180-degree rotation on X axis.
    pub fn rot_x2(&mut self) {
        for b in &mut self.boards {
            let old = *b;
            let mut next = 0;
            // Side faces cycle: U <-> D, F <-> B (rev)
            next |= (old >> 27) & 0x1FF; // D -> U
            next |= (old & 0x1FF) << 27; // U -> D
            next |= Self::rot180((old >> 45) & 0x1FF) << 18; // B -> F (rev)
            next |= Self::rot180((old >> 18) & 0x1FF) << 45; // F -> B (rev)
                                                             // R, L faces rotate 180
            next |= Self::rot180((old >> 9) & 0x1FF) << 9;
            next |= Self::rot180((old >> 36) & 0x1FF) << 36;
            *b = next;
        }
    }
}
