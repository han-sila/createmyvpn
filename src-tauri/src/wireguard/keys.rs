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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn generate_keypair_produces_valid_base64() {
        let kp = generate_keypair();
        let priv_bytes = base64::engine::general_purpose::STANDARD
            .decode(&kp.private_key)
            .expect("private key should be valid base64");
        let pub_bytes = base64::engine::general_purpose::STANDARD
            .decode(&kp.public_key)
            .expect("public key should be valid base64");
        assert_eq!(priv_bytes.len(), 32, "private key must be 32 bytes");
        assert_eq!(pub_bytes.len(), 32, "public key must be 32 bytes");
    }

    #[test]
    fn generate_keypair_produces_unique_keys() {
        let kp1 = generate_keypair();
        let kp2 = generate_keypair();
        assert_ne!(kp1.private_key, kp2.private_key);
        assert_ne!(kp1.public_key, kp2.public_key);
    }

    #[test]
    fn private_and_public_keys_are_different() {
        let kp = generate_keypair();
        assert_ne!(kp.private_key, kp.public_key);
    }
}
