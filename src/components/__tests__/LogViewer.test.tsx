import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import LogViewer from "../../components/LogViewer";

describe("LogViewer", () => {
  it("shows placeholder when no logs", () => {
    render(<LogViewer logs={[]} />);
    expect(screen.getByText("Waiting for events...")).toBeInTheDocument();
  });

  it("renders log entries", () => {
    const logs = ["Connecting to server...", "WireGuard started", "Tunnel active"];
    render(<LogViewer logs={logs} />);
    expect(screen.getByText(/Connecting to server/)).toBeInTheDocument();
    expect(screen.getByText(/WireGuard started/)).toBeInTheDocument();
    expect(screen.getByText(/Tunnel active/)).toBeInTheDocument();
  });

  it("renders line numbers", () => {
    const logs = ["Line one", "Line two"];
    render(<LogViewer logs={logs} />);
    expect(screen.getByText("[01]")).toBeInTheDocument();
    expect(screen.getByText("[02]")).toBeInTheDocument();
  });

  it("renders multiple log entries", () => {
    const logs = Array.from({ length: 20 }, (_, i) => `Log line ${i + 1}`);
    render(<LogViewer logs={logs} />);
    expect(screen.getByText(/Log line 1$/)).toBeInTheDocument();
    expect(screen.getByText(/Log line 20/)).toBeInTheDocument();
  });

  it("has scrollable container", () => {
    const { container } = render(<LogViewer logs={["test"]} />);
    const scrollDiv = container.querySelector(".overflow-y-auto");
    expect(scrollDiv).toBeTruthy();
  });

  it("uses monospace font", () => {
    const { container } = render(<LogViewer logs={["test"]} />);
    const monoDiv = container.querySelector(".font-mono");
    expect(monoDiv).toBeTruthy();
  });
});
