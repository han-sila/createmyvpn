import { useState, useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { Download, Trash2, FileText } from "lucide-react";
import { getLogs, exportLogs, clearLogs } from "../lib/tauri";
import type { ProgressEvent } from "../lib/types";

function LogsPage() {
  const [logs, setLogs] = useState("");
  const [liveEvents, setLiveEvents] = useState<string[]>([]);
  const [downloading, setDownloading] = useState(false);
  const [saveMsg, setSaveMsg] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);

  const fetchLogs = useCallback(async () => {
    try {
      const content = await getLogs();
      setLogs(content);
    } catch (err) {
      console.error("Failed to load logs:", err);
    }
  }, []);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  // Auto-refresh every 3 seconds
  useEffect(() => {
    const interval = setInterval(fetchLogs, 3000);
    return () => clearInterval(interval);
  }, [fetchLogs]);

  // Real-time deploy/destroy events
  useEffect(() => {
    const unlistenDeploy = listen<ProgressEvent>("deploy-progress", (event) => {
      const ts = new Date().toLocaleTimeString();
      setLiveEvents((prev) => [
        ...prev,
        `[${ts}] [DEPLOY ${event.payload.step}/${event.payload.total_steps}] ${event.payload.message}`,
      ]);
    });

    const unlistenDestroy = listen<ProgressEvent>("destroy-progress", (event) => {
      const ts = new Date().toLocaleTimeString();
      setLiveEvents((prev) => [
        ...prev,
        `[${ts}] [DESTROY ${event.payload.step}/${event.payload.total_steps}] ${event.payload.message}`,
      ]);
    });

    return () => {
      unlistenDeploy.then((fn) => fn());
      unlistenDestroy.then((fn) => fn());
    };
  }, []);

  // Auto-scroll to bottom on new content
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs, liveEvents]);

  const allContent =
    liveEvents.length > 0
      ? logs + "\n\n── Live Events ──────────────────────────────\n" + liveEvents.join("\n")
      : logs;

  const handleDownload = async () => {
    setDownloading(true);
    setSaveMsg("");
    try {
      const savedPath = await exportLogs();
      setSaveMsg(`Saved: ${savedPath}`);
      setTimeout(() => setSaveMsg(""), 5000);
    } catch (err) {
      setSaveMsg(`Error: ${String(err)}`);
    } finally {
      setDownloading(false);
    }
  };

  const handleClear = async () => {
    try {
      await clearLogs();
      setLogs("");
      setLiveEvents([]);
      setSaveMsg("");
    } catch (err) {
      console.error("Failed to clear logs:", err);
    }
  };

  const colorize = (line: string) => {
    if (line.includes(" ERROR ") || line.includes("❌")) return "text-red-400";
    if (line.includes(" WARN ")) return "text-yellow-400";
    if (line.includes(" INFO ")) return "text-blue-300";
    if (line.includes(" DEBUG ")) return "text-gray-500";
    if (line.includes("[DEPLOY")) return "text-green-400";
    if (line.includes("[DESTROY")) return "text-orange-400";
    if (line.startsWith("===") || line.startsWith("──")) return "text-gray-500 font-semibold";
    return "text-gray-400";
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold text-white">Logs</h2>
        <div className="flex items-center gap-2">
          <button
            onClick={handleDownload}
            disabled={!logs || downloading}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-300 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
          >
            <Download className="w-3.5 h-3.5" />
            {downloading ? "Saving..." : "Download"}
          </button>
          <button
            onClick={handleClear}
            disabled={!logs}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors disabled:opacity-50"
          >
            <Trash2 className="w-3.5 h-3.5" />
            Clear
          </button>
        </div>
      </div>

      {saveMsg && (
        <p className="text-xs text-gray-400 mb-2 font-mono truncate">{saveMsg}</p>
      )}

      {/* Log viewer */}
      <div
        ref={scrollRef}
        className="flex-1 bg-gray-950 border border-gray-800 rounded-lg p-4 overflow-y-auto font-mono text-xs min-h-0 leading-relaxed select-text cursor-text"
      >
        {allContent.trim().length > 0 ? (
          allContent.split("\n").map((line, i) => (
            <div key={i} className={`${colorize(line)} whitespace-pre-wrap break-all`}>
              {line}
            </div>
          ))
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <FileText className="w-12 h-12 text-gray-700 mb-3" />
            <p className="text-gray-500 text-sm">No logs yet.</p>
            <p className="text-gray-600 text-xs mt-1">
              Deploy, connect, or perform operations to generate log entries.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

export default LogsPage;
