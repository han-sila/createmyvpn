import type { DeploymentStatus, VpnConnectionStatus } from "../lib/types";

interface Props {
  deploymentStatus: DeploymentStatus;
  vpnStatus: VpnConnectionStatus;
}

function StatusBadge({ deploymentStatus, vpnStatus }: Props) {
  let color = "bg-gray-600";
  let text = "Not Deployed";
  let pulse = false;

  if (deploymentStatus === "deployed") {
    if (vpnStatus === "connected") {
      color = "bg-green-500";
      text = "Connected";
      pulse = true;
    } else if (vpnStatus === "connecting" || vpnStatus === "disconnecting") {
      color = "bg-yellow-500";
      text = vpnStatus === "connecting" ? "Connecting..." : "Disconnecting...";
      pulse = true;
    } else {
      color = "bg-primary-400";
      text = "Deployed (Disconnected)";
    }
  } else if (deploymentStatus === "deploying") {
    color = "bg-yellow-500";
    text = "Deploying...";
    pulse = true;
  } else if (deploymentStatus === "destroying") {
    color = "bg-red-500";
    text = "Destroying...";
    pulse = true;
  } else if (deploymentStatus === "failed") {
    color = "bg-red-500";
    text = "Failed";
  }

  return (
    <span className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium text-white bg-gray-800">
      <span className={`w-2 h-2 rounded-full ${color} ${pulse ? "animate-pulse" : ""}`} />
      {text}
    </span>
  );
}

export default StatusBadge;
