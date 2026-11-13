import { useEffect, useState } from "react";
import { Play } from "lucide-react";
import { api, Account, Instance, InstallProgress } from "../lib/api";

interface Props {
  onNavigate: (page: "instances" | "account") => void;
}

const STAGE_LABELS: Record<InstallProgress["stage"], string> = {
  java: "Java",
  client: "Client",
  index: "Index",
  assets: "Ressources",
  libraries: "Bibliothèques",
};

export default function Dashboard({ onNavigate }: Props) {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [account, setAccount] = useState<Account | null>(null);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [launching, setLaunching] = useState(false);
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.listInstances(), api.currentAccount()])
      .then(([list, acc]) => {
        setInstances(list);
        setAccount(acc);
        if (list.length > 0) setSelectedId(list[0].id);
      })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    const unlisten = api.onInstallProgress(setProgress);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const onPlay = async () => {
    if (selectedId === null) return;
    setLaunching(true);
    setError(null);
    setStatus("Préparation du lancement…");
    setProgress(null);
    try {
      const pid = await api.launchInstance(selectedId);
      setStatus(`Minecraft est lancé (processus ${pid}).`);
    } catch (e) {
      setStatus(null);
      setError(String(e));
    } finally {
      setLaunching(false);
    }
  };

  const playDisabled = !account || selectedId === null || launching;
  const playHint = !account
    ? "Connectez un compte pour jouer"
    : selectedId === null
      ? "Créez une instance"
      : "Lancer Minecraft";

  return (
    <>
      <h1 className="page-title">Accueil</h1>
      {error && <div className="alert alert--error" role="alert">{error}</div>}

      <section className="card" aria-labelledby="quick-play">
        <h2 id="quick-play">Lancement rapide</h2>
        <div className="row">
          {instances.length > 0 && (
            <select
              className="input"
              aria-label="Instance à lancer"
              value={selectedId ?? ""}
              onChange={(e) => setSelectedId(Number(e.target.value))}
              disabled={launching}
            >
              {instances.map((i) => (
                <option key={i.id} value={i.id}>
                  {i.name} ({i.mcVersion})
                </option>
              ))}
            </select>
          )}
          <button className="btn" disabled={playDisabled} title={playHint} onClick={onPlay}>
            <Play size={16} aria-hidden="true" />
            {launching ? "Lancement…" : "Jouer"}
          </button>
          {!account && (
            <button className="btn btn--ghost" onClick={() => onNavigate("account")}>
              Se connecter
            </button>
          )}
          {instances.length === 0 && (
            <button className="btn btn--ghost" onClick={() => onNavigate("instances")}>
              Créer une instance
            </button>
          )}
        </div>

        {progress && launching && (
          <p className="muted" aria-live="polite">
            {STAGE_LABELS[progress.stage]} : {progress.done} / {progress.total}
          </p>
        )}
        {status && <p className="muted" aria-live="polite">{status}</p>}
        <p className="muted">
          {account ? `Connecté en tant que ${account.name}.` : "Aucun compte connecté."}
        </p>
      </section>

      <section className="card" aria-labelledby="recent">
        <h2 id="recent">Instances</h2>
        {instances.length === 0 ? (
          <p className="muted">Aucune instance pour le moment.</p>
        ) : (
          <ul className="list">
            {instances.map((i) => (
              <li className="list__item" key={i.id}>
                <span>
                  {i.name}
                  <span className="badge">{i.mcVersion}</span>
                </span>
                <span className="muted">{i.loader}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}
