import { useEffect, useState } from "react";
import { api, Account as AccountInfo } from "../lib/api";

export default function Account() {
  const [account, setAccount] = useState<AccountInfo | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.currentAccount().then(setAccount).catch((e) => setError(String(e)));
  }, []);

  const onLogin = async () => {
    setBusy(true);
    setError(null);
    try {
      setAccount(await api.loginMicrosoft());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const onSignOut = async () => {
    setError(null);
    try {
      await api.signOut();
      setAccount(null);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <>
      <h1 className="page-title">Compte</h1>
      {error && <div className="alert alert--error" role="alert">{error}</div>}

      <section className="card" aria-labelledby="acc-title">
        <h2 id="acc-title">Compte Microsoft</h2>
        {account ? (
          <div className="row" style={{ justifyContent: "space-between" }}>
            <span>
              Connecté : <strong>{account.name}</strong>
            </span>
            <button className="btn btn--ghost" onClick={onSignOut}>Se déconnecter</button>
          </div>
        ) : (
          <>
            <p className="muted">
              La connexion passe par le site officiel de Microsoft. Le launcher ne voit jamais votre mot de passe.
              Un compte possédant Minecraft Java Edition est requis.
            </p>
            <button className="btn" onClick={onLogin} disabled={busy}>
              {busy ? "En attente de la connexion…" : "Se connecter avec Microsoft"}
            </button>
          </>
        )}
      </section>
    </>
  );
}
