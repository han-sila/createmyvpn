import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useVpnStatus } from "../../hooks/useVpnStatus";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("useVpnStatus", () => {
  it("starts with disconnected status", () => {
    mockInvoke.mockResolvedValue("disconnected");
    const { result } = renderHook(() => useVpnStatus());
    expect(result.current.status).toBe("disconnected");
  });

  it("fetches status on mount", async () => {
    mockInvoke.mockResolvedValue("connected");
    const { result } = renderHook(() => useVpnStatus());

    await waitFor(() => {
      expect(result.current.status).toBe("connected");
    });
  });

  it("falls back to disconnected on error", async () => {
    mockInvoke.mockRejectedValue(new Error("fail"));
    const { result } = renderHook(() => useVpnStatus());

    await waitFor(() => {
      expect(result.current.status).toBe("disconnected");
    });
  });

  it("provides a refresh function", async () => {
    mockInvoke.mockResolvedValue("disconnected");
    const { result } = renderHook(() => useVpnStatus());

    await waitFor(() => {
      expect(result.current.status).toBe("disconnected");
    });

    expect(typeof result.current.refresh).toBe("function");
  });
});
