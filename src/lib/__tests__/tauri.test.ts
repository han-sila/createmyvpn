import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  validateCredentials,
  saveCredentials,
  loadCredentials,
  deleteCredentials,
  validateDoCredentials,
  saveDoCredentials,
  loadDoCredentials,
  deleteDoCredentials,
  deployVpn,
  deployDo,
  deployByoVps,
  getDeploymentState,
  resetDeploymentState,
  destroyVpn,
  connectVpn,
  disconnectVpn,
  getVpnStatus,
  getClientConfig,
  getRegions,
  getSettings,
  updateSettings,
  getLogs,
  exportLogs,
  clearLogs,
  exportClientConfig,
} from "../../lib/tauri";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("Credential functions", () => {
  it("validateCredentials invokes with correct args", async () => {
    mockInvoke.mockResolvedValue("arn:aws:iam::123:user/test");
    const result = await validateCredentials("AKID", "SECRET", "us-east-1");
    expect(mockInvoke).toHaveBeenCalledWith("validate_credentials", {
      accessKeyId: "AKID",
      secretAccessKey: "SECRET",
      region: "us-east-1",
    });
    expect(result).toBe("arn:aws:iam::123:user/test");
  });

  it("saveCredentials invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await saveCredentials("AKID", "SECRET");
    expect(mockInvoke).toHaveBeenCalledWith("save_credentials", {
      accessKeyId: "AKID",
      secretAccessKey: "SECRET",
    });
  });

  it("loadCredentials returns creds or null", async () => {
    mockInvoke.mockResolvedValue({
      access_key_id: "AKID",
      secret_access_key: "SECRET",
    });
    const creds = await loadCredentials();
    expect(creds).toEqual({
      access_key_id: "AKID",
      secret_access_key: "SECRET",
    });
  });

  it("loadCredentials returns null when no creds", async () => {
    mockInvoke.mockResolvedValue(null);
    const creds = await loadCredentials();
    expect(creds).toBeNull();
  });

  it("deleteCredentials invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await deleteCredentials();
    expect(mockInvoke).toHaveBeenCalledWith("delete_credentials");
  });
});

describe("DigitalOcean credential functions", () => {
  it("validateDoCredentials invokes with token", async () => {
    mockInvoke.mockResolvedValue("team-name");
    const result = await validateDoCredentials("dop_v1_abc");
    expect(mockInvoke).toHaveBeenCalledWith("validate_do_credentials", {
      apiToken: "dop_v1_abc",
    });
    expect(result).toBe("team-name");
  });

  it("saveDoCredentials invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await saveDoCredentials("dop_v1_abc");
    expect(mockInvoke).toHaveBeenCalledWith("save_do_credentials", {
      apiToken: "dop_v1_abc",
    });
  });

  it("loadDoCredentials returns creds or null", async () => {
    mockInvoke.mockResolvedValue({ api_token: "dop_v1_abc" });
    const creds = await loadDoCredentials();
    expect(creds).toEqual({ api_token: "dop_v1_abc" });
  });

  it("deleteDoCredentials invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await deleteDoCredentials();
    expect(mockInvoke).toHaveBeenCalledWith("delete_do_credentials");
  });
});

