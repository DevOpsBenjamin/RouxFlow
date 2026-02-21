use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Standard face moves: U, D, L, R, F, B
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FaceMove {
    U,
    Up,
    U2,
    D,
    Dp,
    D2,
    L,
    Lp,
    L2,
    R,
    Rp,
    R2,
    F,
    Fp,
    F2,
    B,
    Bp,
    B2,
}

/// Slice moves: M, E, S
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SliceMove {
    M,
    Mp,
    M2,
    E,
    Ep,
    E2,
    S,
    Sp,
    S2,
}

/// Wide moves: u, d, l, r, f, b
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WideMove {
    Uw,
    Uwp,
    Uw2,
    Dw,
    Dwp,
    Dw2,
    Lw,
    Lwp,
    Lw2,
    Rw,
    Rwp,
    Rw2,
    Fw,
    Fwp,
    Fw2,
    Bw,
    Bwp,
    Bw2,
}

/// Global cube rotations: x, y, z
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rotation {
    X,
    Xp,
    X2,
    Y,
    Yp,
    Y2,
    Z,
    Zp,
    Z2,
}

/// Unified Move enum that wraps all categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Move {
    Face(FaceMove),
    Slice(SliceMove),
    Wide(WideMove),
    Rotate(Rotation),
}

impl Serialize for Move {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Move {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Move::ALL
            .iter()
            .find(|m| m.as_str() == raw)
            .copied()
            .ok_or_else(|| serde::de::Error::unknown_variant(&raw, &[]))
    }
}

