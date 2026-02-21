import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import StatusBadge from "../../components/StatusBadge";
import type { DeploymentStatus, VpnConnectionStatus } from "../../lib/types";

describe("StatusBadge", () => {
  const renderBadge = (
    deploymentStatus: DeploymentStatus,
    vpnStatus: VpnConnectionStatus = "disconnected"
  ) => {
    return render(
      <StatusBadge deploymentStatus={deploymentStatus} vpnStatus={vpnStatus} />
    );
  };

  it("shows 'Not Deployed' for not_deployed status", () => {
    renderBadge("not_deployed");
    expect(screen.getByText("Not Deployed")).toBeInTheDocument();
  });

  it("shows 'Deploying...' for deploying status", () => {
    renderBadge("deploying");
    expect(screen.getByText("Deploying...")).toBeInTheDocument();
  });

  it("shows 'Destroying...' for destroying status", () => {
    renderBadge("destroying");
    expect(screen.getByText("Destroying...")).toBeInTheDocument();
  });

  it("shows 'Failed' for failed status", () => {
    renderBadge("failed");
    expect(screen.getByText("Failed")).toBeInTheDocument();
  });

  it("shows 'Connected' when deployed and connected", () => {
    renderBadge("deployed", "connected");
    expect(screen.getByText("Connected")).toBeInTheDocument();
  });

  it("shows 'Connecting...' when deployed and connecting", () => {
    renderBadge("deployed", "connecting");
    expect(screen.getByText("Connecting...")).toBeInTheDocument();
  });

  it("shows 'Disconnecting...' when deployed and disconnecting", () => {
    renderBadge("deployed", "disconnecting");
    expect(screen.getByText("Disconnecting...")).toBeInTheDocument();
  });

  it("shows 'Deployed (Disconnected)' when deployed but disconnected", () => {
    renderBadge("deployed", "disconnected");
    expect(screen.getByText("Deployed (Disconnected)")).toBeInTheDocument();
  });

  it("has pulse animation for active states", () => {
    const { container } = renderBadge("deploying");
    const dot = container.querySelector(".animate-pulse");
    expect(dot).toBeTruthy();
  });

  it("has no pulse for static states", () => {
    const { container } = renderBadge("not_deployed");
    const dot = container.querySelector(".animate-pulse");
    expect(dot).toBeNull();
  });

  it("has no pulse for failed state", () => {
    const { container } = renderBadge("failed");
    const dot = container.querySelector(".animate-pulse");
    expect(dot).toBeNull();
  });
});
