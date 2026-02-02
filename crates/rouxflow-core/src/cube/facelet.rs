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
        match move_str {
            "U" => self.move_u(1), "U'" => self.move_u(3), "U2" => self.move_u(2),
            "D" => self.move_d(1), "D'" => self.move_d(3), "D2" => self.move_d(2),
            "L" => self.move_l(1), "L'" => self.move_l(3), "L2" => self.move_l(2),
            "R" => self.move_r(1), "R'" => self.move_r(3), "R2" => self.move_r(2),
            "F" => self.move_f(1), "F'" => self.move_f(3), "F2" => self.move_f(2),
            "B" => self.move_b(1), "B'" => self.move_b(3), "B2" => self.move_b(2),
            _ => {}
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
            let u = [0, 3, 6]; let f = [0, 3, 6]; let d = [0, 3, 6]; let b = [8, 5, 2];
            for i in 0..3 { self.swap4(0, u[i], 2, f[i], 3, d[i], 5, b[i]); }
        }
    }

    fn move_r(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(1);
            let u = [2, 5, 8]; let f = [2, 5, 8]; let d = [2, 5, 8]; let b = [6, 3, 0];
            for i in 0..3 { self.swap4(0, u[i], 5, b[i], 3, d[i], 2, f[i]); }
        }
    }

    fn move_f(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(2);
            let u = [6, 7, 8]; let r = [0, 3, 6]; let d = [2, 1, 0]; let l = [8, 5, 2];
            for i in 0..3 { self.swap4(0, u[i], 1, r[i], 3, d[i], 4, l[i]); }
        }
    }

    pub fn move_b(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_face(5);
            let u = [2, 1, 0]; let l = [0, 3, 6]; let d = [6, 7, 8]; let r = [8, 5, 2];
            for i in 0..3 { self.swap4(0, u[i], 4, l[i], 3, d[i], 1, r[i]); }
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
