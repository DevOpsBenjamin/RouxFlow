use rouxflow_bitboard::{BitCube, SliceMove};

mod common;
use common::{assert_grid, get_cube_grid};

#[test]
fn slice_m() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::M);
    let grid = get_cube_grid(&cube);
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

#[test]
fn slice_m_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::Mp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "G", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "G", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "G", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "Y", "G", "R", "R", "R", "B", "W", "B"],
        ["O", "O", "O", "G", "Y", "G", "R", "R", "R", "B", "W", "B"],
        ["O", "O", "O", "G", "Y", "G", "R", "R", "R", "B", "W", "B"],
        [" ", " ", " ", "Y", "B", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_m2() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::M2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "Y", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "B", "G", "R", "R", "R", "B", "G", "B"],
        ["O", "O", "O", "G", "B", "G", "R", "R", "R", "B", "G", "B"],
        ["O", "O", "O", "G", "B", "G", "R", "R", "R", "B", "G", "B"],
        [" ", " ", " ", "Y", "W", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_e() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::E);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_e_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::Ep);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_e2() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::E2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_s() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::S);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "Y", "O", "G", "G", "G", "R", "W", "R", "B", "B", "B"],
        ["O", "Y", "O", "G", "G", "G", "R", "W", "R", "B", "B", "B"],
        ["O", "Y", "O", "G", "G", "G", "R", "W", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_s_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::Sp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "W", "O", "G", "G", "G", "R", "Y", "R", "B", "B", "B"],
        ["O", "W", "O", "G", "G", "G", "R", "Y", "R", "B", "B", "B"],
        ["O", "W", "O", "G", "G", "G", "R", "Y", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn slice_s2() {
    let mut cube = BitCube::new_solved();
    cube.apply_slice_move(SliceMove::S2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "R", "O", "G", "G", "G", "R", "O", "R", "B", "B", "B"],
        ["O", "R", "O", "G", "G", "G", "R", "O", "R", "B", "B", "B"],
        ["O", "R", "O", "G", "G", "G", "R", "O", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}
