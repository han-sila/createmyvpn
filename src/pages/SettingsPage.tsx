import { useState, useEffect } from "react";
import { Save, Trash2, Download, CheckCircle2 } from "lucide-react";
import RegionSelector from "../components/RegionSelector";
import {
  getSettings,
  updateSettings,
  loadCredentials,
  deleteCredentials,
  loadDoCredentials,
  deleteDoCredentials,
  exportClientConfig,
} from "../lib/tauri";

function SettingsPage() {
  const [region, setRegion] = useState("us-east-1");
  const [instanceType, setInstanceType] = useState("t2.micro");
  const [wgPort, setWgPort] = useState(51820);
  const [hasCreds, setHasCreds] = useState(false);
  const [hasDoToken, setHasDoToken] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState("");
  const [exportMsg, setExportMsg] = useState("");

  useEffect(() => {
    getSettings().then((s) => {
      setRegion(s.region);
      setInstanceType(s.instance_type);
      setWgPort(s.wireguard_port);
    });
    loadCredentials().then((creds) => setHasCreds(!!creds));
    loadDoCredentials().then((creds) => setHasDoToken(!!creds));
  }, []);

  const handleSave = async () => {
    try {
      await updateSettings(region, instanceType, wgPort);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDeleteCreds = async () => {
    try {
      await deleteCredentials();
      setHasCreds(false);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleDeleteDoToken = async () => {
    try {
      await deleteDoCredentials();
      setHasDoToken(false);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleExportConfig = async () => {
    setExportMsg("");
    try {
      const savedPath = await exportClientConfig();
      setExportMsg(`Saved: ${savedPath}`);
      setTimeout(() => setExportMsg(""), 6000);
    } catch (err) {
      setExportMsg(`Error: ${String(err)}`);
    }
  };

  return (
    <div className="max-w-lg space-y-6">
      <h2 className="text-2xl font-bold text-white">Settings</h2>

      {/* Deployment settings */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-4">
        <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">
          Deployment
        </h3>

        <div>
          <label className="block text-sm font-medium text-gray-400 mb-1.5">
            Default Region
          </label>
          <RegionSelector value={region} onChange={setRegion} />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-400 mb-1.5">
            Instance Type
          </label>
          <select
            value={instanceType}
            onChange={(e) => setInstanceType(e.target.value)}
            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            <option value="t2.micro">t2.micro (1 vCPU, 1GB - Free Tier)</option>
            <option value="t3.micro">t3.micro (2 vCPU, 1GB)</option>
            <option value="t3.small">t3.small (2 vCPU, 2GB)</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-400 mb-1.5">
            WireGuard Port
          </label>
          <input
            type="number"
            value={wgPort}
            onChange={(e) => setWgPort(Number(e.target.value))}
            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>

        <button
          onClick={handleSave}
          className="flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors"
        >
          {saved ? (
            <CheckCircle2 className="w-4 h-4 text-green-400" />
          ) : (
            <Save className="w-4 h-4" />
          )}
          {saved ? "Saved!" : "Save Settings"}
        </button>
      </div>

      {/* AWS Credentials */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-4">
        <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">
          AWS Credentials
        </h3>

        <p className="text-sm text-gray-400">
          {hasCreds
            ? "AWS credentials are saved locally at ~/.createmyvpn/credentials.json"
            : "No credentials saved. Go to Setup to add them."}
        </p>

        {hasCreds && (
          <button
            onClick={handleDeleteCreds}
            className="flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            Delete AWS Credentials
          </button>
        )}
      </div>

      {/* DigitalOcean Credentials */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-4">
        <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">
          DigitalOcean
        </h3>

        <p className="text-sm text-gray-400">
          {hasDoToken
            ? "DO API token is saved locally at ~/.createmyvpn/do_credentials.json"
            : "No DigitalOcean token saved. Go to Setup to add one."}
        </p>

        {hasDoToken && (
          <button
            onClick={handleDeleteDoToken}
            className="flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-red-400 bg-red-500/10 hover:bg-red-500/20 rounded-lg transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            Delete DO Token
          </button>
        )}
      </div>

      {/* Export */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-4">
        <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">
          Export
        </h3>

        <button
          onClick={handleExportConfig}
          className="flex items-center gap-2 px-4 py-2.5 text-sm font-medium text-gray-300 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors"
        >
          <Download className="w-4 h-4" />
          Export WireGuard Config
        </button>

        {exportMsg && (
          <p className="text-xs text-gray-400 font-mono truncate">{exportMsg}</p>
        )}

        <p className="text-xs text-gray-500">
          Download the .conf file to use with the WireGuard app on other devices.
        </p>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3">
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}
    </div>
  );
}

export default SettingsPage;
