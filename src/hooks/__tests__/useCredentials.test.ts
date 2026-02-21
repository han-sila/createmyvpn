import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useCredentials } from "../../hooks/useCredentials";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("useCredentials", () => {
  it("starts with loading=true", () => {
    mockInvoke.mockResolvedValue(null);
    const { result } = renderHook(() => useCredentials());
    expect(result.current.loading).toBe(true);
  });

  it("loads credentials on mount", async () => {
    mockInvoke.mockResolvedValue({
      access_key_id: "AKID",
      secret_access_key: "SECRET",
    });

    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.credentials).toEqual({
      access_key_id: "AKID",
      secret_access_key: "SECRET",
    });
    expect(result.current.hasCredentials).toBe(true);
  });

  it("sets hasCredentials to false when no creds", async () => {
    mockInvoke.mockResolvedValue(null);

    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.credentials).toBeNull();
    expect(result.current.hasCredentials).toBe(false);
  });

  it("handles load error gracefully", async () => {
    mockInvoke.mockRejectedValue(new Error("failed"));

    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.credentials).toBeNull();
    expect(result.current.hasCredentials).toBe(false);
  });

  it("save updates credentials state", async () => {
    mockInvoke.mockResolvedValue(null); // loadCredentials
    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockInvoke.mockResolvedValue(undefined); // saveCredentials
    await act(async () => {
      await result.current.save("NEW_AKID", "NEW_SECRET");
    });

    expect(result.current.credentials).toEqual({
      access_key_id: "NEW_AKID",
      secret_access_key: "NEW_SECRET",
    });
    expect(result.current.hasCredentials).toBe(true);
  });

  it("remove clears credentials", async () => {
    mockInvoke.mockResolvedValue({
      access_key_id: "AKID",
      secret_access_key: "SECRET",
    });

    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.hasCredentials).toBe(true);
    });

    mockInvoke.mockResolvedValue(undefined); // deleteCredentials
    await act(async () => {
      await result.current.remove();
    });

    expect(result.current.credentials).toBeNull();
    expect(result.current.hasCredentials).toBe(false);
  });

  it("validate calls Tauri invoke", async () => {
    mockInvoke.mockResolvedValue(null); // loadCredentials
    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockInvoke.mockResolvedValue("arn:aws:iam::123:user/test");
    await act(async () => {
      const arn = await result.current.validate("AKID", "SECRET", "us-east-1");
      expect(arn).toBe("arn:aws:iam::123:user/test");
    });
  });

  it("refresh reloads credentials", async () => {
    mockInvoke.mockResolvedValue(null);
    const { result } = renderHook(() => useCredentials());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    mockInvoke.mockResolvedValue({
      access_key_id: "REFRESHED",
      secret_access_key: "KEY",
    });

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.credentials?.access_key_id).toBe("REFRESHED");
  });
});
