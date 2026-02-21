/// Userspace WireGuard tunnel engine.
///
/// Uses Cloudflare's `boringtun` for the WireGuard protocol and `tun2` for
/// the virtual network interface. No `wg-quick`, no kernel module.
/// Works on Linux (including WSL2), macOS, and Windows.
///
/// Privilege requirement: creating a TUN device needs CAP_NET_ADMIN on Linux
/// or admin rights on Windows. Set once with:
///     sudo setcap cap_net_admin+ep /path/to/createmyvpn
use std::net::SocketAddr;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519::{PublicKey, StaticSecret};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;

use crate::error::AppError;

use super::config_parser::ParsedClientConfig;

const TUN_NAME: &str = "createmyvpn0";
const MTU: usize = 1420;
/// WireGuard keepalive/handshake timer: how often boringtun's internal timers
/// are serviced.  200 ms is the WireGuard spec recommendation.
const TIMER_INTERVAL_MS: u64 = 200;

// ─── Active tunnel state ────────────────────────────────────────────────────

struct ActiveTunnel {
    /// Dropping (or `send`-ing) this stops the tunnel loop.
    stop_tx: tokio::sync::oneshot::Sender<()>,
    server_ip: String,
    gateway: Option<String>,
}

static TUNNEL: OnceLock<Mutex<Option<ActiveTunnel>>> = OnceLock::new();

fn tunnel_lock() -> &'static Mutex<Option<ActiveTunnel>> {
    TUNNEL.get_or_init(|| Mutex::new(None))
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// On Windows, locate the signed wintun.dll that tun2 requires.
///
/// tun2 will only accept a DLL that is code-signed by "WireGuard LLC".
/// We look for it next to the executable (which is where the installer puts it
/// and where the developer should place it for debug builds).
#[cfg(target_os = "windows")]
fn find_wintun_dll() -> Result<std::ffi::OsString, AppError> {
    let exe_path = std::env::current_exe()
        .map_err(|e| AppError::WireGuard(format!("Cannot determine exe path: {e}")))?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| AppError::WireGuard("Cannot determine exe directory".into()))?;
    let dll_path = exe_dir.join("wintun.dll");
    if dll_path.exists() {
        tracing::info!("Using wintun.dll from: {}", dll_path.display());
        return Ok(dll_path.into_os_string());
    }
    Err(AppError::WireGuard(
        "wintun.dll not found next to the CreateMyVpn executable.\n\
         \n\
         CreateMyVpn bundles the WinTUN driver DLL to create a VPN tunnel.\n\
         The release installer places it automatically, but for development\n\
         or portable builds you must add it manually:\n\
         \n\
         1. Download the WinTUN ZIP from https://www.wintun.net\n\
         2. Open the ZIP and extract:  wintun/bin/amd64/wintun.dll\n\
         3. Place wintun.dll in the same folder as createmyvpn.exe\n\
         4. Restart CreateMyVpn and try connecting again."
            .into(),
    ))
}

