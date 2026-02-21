/// Render the WireGuard client config for connecting to the VPN server.
pub fn render_client_config(
    client_private_key: &str,
    server_public_key: &str,
    endpoint_ip: &str,
    listen_port: u16,
) -> String {
    format!(
        r#"[Interface]
PrivateKey = {client_private_key}
Address = 10.8.0.2/32
DNS = 1.1.1.1

[Peer]
PublicKey = {server_public_key}
Endpoint = {endpoint_ip}:{listen_port}
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
"#,
        client_private_key = client_private_key,
        server_public_key = server_public_key,
        endpoint_ip = endpoint_ip,
        listen_port = listen_port,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_client_config_contains_interface_section() {
        let config = render_client_config("PRIV_KEY", "PUB_KEY", "1.2.3.4", 51820);
        assert!(config.contains("[Interface]"));
        assert!(config.contains("PrivateKey = PRIV_KEY"));
        assert!(config.contains("Address = 10.8.0.2/32"));
        assert!(config.contains("DNS = 1.1.1.1"));
    }

    #[test]
    fn render_client_config_contains_peer_section() {
        let config = render_client_config("PRIV_KEY", "PUB_KEY", "1.2.3.4", 51820);
        assert!(config.contains("[Peer]"));
        assert!(config.contains("PublicKey = PUB_KEY"));
        assert!(config.contains("Endpoint = 1.2.3.4:51820"));
        assert!(config.contains("AllowedIPs = 0.0.0.0/0"));
        assert!(config.contains("PersistentKeepalive = 25"));
    }

    #[test]
    fn render_client_config_uses_custom_port() {
        let config = render_client_config("KEY", "PUB", "10.0.0.1", 12345);
        assert!(config.contains("Endpoint = 10.0.0.1:12345"));
    }

    #[test]
    fn render_client_config_full_tunnel() {
        let config = render_client_config("K", "P", "1.1.1.1", 51820);
        assert!(config.contains("AllowedIPs = 0.0.0.0/0"), "should route all traffic");
    }
}
