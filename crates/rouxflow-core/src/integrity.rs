use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::session::Solve;

type HmacSha256 = Hmac<Sha256>;

// Key injected at compile time via build.rs, XOR-split so it's not
// plaintext in the binary. build.rs reads HMAC_SECRET from .env (local)
// or env var (CI/GitHub Actions). Same key must be set in Supabase Edge Function.
fn get_key() -> [u8; 32] {
    let pad = std::hint::black_box(env!("HMAC_PAD").as_bytes());
    let masked = std::hint::black_box(env!("HMAC_MASKED").as_bytes());
    let mut key = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        let p = decode_nibble(pad[i * 2]) * 16 + decode_nibble(pad[i * 2 + 1]);
        let m = decode_nibble(masked[i * 2]) * 16 + decode_nibble(masked[i * 2 + 1]);
        key[i] = p ^ m;
        i += 1;
    }
    key
}

const fn decode_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        _ => panic!("invalid hex"),
    }
}

/// Build the canonical message string for HMAC signing.
/// Format: "{id}|{time}|{date}|{scramble}|{moves_comma_separated}|{penalty}"
/// Deterministic: no floats, no optional complex fields.
fn canonical_message(solve: &Solve) -> String {
    let scramble = solve.scramble.as_deref().unwrap_or("");
    let moves = solve.moves.join(",");
    let penalty = solve.penalty.as_deref().unwrap_or("");
    format!(
        "{}|{}|{}|{}|{}|{}",
        solve.id, solve.time, solve.date, scramble, moves, penalty
    )
}

/// Compute HMAC-SHA256 over the solve's critical fields. Returns hex-encoded signature.
pub fn sign_solve(solve: &Solve) -> String {
    let key = get_key();
    let msg = canonical_message(solve);
    let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC key length is always 32");
    mac.update(msg.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Verify a solve's integrity field matches the recomputed HMAC.
pub fn verify_solve(solve: &Solve) -> bool {
    match &solve.integrity {
        Some(sig) => sig == &sign_solve(solve),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Solve;

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
    fn sign_is_deterministic() {
        let solve = make_solve();
        let sig1 = sign_solve(&solve);
        let sig2 = sign_solve(&solve);
        assert_eq!(sig1, sig2);
        assert_eq!(sig1.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }

    #[test]
    fn verify_valid_signature() {
        let mut solve = make_solve();
        solve.integrity = Some(sign_solve(&solve));
        assert!(verify_solve(&solve));
    }

    #[test]
    fn verify_fails_without_integrity() {
        let solve = make_solve();
        assert!(!verify_solve(&solve));
    }

    #[test]
    fn tamper_time_invalidates() {
        let mut solve = make_solve();
        solve.integrity = Some(sign_solve(&solve));
        solve.time = 1; // tamper
        assert!(!verify_solve(&solve));
    }

    #[test]
    fn tamper_moves_invalidates() {
        let mut solve = make_solve();
        solve.integrity = Some(sign_solve(&solve));
        solve.moves = vec!["L".to_string()]; // tamper
        assert!(!verify_solve(&solve));
    }

    #[test]
    fn tamper_scramble_invalidates() {
        let mut solve = make_solve();
        solve.integrity = Some(sign_solve(&solve));
        solve.scramble = Some("L D".to_string()); // tamper
        assert!(!verify_solve(&solve));
    }

    #[test]
    fn tamper_penalty_invalidates() {
        let mut solve = make_solve();
        solve.integrity = Some(sign_solve(&solve));
        solve.penalty = Some("+2".to_string()); // tamper
        assert!(!verify_solve(&solve));
    }

    #[test]
    fn dnf_solve_signs_correctly() {
        let mut solve = make_solve();
        solve.time = 0;
        solve.moves = Vec::new();
        solve.penalty = Some("DNF".to_string());
        solve.integrity = Some(sign_solve(&solve));
        assert!(verify_solve(&solve));
    }

    #[test]
    fn different_solves_have_different_signatures() {
        let mut solve1 = make_solve();
        let mut solve2 = make_solve();
        solve2.id = "different-uuid".to_string();
        assert_ne!(sign_solve(&solve1), sign_solve(&solve2));

        solve1.integrity = Some(sign_solve(&solve1));
        solve2.integrity = Some(sign_solve(&solve2));
        assert!(verify_solve(&solve1));
        assert!(verify_solve(&solve2));
    }
}