/// Connect: create TUN device, start the WireGuard packet loop, set up routes.
pub fn connect(config_str: &str) -> Result<(), AppError> {
    // Disconnect any existing tunnel first
    let _ = disconnect();

    let cfg = ParsedClientConfig::parse(config_str)?;
    tracing::info!(
        "Starting userspace WireGuard tunnel to {} ({})",
        cfg.endpoint,
        cfg.vpn_address
    );

    // Decode keys
    let private_bytes = ParsedClientConfig::decode_key(&cfg.private_key_b64)?;
    let public_bytes = ParsedClientConfig::decode_key(&cfg.server_public_key_b64)?;

    let static_secret = StaticSecret::from(private_bytes);
    let peer_public = PublicKey::from(public_bytes);

    let keepalive = cfg.persistent_keepalive.or(Some(25));

    // Create the WireGuard protocol handler
    let tunn = Tunn::new(static_secret, peer_public, None, keepalive, 0, None);

    // Capture current default gateway before we change routing
    let gateway = get_default_gateway();
    tracing::info!("Current default gateway: {:?}", gateway);

    // On Windows, find the signed wintun.dll before creating the TUN device.
    // tun2 defaults to looking for "wintun.dll" via Windows DLL search path, but if
    // no wintun.dll is found LoadLibraryW returns NULL and GetModuleFileNameW(NULL)
    // returns the exe path — causing the signature check to fail against our
    // unsigned dev binary.  Explicitly providing the path avoids this entirely.
    #[cfg(target_os = "windows")]
    let wintun_dll_path = find_wintun_dll()?;

    // Create TUN device
    let mut tun_config = tun2::Configuration::default();
    tun_config
        .tun_name(TUN_NAME)
        .address(&cfg.vpn_address as &str)
        .netmask("255.255.255.255")
        .mtu(MTU as u16)
        .up();

    // Tell tun2 exactly which wintun.dll to use (must be signed by "WireGuard LLC").
    #[cfg(target_os = "windows")]
    tun_config.platform_config(|pc| {
        pc.wintun_file(&wintun_dll_path);
    });

    let tun = tun2::create(&tun_config).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("Operation not permitted") || msg.contains("Access is denied") {
            let exe = std::env::current_exe()
                .unwrap_or_default()
                .display()
                .to_string();
            AppError::WireGuard(format!(
                "Cannot create TUN device — permission denied.\n\
                 \n\
                 Run this once to grant the capability:\n\
                 \n\
                 sudo setcap cap_net_admin+ep {exe}\n\
                 \n\
                 Then restart CreateMyVpn and click Connect again."
            ))
        } else if msg.contains("No such file") || msg.contains("os error 2") {
            AppError::WireGuard(
                "Cannot create TUN device — /dev/net/tun not found.\n\
                 \n\
                 Load the TUN kernel module:\n\
                 \n\
                 sudo modprobe tun\n\
                 \n\
                 To make it persistent across reboots:\n\
                 echo 'tun' | sudo tee /etc/modules-load.d/tun.conf"
                    .into(),
            )
        } else if msg.contains("not signed")
            || msg.contains("not trusted")
            || msg.contains("Signer")
        {
            AppError::WireGuard(format!(
                "Cannot create TUN device — the wintun.dll present is not accepted.\n\
                 tun2 requires a DLL signed by \"WireGuard LLC\".\n\
                 \n\
                 Replace the wintun.dll next to createmyvpn.exe with the official version:\n\
                 1. Download from https://www.wintun.net\n\
                 2. Extract wintun/bin/amd64/wintun.dll from the ZIP\n\
                 3. Place it next to createmyvpn.exe and restart.\n\
                 \n\
                 Original error: {msg}"
            ))
        } else {
            AppError::WireGuard(format!("Failed to create TUN device: {msg}"))
        }
    })?;

    let endpoint = cfg.endpoint;
    let server_ip = endpoint.ip().to_string();

    // Set up routing: send server traffic via real gateway (not TUN, or we loop)
    setup_routes(&server_ip, &gateway, &cfg.allowed_ips)?;

    // Oneshot channel used to stop the tunnel loop cleanly
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn the tunnel loop in its own OS thread (avoids Send constraints on Tunn)
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tunnel runtime");
        rt.block_on(tunnel_loop(tun, tunn, endpoint, stop_rx));
    });

    *tunnel_lock().lock().unwrap() = Some(ActiveTunnel {
        stop_tx,
        server_ip: server_ip.clone(),
        gateway,
    });

    tracing::info!("WireGuard tunnel active — VPN address: {}", cfg.vpn_address);
    Ok(())
}

/// Disconnect: stop the packet loop and remove routes.
pub fn disconnect() -> Result<(), AppError> {
    let mut guard = tunnel_lock().lock().unwrap();
    if let Some(active) = guard.take() {
        tracing::info!("Stopping WireGuard tunnel...");
        // Dropping the sender (or sending) wakes the select! in tunnel_loop
        let _ = active.stop_tx.send(());
        remove_routes(&active.server_ip, &active.gateway);
        tracing::info!("WireGuard tunnel stopped");
    }
    Ok(())
}

