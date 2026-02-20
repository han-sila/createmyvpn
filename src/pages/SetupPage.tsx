import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { KeyRound, CheckCircle2, Loader2, Cloud, Droplets } from "lucide-react";
import RegionSelector from "../components/RegionSelector";
import {
  validateCredentials,
  saveCredentials,
  loadCredentials,
  updateSettings,
  validateDoCredentials,
  saveDoCredentials,
  loadDoCredentials,
} from "../lib/tauri";

function SetupPage() {
  const navigate = useNavigate();

  // Tab
  const [credTab, setCredTab] = useState<"aws" | "do">("aws");

  // AWS state
  const [accessKey, setAccessKey] = useState("");
  const [secretKey, setSecretKey] = useState("");
  const [region, setRegion] = useState("us-east-1");
  const [validating, setValidating] = useState(false);
  const [validated, setValidated] = useState(false);
  const [accountId, setAccountId] = useState("");
  const [awsError, setAwsError] = useState("");

  // DO state
  const [doToken, setDoToken] = useState("");
  const [doValidating, setDoValidating] = useState(false);
  const [doValidated, setDoValidated] = useState(false);
  const [doAccountEmail, setDoAccountEmail] = useState("");
  const [doError, setDoError] = useState("");

  useEffect(() => {
    loadCredentials().then((creds) => {
      if (creds) {
        setAccessKey(creds.access_key_id);
        setSecretKey(creds.secret_access_key);
      }
    });
    loadDoCredentials().then((creds) => {
      if (creds) {
        setDoToken(creds.api_token);
      }
    });
  }, []);

  // AWS handlers
  const handleAwsValidate = async () => {
    setValidating(true);
    setAwsError("");
    setValidated(false);
    try {
      const account = await validateCredentials(accessKey, secretKey, region);
      setAccountId(account);
      setValidated(true);
    } catch (err) {
      setAwsError(String(err));
    } finally {
      setValidating(false);
    }
  };

  const handleAwsSave = async () => {
    try {
      await saveCredentials(accessKey, secretKey);
      await updateSettings(region, "t2.micro", 51820);
      navigate("/dashboard");
    } catch (err) {
      setAwsError(String(err));
    }
  };

  // DO handlers
  const handleDoValidate = async () => {
    setDoValidating(true);
    setDoError("");
    setDoValidated(false);
    try {
      const email = await validateDoCredentials(doToken);
      setDoAccountEmail(email);
      setDoValidated(true);
    } catch (err) {
      setDoError(String(err));
    } finally {
      setDoValidating(false);
    }
  };

  const handleDoSave = async () => {
    try {
      await saveDoCredentials(doToken);
      navigate("/deploy");
    } catch (err) {
      setDoError(String(err));
    }
  };

  return (
    <div className="max-w-lg">
      <h2 className="text-2xl font-bold text-white mb-2">Setup</h2>
      <p className="text-gray-400 text-sm mb-6">
        Connect a cloud provider to deploy your VPN server.
      </p>

      {/* Provider tab switcher */}
      <div className="flex bg-gray-900 border border-gray-800 rounded-lg p-1 mb-5">
        <button
          onClick={() => setCredTab("aws")}
          className={`flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium rounded-md transition-colors ${
            credTab === "aws"
              ? "bg-primary-600/20 text-primary-400"
              : "text-gray-400 hover:text-gray-300"
          }`}
        >
          <Cloud className="w-4 h-4" />
          AWS
        </button>
        <button
          onClick={() => setCredTab("do")}
          className={`flex-1 flex items-center justify-center gap-2 py-2 text-sm font-medium rounded-md transition-colors ${
            credTab === "do"
              ? "bg-primary-600/20 text-primary-400"
              : "text-gray-400 hover:text-gray-300"
          }`}
        >
          <Droplets className="w-4 h-4" />
          DigitalOcean
        </button>
      </div>

      {/* AWS credentials form */}
      {credTab === "aws" && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-5">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              AWS Access Key ID
            </label>
            <input
              type="text"
              value={accessKey}
              onChange={(e) => {
                setAccessKey(e.target.value);
                setValidated(false);
              }}
              placeholder="AKIA..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              AWS Secret Access Key
            </label>
            <input
              type="password"
              value={secretKey}
              onChange={(e) => {
                setSecretKey(e.target.value);
                setValidated(false);
              }}
              placeholder="Your secret key..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              AWS Region
            </label>
            <RegionSelector value={region} onChange={setRegion} />
          </div>

          {awsError && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3">
              <p className="text-sm text-red-400">{awsError}</p>
            </div>
          )}

          {validated && (
            <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-3 flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-green-400" />
              <p className="text-sm text-green-400">
                Credentials valid! Account: {accountId}
              </p>
            </div>
          )}

          <div className="flex gap-3">
            <button
              onClick={handleAwsValidate}
              disabled={!accessKey || !secretKey || validating}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium text-white bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {validating ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <KeyRound className="w-4 h-4" />
              )}
              Validate
            </button>

            <button
              onClick={handleAwsSave}
              disabled={!validated}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Save & Continue
            </button>
          </div>
        </div>
      )}

      {/* DigitalOcean credentials form */}
      {credTab === "do" && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 space-y-5">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1.5">
              API Token
            </label>
            <input
              type="password"
              value={doToken}
              onChange={(e) => {
                setDoToken(e.target.value);
                setDoValidated(false);
              }}
              placeholder="dop_v1_..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white placeholder-gray-500 font-mono focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <p className="mt-1.5 text-xs text-gray-500">
              Generate a token at{" "}
              <span className="text-primary-400 font-mono">
                cloud.digitalocean.com/account/api/tokens
              </span>
              {" "}— Personal access token with Read + Write scope.
            </p>
          </div>

          {doError && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3">
              <p className="text-sm text-red-400">{doError}</p>
            </div>
          )}

          {doValidated && (
            <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-3 flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-green-400" />
              <p className="text-sm text-green-400">
                Token valid! Account: {doAccountEmail}
              </p>
            </div>
          )}

          <div className="flex gap-3">
            <button
              onClick={handleDoValidate}
              disabled={!doToken || doValidating}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium text-white bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {doValidating ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <KeyRound className="w-4 h-4" />
              )}
              Validate
            </button>

            <button
              onClick={handleDoSave}
              disabled={!doValidated}
              className="flex-1 px-4 py-2.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-500 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Save & Continue
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default SetupPage;
