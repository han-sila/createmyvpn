import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { getDeploymentState, deployVpn, destroyVpn } from "../lib/tauri";
import type { DeploymentState, ProgressEvent } from "../lib/types";

export function useDeployment() {
  const [state, setState] = useState<DeploymentState | null>(null);
  const [progress, setProgress] = useState<ProgressEvent[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const s = await getDeploymentState();
      setState(s);
    } catch {
      setState(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlistenDeploy = listen<ProgressEvent>("deploy-progress", (e) => {
      setProgress((prev) => [...prev, e.payload]);
    });
    const unlistenDestroy = listen<ProgressEvent>("destroy-progress", (e) => {
      setProgress((prev) => [...prev, e.payload]);
    });

    return () => {
      unlistenDeploy.then((fn) => fn());
      unlistenDestroy.then((fn) => fn());
    };
  }, []);

  const deploy = async (region: string) => {
    setProgress([]);
    const result = await deployVpn(region);
    setState(result);
    return result;
  };

  const destroy = async () => {
    setProgress([]);
    await destroyVpn();
    setState(null);
  };

  return { state, progress, loading, deploy, destroy, refresh };
}
