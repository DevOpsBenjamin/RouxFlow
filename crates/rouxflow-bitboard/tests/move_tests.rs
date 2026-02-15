use rouxflow_bitboard::{BitCube, Move};

#[test]
fn test_all_move_inversions() {
    for &m in &Move::ALL {
        let mut cube = BitCube::new_solved();
        let inv = m.inverse();

        // Apply move and its inverse
        cube.apply_move_enum(m);
        cube.apply_move_enum(inv);

        let solved = BitCube::new_solved();
        assert_eq!(
            cube, solved,
            "Move {:?} followed by its inverse {:?} failed to return to solved state",
            m, inv
        );
    }
}

#[test]
fn test_double_moves_are_self_inverse() {
    for &m in &Move::ALL {
        if m.as_str().contains('2') {
            let mut cube = BitCube::new_solved();
            cube.apply_move_enum(m);
            cube.apply_move_enum(m);

            let solved = BitCube::new_solved();
            assert_eq!(cube, solved, "Double move {:?} is not its own inverse", m);
        }
    }
}

#[test]
fn test_triple_prime_is_move() {
    for &m in &Move::ALL {
        let (_base, is_prime, is_double) = match m.as_str() {
            s if s.ends_with('2') => (s[0..s.len() - 1].to_string(), false, true),
            s if s.ends_with('\'') => (s[0..s.len() - 1].to_string(), true, false),
            s => (s.to_string(), false, false),
        };

        if !is_prime && !is_double {
            let mut cube1 = BitCube::new_solved();
            let mut cube2 = BitCube::new_solved();

            // Apply single move
            cube1.apply_move_enum(m);

            // Apply prime move 3 times
            let prime = m.inverse();
            cube2.apply_move_enum(prime);
            cube2.apply_move_enum(prime);
            cube2.apply_move_enum(prime);

            assert_eq!(
                cube1, cube2,
                "Triple inverse of {:?} should equal {:?}",
                m, m
            );
        }
    }
}
