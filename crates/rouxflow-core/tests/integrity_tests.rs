use rouxflow_core::integrity::{sign_solve, verify_solve};
use rouxflow_core::session::Solve;

fn make_solve() -> Solve {
    Solve {
        id: "test-uuid-1234".to_string(),
        time: 12345,
        moves: vec!["R".to_string(), "U".to_string(), "R'".to_string()],
        date: 1707900000000,
        is_valid: true,
        scramble: Some("R U R' U'".to_string()),
        timed_moves: None,
        penalty: None,
        deleted_at: None,
        integrity: None,
    }
}

#[test]
fn sign_produces_consistent_output() {
    let solve = make_solve();
    let sig1 = sign_solve(&solve);
    let sig2 = sign_solve(&solve);
    assert_eq!(sig1, sig2, "Signing same solve twice must produce same result");
    assert_eq!(sig1.len(), 64, "HMAC-SHA256 hex should be 64 chars");
}

#[test]
fn modifying_time_changes_signature() {
    let mut solve = make_solve();
    let original = sign_solve(&solve);
    solve.time = 99999;
    let tampered = sign_solve(&solve);
    assert_ne!(original, tampered);
}

#[test]
fn modifying_moves_changes_signature() {
    let mut solve = make_solve();
    let original = sign_solve(&solve);
    solve.moves = vec!["L".to_string()];
    let tampered = sign_solve(&solve);
    assert_ne!(original, tampered);
}

#[test]
fn modifying_scramble_changes_signature() {
    let mut solve = make_solve();
    let original = sign_solve(&solve);
    solve.scramble = Some("L D L'".to_string());
    let tampered = sign_solve(&solve);
    assert_ne!(original, tampered);
}

#[test]
fn modifying_penalty_changes_signature() {
    let mut solve = make_solve();
    let original = sign_solve(&solve);
    solve.penalty = Some("+2".to_string());
    let tampered = sign_solve(&solve);
    assert_ne!(original, tampered);
}

#[test]
fn verify_returns_true_for_valid() {
    let mut solve = make_solve();
    solve.integrity = Some(sign_solve(&solve));
    assert!(verify_solve(&solve));
}

#[test]
fn verify_returns_false_for_tampered() {
    let mut solve = make_solve();
    solve.integrity = Some(sign_solve(&solve));
    solve.time = 1; // tamper
    assert!(!verify_solve(&solve));
}

#[test]
fn verify_returns_false_without_integrity() {
    let solve = make_solve();
    assert!(!verify_solve(&solve));
}

#[test]
fn dnf_solve_gets_valid_signature() {
    let mut solve = Solve {
        id: "dnf-uuid".to_string(),
        time: 0,
        moves: Vec::new(),
        date: 1707900000000,
        is_valid: true,
        scramble: Some("R U R' U'".to_string()),
        timed_moves: None,
        penalty: Some("DNF".to_string()),
        deleted_at: None,
        integrity: None,
    };
    solve.integrity = Some(sign_solve(&solve));
    assert!(verify_solve(&solve));
}
