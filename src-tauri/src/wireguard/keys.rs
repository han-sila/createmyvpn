use base64::Engine;
use boringtun::x25519::{PublicKey, StaticSecret};

/// A WireGuard key pair (private + public) as base64 strings.
pub struct WgKeyPair {
    pub private_key: String,
    pub public_key: String,
}

/// Generate a Curve25519 key pair for WireGuard using x25519-dalek.
pub fn generate_keypair() -> WgKeyPair {
    let secret = StaticSecret::random_from_rng(rand::thread_rng());
    let public = PublicKey::from(&secret);

    let private_b64 = base64::engine::general_purpose::STANDARD.encode(secret.to_bytes());
    let public_b64 = base64::engine::general_purpose::STANDARD.encode(public.to_bytes());

    WgKeyPair {
        private_key: private_b64,
        public_key: public_b64,
    }
}
