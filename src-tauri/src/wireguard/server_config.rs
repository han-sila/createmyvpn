/// Render the WireGuard server config (wg0.conf) with iptables NAT rules.
pub fn render_server_config(
    server_private_key: &str,
    client_public_key: &str,
    listen_port: u16,
) -> String {
    format!(
        r#"[Interface]
Address = 10.8.0.1/24
ListenPort = {listen_port}
PrivateKey = {server_private_key}

# NAT masquerading rules
PostUp = iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
PostUp = iptables -A FORWARD -i wg0 -j ACCEPT
PostUp = iptables -A FORWARD -o wg0 -j ACCEPT
PostDown = iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE
PostDown = iptables -D FORWARD -i wg0 -j ACCEPT
PostDown = iptables -D FORWARD -o wg0 -j ACCEPT

[Peer]
PublicKey = {client_public_key}
AllowedIPs = 10.8.0.2/32
"#,
        listen_port = listen_port,
        server_private_key = server_private_key,
        client_public_key = client_public_key,
    )
}
