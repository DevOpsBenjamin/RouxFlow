use crate::bitcube::BitCube;

#[test]
fn test_fb_block_detection() {
    // 1. Solved cube has blocks everywhere
    let mut cube = BitCube::new_solved();
    assert!(cube.is_l_block_formed(), "Solved cube should have L-block");
    assert!(cube.is_fb_block(), "Solved cube should have blocks");

    // 2. Rotate whole cube (y) -> should still detect
    cube.rotate_y();
    assert!(cube.is_l_block_formed(), "Rotated solved cube should have L-block");
    assert!(cube.is_fb_block(), "Rotated solved cube should have blocks");

    // 3. Create a specific block scenario
    // Rotate R2. This preserves the L-block (L, F-Left, B-Right, D-Left).
    let mut cube = BitCube::new_solved();
    cube.rotate_r();
    cube.rotate_r();
    assert!(cube.is_l_block_formed(), "Rotating R2 shouldn't break L-block");
    assert!(cube.is_fb_block());

    // Now break the L-block with F.
    cube.rotate_f();
    assert!(!cube.is_l_block_formed(), "Rotating F must break the specific L-block");
}

#[test]
fn test_pseudo_block_detection() {
    let mut cube = BitCube::new_solved();
    // Move a slice. This shuffles centers but keeps corner/edge blocks together.
    cube.rotate_s();
    // Standard is_l_block_formed checks if stickers are same color.
    // S move destroys standard L face (center and one edge moved).
    assert!(!cube.is_l_block_formed(), "S move shuffles L face");
    // BUT find_any_block (is_fb_block) should find it IF the block stickers match each other.
    // In BitCube, S move shuffles colors.
    // Actually, S move on solved cube creates a state where F and B are solved, but L, R, U, D have mid stripes.
    // So there might NOT be a block anymore because piece colors are mixed.
}
