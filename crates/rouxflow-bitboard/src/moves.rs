use crate::bitcube::BitCube;
use crate::move_indices::*;

impl BitCube {
    /// High-level entry point to apply any move using the Move enum.
    /// This is type-safe and avoids string parsing for performance.
    pub fn apply_move_enum(&mut self, m: Move) {
        match m {
            Move::Face(fm) => self.apply_face_move(fm),
            Move::Slice(sm) => self.apply_slice_move(sm),
            Move::Wide(wm) => self.apply_wide_move(wm),
            Move::Rotate(rm) => self.apply_rotation(rm),
        }
    }

    /// Convenience method to apply a move by its standard notation string (e.g., "U", "R'", "f2").
    /// Note: This is slower than apply_move_enum because it involves string matching.
    pub fn apply_move(&mut self, move_str: &str) {
        let clean_move = move_str.trim();
        if clean_move.is_empty() {
            return;
        }

        // Determine if it's a double move (2) or prime move (')
        let (m, count) = if clean_move.ends_with("2'") || clean_move.ends_with('2') {
            (&clean_move[0..clean_move.len() - 1], 2)
        } else if clean_move.ends_with('\'') {
            (&clean_move[0..clean_move.len() - 1], 3)
        } else {
            (clean_move, 1)
        };

        // Apply the base rotation N times
        for _ in 0..count {
            match m {
                "U" => self.face_u(),
                "D" => self.face_d(),
                "L" => self.face_l(),
                "R" => self.face_r(),
                "F" => self.face_f(),
                "B" => self.face_b(),
                "M" => self.slice_m(),
                "S" => self.slice_s(),
                "E" => self.slice_e(),
                "r" => self.wide_rw(),
                "l" => self.wide_lw(),
                "f" => self.wide_fw(),
                "b" => self.wide_bw(),
                "u" => self.wide_uw(),
                "d" => self.wide_dw(),
                "x" => self.rot_x(),
                "y" => self.rot_y(),
                "z" => self.rot_z(),
                _ => {}
            }
        }
    }
}
