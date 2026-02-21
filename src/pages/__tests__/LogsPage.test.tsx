import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import LogsPage from "../LogsPage";
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
      case "get_logs":
        return "2024-01-01 12:00:00 [INFO] Application started\n2024-01-01 12:00:01 [INFO] Ready";
      case "export_logs":
        return "/home/user/logs/createmyvpn.log";
      case "clear_logs":
        return null;
      default:
        return null;
    }
  });
});

function renderPage() {
  return render(
    <MemoryRouter>
      <LogsPage />
    </MemoryRouter>,
  );
}

describe("LogsPage", () => {
  it("renders the Logs heading", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText("Logs")).toBeInTheDocument();
    });
  });

  it("fetches and displays log content", async () => {
    renderPage();
    await waitFor(() => {
      expect(screen.getByText(/Application started/)).toBeInTheDocument();
    });
  });

  it("calls get_logs on mount", async () => {
    renderPage();
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_logs");
    });
  });

  it("has a download/export button", async () => {
    renderPage();
    await waitFor(() => {
      const downloadBtn = screen.getByText(/Save|Download|Export/i);
      expect(downloadBtn).toBeInTheDocument();
    });
  });

  it("has a clear button", async () => {
    renderPage();
    await waitFor(() => {
      const clearBtn = screen.getByText(/Clear/i);
      expect(clearBtn).toBeInTheDocument();
    });
  });

  it("calls export_logs when download is clicked", async () => {
    renderPage();
    // Wait for logs to load so the Download button is enabled
    await waitFor(() => {
      const btn = screen.getByRole("button", { name: /Download/i });
      expect(btn).not.toBeDisabled();
    });
    fireEvent.click(screen.getByRole("button", { name: /Download/i }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("export_logs");
    });
  });

  it("subscribes to deploy-progress and destroy-progress events", () => {
    renderPage();
    expect(mockListen).toHaveBeenCalledWith("deploy-progress", expect.any(Function));
    expect(mockListen).toHaveBeenCalledWith("destroy-progress", expect.any(Function));
  });
});
