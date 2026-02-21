import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import App from "../App";
import { invoke } from "@tauri-apps/api/core";

// Mock logo import
vi.mock("../assets/logo.png", () => ({ default: "logo.png" }));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
  // Default mocks for data loaded on page mount
  mockInvoke.mockImplementation(async (cmd: string) => {
    switch (cmd) {
      case "load_credentials":
        return null;
      case "load_do_credentials":
        return null;
      case "get_deployment_state":
        return {
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
      case "get_vpn_status":
        return "disconnected";
      case "get_settings":
        return {
          region: "us-east-1",
          instance_type: "t2.micro",
          wireguard_port: 51820,
        };
      case "get_logs":
        return "";
      case "get_regions":
        return [{ code: "us-east-1", name: "US East (N. Virginia)" }];
      default:
        return null;
    }
  });
});

describe("App Integration — Routing", () => {
  it("renders the sidebar with navigation", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("CreateMyVPN")).toBeInTheDocument();
    });
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("Setup")).toBeInTheDocument();
    expect(screen.getByText("Deploy")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
    expect(screen.getByText("Logs")).toBeInTheDocument();
  });

  it("shows version in sidebar", async () => {
    render(<App />);
    await waitFor(() => {
      expect(screen.getByText("v0.1.0")).toBeInTheDocument();
    });
  });
});

describe("App Integration — Default Route", () => {
  it("redirects / to /dashboard", async () => {
    render(<App />);
    await waitFor(() => {
      // Dashboard page should render for not_deployed state
      expect(screen.getByText("Dashboard")).toBeInTheDocument();
    });
  });
});
