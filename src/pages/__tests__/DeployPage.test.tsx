import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import DeployPage from "../DeployPage";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

beforeEach(() => {
  mockInvoke.mockReset();
  mockListen.mockReset();
  mockListen.mockImplementation(() => Promise.resolve(() => {}));
  mockInvoke.mockImplementation(async (cmd: string) => {
    switch (cmd) {
      case "get_settings":
        return {
          region: "us-east-1",
          instance_type: "t2.micro",
          wireguard_port: 51820,
        };
      case "load_credentials":
        return { access_key_id: "AKID", secret_access_key: "SECRET" };
      case "load_do_credentials":
        return { api_token: "dop_v1_test" };
      case "get_deployment_state":
        return { status: "not_deployed" };
      default:
        return null;
    }
  });
});

function renderPage() {
  return render(
    <MemoryRouter>
      <DeployPage />
    </MemoryRouter>,
  );
}

describe("DeployPage", () => {
  it("renders the deploy heading", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /Deploy VPN/i })).toBeInTheDocument();
    });
  });

  it("shows AWS mode tab", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /^AWS$/i })).toBeInTheDocument();
    });
  });

  it("shows DigitalOcean mode tab", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /DigitalOcean/i })).toBeInTheDocument();
    });
  });

  it("shows BYO mode tab", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Your Own Server/i })).toBeInTheDocument();
    });
  });

  it("shows Deploy button", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Deploy VPN|Launch/i)).toBeInTheDocument();
    });
  });

  it("shows auto-destroy timer options", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Never|manual/i)).toBeInTheDocument();
    });
  });

  it("shows region selector when in AWS mode", async () => {
    renderPage();
    await waitFor(() => {
      // Region selector shows region options
      expect(screen.getByText(/Region/i)).toBeInTheDocument();
    });
  });

  it("can switch to BYO mode", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Your Own Server/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /Your Own Server/i }));
    await waitFor(() => {
      expect(screen.getByText(/Server IP Address/i)).toBeInTheDocument();
    });
  });

  it("subscribes to deploy-progress events", () => {
    renderPage();
    expect(mockListen).toHaveBeenCalledWith("deploy-progress", expect.any(Function));
  });
});
