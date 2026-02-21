import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import SetupPage from "../SetupPage";
import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation(async (cmd: string) => {
    switch (cmd) {
      case "load_credentials":
        return null;
      case "load_do_credentials":
        return null;
      default:
        return null;
    }
  });
});

function renderPage() {
  return render(
    <MemoryRouter>
      <SetupPage />
    </MemoryRouter>,
  );
}

describe("SetupPage", () => {
  it("renders the page heading", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: /Setup/i })).toBeInTheDocument();
    });
  });

  it("shows AWS tab by default", async () => {
    renderPage();
    await waitFor(() => {
      // Should show the AWS tab button
      expect(screen.getByRole("button", { name: /^AWS$/i })).toBeInTheDocument();
    });
  });

  it("shows DigitalOcean tab", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/DigitalOcean/i)).toBeInTheDocument();
    });
  });

  it("can switch to DO tab", async () => {
    renderPage();
    await waitFor(() => {
      const doTab = screen.getByText(/DigitalOcean/i);
      fireEvent.click(doTab);
    });
    await waitFor(() => {
      // Should now show DO token input area
      expect(screen.getByText(/API Token/i)).toBeInTheDocument();
    });
  });

  it("has Access Key input on AWS tab", async () => {
    renderPage();
    await waitFor(() => {
      const input = screen.getByPlaceholderText(/AKIA/i);
      expect(input).toBeInTheDocument();
    });
  });

  it("has a Validate button", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Validate/i)).toBeInTheDocument();
    });
  });

  it("loads saved credentials on mount", async () => {
    renderPage();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("load_credentials");
      expect(mockInvoke).toHaveBeenCalledWith("load_do_credentials");
    });
  });

  it("populates inputs with saved credentials", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case "load_credentials":
          return { access_key_id: "AKIATEST123", secret_access_key: "secretXYZ" };
        case "load_do_credentials":
          return null;
        default:
          return null;
      }
    });

    renderPage();
    await waitFor(() => {
      expect(screen.getByDisplayValue("AKIATEST123")).toBeInTheDocument();
    });
  });

  it("has a region selector", async () => {
    renderPage();
    await waitFor(() => {
      // RegionSelector renders a select element
      expect(screen.getByText(/Region/i)).toBeInTheDocument();
    });
  });
});
