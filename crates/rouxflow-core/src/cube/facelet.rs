use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    White = 0,
    Yellow = 1,
    Green = 2,
    Blue = 3,
    Red = 4,
    Orange = 5,
}

/// A Rubik's Cube represented by its 54 facelets.
/// Face order: U(0), R(1), F(2), D(3), L(4), B(5)
/// Each face is 3x3 (9 facelets), indexed 0-8 (top-left to bottom-right)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaceletCube {
    pub facelets: Vec<Color>,
}

impl Default for FaceletCube {
    fn default() -> Self {
        Self::new()
    }
}

impl FaceletCube {
    pub fn new() -> Self {
        let mut facelets = vec![Color::White; 54];
        for i in 0..9 { facelets[i + 0] = Color::White; }  // U: White
        for i in 0..9 { facelets[i + 9] = Color::Red; }    // R: Red
        for i in 0..9 { facelets[i + 18] = Color::Green; } // F: Green
        for i in 0..9 { facelets[i + 27] = Color::Yellow; }// D: Yellow
        for i in 0..9 { facelets[i + 36] = Color::Orange; }// L: Orange
        for i in 0..9 { facelets[i + 45] = Color::Blue; }  // B: Blue
        Self { facelets }
    }

    fn rotate_face(&mut self, face_idx: usize) {
        let start = face_idx * 9;
        let old = self.facelets.clone();
        self.facelets[start + 0] = old[start + 6];
        self.facelets[start + 1] = old[start + 3];
        self.facelets[start + 2] = old[start + 0];
        self.facelets[start + 3] = old[start + 7];
        self.facelets[start + 4] = old[start + 4];
        self.facelets[start + 5] = old[start + 1];
        self.facelets[start + 6] = old[start + 8];
        self.facelets[start + 7] = old[start + 5];
        self.facelets[start + 8] = old[start + 2];
    }

    pub fn apply_move(&mut self, move_str: &str) {
        // Handle potential concatenated moves like S'S or x2y
        let clean_move = move_str.trim();
        if clean_move.is_empty() { return; }

        let (m, count) = if clean_move.ends_with("2'") {
            (&clean_move[0..clean_move.len()-2], 2)
        } else if clean_move.ends_with('2') {
            (&clean_move[0..clean_move.len()-1], 2)
        } else if clean_move.ends_with('\'') {
            (&clean_move[0..clean_move.len()-1], 3)
        } else {
            (clean_move, 1)
        };

        match m {
            "U" => self.move_u(count), "D" => self.move_d(count),
            "L" => self.move_l(count), "R" => self.move_r(count),
            "F" => self.move_f(count), "B" => self.move_b(count),
            "M" => self.move_m(count), "S" => self.move_s(count), "E" => self.move_e(count),
            "x" => self.rotate_cube_x(count), "y" => self.rotate_cube_y(count), "z" => self.rotate_cube_z(count),
            "r" => { self.move_r(count); self.move_m(3 * count % 4); },
            "l" => { self.move_l(count); self.move_m(count); },
            _ => {
                // Handle split cases like S'S
                if clean_move.starts_with("S'S") {
                    self.apply_move("S'"); self.apply_move("S");
                }
            }
        }
        
        #[cfg(debug_assertions)]
        self.validate();
    }

    /// Ensure cube integrity (9 of each color)
    fn validate(&self) {
        let mut counts = [0; 6];
        for &color in &self.facelets {
            counts[color as usize] += 1;
        }
        for (i, &count) in counts.iter().enumerate() {
            if count != 9 {
                eprintln!("CORRUPTION DETECTED: Color {:?} has count {}", i, count);
            }
        }
    }

    fn swap4(&mut self, f1: usize, i1: usize, f2: usize, i2: usize, f3: usize, i3: usize, f4: usize, i4: usize) {
        let tmp = self.facelets[f1 * 9 + i1];
        self.facelets[f1 * 9 + i1] = self.facelets[f4 * 9 + i4];
        self.facelets[f4 * 9 + i4] = self.facelets[f3 * 9 + i3];
        self.facelets[f3 * 9 + i3] = self.facelets[f2 * 9 + i2];
        self.facelets[f2 * 9 + i2] = tmp;
    }

