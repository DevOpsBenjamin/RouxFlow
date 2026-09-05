use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

fn main() {
    let secret_hex = get_secret();
    assert_eq!(secret_hex.len(), 64, "HMAC_SECRET must be 64 hex chars (32 bytes)");
    let secret = decode_hex(&secret_hex);

    // XOR-split the key so it's not plaintext in the compiled binary.
    // Random pad changes every build.
    let pad = random_bytes(32);
    let masked: Vec<u8> = secret.iter().zip(pad.iter()).map(|(s, p)| s ^ p).collect();

    println!("cargo:rustc-env=HMAC_PAD={}", encode_hex(&pad));
    println!("cargo:rustc-env=HMAC_MASKED={}", encode_hex(&masked));
}

fn get_secret() -> String {
    // 1. Try environment variable directly (CI / GitHub Actions)
    if let Ok(secret) = std::env::var("HMAC_SECRET") {
        if !secret.is_empty() {
            return secret;
        }
    }

    // 2. Fallback: read from root .env file (local dev)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let env_path = std::path::Path::new(&manifest_dir).join("../../.env");
    println!("cargo:rerun-if-changed={}", env_path.display());

    if let Ok(content) = std::fs::read_to_string(&env_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some(value) = line.strip_prefix("HMAC_SECRET=") {
                let value = value.trim();
                if !value.is_empty() {
                    return value.to_string();
                }
            }
        }
    }

    panic!(
        "HMAC_SECRET not set! Either:\n  \
         - Set it in .env at repo root\n  \
         - Export HMAC_SECRET as an environment variable\n  \
         - In GitHub Actions, use a repository secret"
    );
}

fn decode_hex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("invalid hex"))
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn random_bytes(n: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(n);
    let state = RandomState::new();
    let mut counter = 0u64;
    while bytes.len() < n {
        let mut hasher = state.build_hasher();
        hasher.write_u64(counter);
        counter += 1;
        bytes.extend_from_slice(&hasher.finish().to_le_bytes());
    }
    bytes.truncate(n);
    bytes
}
