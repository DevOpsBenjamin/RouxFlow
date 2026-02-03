use rouxflow_bitboard::BitCube;
use rouxflow_core::cube::facelet::FaceletCube;

pub trait FromFacelet {
    fn from_facelet(facelet: &FaceletCube) -> Self;
}

impl FromFacelet for BitCube {
    fn from_facelet(facelet_cube: &FaceletCube) -> Self {
        let mut boards = [0u64; 6];
        for (i, &color) in facelet_cube.facelets.iter().enumerate() {
            boards[color as usize] |= 1 << i;
        }
        BitCube { boards }
    }
}

pub trait ToFacelet {
    fn to_facelet(&self) -> FaceletCube;
}

impl ToFacelet for BitCube {
    fn to_facelet(&self) -> FaceletCube {
        use rouxflow_core::cube::facelet::Color;
        let mut facelets = vec![Color::White; 54];
        for i in 0..54 {
            for color_idx in 0..6 {
                if (self.boards[color_idx] & (1 << i)) != 0 {
                    facelets[i] = match color_idx {
                        0 => Color::White,
                        1 => Color::Yellow,
                        2 => Color::Green,
                        3 => Color::Blue,
                        4 => Color::Red,
                        5 => Color::Orange,
                        _ => unreachable!(),
                    };
                    break;
                }
            }
        }
        FaceletCube { facelets }
    }
}
