//! HMAC-SHA256 request signing for Supabase leaderboard protection.
//!
//! The secret key is compiled into the WASM binary.
//! Each POST to the leaderboard includes an `X-Signature` header.
//!
//! NOTE: WASM can be decompiled (wasm2wat), so this is a barrier,
//! not a guarantee. Server-side Edge Function verification is the
//! real line of defence.

// TODO: Implement HMAC signing when leaderboard feature is built.
// Dependencies needed: hmac, sha2
//
// pub fn sign_request(body: &[u8]) -> String {
//     use hmac::{Hmac, Mac};
//     use sha2::Sha256;
//     type HmacSha256 = Hmac<Sha256>;
//     let secret = include_bytes!("../signing_key.bin"); // or const
//     let mut mac = HmacSha256::new_from_slice(secret).unwrap();
//     mac.update(body);
//     hex::encode(mac.finalize().into_bytes())
// }
