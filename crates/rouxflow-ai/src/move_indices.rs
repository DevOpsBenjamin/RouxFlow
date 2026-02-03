#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    U, Up, U2,
    D, Dp, D2,
    L, Lp, L2,
    R, Rp, R2,
    F, Fp, F2,
    B, Bp, B2,
    M, Mp, M2,
    Rw, Rwp, Rw2,
}

impl Move {
    pub const ALL: [Move; 21] = [
        Move::U, Move::Up, Move::U2,
        Move::D, Move::Dp, Move::D2,
        Move::L, Move::Lp, Move::L2,
        Move::Rw, Move::Rwp, Move::Rw2,
        Move::F, Move::Fp, Move::F2,
        Move::B, Move::Bp, Move::B2,
        Move::M, Move::Mp, Move::M2,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Move::U => "U", Move::Up => "U'", Move::U2 => "U2",
            Move::D => "D", Move::Dp => "D'", Move::D2 => "D2",
            Move::L => "L", Move::Lp => "L'", Move::L2 => "L2",
            Move::R => "R", Move::Rp => "R'", Move::R2 => "R2",
            Move::F => "F", Move::Fp => "F'", Move::F2 => "F2",
            Move::B => "B", Move::Bp => "B'", Move::B2 => "B2",
            Move::M => "M", Move::Mp => "M'", Move::M2 => "M2",
            Move::Rw => "r", Move::Rwp => "r'", Move::Rw2 => "r2",
        }
    }

    pub fn face(&self) -> u8 {
        match self {
            Move::U | Move::Up | Move::U2 => 0,
            Move::D | Move::Dp | Move::D2 => 1,
            Move::L | Move::Lp | Move::L2 => 2,
            Move::R | Move::Rp | Move::R2 => 3,
            Move::F | Move::Fp | Move::F2 => 4,
            Move::B | Move::Bp | Move::B2 => 5,
            Move::M | Move::Mp | Move::M2 => 6,
            Move::Rw | Move::Rwp | Move::Rw2 => 3, // r is basically R
        }
    }

    pub fn inverse(&self) -> Move {
        match self {
            Move::U  => Move::Up, Move::Up => Move::U, Move::U2 => Move::U2,
            Move::D  => Move::Dp, Move::Dp => Move::D, Move::D2 => Move::D2,
            Move::L  => Move::Lp, Move::Lp => Move::L, Move::L2 => Move::L2,
            Move::R  => Move::Rp, Move::Rp => Move::R, Move::R2 => Move::R2,
            Move::F  => Move::Fp, Move::Fp => Move::F, Move::F2 => Move::F2,
            Move::B  => Move::Bp, Move::Bp => Move::B, Move::B2 => Move::B2,
            Move::M  => Move::Mp, Move::Mp => Move::M, Move::M2 => Move::M2,
            Move::Rw  => Move::Rwp, Move::Rwp => Move::Rw, Move::Rw2 => Move::Rw2,
        }
    }
}
