import { describe, it, expect } from "vitest";
import {
  AWS_REGIONS,
  DO_REGIONS,
  type DeploymentStatus,
  type VpnConnectionStatus,
  type DeploymentState,
  type AppSettings,
  type ProgressEvent,
  type AwsCredentials,
  type DoCredentials,
} from "../../lib/types";

describe("AWS_REGIONS", () => {
  it("contains at least 10 regions", () => {
    expect(AWS_REGIONS.length).toBeGreaterThanOrEqual(10);
  });

  it("each region has a code and name", () => {
    for (const region of AWS_REGIONS) {
      expect(region.code).toBeTruthy();
      expect(region.name).toBeTruthy();
    }
  });

  it("includes us-east-1", () => {
    const usEast = AWS_REGIONS.find((r) => r.code === "us-east-1");
    expect(usEast).toBeDefined();
    expect(usEast!.name).toBe("US East (N. Virginia)");
  });

  it("has no duplicate region codes", () => {
    const codes = AWS_REGIONS.map((r) => r.code);
    expect(new Set(codes).size).toBe(codes.length);
  });
});

describe("DO_REGIONS", () => {
  it("contains at least 5 regions", () => {
    expect(DO_REGIONS.length).toBeGreaterThanOrEqual(5);
  });

  it("each region has a code and name", () => {
    for (const region of DO_REGIONS) {
      expect(region.code).toBeTruthy();
      expect(region.name).toBeTruthy();
    }
  });

  it("includes nyc1", () => {
    const nyc = DO_REGIONS.find((r) => r.code === "nyc1");
    expect(nyc).toBeDefined();
    expect(nyc!.name).toBe("New York 1");
  });

  it("has no duplicate region codes", () => {
    const codes = DO_REGIONS.map((r) => r.code);
    expect(new Set(codes).size).toBe(codes.length);
  });
});

describe("Type shape validation", () => {
  it("DeploymentState has correct shape", () => {
    const state: DeploymentState = {
      status: "not_deployed",
      deployment_mode: null,
      region: null,
      vpc_id: null,
      igw_id: null,
      subnet_id: null,
      route_table_id: null,
      security_group_id: null,
      key_pair_name: null,
      instance_id: null,
      allocation_id: null,
      association_id: null,
      elastic_ip: null,
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
    expect(state.status).toBe("not_deployed");
    expect(state.deployment_mode).toBeNull();
  });

  it("AppSettings has correct defaults", () => {
    const settings: AppSettings = {
      region: "us-east-1",
      instance_type: "t2.micro",
      wireguard_port: 51820,
    };
    expect(settings.region).toBe("us-east-1");
    expect(settings.wireguard_port).toBe(51820);
  });

  it("ProgressEvent has correct shape", () => {
    const event: ProgressEvent = {
      step: 1,
      total_steps: 10,
      message: "Creating VPC",
      status: "running",
    };
    expect(event.step).toBe(1);
    expect(event.status).toBe("running");
  });

  it("DeploymentStatus literals are valid", () => {
    const statuses: DeploymentStatus[] = [
      "not_deployed",
      "deploying",
      "deployed",
      "destroying",
      "failed",
    ];
    expect(statuses).toHaveLength(5);
  });

  it("VpnConnectionStatus literals are valid", () => {
    const statuses: VpnConnectionStatus[] = [
      "disconnected",
      "connecting",
      "connected",
      "disconnecting",
    ];
    expect(statuses).toHaveLength(4);
  });

  it("AwsCredentials has required fields", () => {
    const creds: AwsCredentials = {
      access_key_id: "AKIAIOSFODNN7EXAMPLE",
      secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    };
    expect(creds.access_key_id).toBeTruthy();
    expect(creds.secret_access_key).toBeTruthy();
  });

  it("DoCredentials has required fields", () => {
    const creds: DoCredentials = {
      api_token: "dop_v1_abc123",
    };
    expect(creds.api_token).toBeTruthy();
  });
});
