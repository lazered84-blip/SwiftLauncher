import { useState } from "react";
import { Home, Box, User, Settings as SettingsIcon } from "lucide-react";
import Dashboard from "./pages/Dashboard";
import Instances from "./pages/Instances";
import Account from "./pages/Account";
import Settings from "./pages/Settings";
import { Disclaimer } from "./components/Disclaimer";
import { isDemoMode } from "./lib/api";

type Page = "dashboard" | "instances" | "account" | "settings";

const NAV: { id: Page; label: string; Icon: typeof Home }[] = [
  { id: "dashboard", label: "Accueil", Icon: Home },
  { id: "instances", label: "Instances", Icon: Box },
  { id: "account", label: "Compte", Icon: User },
  { id: "settings", label: "Réglages", Icon: SettingsIcon },
];

export default function App() {
  const [page, setPage] = useState<Page>("dashboard");

  return (
    <div className="app">
      <nav className="sidebar" aria-label="Navigation principale">
        <div className="sidebar__brand">Launcher</div>
        {NAV.map(({ id, label, Icon }) => (
          <button
            key={id}
            className="nav-item"
            aria-current={page === id ? "page" : undefined}
            onClick={() => setPage(id)}
          >
            <Icon size={18} aria-hidden="true" />
            {label}
          </button>
        ))}
      </nav>

      <main className="main">
        {isDemoMode && (
          <div className="alert" role="status">
            Mode démonstration (navigateur) : données fictives, aucun jeu n'est lancé. Pour le vrai launcher,
            utilisez <code>npm run tauri dev</code>.
          </div>
        )}
        {page === "dashboard" && <Dashboard onNavigate={setPage} />}
        {page === "instances" && <Instances />}
        {page === "account" && <Account />}
        {page === "settings" && <Settings />}
        <Disclaimer />
      </main>
    </div>
  );
}
