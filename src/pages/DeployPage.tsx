import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import {
  Rocket,
  Loader2,
  CheckCircle2,
  XCircle,
  Cloud,
  Server,
  Droplets,
} from "lucide-react";
import RegionSelector from "../components/RegionSelector";
import ProgressStepper from "../components/ProgressStepper";
import type { ProgressEvent, DeploymentState } from "../lib/types";
import { DO_REGIONS } from "../lib/types";
import {
  deployVpn,
  deployByoVps,
  deployDo,
  getSettings,
  loadCredentials,
  loadDoCredentials,
  getDeploymentState,
} from "../lib/tauri";

const AUTO_DESTROY_OPTIONS = [
  { value: "", label: "Never (manual destroy only)" },
  { value: "1", label: "After 1 hour" },
  { value: "2", label: "After 2 hours" },
  { value: "4", label: "After 4 hours" },
  { value: "8", label: "After 8 hours" },
  { value: "24", label: "After 24 hours" },
];

const DO_SIZES = [
  { value: "s-1vcpu-512mb-10gb", label: "Basic — 512MB RAM (~$4/mo)" },
  { value: "s-1vcpu-1gb", label: "Standard — 1GB RAM (~$6/mo)" },
  { value: "s-2vcpu-2gb", label: "Performance — 2GB RAM (~$18/mo)" },
];

