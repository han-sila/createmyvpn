import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import Layout from "./components/Layout";
import SetupPage from "./pages/SetupPage";
import DashboardPage from "./pages/DashboardPage";
import DeployPage from "./pages/DeployPage";
import SettingsPage from "./pages/SettingsPage";
import LogsPage from "./pages/LogsPage";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="setup" element={<SetupPage />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="deploy" element={<DeployPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="logs" element={<LogsPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
