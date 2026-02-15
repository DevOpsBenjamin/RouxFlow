use rouxflow_bitboard::{BitCube, FaceMove};

mod common;
use common::{assert_grid, get_cube_grid};

#[test]
fn face_u() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::U);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_u_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::Up);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_u2() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::U2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_d() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::D);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["B", "B", "B", "O", "O", "O", "G", "G", "G", "R", "R", "R"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_d_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::Dp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["G", "G", "G", "R", "R", "R", "B", "B", "B", "O", "O", "O"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_d2() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::D2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["O", "O", "O", "G", "G", "G", "R", "R", "R", "B", "B", "B"],
        ["R", "R", "R", "B", "B", "B", "O", "O", "O", "G", "G", "G"],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_l() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::L);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "B", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "W", "G", "G", "R", "R", "R", "B", "B", "Y"],
        ["O", "O", "O", "W", "G", "G", "R", "R", "R", "B", "B", "Y"],
        ["O", "O", "O", "W", "G", "G", "R", "R", "R", "B", "B", "Y"],
        [" ", " ", " ", "G", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_l_prime() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::Lp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "G", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "Y", "G", "G", "R", "R", "R", "B", "B", "W"],
        ["O", "O", "O", "Y", "G", "G", "R", "R", "R", "B", "B", "W"],
        ["O", "O", "O", "Y", "G", "G", "R", "R", "R", "B", "B", "W"],
        [" ", " ", " ", "B", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn face_l2() {
    let mut cube = BitCube::new_solved();
    cube.apply_face_move(FaceMove::L2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "Y", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "W", " ", " ", " ", " ", " ", " "],
        ["O", "O", "O", "B", "G", "G", "R", "R", "R", "B", "B", "G"],
        ["O", "O", "O", "B", "G", "G", "R", "R", "R", "B", "B", "G"],
        ["O", "O", "O", "B", "G", "G", "R", "R", "R", "B", "B", "G"],
        [" ", " ", " ", "W", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "Y", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

// We will add R, L, D, F, B tests below one by one later...
