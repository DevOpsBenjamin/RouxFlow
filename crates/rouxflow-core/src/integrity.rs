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

