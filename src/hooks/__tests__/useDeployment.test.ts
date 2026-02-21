import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useDeployment } from "../../hooks/useDeployment";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

const mockDefaultState = {
  status: "not_deployed" as const,
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

const mockDeployedState = {
  ...mockDefaultState,
  status: "deployed" as const,
  deployment_mode: "aws",
  region: "us-east-1",
  elastic_ip: "1.2.3.4",
};

describe("useDeployment", () => {
  it("starts with loading=true", () => {
    mockInvoke.mockResolvedValue(mockDefaultState);
    const { result } = renderHook(() => useDeployment());
    expect(result.current.loading).toBe(true);
  });

  it("loads deployment state on mount", async () => {
    mockInvoke.mockResolvedValue(mockDefaultState);
    const { result } = renderHook(() => useDeployment());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.state).toEqual(mockDefaultState);
  });

  it("handles load error gracefully", async () => {
    mockInvoke.mockRejectedValue(new Error("failed"));
    const { result } = renderHook(() => useDeployment());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.state).toBeNull();
  });

  it("deploy calls Tauri and updates state", async () => {
    mockInvoke.mockResolvedValue(mockDefaultState);
    const { result } = renderHook(() => useDeployment());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockInvoke.mockResolvedValue(mockDeployedState);
    await act(async () => {
      const deployResult = await result.current.deploy("us-east-1");
      expect(deployResult.status).toBe("deployed");
    });

    expect(result.current.state?.status).toBe("deployed");
  });

  it("deploy clears previous progress", async () => {
    mockInvoke.mockResolvedValue(mockDefaultState);
    const { result } = renderHook(() => useDeployment());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockInvoke.mockResolvedValue(mockDeployedState);
    await act(async () => {
      await result.current.deploy("us-east-1");
    });

    expect(result.current.progress).toEqual([]);
  });

  it("destroy calls Tauri and clears state", async () => {
    mockInvoke.mockResolvedValue(mockDeployedState);
    const { result } = renderHook(() => useDeployment());

    await waitFor(() => {
      expect(result.current.state?.status).toBe("deployed");
    });

    mockInvoke.mockResolvedValue(undefined);
    await act(async () => {
      await result.current.destroy();
    });

    expect(result.current.state).toBeNull();
  });

  it("refresh reloads state", async () => {
    mockInvoke.mockResolvedValue(mockDefaultState);
    const { result } = renderHook(() => useDeployment());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockInvoke.mockResolvedValue(mockDeployedState);
    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.state?.status).toBe("deployed");
  });
});
