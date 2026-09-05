use rouxflow_bitboard::{BitCube, FaceMove, Rotation};

mod common;
use common::{assert_grid, get_cube_grid};

fn get_scrambled_cube() -> BitCube {
    let mut cube = BitCube::new_solved();
    let scramble = vec![
        FaceMove::Bp,
        FaceMove::F2,
        FaceMove::U,
        FaceMove::L2,
        FaceMove::R2,
        FaceMove::Bp,
        FaceMove::Dp,
        FaceMove::B2,
        FaceMove::Fp,
        FaceMove::D2,
        FaceMove::R2,
        FaceMove::Bp,
        FaceMove::U2,
        FaceMove::Dp,
        FaceMove::L2,
        FaceMove::Up,
        FaceMove::Bp,
        FaceMove::Dp,
        FaceMove::B2,
        FaceMove::L,
    ];
    for m in scramble {
        cube.apply_face_move(m);
    }
    cube
}

#[test]
fn rot_x() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::X);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "G", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "G", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "O", " ", " ", " ", " ", " ", " "],
        ["G", "B", "R", "G", "O", "B", "Y", "R", "Y", "G", "R", "O"],
        ["Y", "O", "Y", "B", "Y", "Y", "G", "R", "O", "W", "W", "O"],
        ["W", "R", "R", "G", "W", "B", "W", "O", "B", "Y", "W", "R"],
        [" ", " ", " ", "W", "R", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "G", "R", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_x_prime() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Xp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "W", "R", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "G", "R", " ", " ", " ", " ", " ", " "],
        ["R", "R", "W", "R", "W", "Y", "B", "O", "W", "B", "W", "G"],
        ["Y", "O", "Y", "O", "W", "W", "O", "R", "G", "Y", "Y", "B"],
        ["R", "B", "G", "O", "R", "G", "Y", "R", "Y", "B", "O", "G"],
        [" ", " ", " ", "W", "G", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "G", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "B", "O", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_x2() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::X2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "G", "O", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "Y", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "W", "B", " ", " ", " ", " ", " ", " "],
        ["R", "Y", "R", "W", "R", "O", "W", "G", "Y", "O", "B", "Y"],
        ["B", "O", "R", "Y", "B", "G", "O", "R", "R", "B", "G", "W"],
        ["G", "Y", "W", "B", "G", "R", "B", "O", "Y", "O", "G", "W"],
        [" ", " ", " ", "R", "W", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "R", "G", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_y() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Y);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "O", "O", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "W", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "W", "Y", " ", " ", " ", " ", " ", " "],
        ["W", "G", "O", "Y", "O", "B", "R", "G", "B", "W", "Y", "G"],
        ["W", "G", "B", "R", "R", "O", "G", "B", "Y", "R", "O", "B"],
        ["Y", "B", "O", "Y", "G", "W", "O", "R", "W", "R", "Y", "R"],
        [" ", " ", " ", "B", "Y", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "Y", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "B", "G", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_y_prime() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Yp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "Y", "W", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "O", "O", " ", " ", " ", " ", " ", " "],
        ["R", "G", "B", "W", "Y", "G", "W", "G", "O", "Y", "O", "B"],
        ["G", "B", "Y", "R", "O", "B", "W", "G", "B", "R", "R", "O"],
        ["O", "R", "W", "R", "Y", "R", "Y", "B", "O", "Y", "G", "W"],
        [" ", " ", " ", "G", "B", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "Y", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "Y", "B", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_y2() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Y2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "G", "R", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "R", " ", " ", " ", " ", " ", " "],
        ["Y", "O", "B", "R", "G", "B", "W", "Y", "G", "W", "G", "O"],
        ["R", "R", "O", "G", "B", "Y", "R", "O", "B", "W", "G", "B"],
        ["Y", "G", "W", "O", "R", "W", "R", "Y", "R", "Y", "B", "O"],
        [" ", " ", " ", "B", "W", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "O", "G", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_z() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Z);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "R", "R", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "O", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "R", "B", "G", " ", " ", " ", " ", " ", " "],
        ["G", "B", "G", "Y", "W", "W", "O", "O", "R", "B", "Y", "W"],
        ["W", "Y", "O", "B", "G", "G", "R", "W", "W", "G", "B", "R"],
        ["B", "Y", "B", "O", "B", "O", "G", "W", "Y", "R", "G", "O"],
        [" ", " ", " ", "Y", "R", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "G", "R", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "O", "B", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_z_prime() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Zp);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "B", "O", "W", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "O", "R", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "R", "Y", " ", " ", " ", " ", " ", " "],
        ["Y", "W", "G", "O", "B", "O", "B", "Y", "B", "O", "G", "R"],
        ["W", "W", "R", "G", "G", "B", "O", "Y", "W", "R", "B", "G"],
        ["R", "O", "O", "W", "W", "Y", "G", "B", "G", "W", "Y", "B"],
        [" ", " ", " ", "G", "B", "R", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "O", "Y", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "R", "R", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}

#[test]
fn rot_z2() {
    let mut cube = get_scrambled_cube();
    cube.apply_rotation(Rotation::Z2);
    let grid = get_cube_grid(&cube);
    #[rustfmt::skip]
    let expected = [
        [" ", " ", " ", "B", "W", "G", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "Y", "B", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "B", "O", "G", " ", " ", " ", " ", " ", " "],
        ["W", "G", "Y", "O", "B", "Y", "R", "Y", "R", "W", "R", "O"],
        ["O", "R", "R", "B", "G", "W", "B", "O", "R", "Y", "B", "G"],
        ["B", "O", "Y", "O", "G", "W", "G", "Y", "W", "B", "G", "R"],
        [" ", " ", " ", "G", "R", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "W", "W", "O", " ", " ", " ", " ", " ", " "],
        [" ", " ", " ", "Y", "W", "R", " ", " ", " ", " ", " ", " "],
    ];
    assert_grid(grid, expected);
}
