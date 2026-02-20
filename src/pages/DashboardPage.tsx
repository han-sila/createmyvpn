import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import {
  Wifi,
  WifiOff,
  Globe,
  Trash2,
  Copy,
  CheckCircle2,
  XCircle,
  Loader2,
  QrCode,
  X,
} from "lucide-react";
import { QRCodeSVG } from "qrcode.react";
import StatusBadge from "../components/StatusBadge";
import ConfirmDialog from "../components/ConfirmDialog";
import type { DeploymentState, VpnConnectionStatus } from "../lib/types";
import {
  getDeploymentState,
  getVpnStatus,
  connectVpn,
  disconnectVpn,
  destroyVpn,
  getClientConfig,
  resetDeploymentState,
} from "../lib/tauri";

function formatCountdown(isoString: string): string {
  const target = new Date(isoString).getTime();
  const diff = target - Date.now();
  if (diff <= 0) return "Expired";
  const h = Math.floor(diff / 3_600_000);
  const m = Math.floor((diff % 3_600_000) / 60_000);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function DashboardPage() {
  const navigate = useNavigate();
  const [deployment, setDeployment] = useState<DeploymentState | null>(null);
  const [vpnStatus, setVpnStatus] = useState<VpnConnectionStatus>("disconnected");
  const [showDestroy, setShowDestroy] = useState(false);
  const [destroying, setDestroying] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [copied, setCopied] = useState(false);
  const [resetting, setResetting] = useState(false);
  const [error, setError] = useState("");

  // QR code
  const [showQr, setShowQr] = useState(false);
  const [qrConfig, setQrConfig] = useState<string | null>(null);

  // Auto-destroy countdown
  const [countdown, setCountdown] = useState<string | null>(null);

  const refreshState = useCallback(async () => {
    try {
      const state = await getDeploymentState();
      setDeployment(state);
      if (state.status === "deployed") {
        const status = await getVpnStatus();
        setVpnStatus(status);
      }
    } catch (err) {
      console.error("Failed to load state:", err);
    }
  }, []);

  useEffect(() => {
    refreshState();
    const interval = setInterval(refreshState, 5000);
    return () => clearInterval(interval);
  }, [refreshState]);

  // Countdown ticker for auto-destroy timer
  useEffect(() => {
    if (!deployment?.auto_destroy_at) {
      setCountdown(null);
      return;
    }
    const update = () => setCountdown(formatCountdown(deployment.auto_destroy_at!));
    update();
    const interval = setInterval(update, 60_000);
    return () => clearInterval(interval);
  }, [deployment?.auto_destroy_at]);

  const handleConnect = async () => {
    setConnecting(true);
    setError("");
    try {
      await connectVpn();
      setVpnStatus("connected");
    } catch (err) {
      setError(String(err));
    } finally {
      setConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    setConnecting(true);
    setError("");
    try {
      await disconnectVpn();
      setVpnStatus("disconnected");
    } catch (err) {
      setError(String(err));
    } finally {
      setConnecting(false);
    }
  };

  const handleDestroy = async () => {
    setShowDestroy(false);
    setDestroying(true);
    setError("");
    try {
      await destroyVpn();
      setDeployment(null);
      setVpnStatus("disconnected");
    } catch (err) {
      setError(String(err));
    } finally {
      setDestroying(false);
    }
  };

  const handleCopyConfig = async () => {
    const config = await getClientConfig();
    if (config) {
      await navigator.clipboard.writeText(config);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleShowQr = async () => {
    const config = await getClientConfig();
    setQrConfig(config);
    setShowQr(true);
  };

  const isByo = deployment?.deployment_mode === "byo";
  const isDo = deployment?.deployment_mode === "do";
  const isDeployed = deployment?.status === "deployed";
  const isConnected = vpnStatus === "connected";

  const destroyDialogMessage = isByo
    ? "This will stop WireGuard on your server and remove the local VPN config. Your server itself will NOT be deleted."
    : isDo
      ? "This will permanently delete your DigitalOcean Droplet, Firewall, and SSH key. Your WireGuard client config will stop working. This cannot be undone."
      : "This will permanently delete all AWS resources (EC2 instance, VPC, Elastic IP, etc.). Your WireGuard client config will stop working. This action cannot be undone.";

  // ── Stuck-state ───────────────────────────────────────────────────────────
  if (
    deployment &&
    (deployment.status === "deploying" || deployment.status === "destroying")
  ) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <Loader2 className="w-12 h-12 text-primary-400 animate-spin mb-4" />
        <h2 className="text-xl font-bold text-white mb-2">
          {deployment.status === "deploying"
            ? "Deployment in Progress"
            : "Destroying Server…"}
        </h2>
        <p className="text-gray-400 text-sm mb-6 max-w-sm">
          This operation appears to be running. If it started in a previous
          session and is stuck, refresh to re-check or restart the app.
        </p>
        <div className="flex gap-3">
          <button
            onClick={refreshState}
            className="px-6 py-2.5 text-sm font-medium text-white bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            Refresh Status
          </button>
          <button
            onClick={() => navigate("/logs")}
            className="px-6 py-2.5 text-sm font-medium text-primary-400 bg-primary-600/10 hover:bg-primary-600/20 rounded-lg transition-colors"
          >
            View Logs
          </button>
        </div>
      </div>
    );
  }

  // ── Failed state ──────────────────────────────────────────────────────────
  if (deployment && deployment.status === "failed") {
    return (
      <div className="max-w-lg">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-white">Dashboard</h2>
          <StatusBadge deploymentStatus={deployment.status} vpnStatus={vpnStatus} />
        </div>

        <div className="bg-red-500/10 border border-red-500/30 rounded-xl p-6 mb-4">
          <div className="flex items-center gap-3 mb-3">
            <XCircle className="w-6 h-6 text-red-400 flex-shrink-0" />
            <h3 className="text-lg font-semibold text-white">Deployment Failed</h3>
          </div>
          {deployment.error_message && (
            <p className="text-sm text-red-300 mb-3">{deployment.error_message}</p>
          )}
          <p className="text-xs text-gray-400 mb-4">
            Some AWS resources may have been partially created. You can attempt
            to destroy them with the button below, or reset the state and deploy
            again.
          </p>
          <div className="flex gap-3">
            <button
              onClick={async () => {
                setResetting(true);
                setError("");
                try {
                  await resetDeploymentState();
                  setDeployment(null);
                } catch (err) {
                  setError(String(err));
                } finally {
                  setResetting(false);
                }
              }}
              disabled={resetting}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors disabled:opacity-50"
            >
              {resetting ? "Resetting…" : "Reset & Deploy Again"}
            </button>
            {deployment.instance_id && (
              <button
                onClick={() => setShowDestroy(true)}
                disabled={destroying}
                className="flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors disabled:opacity-50"
              >
                <Trash2 className="w-4 h-4" />
                Destroy
              </button>
            )}
          </div>
          {error && (
            <div className="mt-3 bg-red-500/10 border border-red-500/30 rounded-lg p-3">
              <p className="text-xs text-red-300 font-mono break-all">{error}</p>
            </div>
          )}
        </div>

        <ConfirmDialog
          isOpen={showDestroy}
          title={isDo ? "Delete Droplet?" : "Destroy VPN Server?"}
          message={destroyDialogMessage}
          confirmLabel={isDo ? "Delete Everything" : "Destroy Everything"}
          onConfirm={handleDestroy}
          onCancel={() => setShowDestroy(false)}
        />
      </div>
    );
  }

  // ── No deployment ─────────────────────────────────────────────────────────
  if (!deployment || deployment.status === "not_deployed") {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <div className="w-20 h-20 rounded-full bg-gray-800 flex items-center justify-center mb-6">
          <Globe className="w-10 h-10 text-gray-600" />
        </div>
        <h2 className="text-2xl font-bold text-white mb-2">No VPN Deployed</h2>
        <p className="text-gray-400 text-sm mb-6 max-w-sm">
          Deploy your own private VPN server to get started. It only takes a
          couple of minutes.
        </p>
        <button
          onClick={() => navigate("/deploy")}
          className="px-6 py-3 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
        >
          Deploy VPN
        </button>
      </div>
    );
  }

  return (
    <div className="max-w-lg">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-white">Dashboard</h2>
        <StatusBadge
          deploymentStatus={deployment.status}
          vpnStatus={vpnStatus}
        />
      </div>

      {/* Notice banner */}
      {deployment.error_message && (
        <div className="mb-4 bg-yellow-500/10 border border-yellow-500/30 rounded-xl p-4">
          <p className="text-sm font-medium text-yellow-300 mb-1">⚠ Notice</p>
          <p className="text-xs text-yellow-200/80">{deployment.error_message}</p>
        </div>
      )}

      {/* Connection card */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-4">
        <div className="flex items-center justify-center mb-6">
          <div
            className={`w-24 h-24 rounded-full flex items-center justify-center transition-all duration-700 ${
              isConnected
                ? "bg-green-500/20 ring-4 ring-green-500/20 animate-glow-pulse"
                : "bg-gray-800"
            }`}
          >
            {isConnected ? (
              <Wifi className="w-12 h-12 text-green-400" />
            ) : (
              <WifiOff className="w-12 h-12 text-gray-500" />
            )}
          </div>
        </div>

        <div className="text-center mb-6">
          <p className="text-lg font-semibold text-white">
            {isConnected ? "VPN Connected" : "VPN Disconnected"}
          </p>
          <p className="text-sm text-gray-400 mt-1">
            {isConnected
              ? "Your traffic is secured through your private server"
              : "Click connect to route traffic through your VPN"}
          </p>
        </div>

        {isDeployed && (
          <button
            onClick={isConnected ? handleDisconnect : handleConnect}
            disabled={connecting || destroying}
            className={`w-full py-3 text-sm font-medium rounded-lg transition-colors ${
              isConnected
                ? "bg-red-600 hover:bg-red-500 text-white"
                : "bg-green-600 hover:bg-green-500 text-white"
            } disabled:opacity-50`}
          >
            {connecting
              ? "Please wait..."
              : isConnected
                ? "Disconnect"
                : "Connect"}
          </button>
        )}
      </div>

      {/* Server info */}
      {isDeployed && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-4 mb-4">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-gray-500 text-xs mb-1">Server IP</p>
              <p className="text-white font-mono">
                {deployment.elastic_ip || "—"}
              </p>
            </div>
            <div>
              <p className="text-gray-500 text-xs mb-1">
                {isByo ? "Mode" : "Region"}
              </p>
              <p className="text-white">
                {isByo
                  ? "Custom Server"
                  : isDo
                    ? `DO / ${deployment.region || "—"}`
                    : deployment.region || "—"}
              </p>
            </div>
            <div>
              <p className="text-gray-500 text-xs mb-1">Deployed</p>
              <p className="text-white">
                {deployment.deployed_at
                  ? new Date(deployment.deployed_at).toLocaleDateString()
                  : "—"}
              </p>
            </div>
            <div>
              {countdown ? (
                <>
                  <p className="text-gray-500 text-xs mb-1">Auto-destroys in</p>
                  <p className="text-amber-400 font-mono text-sm">{countdown}</p>
                </>
              ) : isByo ? (
                <>
                  <p className="text-gray-500 text-xs mb-1">Server</p>
                  <p className="text-white text-xs">
                    {deployment.ssh_user || "ubuntu"}@{deployment.elastic_ip || "—"}
                  </p>
                </>
              ) : (
                <>
                  <p className="text-gray-500 text-xs mb-1">Instance</p>
                  <p className="text-white font-mono text-xs">
                    {deployment.instance_id
                      ? deployment.instance_id.slice(0, 15) + "..."
                      : "—"}
                  </p>
                </>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-2">
        <button
          onClick={handleCopyConfig}
          className="flex-1 flex items-center justify-center gap-2 px-3 py-2.5 text-sm font-medium text-gray-300 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors"
        >
          {copied ? (
            <CheckCircle2 className="w-4 h-4 text-green-400" />
          ) : (
            <Copy className="w-4 h-4" />
          )}
          {copied ? "Copied!" : "Copy Config"}
        </button>

        <button
          onClick={handleShowQr}
          className="flex items-center justify-center gap-2 px-3 py-2.5 text-sm font-medium text-gray-300 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors"
        >
          <QrCode className="w-4 h-4" />
          Scan
        </button>

        <button
          onClick={() => setShowDestroy(true)}
          disabled={destroying}
          className="flex items-center justify-center gap-2 px-3 py-2.5 text-sm font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors disabled:opacity-50"
        >
          <Trash2 className="w-4 h-4" />
          Destroy
        </button>
      </div>

      {error && (
        <div className="mt-4 bg-red-500/10 border border-red-500/30 rounded-lg p-3">
          <p className="text-sm font-medium text-red-400 mb-1">
            Connection Error
          </p>
          <pre className="text-xs text-red-300 whitespace-pre-wrap break-words font-mono">
            {error}
          </pre>
          <button
            onClick={() => navigate("/logs")}
            className="mt-2 text-xs text-primary-400 hover:text-primary-300 underline"
          >
            View full logs for details
          </button>
        </div>
      )}

      <ConfirmDialog
        isOpen={showDestroy}
        title={
          isByo
            ? "Disconnect Server?"
            : isDo
              ? "Delete Droplet?"
              : "Destroy VPN Server?"
        }
        message={destroyDialogMessage}
        confirmLabel={
          isByo ? "Disconnect" : isDo ? "Delete Everything" : "Destroy Everything"
        }
        onConfirm={handleDestroy}
        onCancel={() => setShowDestroy(false)}
      />

      {/* QR code modal */}
      {showQr && (
        <div
          className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
          onClick={() => setShowQr(false)}
        >
          <div
            className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-4 max-w-xs w-full"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-white">Scan on Mobile</h3>
              <button
                onClick={() => setShowQr(false)}
                className="text-gray-500 hover:text-gray-300 transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {qrConfig ? (
              <>
                <div className="bg-white p-3 rounded-lg flex items-center justify-center">
                  <QRCodeSVG value={qrConfig} size={220} />
                </div>
                <p className="text-xs text-gray-500 text-center">
                  Open <span className="text-gray-300">WireGuard</span> on iOS
                  or Android → tap <span className="text-gray-300">+</span> →{" "}
                  <span className="text-gray-300">Scan from QR Code</span>
                </p>
              </>
            ) : (
              <p className="text-sm text-gray-400 text-center py-4">
                No VPN config available. Deploy a server first.
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default DashboardPage;
