use rouxflow_bitboard::{BitCube, WideMove};

mod common;
use common::{assert_grid, get_cube_grid};

#[test]
fn wide_rw() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Rw);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "G", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "G", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "G", "G", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "Y", "Y", "R", "R", "R", "W", "W", "B"],
        ["O", "O", "O", "G", "Y", "Y", "R", "R", "R", "W", "W", "B"],
        ["O", "O", "O", "G", "Y", "Y", "R", "R", "R", "W", "W", "B"],
        [" ", " ", " ", "Y", "B", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "B", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_rw_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Rwp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "B", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "B", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "B", "B", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "W", "W", "R", "R", "R", "Y", "Y", "B"],
        ["O", "O", "O", "G", "W", "W", "R", "R", "R", "Y", "Y", "B"],
        ["O", "O", "O", "G", "W", "W", "R", "R", "R", "Y", "Y", "B"],
        [" ", " ", " ", "Y", "G", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "G", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "G", "G", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_rw2() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Rw2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "Y", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "B", "B", "R", "R", "R", "G", "G", "B"],
        ["O", "O", "O", "G", "B", "B", "R", "R", "R", "G", "G", "B"],
        ["O", "O", "O", "G", "B", "B", "R", "R", "R", "G", "G", "B"],
        [" ", " ", " ", "Y", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "W", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_lw() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Lw);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "B", "B", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "B", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "B", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "W", "W", "G", "R", "R", "R", "B", "Y", "Y"],
        ["O", "O", "O", "W", "W", "G", "R", "R", "R", "B", "Y", "Y"],
        ["O", "O", "O", "W", "W", "G", "R", "R", "R", "B", "Y", "Y"],
        [" ", " ", " ", "G", "G", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "G", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "G", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_lw_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Lwp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "G", "G", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "G", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "G", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "Y", "Y", "G", "R", "R", "R", "B", "W", "W"],
        ["O", "O", "O", "Y", "Y", "G", "R", "R", "R", "B", "W", "W"],
        ["O", "O", "O", "Y", "Y", "G", "R", "R", "R", "B", "W", "W"],
        [" ", " ", " ", "B", "B", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "B", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "B", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_lw2() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Lw2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "Y", "Y", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "B", "B", "G", "R", "R", "R", "B", "G", "G"],
        ["O", "O", "O", "B", "B", "G", "R", "R", "R", "B", "G", "G"],
        ["O", "O", "O", "B", "B", "G", "R", "R", "R", "B", "G", "G"],
        [" ", " ", " ", "W", "W", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_uw() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Uw);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_uw_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Uwp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_uw2() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Uw2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_dw() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Dw);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_dw_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Dwp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_dw2() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Dw2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_fw() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Fw);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        ["O", "Y", "Y", "G", "G", "G", "W", "W", "R", "B", "B", "B"],
        ["O", "Y", "Y", "G", "G", "G", "W", "W", "R", "B", "B", "B"],
        ["O", "Y", "Y", "G", "G", "G", "W", "W", "R", "B", "B", "B"],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_fw_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Fwp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        ["O", "W", "W", "G", "G", "G", "Y", "Y", "R", "B", "B", "B"],
        ["O", "W", "W", "G", "G", "G", "Y", "Y", "R", "B", "B", "B"],
        ["O", "W", "W", "G", "G", "G", "Y", "Y", "R", "B", "B", "B"],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_fw2() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Fw2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        ["O", "R", "R", "G", "G", "G", "O", "O", "R", "B", "B", "B"],
        ["O", "R", "R", "G", "G", "G", "O", "O", "R", "B", "B", "B"],
        ["O", "R", "R", "G", "G", "G", "O", "O", "R", "B", "B", "B"],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_bw() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Bw);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["W", "W", "O", "G", "G", "G", "R", "Y", "Y", "B", "B", "B"],
        ["W", "W", "O", "G", "G", "G", "R", "Y", "Y", "B", "B", "B"],
        ["W", "W", "O", "G", "G", "G", "R", "Y", "Y", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_bw_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Bwp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "O", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["Y", "Y", "O", "G", "G", "G", "R", "W", "W", "B", "B", "B"],
        ["Y", "Y", "O", "G", "G", "G", "R", "W", "W", "B", "B", "B"],
        ["Y", "Y", "O", "G", "G", "G", "R", "W", "W", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "R", "R", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn wide_bw2() {
    let mut cube = BitCube::new_solved();
    cube.apply_wide_move(WideMove::Bw2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["R", "R", "O", "G", "G", "G", "R", "O", "O", "B", "B", "B"],
        ["R", "R", "O", "G", "G", "G", "R", "O", "O", "B", "B", "B"],
        ["R", "R", "O", "G", "G", "G", "R", "O", "O", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}
