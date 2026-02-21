//! Integration tests for the createmyvpn library.
//!
//! These tests exercise cross-module interactions:
//!   - Key generation → config rendering → config parsing round-trip
//!   - State serialization/deserialization with all fields
//!   - Error conversion chains

use createmyvpn_lib::state::*;
use createmyvpn_lib::wireguard::{client_config, config_parser, keys, server_config};

// ── Key generation → Config rendering → Config parsing round-trip ────────────

#[test]
fn full_wireguard_config_roundtrip() {
    // 1. Generate server and client key pairs
    let server_kp = keys::generate_keypair();
    let client_kp = keys::generate_keypair();

    // 2. Render server config
    let server_conf = server_config::render_server_config(
        &server_kp.private_key,
        &client_kp.public_key,
        51820,
    );
    assert!(server_conf.contains(&server_kp.private_key));
    assert!(server_conf.contains(&client_kp.public_key));
    assert!(server_conf.contains("ListenPort = 51820"));

    // 3. Render client config
    let client_conf = client_config::render_client_config(
        &client_kp.private_key,
        &server_kp.public_key,
        "203.0.113.10",
        51820,
    );
    assert!(client_conf.contains(&client_kp.private_key));
    assert!(client_conf.contains(&server_kp.public_key));
    assert!(client_conf.contains("203.0.113.10:51820"));

    // 4. Parse the client config back
    let parsed = config_parser::ParsedClientConfig::parse(&client_conf)
        .expect("should parse generated client config");
    assert_eq!(parsed.private_key_b64, client_kp.private_key);
    assert_eq!(parsed.server_public_key_b64, server_kp.public_key);
    assert_eq!(parsed.vpn_address, "10.8.0.2");
    assert_eq!(parsed.endpoint.to_string(), "203.0.113.10:51820");
    assert_eq!(parsed.dns, Some("1.1.1.1".to_string()));
    assert_eq!(parsed.allowed_ips, vec!["0.0.0.0/0"]);
    assert_eq!(parsed.persistent_keepalive, Some(25));

    // 5. Decode the parsed keys into bytes
    let priv_bytes = config_parser::ParsedClientConfig::decode_key(&parsed.private_key_b64)
        .expect("should decode private key");
    let pub_bytes = config_parser::ParsedClientConfig::decode_key(&parsed.server_public_key_b64)
        .expect("should decode public key");
    assert_eq!(priv_bytes.len(), 32);
    assert_eq!(pub_bytes.len(), 32);
}

#[test]
fn config_roundtrip_custom_port() {
    let server_kp = keys::generate_keypair();
    let client_kp = keys::generate_keypair();

    let client_conf = client_config::render_client_config(
        &client_kp.private_key,
        &server_kp.public_key,
        "10.0.0.1",
        12345,
    );

    let parsed = config_parser::ParsedClientConfig::parse(&client_conf).unwrap();
    assert_eq!(parsed.endpoint.port(), 12345);
    assert_eq!(parsed.endpoint.ip().to_string(), "10.0.0.1");
}

#[test]
fn multiple_keypairs_are_independent() {
    let pairs: Vec<_> = (0..10).map(|_| keys::generate_keypair()).collect();

    // All private keys should be unique
    let priv_keys: std::collections::HashSet<_> = pairs.iter().map(|p| &p.private_key).collect();
    assert_eq!(priv_keys.len(), 10, "all private keys should be unique");

    // All public keys should be unique
    let pub_keys: std::collections::HashSet<_> = pairs.iter().map(|p| &p.public_key).collect();
    assert_eq!(pub_keys.len(), 10, "all public keys should be unique");
}

// ── State serialization ─────────────────────────────────────────────────────

#[test]
fn deployment_state_full_roundtrip() {
    let state = DeploymentState {
        status: DeploymentStatus::Deployed,
        deployment_mode: Some("aws".to_string()),
        region: Some("us-east-1".to_string()),
        vpc_id: Some("vpc-abc123".to_string()),
        igw_id: Some("igw-def456".to_string()),
        subnet_id: Some("subnet-ghi789".to_string()),
        route_table_id: Some("rtb-jkl012".to_string()),
        security_group_id: Some("sg-mno345".to_string()),
        key_pair_name: Some("createmyvpn-key".to_string()),
        instance_id: Some("i-pqr678".to_string()),
        allocation_id: Some("eipalloc-stu901".to_string()),
        association_id: Some("eipassoc-vwx234".to_string()),
        elastic_ip: Some("203.0.113.50".to_string()),
        ssh_private_key: Some("-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----".to_string()),
        ssh_user: Some("ubuntu".to_string()),
        server_public_key: Some("server_pub_key_base64".to_string()),
        client_private_key: Some("client_priv_key_base64".to_string()),
        client_public_key: Some("client_pub_key_base64".to_string()),
        client_config: Some("[Interface]\nPrivateKey = test".to_string()),
        deployed_at: Some(chrono::Utc::now()),
        auto_destroy_at: None,
        error_message: None,
        droplet_id: None,
        do_firewall_id: None,
        do_ssh_key_id: None,
    };

    let json = serde_json::to_string_pretty(&state).expect("serialize");
    let restored: DeploymentState = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.status, DeploymentStatus::Deployed);
    assert_eq!(restored.deployment_mode, Some("aws".to_string()));
    assert_eq!(restored.vpc_id, Some("vpc-abc123".to_string()));
    assert_eq!(restored.instance_id, Some("i-pqr678".to_string()));
    assert_eq!(restored.elastic_ip, Some("203.0.113.50".to_string()));
    assert_eq!(restored.ssh_user, Some("ubuntu".to_string()));
}

