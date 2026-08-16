// SPDX-License-Identifier: BUSL-1.1
//! Generate an Ed25519 license-issuing keypair.
//! Public key → stdout (hex, 32 bytes). Private seed is written to
//! `config/secrets/license-ed25519.seed` (gitignored) when run from the repo root.
//!
//!   cargo run -p lab-kit-report --example gen_license_keypair

use ed25519_dalek::{SigningKey, VerifyingKey};
use std::io::Write;

fn main() {
    let seed: [u8; 32] = rand_seed();
    let sk = SigningKey::from_bytes(&seed);
    let vk: VerifyingKey = sk.verifying_key();
    println!("public_hex={}", hex(vk.as_bytes()));
    println!("seed_hex={}", hex(&seed));
    let dest = std::path::Path::new("config/secrets/license-ed25519.seed");
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::File::create(dest) {
        let _ = writeln!(f, "{}", hex(&seed));
        eprintln!("wrote {}", dest.display());
    }
    let pub_dest = std::path::Path::new("crates/lab-kit-report/keys/license-ed25519.pub");
    if let Some(parent) = pub_dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::File::create(pub_dest) {
        let _ = writeln!(f, "{}", hex(vk.as_bytes()));
        eprintln!("wrote {}", pub_dest.display());
    }
}

fn rand_seed() -> [u8; 32] {
    let mut seed = [0u8; 32];
    getrandom_fill(&mut seed);
    seed
}

fn getrandom_fill(buf: &mut [u8]) {
    // Avoid extra deps: read from /dev/urandom (Unix).
    let bytes = std::fs::read("/dev/urandom").expect("urandom");
    buf.copy_from_slice(&bytes[..32]);
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
