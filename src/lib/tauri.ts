import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  AwsCredentials,
  AwsRegion,
  DeploymentState,
  DoCredentials,
  VpnConnectionStatus,
} from "./types";

// Credentials
export async function validateCredentials(
  accessKeyId: string,
  secretAccessKey: string,
  region: string
): Promise<string> {
  return invoke("validate_credentials", {
    accessKeyId,
    secretAccessKey,
    region,
  });
}

export async function saveCredentials(
  accessKeyId: string,
  secretAccessKey: string
): Promise<void> {
  return invoke("save_credentials", { accessKeyId, secretAccessKey });
}

export async function loadCredentials(): Promise<AwsCredentials | null> {
  return invoke("load_credentials");
}

export async function deleteCredentials(): Promise<void> {
  return invoke("delete_credentials");
}

// DigitalOcean Credentials
export async function validateDoCredentials(apiToken: string): Promise<string> {
  return invoke("validate_do_credentials", { apiToken });
}

export async function saveDoCredentials(apiToken: string): Promise<void> {
  return invoke("save_do_credentials", { apiToken });
}

export async function loadDoCredentials(): Promise<DoCredentials | null> {
  return invoke("load_do_credentials");
}

export async function deleteDoCredentials(): Promise<void> {
  return invoke("delete_do_credentials");
}

// Deploy
export async function deployVpn(
  region: string,
  autoDestroyHours?: number
): Promise<DeploymentState> {
  return invoke("deploy_vpn", {
    region,
    autoDestroyHours: autoDestroyHours ?? null,
  });
}

export async function deployDo(
  region: string,
  size: string,
  autoDestroyHours?: number
): Promise<DeploymentState> {
  return invoke("deploy_do", {
    region,
    size,
    autoDestroyHours: autoDestroyHours ?? null,
  });
}

export async function deployByoVps(
  serverIp: string,
  sshPrivateKey: string,
  sshUser: string,
  sshPort: number,
  autoDestroyHours?: number
): Promise<DeploymentState> {
  return invoke("deploy_byo_vps", {
    serverIp,
    sshPrivateKey,
    sshUser,
    sshPort,
    autoDestroyHours: autoDestroyHours ?? null,
  });
}

export async function getDeploymentState(): Promise<DeploymentState> {
  return invoke("get_deployment_state");
}

export async function resetDeploymentState(): Promise<void> {
  return invoke("reset_deployment_state");
}

// Destroy
export async function destroyVpn(): Promise<void> {
  return invoke("destroy_vpn");
}

// Connect
export async function connectVpn(): Promise<void> {
  return invoke("connect_vpn");
}

export async function disconnectVpn(): Promise<void> {
  return invoke("disconnect_vpn");
}

export async function getVpnStatus(): Promise<VpnConnectionStatus> {
  return invoke("get_vpn_status");
}

export async function getClientConfig(): Promise<string | null> {
  return invoke("get_client_config");
}

// Settings
export async function getRegions(): Promise<AwsRegion[]> {
  return invoke("get_regions");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function updateSettings(
  region: string,
  instanceType: string,
  wireguardPort: number
): Promise<void> {
  return invoke("update_settings", {
    region,
    instanceType,
    wireguardPort,
  });
}

// Logs
export async function getLogs(): Promise<string> {
  return invoke("get_logs");
}

export async function exportLogs(): Promise<string> {
  return invoke("export_logs");
}

export async function clearLogs(): Promise<void> {
  return invoke("clear_logs");
}

export async function exportClientConfig(): Promise<string> {
  return invoke("export_client_config");
}