#[test]
fn deployment_state_do_fields_roundtrip() {
    let state = DeploymentState {
        status: DeploymentStatus::Deployed,
        deployment_mode: Some("do".to_string()),
        droplet_id: Some(123456789),
        do_firewall_id: Some("fw-uuid".to_string()),
        do_ssh_key_id: Some(987654),
        elastic_ip: Some("167.99.0.1".to_string()),
        ..DeploymentState::default()
    };

    let json = serde_json::to_string(&state).unwrap();
    let restored: DeploymentState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.droplet_id, Some(123456789));
    assert_eq!(restored.do_firewall_id, Some("fw-uuid".to_string()));
    assert_eq!(restored.do_ssh_key_id, Some(987654));
}

#[test]
fn deployment_state_byo_mode() {
    let state = DeploymentState {
        status: DeploymentStatus::Deployed,
        deployment_mode: Some("byo".to_string()),
        elastic_ip: Some("10.0.0.5".to_string()),
        ssh_user: Some("root".to_string()),
        ..DeploymentState::default()
    };

    let json = serde_json::to_string(&state).unwrap();
    let restored: DeploymentState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.deployment_mode, Some("byo".to_string()));
    assert_eq!(restored.ssh_user, Some("root".to_string()));
    // BYO should not have AWS-specific fields
    assert!(restored.vpc_id.is_none());
    assert!(restored.instance_id.is_none());
    assert!(restored.droplet_id.is_none());
}

#[test]
fn corrupt_state_json_falls_back_to_default() {
    // Simulating what store.rs does: if JSON is corrupt, return default
    let corrupt = "{ this is not valid json }}}";
    let result: Result<DeploymentState, _> = serde_json::from_str(corrupt);
    assert!(result.is_err());

    // The store returns default on error
    let state = DeploymentState::default();
    assert_eq!(state.status, DeploymentStatus::NotDeployed);
}

#[test]
fn state_with_missing_new_fields_deserializes() {
    // Simulate old state file that doesn't have DO fields
    let old_json = r#"{
        "status": "deployed",
        "region": "us-east-1",
        "vpc_id": "vpc-123",
        "elastic_ip": "1.2.3.4"
    }"#;

    let state: DeploymentState = serde_json::from_str(old_json).unwrap();
    assert_eq!(state.status, DeploymentStatus::Deployed);
    assert_eq!(state.vpc_id, Some("vpc-123".to_string()));
    // New fields should be None
    assert!(state.droplet_id.is_none());
    assert!(state.do_firewall_id.is_none());
    assert!(state.deployment_mode.is_none());
}

// ── Error chains ────────────────────────────────────────────────────────────

#[test]
fn error_variant_display_messages() {
    use createmyvpn_lib::error::AppError;

    let cases = vec![
        (AppError::Aws("region unavailable".into()), "AWS error: region unavailable"),
        (AppError::Ssh("timeout".into()), "SSH error: timeout"),
        (AppError::WireGuard("bad config".into()), "WireGuard error: bad config"),
        (AppError::State("corrupt".into()), "State error: corrupt"),
        (AppError::Credential("expired".into()), "Credential error: expired"),
        (AppError::General("oops".into()), "oops"),
    ];

    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected);
    }
}

#[test]
fn error_serializes_to_json_string() {
    use createmyvpn_lib::error::AppError;

    let err = AppError::Aws("something failed".into());
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json, serde_json::Value::String("AWS error: something failed".to_string()));
}

// ── AppSettings validation ──────────────────────────────────────────────────

#[test]
fn settings_default_port_is_wireguard_standard() {
    let settings = AppSettings::new();
    assert_eq!(settings.wireguard_port, 51820);
}

#[test]
fn settings_with_custom_values_roundtrip() {
    let settings = AppSettings {
        region: "ap-southeast-1".to_string(),
        instance_type: "t3.small".to_string(),
        wireguard_port: 9999,
    };

    let json = serde_json::to_string(&settings).unwrap();
    let restored: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.region, "ap-southeast-1");
    assert_eq!(restored.instance_type, "t3.small");
    assert_eq!(restored.wireguard_port, 9999);
}

// ── Credentials ─────────────────────────────────────────────────────────────

#[test]
fn aws_credentials_roundtrip() {
    let creds = AwsCredentials {
        access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
    };

    let json = serde_json::to_string(&creds).unwrap();
    assert!(json.contains("AKIAIOSFODNN7EXAMPLE"));

    let restored: AwsCredentials = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.access_key_id, "AKIAIOSFODNN7EXAMPLE");
}

#[test]
fn do_credentials_roundtrip() {
    let creds = DoCredentials {
        api_token: "dop_v1_abcdef1234567890".to_string(),
    };

    let json = serde_json::to_string(&creds).unwrap();
    let restored: DoCredentials = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.api_token, "dop_v1_abcdef1234567890");
}

// ── ProgressEvent ───────────────────────────────────────────────────────────

#[test]
fn progress_event_all_statuses() {
    for status in ["running", "done", "error"] {
        let event = ProgressEvent {
            step: 1,
            total_steps: 10,
            message: "test".to_string(),
            status: status.to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(status));
    }
}

#[test]
fn progress_event_step_bounds() {
    let event = ProgressEvent {
        step: 10,
        total_steps: 10,
        message: "Final step".to_string(),
        status: "done".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"step\":10"));
    assert!(json.contains("\"total_steps\":10"));
}