describe("Deploy functions", () => {
  const mockState = {
    status: "deployed" as const,
    deployment_mode: "aws",
    region: "us-east-1",
    vpc_id: "vpc-123",
    igw_id: null,
    subnet_id: null,
    route_table_id: null,
    security_group_id: null,
    key_pair_name: null,
    instance_id: null,
    allocation_id: null,
    association_id: null,
    elastic_ip: "1.2.3.4",
    ssh_private_key: null,
    ssh_user: null,
    server_public_key: null,
    client_private_key: null,
    client_public_key: null,
    client_config: null,
    deployed_at: null,
    auto_destroy_at: null,
    error_message: null,
    droplet_id: null,
    do_firewall_id: null,
    do_ssh_key_id: null,
  };

  it("deployVpn invokes with region and null autoDestroy", async () => {
    mockInvoke.mockResolvedValue(mockState);
    const result = await deployVpn("us-east-1");
    expect(mockInvoke).toHaveBeenCalledWith("deploy_vpn", {
      region: "us-east-1",
      autoDestroyHours: null,
    });
    expect(result.status).toBe("deployed");
  });

  it("deployVpn passes autoDestroyHours when set", async () => {
    mockInvoke.mockResolvedValue(mockState);
    await deployVpn("us-west-2", 4);
    expect(mockInvoke).toHaveBeenCalledWith("deploy_vpn", {
      region: "us-west-2",
      autoDestroyHours: 4,
    });
  });

  it("deployDo invokes with region, size, autoDestroy", async () => {
    mockInvoke.mockResolvedValue({ ...mockState, deployment_mode: "do" });
    await deployDo("nyc1", "s-1vcpu-1gb", 2);
    expect(mockInvoke).toHaveBeenCalledWith("deploy_do", {
      region: "nyc1",
      size: "s-1vcpu-1gb",
      autoDestroyHours: 2,
    });
  });

  it("deployByoVps invokes with server details", async () => {
    mockInvoke.mockResolvedValue({ ...mockState, deployment_mode: "byo" });
    await deployByoVps("10.0.0.1", "key-data", "root", 22);
    expect(mockInvoke).toHaveBeenCalledWith("deploy_byo_vps", {
      serverIp: "10.0.0.1",
      sshPrivateKey: "key-data",
      sshUser: "root",
      sshPort: 22,
      autoDestroyHours: null,
    });
  });

  it("getDeploymentState returns state", async () => {
    mockInvoke.mockResolvedValue(mockState);
    const result = await getDeploymentState();
    expect(result.status).toBe("deployed");
  });

  it("resetDeploymentState invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await resetDeploymentState();
    expect(mockInvoke).toHaveBeenCalledWith("reset_deployment_state");
  });
});

describe("Destroy / Connect / Status functions", () => {
  it("destroyVpn invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await destroyVpn();
    expect(mockInvoke).toHaveBeenCalledWith("destroy_vpn");
  });

  it("connectVpn invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await connectVpn();
    expect(mockInvoke).toHaveBeenCalledWith("connect_vpn");
  });

  it("disconnectVpn invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await disconnectVpn();
    expect(mockInvoke).toHaveBeenCalledWith("disconnect_vpn");
  });

  it("getVpnStatus returns status string", async () => {
    mockInvoke.mockResolvedValue("connected");
    const status = await getVpnStatus();
    expect(status).toBe("connected");
  });

  it("getClientConfig returns config or null", async () => {
    mockInvoke.mockResolvedValue("[Interface]\nPrivateKey = ...");
    const config = await getClientConfig();
    expect(config).toContain("[Interface]");
  });
});

describe("Settings functions", () => {
  it("getSettings returns settings object", async () => {
    const settings = {
      region: "us-east-1",
      instance_type: "t2.micro",
      wireguard_port: 51820,
    };
    mockInvoke.mockResolvedValue(settings);
    const result = await getSettings();
    expect(result.wireguard_port).toBe(51820);
  });

  it("updateSettings invokes with correct args", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await updateSettings("eu-west-1", "t3.micro", 51821);
    expect(mockInvoke).toHaveBeenCalledWith("update_settings", {
      region: "eu-west-1",
      instanceType: "t3.micro",
      wireguardPort: 51821,
    });
  });

  it("getRegions returns array", async () => {
    mockInvoke.mockResolvedValue([{ code: "us-east-1", name: "US East" }]);
    const regions = await getRegions();
    expect(regions).toHaveLength(1);
  });
});

describe("Log functions", () => {
  it("getLogs returns log string", async () => {
    mockInvoke.mockResolvedValue("2026-02-21 log line");
    const logs = await getLogs();
    expect(logs).toContain("2026");
  });

  it("exportLogs returns file path", async () => {
    mockInvoke.mockResolvedValue("/home/user/Downloads/logs.txt");
    const path = await exportLogs();
    expect(path).toContain("logs");
  });

  it("clearLogs invokes correctly", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await clearLogs();
    expect(mockInvoke).toHaveBeenCalledWith("clear_logs");
  });

  it("exportClientConfig returns file path", async () => {
    mockInvoke.mockResolvedValue("/home/user/Downloads/client.conf");
    const path = await exportClientConfig();
    expect(path).toContain("client.conf");
  });
});
