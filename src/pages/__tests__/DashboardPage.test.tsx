import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import DashboardPage from "../DashboardPage";
import { invoke } from "@tauri-apps/api/core";

// Mock qrcode.react
vi.mock("qrcode.react", () => ({
  QRCodeSVG: () => null,
}));

const mockInvoke = vi.mocked(invoke);

const notDeployed = {
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

const deployed = {
  ...notDeployed,
  status: "deployed",
  deployment_mode: "aws",
  region: "us-east-1",
  elastic_ip: "203.0.113.50",
  deployed_at: new Date().toISOString(),
};

beforeEach(() => {
  mockInvoke.mockReset();
});

function renderPage() {
  return render(
    <MemoryRouter>
      <DashboardPage />
    </MemoryRouter>,
  );
}

describe("DashboardPage — Not Deployed", () => {
  beforeEach(() => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case "get_deployment_state":
          return notDeployed;
        case "get_vpn_status":
          return "disconnected";
        default:
          return null;
      }
    });
  });

  it("renders dashboard heading", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/No VPN Deployed/i)).toBeInTheDocument();
    });
  });

  it("shows not deployed status", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /No VPN Deployed/i })).toBeInTheDocument();
    });
  });

  it("calls get_deployment_state on mount", async () => {
    renderPage();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_deployment_state");
    });
  });
});

describe("DashboardPage — Deployed", () => {
  beforeEach(() => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case "get_deployment_state":
          return deployed;
        case "get_vpn_status":
          return "disconnected";
        default:
          return null;
      }
    });
  });

  it("shows the server IP when deployed", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/203\.0\.113\.50/)).toBeInTheDocument();
    });
  });

  it("shows region info when deployed", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/us-east-1/)).toBeInTheDocument();
    });
  });

  it("shows connect/disconnect buttons when deployed", async () => {
    renderPage();
    await waitFor(() => {
      const btn = screen.getByRole("button", { name: /^Connect$|^Disconnect$/i });
      expect(btn).toBeInTheDocument();
    });
  });

  it("shows destroy button when deployed", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Destroy/i)).toBeInTheDocument();
    });
  });
});

describe("DashboardPage — Error state", () => {
  it("shows error message when deployment failed", async () => {
    const failedState = {
      ...notDeployed,
      status: "failed",
      error_message: "EC2 launch failed: quota exceeded",
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case "get_deployment_state":
          return failedState;
        case "get_vpn_status":
          return "disconnected";
        default:
          return null;
      }
    });

    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/quota exceeded/i)).toBeInTheDocument();
    });
  });
});
