fn main() {
    // On Windows RELEASE builds, embed a manifest requesting administrator privileges.
    // This is required for WinTUN to create a virtual network adapter.
    // Debug builds (tauri dev / cargo run) do NOT embed this manifest so the dev
    // binary can be launched without UAC elevation. VPN connect won't work in debug
    // without manual elevation, but the full UI is testable without admin rights.
    #[allow(unused_mut)]
    let mut attributes = tauri_build::Attributes::new();

    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        let windows = tauri_build::WindowsAttributes::new()
            .app_manifest(include_str!("createmyvpn.exe.manifest"));
        attributes = attributes.windows_attributes(windows);
    }

    tauri_build::try_build(attributes).expect("failed to run tauri-build");
}
