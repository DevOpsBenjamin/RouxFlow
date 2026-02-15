use std::fmt;

/// A Rubik's Cube represented by bitboards for high-performance search.
/// Each color (White, Yellow, Green, Blue, Red, Orange) has a 64-bit integer.
/// Indices 0-53 are used (out of 64 bits).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitCube {
    pub boards: [u64; 6],
}

impl BitCube {
    /// Optimized constructor that sets bits directly.
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

    pub fn get_color_at(&self, idx: usize) -> usize {
        for i in 0..6 {
            if (self.boards[i] & (1 << idx)) != 0 { return i; }
        }
        0
    }
}

impl fmt::Display for BitCube {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\x1b[0m";
        const COLORS: [(&str, char); 6] = [
            ("\x1b[97m", 'W'),  // White
            ("\x1b[93m", 'Y'),  // Yellow
            ("\x1b[32m", 'G'),  // Green
            ("\x1b[34m", 'B'),  // Blue
            ("\x1b[31m", 'R'),  // Red
            ("\x1b[38;5;208m", 'O'), // Orange
        ];

        let sticker = |bit_idx: usize| -> (char, &'static str) {
            let c = self.get_color_at(bit_idx);
            (COLORS[c].1, COLORS[c].0)
        };

        let print_row = |f: &mut fmt::Formatter<'_>, face_offset: usize, row: usize| -> fmt::Result {
            for col in 0..3 {
                let (ch, color) = sticker(face_offset + row * 3 + col);
                write!(f, "{}{}{} ", color, ch, RESET)?;
            }
            Ok(())
        };

        let pad = "      ";
        for row in 0..3 {
            write!(f, "{}", pad)?;
            print_row(f, 0, row)?;
            writeln!(f)?;
        }
        for row in 0..3 {
            print_row(f, 36, row)?; // L
            print_row(f, 18, row)?; // F
            print_row(f, 9, row)?;  // R
            print_row(f, 45, row)?; // B
            writeln!(f)?;
        }
        for row in 0..3 {
            write!(f, "{}", pad)?;
            print_row(f, 27, row)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