function DeployPage() {
  const navigate = useNavigate();

  // Mode
  const [mode, setMode] = useState<"aws" | "do" | "byo">("aws");

  // AWS form
  const [region, setRegion] = useState("us-east-1");
  const [hasCreds, setHasCreds] = useState(false);

  // DO form
  const [doRegion, setDoRegion] = useState("nyc1");
  const [doSize, setDoSize] = useState("s-1vcpu-512mb-10gb");
  const [hasDoCredentials, setHasDoCredentials] = useState(false);

  // BYO form
  const [serverIp, setServerIp] = useState("");
  const [sshKey, setSshKey] = useState("");
  const [sshUser, setSshUser] = useState("ubuntu");
  const [sshPort, setSshPort] = useState(22);

  // Shared
  const [autoDestroyHours, setAutoDestroyHours] = useState<number | undefined>(
    undefined
  );
  const [deploying, setDeploying] = useState(false);
  const [done, setDone] = useState(false);
  const [error, setError] = useState("");
  const [steps, setSteps] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [attempted, setAttempted] = useState(false);
  const [existingDeployment, setExistingDeployment] =
    useState<DeploymentState | null>(null);

  useEffect(() => {
    loadCredentials().then((creds) => setHasCreds(!!creds));
    loadDoCredentials().then((creds) => setHasDoCredentials(!!creds));
    getSettings().then((s) => setRegion(s.region));
  }, []);

  useEffect(() => {
    const checkState = () => {
      getDeploymentState()
        .then((s) => {
          if (
            s.status === "deployed" ||
            s.status === "deploying" ||
            s.status === "destroying"
          ) {
            setExistingDeployment(s);
          } else {
            setExistingDeployment(null);
          }
        })
        .catch(() => setExistingDeployment(null));
    };
    checkState();
    const interval = setInterval(checkState, 3000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const unlisten = listen<ProgressEvent>("deploy-progress", (event) => {
      const progress = event.payload;
      setCurrentStep(progress.step);
      setSteps((prev) => {
        const updated = [...prev];
        if (updated.length < progress.step) {
          updated.push(progress);
        } else {
          updated[progress.step - 1] = progress;
        }
        return updated;
      });
      if (progress.status === "done") {
        setDone(true);
        setDeploying(false);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleReset = () => {
    setAttempted(false);
    setError("");
    setSteps([]);
    setDone(false);
    setCurrentStep(0);
  };

  const handleSetMode = (m: "aws" | "do" | "byo") => {
    setMode(m);
    handleReset();
  };

  const startDeploy = async (deployFn: () => Promise<unknown>) => {
    try {
      const currentState = await getDeploymentState();
      if (
        currentState.status === "deployed" ||
        currentState.status === "deploying" ||
        currentState.status === "destroying"
      ) {
        setExistingDeployment(currentState);
        return;
      }
    } catch {
      // No state file = not deployed, safe to proceed
    }
    setDeploying(true);
    setAttempted(true);
    setError("");
    setSteps([]);
    setDone(false);
    setCurrentStep(0);
    try {
      await deployFn();
    } catch (err) {
      setError(String(err));
      setDeploying(false);
    }
  };

  const handleAwsDeploy = () =>
    startDeploy(() => deployVpn(region, autoDestroyHours));

  const handleDoDeploy = () =>
    startDeploy(() => deployDo(doRegion, doSize, autoDestroyHours));

  const handleByoDeploy = () =>
    startDeploy(() =>
      deployByoVps(serverIp, sshKey, sshUser, sshPort, autoDestroyHours)
    );

  // Block if server already deployed/deploying/destroying
  if (existingDeployment) {
    const isDeploying = existingDeployment.status === "deploying";
    const isDestroying = existingDeployment.status === "destroying";
    return (
      <div className="flex flex-col items-center justify-center h-full text-center">
        <div
          className={`w-20 h-20 rounded-full flex items-center justify-center mb-6 ${
            isDestroying ? "bg-yellow-500/20" : "bg-green-500/20"
          }`}
        >
          {isDeploying || isDestroying ? (
            <Loader2
              className={`w-10 h-10 animate-spin ${
                isDestroying ? "text-yellow-400" : "text-primary-400"
              }`}
            />
          ) : (
            <CheckCircle2 className="w-10 h-10 text-green-400" />
          )}
        </div>
        <h2 className="text-2xl font-bold text-white mb-2">
          {isDeploying
            ? "Deployment Running"
            : isDestroying
              ? "Server Being Destroyed"
              : "Server Already Deployed"}
        </h2>
        <p className="text-gray-400 text-sm mb-6 max-w-sm">
          {isDeploying
            ? "A deployment is currently running. Check the Dashboard for its status."
            : isDestroying
              ? "The server is being destroyed. Please wait before starting a new deployment."
              : "Your VPN server is already running. Go to the Dashboard to manage it."}
        </p>
        <button
          onClick={() => navigate("/dashboard")}
          className="px-6 py-3 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
        >
          Go to Dashboard
        </button>
      </div>
    );
  }

  return (
    <div className="max-w-lg">
      <h2 className="text-2xl font-bold text-white mb-2">Deploy VPN</h2>
      <p className="text-gray-400 text-sm mb-5">
        Deploy WireGuard on AWS, DigitalOcean, or install it on your own server.
      </p>

      {/* Tab switcher */}
      <div className="flex bg-gray-900 border border-gray-800 rounded-lg p-1 mb-5">
        <button
          onClick={() => handleSetMode("aws")}
          className={`flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium rounded-md transition-colors ${
            mode === "aws"
              ? "bg-primary-600/20 text-primary-400"
              : "text-gray-400 hover:text-gray-300"
          }`}
        >
          <Cloud className="w-4 h-4" />
          AWS
        </button>
        <button
          onClick={() => handleSetMode("do")}
          className={`flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium rounded-md transition-colors ${
            mode === "do"
              ? "bg-primary-600/20 text-primary-400"
              : "text-gray-400 hover:text-gray-300"
          }`}
        >
          <Droplets className="w-4 h-4" />
          DigitalOcean
        </button>
        <button
          onClick={() => handleSetMode("byo")}
          className={`flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium rounded-md transition-colors ${
            mode === "byo"
              ? "bg-primary-600/20 text-primary-400"
              : "text-gray-400 hover:text-gray-300"
          }`}
        >
          <Server className="w-4 h-4" />
          Your Own Server
        </button>
      </div>

      {/* AWS form */}
      {mode === "aws" && !attempted && (
        <>
          {!hasCreds ? (
            <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 text-center">
              <p className="text-sm text-gray-400 mb-4">
                Set up your AWS credentials before deploying.
              </p>
              <button
                onClick={() => navigate("/setup")}
                className="px-6 py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
              >
                Go to Setup
              </button>
            </div>
          ) : (
            <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-5">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1.5">
                  Deploy Region
                </label>
                <RegionSelector value={region} onChange={setRegion} />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1.5">
                  Auto-Destroy
                </label>
                <select
                  value={autoDestroyHours ?? ""}
                  onChange={(e) =>
                    setAutoDestroyHours(
                      e.target.value ? Number(e.target.value) : undefined
                    )
                  }
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  {AUTO_DESTROY_OPTIONS.map((o) => (
                    <option key={o.value} value={o.value}>
                      {o.label}
                    </option>
                  ))}
                </select>
              </div>

              <button
                onClick={handleAwsDeploy}
                className="w-full flex items-center justify-center gap-2 px-4 py-3 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
              >
                <Rocket className="w-4 h-4" />
                Deploy VPN Server
              </button>

              <p className="text-xs text-gray-500 text-center">
                Creates an EC2 instance, VPC, and related resources in your AWS
                account. ~$3–5/month while running.
              </p>
            </div>
          )}
        </>
      )}

      {/* DigitalOcean form */}
      {mode === "do" && !attempted && (
        <>
          {!hasDoCredentials ? (
            <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 text-center">
              <p className="text-sm text-gray-400 mb-4">
                Set up your DigitalOcean API token before deploying.
              </p>
              <button
                onClick={() => navigate("/setup")}
                className="px-6 py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
              >
                Go to Setup
              </button>
            </div>
          ) : (
            <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-5">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1.5">
                  Region
                </label>
                <select
                  value={doRegion}
                  onChange={(e) => setDoRegion(e.target.value)}
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  {DO_REGIONS.map((r) => (
                    <option key={r.code} value={r.code}>
                      {r.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1.5">
                  Droplet Size
                </label>
                <select
                  value={doSize}
                  onChange={(e) => setDoSize(e.target.value)}
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  {DO_SIZES.map((s) => (
                    <option key={s.value} value={s.value}>
                      {s.label}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1.5">
                  Auto-Destroy
                </label>
                <select
                  value={autoDestroyHours ?? ""}
                  onChange={(e) =>
                    setAutoDestroyHours(
                      e.target.value ? Number(e.target.value) : undefined
                    )
                  }
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  {AUTO_DESTROY_OPTIONS.map((o) => (
                    <option key={o.value} value={o.value}>
                      {o.label}
                    </option>
                  ))}
                </select>
              </div>

              <button
                onClick={handleDoDeploy}
                className="w-full flex items-center justify-center gap-2 px-4 py-3 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
              >
                <Rocket className="w-4 h-4" />
                Deploy Droplet
              </button>

              <p className="text-xs text-gray-500 text-center">
                Creates a DigitalOcean Droplet with WireGuard pre-configured.
                Billed by the hour.
              </p>
            </div>
          )}
        </>
      )}

      {/* BYO form */}
      {mode === "byo" && !attempted && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-5">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Server IP Address
            </label>
            <input
              type="text"
              value={serverIp}
              onChange={(e) => setServerIp(e.target.value)}
              placeholder="1.2.3.4"
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white placeholder-gray-500 font-mono focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1.5">
                SSH User
              </label>
              <input
                type="text"
                value={sshUser}
                onChange={(e) => setSshUser(e.target.value)}
                placeholder="ubuntu"
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1.5">
                SSH Port
              </label>
              <input
                type="number"
                value={sshPort}
                onChange={(e) => setSshPort(Number(e.target.value))}
                className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              SSH Private Key
            </label>
            <textarea
              value={sshKey}
              onChange={(e) => setSshKey(e.target.value)}
              placeholder={"-----BEGIN OPENSSH PRIVATE KEY-----\n...\n-----END OPENSSH PRIVATE KEY-----"}
              rows={6}
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-xs text-white placeholder-gray-500 font-mono focus:outline-none focus:ring-2 focus:ring-primary-500 resize-none"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              Auto-Destroy
            </label>
            <select
              value={autoDestroyHours ?? ""}
              onChange={(e) =>
                setAutoDestroyHours(
                  e.target.value ? Number(e.target.value) : undefined
                )
              }
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              {AUTO_DESTROY_OPTIONS.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </div>

          <button
            onClick={handleByoDeploy}
            disabled={!serverIp || !sshKey}
            className="w-full flex items-center justify-center gap-2 px-4 py-3 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Rocket className="w-4 h-4" />
            Install WireGuard
          </button>

          <p className="text-xs text-gray-500 text-center">
            Works with any Ubuntu 22.04/24.04 server. Hetzner (~€4/mo),
            DigitalOcean (~$5/mo), or a home server. No cloud account needed.
          </p>
        </div>
      )}

      {/* Progress view (shared for all modes) */}
      {attempted && (
        <div className="space-y-4">
          <div className="bg-gray-900 border border-gray-800 rounded-xl p-6">
            <div className="flex items-center gap-3 mb-4">
              {done ? (
                <CheckCircle2 className="w-5 h-5 text-green-400" />
              ) : error ? (
                <XCircle className="w-5 h-5 text-red-400" />
              ) : (
                <Loader2 className="w-5 h-5 text-primary-400 animate-spin" />
              )}
              <h3 className="text-lg font-semibold text-white">
                {done
                  ? "Deployment Complete!"
                  : error
                    ? "Deployment Failed"
                    : mode === "aws"
                      ? "Deploying to AWS..."
                      : mode === "do"
                        ? "Deploying to DigitalOcean..."
                        : "Configuring Server..."}
              </h3>
            </div>

            <ProgressStepper steps={steps} currentStep={currentStep} />

            {error && (
              <div className="mt-4 bg-red-500/10 border border-red-500/30 rounded-lg p-3">
                <p className="text-xs font-mono text-red-400 break-all">
                  {error}
                </p>
              </div>
            )}
          </div>

          {done && (
            <button
              onClick={() => navigate("/dashboard")}
              className="w-full px-4 py-3 text-sm font-medium text-white bg-green-600 hover:bg-green-500 rounded-lg transition-colors"
            >
              Go to Dashboard
            </button>
          )}

          {error && !deploying && (
            <button
              onClick={handleReset}
              className="w-full px-4 py-3 text-sm font-medium text-white bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
            >
              ← Try Again
            </button>
          )}
        </div>
      )}
    </div>
  );
}

export default DeployPage;