/// Returns true if the tunnel thread is alive.
pub fn is_active() -> bool {
    tunnel_lock().lock().unwrap().is_some()
}

// ─── Packet loop ────────────────────────────────────────────────────────────

async fn tunnel_loop(
    tun: tun2::platform::Device,
    mut tunn: Tunn,
    endpoint: SocketAddr,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let tun = match tun2::AsyncDevice::new(tun) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create async TUN device: {}", e);
            return;
        }
    };

    let (mut tun_writer, mut tun_reader) = match tun.split() {
        Ok(pair) => pair,
        Err(e) => {
            tracing::error!("Failed to split async TUN device: {}", e);
            return;
        }
    };

    let udp = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to bind UDP socket: {}", e);
            return;
        }
    };
    if let Err(e) = udp.connect(endpoint).await {
        tracing::error!("Failed to connect UDP socket to {}: {}", endpoint, e);
        return;
    }

    let mut tun_buf = vec![0u8; 65536];
    let mut udp_buf = vec![0u8; 65536];
    let mut out_buf = vec![0u8; 65536];

    tracing::info!("Tunnel packet loop started, endpoint: {}", endpoint);

    // Fires every TIMER_INTERVAL_MS for WireGuard keepalives
    let mut timer = tokio::time::interval(Duration::from_millis(TIMER_INTERVAL_MS));

    loop {
        tokio::select! {
            biased;

            // ── Stop signal ──────────────────────────────────────────────────
            _ = &mut stop_rx => {
                tracing::info!("Tunnel loop received stop signal");
                break;
            }

            // ── Timer: keepalives ────────────────────────────────────────────
            _ = timer.tick() => {
                // Send WireGuard keepalives / handshake retries
                match tunn.update_timers(&mut out_buf) {
                    TunnResult::WriteToNetwork(pkt) => { let _ = udp.send(pkt).await; }
                    TunnResult::Err(e) => tracing::warn!("WireGuard timer error: {:?}", e),
                    _ => {}
                }
            }

            // ── Outgoing from TUN (local apps) → encrypt → UDP ──────────────
            result = tun_reader.read(&mut tun_buf) => {
                match result {
                    Ok(n) if n > 0 => {
                        match tunn.encapsulate(&tun_buf[..n], &mut out_buf) {
                            TunnResult::WriteToNetwork(pkt) => { let _ = udp.send(pkt).await; }
                            TunnResult::Err(e) => tracing::debug!("Encapsulate error: {:?}", e),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // ── Incoming UDP (server → client) → decrypt → TUN ───────────────
            result = udp.recv(&mut udp_buf) => {
                match result {
                    Ok(n) => {
                        let mut data_slice: &[u8] = &udp_buf[..n];
                        loop {
                            match tunn.decapsulate(None, data_slice, &mut out_buf) {
                                TunnResult::WriteToTunnelV4(payload, _)
                                | TunnResult::WriteToTunnelV6(payload, _) => {
                                    let _ = tun_writer.write_all(payload).await;
                                    data_slice = &[];
                                }
                                TunnResult::WriteToNetwork(pkt) => {
                                    let _ = udp.send(pkt).await;
                                    data_slice = &[];
                                }
                                TunnResult::Done => break,
                                TunnResult::Err(e) => {
                                    tracing::debug!("Decapsulate error: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => tracing::debug!("UDP recv error: {}", e),
                }
            }
        }
    }

    tracing::info!("Tunnel packet loop exited");
}

// ─── Routing ────────────────────────────────────────────────────────────────

fn get_default_gateway() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Method 1: ip route show default
        // Output: "default via 192.168.1.1 dev eth0 ..."
        if let Ok(out) = Command::new("ip").args(["route", "show", "default"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for part in s.split_whitespace().collect::<Vec<_>>().windows(2) {
                if part[0] == "via" {
                    return Some(part[1].to_string());
                }
            }
        }

        // Method 2: ip route get 8.8.8.8 (fallback for WSL2 / non-standard setups)
        // Output: "8.8.8.8 via 172.17.0.1 dev eth0 src ..."
        if let Ok(out) = Command::new("ip").args(["route", "get", "8.8.8.8"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for part in s.split_whitespace().collect::<Vec<_>>().windows(2) {
                if part[0] == "via" {
                    return Some(part[1].to_string());
                }
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    {
        let out = Command::new("route")
            .args(["-n", "get", "default"])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines() {
            let l = line.trim();
            if l.starts_with("gateway:") {
                return Some(l.split(':').nth(1)?.trim().to_string());
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        // Query the lowest-metric default route via PowerShell.
        // Output is just the next-hop IP, e.g. "192.168.1.1"
        let out = Command::new("powershell")
            .args([
                "-NoProfile", "-NonInteractive", "-Command",
                "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' | \
                  Sort-Object RouteMetric | \
                  Select-Object -First 1).NextHop",
            ])
            .output()
            .ok()?;
        let gw = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if gw.is_empty() || gw == "0.0.0.0" {
            None
        } else {
            Some(gw)
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

/// Returns the Windows interface index for the named adapter (e.g. "createmyvpn0").
/// Used to specify which interface `route add` should use.
#[cfg(target_os = "windows")]
fn get_tun_if_index(name: &str) -> Option<u32> {
    let cmd = format!(
        "(Get-NetAdapter -Name '{}' -ErrorAction SilentlyContinue).ifIndex",
        name
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &cmd])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<u32>()
        .ok()
}

fn setup_routes(
    server_ip: &str,
    gateway: &Option<String>,
    allowed_ips: &[String],
) -> Result<(), AppError> {
    #[cfg(target_os = "linux")]
    {
        let full_tunnel = allowed_ips.iter().any(|ip| ip == "0.0.0.0/0");

        // 1. Pin the WireGuard server itself to the real gateway to avoid routing loop.
        //    Without this, when full-tunnel is active, WireGuard handshake packets
        //    themselves would be routed through the TUN → infinite loop → no connection.
        if let Some(gw) = gateway {
            let out = Command::new("ip")
                .args(["route", "add", server_ip, "via", gw])
                .output()
                .map_err(|e| AppError::WireGuard(format!("ip route add (server): {}", e)))?;
            if !out.status.success() {
                let err = String::from_utf8_lossy(&out.stderr);
                // "RTNETLINK answers: File exists" is harmless
                if !err.contains("File exists") {
                    tracing::warn!("ip route add server: {}", err);
                }
            }
        } else if full_tunnel {
            return Err(AppError::WireGuard(
                "Cannot set up full-tunnel VPN routing: the system's default gateway \
                 could not be detected.\n\
                 \n\
                 Without a known gateway, WireGuard handshake packets would loop through \
                 the tunnel and the connection would never establish.\n\
                 \n\
                 Check that a default route exists:\n\
                 \n\
                 ip route show default\n\
                 \n\
                 If missing, add one (replace GW and DEV with your values):\n\
                 \n\
                 sudo ip route add default via GW dev DEV"
                    .into(),
            ));
        }

        // 2. Route all requested traffic via TUN
        for cidr in allowed_ips {
            if cidr == "0.0.0.0/0" {
                // Split into two /1s so we don't override the server route above
                for half in &["0.0.0.0/1", "128.0.0.0/1"] {
                    run_ip_route_add(half)?;
                }
            } else {
                run_ip_route_add(cidr)?;
            }
        }

        tracing::info!("Routes configured");
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(gw) = gateway {
            let _ = Command::new("route")
                .args(["add", &format!("{}/32", server_ip), gw])
                .output();
        }
        for cidr in allowed_ips {
            if cidr == "0.0.0.0/0" {
                let _ = Command::new("route")
                    .args(["add", "-net", "0.0.0.0/1", "-interface", TUN_NAME])
                    .output();
                let _ = Command::new("route")
                    .args(["add", "-net", "128.0.0.0/1", "-interface", TUN_NAME])
                    .output();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let full_tunnel = allowed_ips.iter().any(|ip| ip == "0.0.0.0/0");

        // 1. Pin the WireGuard server's IP to the real gateway BEFORE redirecting
        //    all traffic through the TUN — otherwise the handshake packets loop.
        if let Some(gw) = gateway {
            let out = Command::new("route")
                .args(["add", server_ip, "mask", "255.255.255.255", gw])
                .output();
            if let Ok(o) = out {
                if !o.status.success() {
                    let err = String::from_utf8_lossy(&o.stderr);
                    if !err.contains("already exists") {
                        tracing::warn!("route add server {}: {}", server_ip, err.trim());
                    }
                }
            }
        } else if full_tunnel {
            return Err(AppError::WireGuard(
                "Cannot set up full-tunnel VPN routing on Windows: \
                 default gateway not detected.\n\
                 \n\
                 Check in PowerShell:\n\
                 Get-NetRoute -DestinationPrefix '0.0.0.0/0'"
                    .into(),
            ));
        }

        // 2. Route AllowedIPs traffic via the TUN adapter.
        //    Split 0.0.0.0/0 into two /1s so it has lower precedence than the
        //    server route pinned above (Windows matches most-specific first).
        if full_tunnel {
            if let Some(idx) = get_tun_if_index(TUN_NAME) {
                let idx_s = idx.to_string();
                for (net, mask) in &[("0.0.0.0", "128.0.0.0"), ("128.0.0.0", "128.0.0.0")] {
                    let out = Command::new("route")
                        .args(["add", net, "mask", mask, "0.0.0.0", "metric", "6", "IF", &idx_s])
                        .output();
                    if let Ok(o) = out {
                        if !o.status.success() {
                            let err = String::from_utf8_lossy(&o.stderr);
                            if !err.contains("already exists") {
                                tracing::warn!("route add {}/{}: {}", net, mask, err.trim());
                            }
                        }
                    }
                }
                tracing::info!("Windows routes configured via interface index {}", idx);
            } else {
                tracing::warn!(
                    "Could not resolve interface index for '{}' — \
                     traffic may not route through the VPN",
                    TUN_NAME
                );
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_ip_route_add(cidr: &str) -> Result<(), AppError> {
    let out = Command::new("ip")
        .args(["route", "add", cidr, "dev", TUN_NAME])
        .output()
        .map_err(|e| AppError::WireGuard(format!("ip route add {}: {}", cidr, e)))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        if !err.contains("File exists") {
            tracing::warn!("ip route add {}: {}", cidr, err);
        }
    }
    Ok(())
}

fn remove_routes(server_ip: &str, gateway: &Option<String>) {
    #[cfg(target_os = "linux")]
    {
        for half in &["0.0.0.0/1", "128.0.0.0/1"] {
            let _ = Command::new("ip")
                .args(["route", "del", half, "dev", TUN_NAME])
                .output();
        }
        if let Some(gw) = gateway {
            let _ = Command::new("ip")
                .args(["route", "del", server_ip, "via", gw])
                .output();
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("route")
            .args(["delete", "-net", "0.0.0.0/1"])
            .output();
        let _ = Command::new("route")
            .args(["delete", "-net", "128.0.0.0/1"])
            .output();
        if let Some(gw) = gateway {
            let _ = Command::new("route")
                .args(["delete", &format!("{}/32", server_ip), gw])
                .output();
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Remove TUN traffic routes (the two /1 halves of 0.0.0.0/0)
        let _ = Command::new("route")
            .args(["delete", "0.0.0.0", "mask", "128.0.0.0"])
            .output();
        let _ = Command::new("route")
            .args(["delete", "128.0.0.0", "mask", "128.0.0.0"])
            .output();
        // Remove the server pin route
        let _ = Command::new("route")
            .args(["delete", server_ip, "mask", "255.255.255.255"])
            .output();
        let _ = gateway; // not needed on Windows
    }
}
