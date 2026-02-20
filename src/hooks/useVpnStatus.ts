import { useState, useEffect, useCallback } from "react";
import { getVpnStatus } from "../lib/tauri";
import type { VpnConnectionStatus } from "../lib/types";

export function useVpnStatus(pollInterval = 5000) {
  const [status, setStatus] = useState<VpnConnectionStatus>("disconnected");

  const refresh = useCallback(async () => {
    try {
      const s = await getVpnStatus();
      setStatus(s);
    } catch {
      setStatus("disconnected");
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, pollInterval);
    return () => clearInterval(interval);
  }, [refresh, pollInterval]);

  return { status, refresh };
}
