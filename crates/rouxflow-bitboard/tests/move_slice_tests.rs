use rouxflow_bitboard::{BitCube, SliceMove};

mod common;
use common::{assert_grid, get_cube_grid};

#[test]
fn slice_m() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::M);
    let grid = get_cube_grid(&cube);

    // M move: rotates the middle slice downwards (follows L direction).
    // Middle columns of U, F, D, B are affected.
    // U mid -> F mid, F mid -> D mid, D mid -> B mid (reversed), B mid -> U mid (reversed).
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "B", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "B", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "B", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "W", "G", "R", "R", "R", "B", "Y", "B"],
        ["O", "O", "O", "G", "W", "G", "R", "R", "R", "B", "Y", "B"],
        ["O", "O", "O", "G", "W", "G", "R", "R", "R", "B", "Y", "B"],
        [" ", " ", " ", "Y", "G", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "G", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "G", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}
