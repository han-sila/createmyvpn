pub mod aws {
    pub mod ami;
    pub mod client;
    pub mod ec2;
    pub mod security_group;
    pub mod teardown;
    pub mod vpc;
}

pub mod do_cloud {
    pub mod client;
    pub mod droplet;
    pub mod firewall;
    pub mod key;
}

pub mod commands {
    pub mod byo;
    pub mod connect;
    pub mod credentials;
    pub mod credentials_do;
    pub mod deploy;
    pub mod deploy_do;
    pub mod destroy;
    pub mod logs;
    pub mod settings;
    pub mod timer;
}

pub mod persistence {
    pub mod store;
}

pub mod ssh {
    pub mod client;
    pub mod configure;
}

pub mod wireguard {
    pub mod client_config;
    pub mod config_parser;
    pub mod keys;
    pub mod server_config;
    pub mod userspace;
    pub mod tunnel;
}

pub mod error;
pub mod state;

pub fn run() {
    // ── Logging: stderr + file ─────────────────────────────────────────────
    let log_dir = dirs::home_dir()
        .expect("Cannot find home directory")
        .join(".createmyvpn")
        .join("logs");
    std::fs::create_dir_all(&log_dir).expect("Cannot create log directory");

    let log_path = log_dir.join("createmyvpn.log");

    // ── Fresh-session cleanup ─────────────────────────────────────────────
    // Truncate the log file so each run starts with a clean slate.
    let _ = std::fs::write(&log_path, "");
    // Delete stored AWS credentials — they are entered fresh each session.
    // Exception: if an auto-destroy timer is pending, keep credentials so the
    // timer can call destroy_vpn (which needs them for AWS teardown).
    let has_pending_timer = persistence::store::load_state()
        .ok()
        .and_then(|s| s.auto_destroy_at)
        .map(|t| t > chrono::Utc::now())
        .unwrap_or(false);
    if !has_pending_timer {
        let _ = persistence::store::delete_credentials();
        let _ = persistence::store::delete_do_credentials();
    }

    // Write a session separator
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(f, "\n========================================");
            let _ = writeln!(f, "CreateMyVpn session started: {}", chrono::Utc::now());
            let _ = writeln!(f, "========================================");
        }
    }

    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

    let registry = tracing_subscriber::registry().with(
        fmt::layer().with_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "createmyvpn_lib=info".into()),
        ),
    );

    // File logging is best-effort — if the log file can't be opened (e.g. locked
    // by a previous crashed process), the app still starts with stderr-only logging.
    if let Ok(log_file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        registry
            .with(
                fmt::layer()
                    .with_writer(std::sync::Mutex::new(log_file))
                    .with_ansi(false)
                    .with_filter(EnvFilter::new("createmyvpn_lib=debug")),
            )
            .init();
    } else {
        registry.init();
    }

    // ── Startup state recovery ────────────────────────────────────────────
    // If the app was closed mid-deploy or mid-destroy, the persisted status
    // will be stuck at "Deploying" or "Destroying". Correct those on startup
    // so the UI can offer a sensible recovery path instead of spinning forever.
    if let Ok(mut st) = persistence::store::load_state() {
        match st.status {
            state::DeploymentStatus::Deploying => {
                tracing::warn!("Startup: found stuck 'Deploying' state — resetting to Failed");
                st.status = state::DeploymentStatus::Failed;
                st.error_message = Some(
                    "The deployment was interrupted (app was closed mid-deploy). \
                     You can retry from the Deploy page."
                        .into(),
                );
                let _ = persistence::store::save_state(&st);
            }
            state::DeploymentStatus::Destroying => {
                tracing::warn!(
                    "Startup: found stuck 'Destroying' state — \
                     resetting to Deployed so the user can retry"
                );
                st.status = state::DeploymentStatus::Deployed;
                // Clear the auto-destroy timer — the resources were already being
                // deleted, so we don't want to re-spawn a timer that fires and
                // tries to destroy partially-removed infrastructure.
                st.auto_destroy_at = None;
                st.error_message = Some(
                    "A previous destroy attempt was interrupted. \
                     Your server may still be running — please destroy it again."
                        .into(),
                );
                let _ = persistence::store::save_state(&st);
            }
            state::DeploymentStatus::Deployed => {
                // Re-spawn auto-destroy timer if it was set and is still in the future.
                if let Some(at) = st.auto_destroy_at {
                    if at > chrono::Utc::now() {
                        tracing::info!("Startup: re-spawning auto-destroy timer (fires at {})", at);
                        // Timer is spawned after Tauri builder runs; store at for use below.
                        // We use a flag here since AppHandle isn't available yet.
                    } else {
                        // Timer expired while app was closed — mark for immediate destroy.
                        tracing::warn!("Startup: auto-destroy timer expired while app was closed");
                        st.error_message = Some(
                            "Auto-destroy timer expired while the app was closed. \
                             Use the Destroy button to clean up manually."
                                .into(),
                        );
                        st.auto_destroy_at = None;
                        let _ = persistence::store::save_state(&st);
                    }
                }
            }
            _ => {}
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Re-spawn auto-destroy timer if it survived an app restart.
            if let Ok(st) = persistence::store::load_state() {
                if st.status == state::DeploymentStatus::Deployed {
                    if let Some(at) = st.auto_destroy_at {
                        if at > chrono::Utc::now() {
                            commands::timer::spawn_auto_destroy_timer(app.handle().clone(), at);
                        }
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::credentials::validate_credentials,
            commands::credentials::save_credentials,
            commands::credentials::load_credentials,
            commands::credentials::delete_credentials,
            commands::credentials_do::validate_do_credentials,
            commands::credentials_do::save_do_credentials,
            commands::credentials_do::load_do_credentials,
            commands::credentials_do::delete_do_credentials,
            commands::deploy::deploy_vpn,
            commands::deploy::get_deployment_state,
            commands::deploy::reset_deployment_state,
            commands::deploy_do::deploy_do,
            commands::destroy::destroy_vpn,
            commands::byo::deploy_byo_vps,
            commands::connect::connect_vpn,
            commands::connect::disconnect_vpn,
            commands::connect::get_vpn_status,
            commands::connect::get_client_config,
            commands::settings::get_regions,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::logs::get_logs,
            commands::logs::export_logs,
            commands::logs::clear_logs,
            commands::settings::export_client_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running createmyvpn");
}
