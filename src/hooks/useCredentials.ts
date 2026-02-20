import { useState, useEffect, useCallback } from "react";
import {
  loadCredentials,
  saveCredentials,
  deleteCredentials,
  validateCredentials,
} from "../lib/tauri";
import type { AwsCredentials } from "../lib/types";

export function useCredentials() {
  const [credentials, setCredentials] = useState<AwsCredentials | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const creds = await loadCredentials();
      setCredentials(creds);
    } catch {
      setCredentials(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const save = async (accessKeyId: string, secretAccessKey: string) => {
    await saveCredentials(accessKeyId, secretAccessKey);
    setCredentials({ access_key_id: accessKeyId, secret_access_key: secretAccessKey });
  };

  const validate = async (
    accessKeyId: string,
    secretAccessKey: string,
    region: string
  ) => {
    return validateCredentials(accessKeyId, secretAccessKey, region);
  };

  const remove = async () => {
    await deleteCredentials();
    setCredentials(null);
  };

  return {
    credentials,
    loading,
    hasCredentials: !!credentials,
    save,
    validate,
    remove,
    refresh,
  };
}