    fn move_u(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(0);
            for i in 0..3 { self.swap4(2, i, 4, i, 5, i, 1, i); }
        }
    }

    fn move_d(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(3);
            for i in 6..9 { self.swap4(2, i, 1, i, 5, i, 4, i); }
        }
    }

    fn move_l(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(4);
            let u_idx = [0, 3, 6]; let f_idx = [0, 3, 6]; let d_idx = [0, 3, 6]; let b_idx = [8, 5, 2];
            for i in 0..3 { self.swap4(0, u_idx[i], 2, f_idx[i], 3, d_idx[i], 5, b_idx[i]); }
        }
    }

    fn move_r(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(1);
            let u_idx = [2, 5, 8]; let f_idx = [2, 5, 8]; let d_idx = [2, 5, 8]; let b_idx = [6, 3, 0];
            for i in 0..3 { self.swap4(0, u_idx[i], 5, b_idx[i], 3, d_idx[i], 2, f_idx[i]); }
        }
    }

    fn move_f(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(2);
            let u_idx = [6, 7, 8]; let r_idx = [0, 3, 6]; let d_idx = [2, 1, 0]; let l_idx = [8, 5, 2];
            for i in 0..3 { self.swap4(0, u_idx[i], 1, r_idx[i], 3, d_idx[i], 4, l_idx[i]); }
        }
    }

    fn move_b(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(5);
            let u_idx = [2, 1, 0]; let l_idx = [0, 3, 6]; let d_idx = [6, 7, 8]; let r_idx = [8, 5, 2];
            for i in 0..3 { self.swap4(0, u_idx[i], 4, l_idx[i], 3, d_idx[i], 1, r_idx[i]); }
        }
    }

    fn move_m(&mut self, count: usize) {
        for _ in 0..count {
            // M follows L (U -> F -> D -> B)
            let u_idx = [1, 4, 7]; let f_idx = [1, 4, 7]; let d_idx = [1, 4, 7]; let b_idx = [7, 4, 1];
            for i in 0..3 { self.swap4(0, u_idx[i], 2, f_idx[i], 3, d_idx[i], 5, b_idx[i]); }
        }
    }

    fn move_s(&mut self, count: usize) {
        for _ in 0..count {
            // S follows F (U -> R -> D -> L)
            let u_idx = [3, 4, 5]; let r_idx = [1, 4, 7]; let d_idx = [5, 4, 3]; let l_idx = [7, 4, 1];
            for i in 0..3 { self.swap4(0, u_idx[i], 1, r_idx[i], 3, d_idx[i], 4, l_idx[i]); }
        }
    }

    fn move_e(&mut self, count: usize) {
        for _ in 0..count {
            // E follows D (F -> R -> B -> L)
            let f_idx = [3, 4, 5]; let r_idx = [3, 4, 5]; let b_idx = [3, 4, 5]; let l_idx = [3, 4, 5];
            for i in 0..3 { self.swap4(2, f_idx[i], 1, r_idx[i], 5, b_idx[i], 4, l_idx[i]); }
        }
    }

    fn rotate_cube_x(&mut self, count: usize) {
        for _ in 0..count {
            self.move_r(1); self.move_m(3); self.move_l(3);
        }
    }

    fn rotate_cube_y(&mut self, count: usize) {
        for _ in 0..count {
            self.move_u(1); self.move_e(3); self.move_d(3);
        }
    }

    fn rotate_cube_z(&mut self, count: usize) {
        for _ in 0..count {
            self.move_f(1); self.move_s(1); self.move_b(3);
        }
    }

    /// Dump a human-readable 2D representation of the cube
    pub fn dump_debug(&self) {
        let f = &self.facelets;
        let c = |idx: usize| match f[idx] {
            Color::White => "W",
            Color::Yellow => "Y",
            Color::Green => "G",
            Color::Blue => "B",
            Color::Red => "R",
            Color::Orange => "O",
        };

        println!("      +-------+");
        for r in 0..3 {
            println!("      | {} {} {} |", c(0*9 + r*3), c(0*9 + r*3+1), c(0*9 + r*3+2));
        }
        println!("+-------+-------+-------+-------+");
        for r in 0..3 {
            println!("| {} {} {} | {} {} {} | {} {} {} | {} {} {} |",
                c(4*9 + r*3), c(4*9 + r*3+1), c(4*9 + r*3+2), // L
                c(2*9 + r*3), c(2*9 + r*3+1), c(2*9 + r*3+2), // F
                c(1*9 + r*3), c(1*9 + r*3+1), c(1*9 + r*3+2), // R
                c(5*9 + r*3), c(5*9 + r*3+1), c(5*9 + r*3+2), // B
            );
        }
        println!("+-------+-------+-------+-------+");
        for r in 0..3 {
            println!("      | {} {} {} |", c(3*9 + r*3), c(3*9 + r*3+1), c(3*9 + r*3+2));
        }
        println!("      +-------+");
    }
}