impl Move {
    pub const ALL: [Move; 54] = [
        Move::Face(FaceMove::U),
        Move::Face(FaceMove::Up),
        Move::Face(FaceMove::U2),
        Move::Face(FaceMove::D),
        Move::Face(FaceMove::Dp),
        Move::Face(FaceMove::D2),
        Move::Face(FaceMove::L),
        Move::Face(FaceMove::Lp),
        Move::Face(FaceMove::L2),
        Move::Face(FaceMove::R),
        Move::Face(FaceMove::Rp),
        Move::Face(FaceMove::R2),
        Move::Face(FaceMove::F),
        Move::Face(FaceMove::Fp),
        Move::Face(FaceMove::F2),
        Move::Face(FaceMove::B),
        Move::Face(FaceMove::Bp),
        Move::Face(FaceMove::B2),
        Move::Slice(SliceMove::M),
        Move::Slice(SliceMove::Mp),
        Move::Slice(SliceMove::M2),
        Move::Slice(SliceMove::E),
        Move::Slice(SliceMove::Ep),
        Move::Slice(SliceMove::E2),
        Move::Slice(SliceMove::S),
        Move::Slice(SliceMove::Sp),
        Move::Slice(SliceMove::S2),
        Move::Wide(WideMove::Uw),
        Move::Wide(WideMove::Uwp),
        Move::Wide(WideMove::Uw2),
        Move::Wide(WideMove::Dw),
        Move::Wide(WideMove::Dwp),
        Move::Wide(WideMove::Dw2),
        Move::Wide(WideMove::Lw),
        Move::Wide(WideMove::Lwp),
        Move::Wide(WideMove::Lw2),
        Move::Wide(WideMove::Rw),
        Move::Wide(WideMove::Rwp),
        Move::Wide(WideMove::Rw2),
        Move::Wide(WideMove::Fw),
        Move::Wide(WideMove::Fwp),
        Move::Wide(WideMove::Fw2),
        Move::Wide(WideMove::Bw),
        Move::Wide(WideMove::Bwp),
        Move::Wide(WideMove::Bw2),
        Move::Rotate(Rotation::X),
        Move::Rotate(Rotation::Xp),
        Move::Rotate(Rotation::X2),
        Move::Rotate(Rotation::Y),
        Move::Rotate(Rotation::Yp),
        Move::Rotate(Rotation::Y2),
        Move::Rotate(Rotation::Z),
        Move::Rotate(Rotation::Zp),
        Move::Rotate(Rotation::Z2),
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Move::Face(m) => match m {
                FaceMove::U => "U",
                FaceMove::Up => "U'",
                FaceMove::U2 => "U2",
                FaceMove::D => "D",
                FaceMove::Dp => "D'",
                FaceMove::D2 => "D2",
                FaceMove::L => "L",
                FaceMove::Lp => "L'",
                FaceMove::L2 => "L2",
                FaceMove::R => "R",
                FaceMove::Rp => "R'",
                FaceMove::R2 => "R2",
                FaceMove::F => "F",
                FaceMove::Fp => "F'",
                FaceMove::F2 => "F2",
                FaceMove::B => "B",
                FaceMove::Bp => "B'",
                FaceMove::B2 => "B2",
            },
            Move::Slice(m) => match m {
                SliceMove::M => "M",
                SliceMove::Mp => "M'",
                SliceMove::M2 => "M2",
                SliceMove::E => "E",
                SliceMove::Ep => "E'",
                SliceMove::E2 => "E2",
                SliceMove::S => "S",
                SliceMove::Sp => "S'",
                SliceMove::S2 => "S2",
            },
            Move::Wide(m) => match m {
                WideMove::Uw => "u",
                WideMove::Uwp => "u'",
                WideMove::Uw2 => "u2",
                WideMove::Dw => "d",
                WideMove::Dwp => "d'",
                WideMove::Dw2 => "d2",
                WideMove::Lw => "l",
                WideMove::Lwp => "l'",
                WideMove::Lw2 => "l2",
                WideMove::Rw => "r",
                WideMove::Rwp => "r'",
                WideMove::Rw2 => "r2",
                WideMove::Fw => "f",
                WideMove::Fwp => "f'",
                WideMove::Fw2 => "f2",
                WideMove::Bw => "b",
                WideMove::Bwp => "b'",
                WideMove::Bw2 => "b2",
            },
            Move::Rotate(m) => match m {
                Rotation::X => "x",
                Rotation::Xp => "x'",
                Rotation::X2 => "x2",
                Rotation::Y => "y",
                Rotation::Yp => "y'",
                Rotation::Y2 => "y2",
                Rotation::Z => "z",
                Rotation::Zp => "z'",
                Rotation::Z2 => "z2",
            },
        }
    }

    pub fn face(&self) -> u8 {
        match self {
            Move::Face(m) => match m {
                FaceMove::U | FaceMove::Up | FaceMove::U2 => 0,
                FaceMove::D | FaceMove::Dp | FaceMove::D2 => 1,
                FaceMove::L | FaceMove::Lp | FaceMove::L2 => 2,
                FaceMove::R | FaceMove::Rp | FaceMove::R2 => 3,
                FaceMove::F | FaceMove::Fp | FaceMove::F2 => 4,
                FaceMove::B | FaceMove::Bp | FaceMove::B2 => 5,
            },
            Move::Wide(m) => match m {
                WideMove::Uw | WideMove::Uwp | WideMove::Uw2 => 0,
                WideMove::Dw | WideMove::Dwp | WideMove::Dw2 => 1,
                WideMove::Lw | WideMove::Lwp | WideMove::Lw2 => 2,
                WideMove::Rw | WideMove::Rwp | WideMove::Rw2 => 3,
                WideMove::Fw | WideMove::Fwp | WideMove::Fw2 => 4,
                WideMove::Bw | WideMove::Bwp | WideMove::Bw2 => 5,
            },
            Move::Slice(_) => 6,
            Move::Rotate(_) => 7,
        }
    }

    pub fn inverse(&self) -> Move {
        match self {
            Move::Face(m) => Move::Face(match m {
                FaceMove::U => FaceMove::Up,
                FaceMove::Up => FaceMove::U,
                FaceMove::U2 => FaceMove::U2,
                FaceMove::D => FaceMove::Dp,
                FaceMove::Dp => FaceMove::D,
                FaceMove::D2 => FaceMove::D2,
                FaceMove::L => FaceMove::Lp,
                FaceMove::Lp => FaceMove::L,
                FaceMove::L2 => FaceMove::L2,
                FaceMove::R => FaceMove::Rp,
                FaceMove::Rp => FaceMove::R,
                FaceMove::R2 => FaceMove::R2,
                FaceMove::F => FaceMove::Fp,
                FaceMove::Fp => FaceMove::F,
                FaceMove::F2 => FaceMove::F2,
                FaceMove::B => FaceMove::Bp,
                FaceMove::Bp => FaceMove::B,
                FaceMove::B2 => FaceMove::B2,
            }),
            Move::Slice(m) => Move::Slice(match m {
                SliceMove::M => SliceMove::Mp,
                SliceMove::Mp => SliceMove::M,
                SliceMove::M2 => SliceMove::M2,
                SliceMove::E => SliceMove::Ep,
                SliceMove::Ep => SliceMove::E,
                SliceMove::E2 => SliceMove::E2,
                SliceMove::S => SliceMove::Sp,
                SliceMove::Sp => SliceMove::S,
                SliceMove::S2 => SliceMove::S2,
            }),
            Move::Wide(m) => Move::Wide(match m {
                WideMove::Uw => WideMove::Uwp,
                WideMove::Uwp => WideMove::Uw,
                WideMove::Uw2 => WideMove::Uw2,
                WideMove::Dw => WideMove::Dwp,
                WideMove::Dwp => WideMove::Dw,
                WideMove::Dw2 => WideMove::Dw2,
                WideMove::Lw => WideMove::Lwp,
                WideMove::Lwp => WideMove::Lw,
                WideMove::Lw2 => WideMove::Lw2,
                WideMove::Rw => WideMove::Rwp,
                WideMove::Rwp => WideMove::Rw,
                WideMove::Rw2 => WideMove::Rw2,
                WideMove::Fw => WideMove::Fwp,
                WideMove::Fwp => WideMove::Fw,
                WideMove::Fw2 => WideMove::Fw2,
                WideMove::Bw => WideMove::Bwp,
                WideMove::Bwp => WideMove::Bw,
                WideMove::Bw2 => WideMove::Bw2,
            }),
            Move::Rotate(m) => Move::Rotate(match m {
                Rotation::X => Rotation::Xp,
                Rotation::Xp => Rotation::X,
                Rotation::X2 => Rotation::X2,
                Rotation::Y => Rotation::Yp,
                Rotation::Yp => Rotation::Y,
                Rotation::Y2 => Rotation::Y2,
                Rotation::Z => Rotation::Zp,
                Rotation::Zp => Rotation::Z,
                Rotation::Z2 => Rotation::Z2,
            }),
        }
    }
}
