use rouxflow_bitboard::BitCube;

pub type CubeGrid = [[char; 12]; 9];

pub fn get_cube_grid(cube: &BitCube) -> CubeGrid {
    let mut grid = [[' '; 12]; 9];
    let sticker = |idx: usize| match cube.get_color_at(idx) {
        0 => 'W',
        1 => 'Y',
        2 => 'G',
        3 => 'B',
        4 => 'R',
        5 => 'O',
        _ => '?',
    };
    for r in 0..3 {
        for c in 0..3 {
            grid[r][3 + c] = sticker(0 + r * 3 + c);
        }
    }
    for r in 0..3 {
        for c in 0..3 {
            grid[3 + r][0 + c] = sticker(36 + r * 3 + c);
        }
        for c in 0..3 {
            grid[3 + r][3 + c] = sticker(18 + r * 3 + c);
        }
        for c in 0..3 {
            grid[3 + r][6 + c] = sticker(9 + r * 3 + c);
        }
        for c in 0..3 {
            grid[3 + r][9 + c] = sticker(45 + r * 3 + c);
        }
    }
    for r in 0..3 {
        for c in 0..3 {
            grid[6 + r][3 + c] = sticker(27 + r * 3 + c);
        }
    }
    grid
}

pub fn assert_grid(actual_grid: CubeGrid, expected: [[&str; 12]; 9]) {
    for r in 0..9 {
        for c in 0..12 {
            let actual = actual_grid[r][c];
            let exp = expected[r][c].chars().next().unwrap_or(' ');
            if actual != exp {
                for rr in 0..9 {
                    for cc in 0..12 {
                        print!("{} ", actual_grid[rr][cc]);
                    }
                    println!();
                }
                panic!(
                    "Grid mismatch at row {}, col {}! Actual: '{}', Expected: '{}'",
                    r, c, actual, exp
                );
            }
        }
    }
}
