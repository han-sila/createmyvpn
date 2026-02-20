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
