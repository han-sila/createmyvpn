import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import SettingsPage from "../SettingsPage";
import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
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
        return null;
      case "update_settings":
        return null;
      case "delete_credentials":
        return null;
      default:
        return null;
    }
  });
});

function renderPage() {
  return render(
    <MemoryRouter>
      <SettingsPage />
    </MemoryRouter>,
  );
}

describe("SettingsPage", () => {
  it("renders the Settings heading", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText("Settings")).toBeInTheDocument();
    });
  });

  it("displays deployment section", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText("Deployment")).toBeInTheDocument();
    });
  });

  it("shows default WireGuard port", async () => {
    renderPage();
    await waitFor(() => {
      const portInput = screen.getByDisplayValue("51820");
      expect(portInput).toBeInTheDocument();
    });
  });

  it("shows Save button", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Save Settings/i })).toBeInTheDocument();
    });
  });

  it("calls update_settings on save", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Save Settings/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: /Save Settings/i }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("update_settings", expect.any(Object));
    });
  });

  it("shows AWS credentials section when credentials exist", async () => {
    renderPage();
    await waitFor(() => {
      // Should show a delete button for AWS credentials
      expect(screen.getByText(/AWS/i)).toBeInTheDocument();
    });
  });

  it("calls delete_credentials when removing AWS creds", async () => {
    renderPage();
    await waitFor(() => {
      // Find the delete button for AWS credentials
      const deleteButtons = screen.getAllByRole("button");
      const deleteAwsBtn = deleteButtons.find(
        (btn) => btn.textContent?.includes("Delete") || btn.textContent?.includes("Remove"),
      );
      if (deleteAwsBtn) {
        fireEvent.click(deleteAwsBtn);
      }
    });
  });

  it("shows Export Config button", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Export WireGuard Config/i })).toBeInTheDocument();
    });
  });
});
