use std::net::SocketAddr;

use crate::error::AppError;

/// Parsed representation of a WireGuard client .conf file.
///
/// Example config:
/// ```ini
/// [Interface]
/// PrivateKey = <base64>
/// Address = 10.0.0.2/32
/// DNS = 1.1.1.1
///
/// [Peer]
/// PublicKey = <base64>
/// Endpoint = 1.2.3.4:51820
/// AllowedIPs = 0.0.0.0/0
/// PersistentKeepalive = 25
/// ```
#[derive(Debug, Clone)]
pub struct ParsedClientConfig {
    pub private_key_b64: String,
    pub vpn_address: String, // e.g. "10.0.0.2"
    pub dns: Option<String>,
    pub server_public_key_b64: String,
    pub endpoint: SocketAddr,
    pub allowed_ips: Vec<String>,
    pub persistent_keepalive: Option<u16>,
}

impl ParsedClientConfig {
    pub fn parse(conf: &str) -> Result<Self, AppError> {
        let mut private_key = None;
        let mut address = None;
        let mut dns = None;
        let mut server_public_key = None;
        let mut endpoint_str = None;
        let mut allowed_ips = Vec::new();
        let mut keepalive = None;

        let mut section = "";

        for line in conf.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') {
                section = line;
                continue;
            }

            let (key, val) = match line.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => continue,
            };

            match section {
                "[Interface]" => match key {
                    "PrivateKey" => private_key = Some(val.to_string()),
                    "Address" => {
                        // Strip CIDR prefix, keep just the IP
                        let ip = val.split('/').next().unwrap_or(val);
                        address = Some(ip.to_string());
                    }
                    "DNS" => dns = Some(val.to_string()),
                    _ => {}
                },
                "[Peer]" => match key {
                    "PublicKey" => server_public_key = Some(val.to_string()),
                    "Endpoint" => endpoint_str = Some(val.to_string()),
                    "AllowedIPs" => {
                        for cidr in val.split(',') {
                            allowed_ips.push(cidr.trim().to_string());
                        }
                    }
                    "PersistentKeepalive" => {
                        keepalive = val.parse::<u16>().ok();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let private_key_b64 = private_key
            .ok_or_else(|| AppError::WireGuard("Config missing [Interface] PrivateKey".into()))?;
        let vpn_address = address
            .ok_or_else(|| AppError::WireGuard("Config missing [Interface] Address".into()))?;
        let server_public_key_b64 = server_public_key
            .ok_or_else(|| AppError::WireGuard("Config missing [Peer] PublicKey".into()))?;
        let endpoint_raw = endpoint_str
            .ok_or_else(|| AppError::WireGuard("Config missing [Peer] Endpoint".into()))?;

        let endpoint: SocketAddr = endpoint_raw.parse().map_err(|_| {
            AppError::WireGuard(format!("Invalid endpoint address: {}", endpoint_raw))
        })?;

        Ok(ParsedClientConfig {
            private_key_b64,
            vpn_address,
            dns,
            server_public_key_b64,
            endpoint,
            allowed_ips,
            persistent_keepalive: keepalive,
        })
    }

    /// Decode a base64 WireGuard key into a 32-byte array.
    pub fn decode_key(b64: &str) -> Result<[u8; 32], AppError> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| AppError::WireGuard(format!("Failed to decode WireGuard key: {}", e)))?;

        bytes.try_into().map_err(|_| {
            AppError::WireGuard("WireGuard key must be exactly 32 bytes".into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CONFIG: &str = r#"[Interface]
PrivateKey = yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=
Address = 10.8.0.2/32
DNS = 1.1.1.1

[Peer]
PublicKey = xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=
Endpoint = 1.2.3.4:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
"#;

    #[test]
    fn parse_valid_config() {
        let parsed = ParsedClientConfig::parse(VALID_CONFIG).unwrap();
        assert_eq!(parsed.private_key_b64, "yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=");
        assert_eq!(parsed.vpn_address, "10.8.0.2");
        assert_eq!(parsed.dns, Some("1.1.1.1".to_string()));
        assert_eq!(parsed.server_public_key_b64, "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=");
        assert_eq!(parsed.endpoint.to_string(), "1.2.3.4:51820");
        assert_eq!(parsed.allowed_ips, vec!["0.0.0.0/0"]);
        assert_eq!(parsed.persistent_keepalive, Some(25));
    }

    #[test]
    fn parse_strips_cidr_from_address() {
        let config = VALID_CONFIG.replace("10.8.0.2/32", "10.0.0.5/24");
        let parsed = ParsedClientConfig::parse(&config).unwrap();
        assert_eq!(parsed.vpn_address, "10.0.0.5");
    }

    #[test]
    fn parse_missing_private_key_errors() {
        let config = r#"[Interface]
Address = 10.8.0.2/32

[Peer]
PublicKey = xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=
Endpoint = 1.2.3.4:51820
AllowedIPs = 0.0.0.0/0
"#;
        let result = ParsedClientConfig::parse(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("PrivateKey"));
    }

    #[test]
    fn parse_missing_endpoint_errors() {
        let config = r#"[Interface]
PrivateKey = yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=
Address = 10.8.0.2/32

[Peer]
PublicKey = xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=
AllowedIPs = 0.0.0.0/0
"#;
        let result = ParsedClientConfig::parse(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Endpoint"));
    }

    #[test]
    fn parse_invalid_endpoint_errors() {
        let config = VALID_CONFIG.replace("1.2.3.4:51820", "not-an-address");
        let result = ParsedClientConfig::parse(&config);
        assert!(result.is_err());
    }

    #[test]
    fn parse_multiple_allowed_ips() {
        let config = VALID_CONFIG.replace("AllowedIPs = 0.0.0.0/0", "AllowedIPs = 10.0.0.0/8, 192.168.1.0/24");
        let parsed = ParsedClientConfig::parse(&config).unwrap();
        assert_eq!(parsed.allowed_ips, vec!["10.0.0.0/8", "192.168.1.0/24"]);
    }

    #[test]
    fn parse_optional_dns_missing() {
        let config = VALID_CONFIG.replace("DNS = 1.1.1.1\n", "");
        let parsed = ParsedClientConfig::parse(&config).unwrap();
        assert_eq!(parsed.dns, None);
    }

    #[test]
    fn parse_ignores_comments_and_blank_lines() {
        let config = format!("# This is a comment\n\n{}", VALID_CONFIG);
        let parsed = ParsedClientConfig::parse(&config).unwrap();
        assert_eq!(parsed.vpn_address, "10.8.0.2");
    }

    #[test]
    fn decode_key_valid_32_bytes() {
        // 32 zero bytes in base64
        let b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let bytes = ParsedClientConfig::decode_key(b64).unwrap();
        assert_eq!(bytes.len(), 32);
        assert!(bytes.iter().all(|&b| b == 0));
    }

    #[test]
    fn decode_key_invalid_base64_errors() {
        let result = ParsedClientConfig::decode_key("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn decode_key_wrong_length_errors() {
        // 16 bytes, not 32
        let b64 = "AAAAAAAAAAAAAAAAAAAAAA==";
        let result = ParsedClientConfig::decode_key(b64);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("32 bytes"));
    }
}
