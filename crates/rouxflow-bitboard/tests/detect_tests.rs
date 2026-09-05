use rouxflow_bitboard::BitCube;

#[test]
fn test_fb_block_detection() {
    let mut cube = BitCube::new_solved();
    assert!(cube.is_l_block_formed(), "Solved cube should have L-block");
    assert!(cube.is_fb_block(), "Solved cube should have blocks");

    cube.rot_y();
    assert!(
        cube.is_l_block_formed(),
        "Rotated solved cube should have L-block"
    );
    assert!(cube.is_fb_block(), "Rotated solved cube should have blocks");

    let mut cube = BitCube::new_solved();
    cube.face_r2(); // Using the new explicit method
    assert!(
        cube.is_l_block_formed(),
        "Rotating R2 shouldn't break L-block"
    );
    assert!(cube.is_fb_block());

    cube.face_f();
    assert!(
        !cube.is_l_block_formed(),
        "Rotating F must break the specific L-block"
    );
}

#[test]
fn test_pseudo_block_detection() {
    let mut cube = BitCube::new_solved();
    cube.slice_s();
    assert!(!cube.is_l_block_formed(), "S move shuffles L face");
    // is_fb_block is still WIP for non-monochromatic pseudo-blocks if centers shifted
}
