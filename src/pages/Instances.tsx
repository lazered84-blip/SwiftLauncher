import { useEffect, useState, FormEvent } from "react";
import { Trash2 } from "lucide-react";
import { api, Instance, VersionEntry } from "../lib/api";
import { validateInstanceName } from "../lib/validation";

export default function Instances() {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [versions, setVersions] = useState<VersionEntry[]>([]);
  const [name, setName] = useState("");
  const [version, setVersion] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loadingVersions, setLoadingVersions] = useState(true);
  const [confirmId, setConfirmId] = useState<number | null>(null);

  const refresh = () => api.listInstances().then(setInstances).catch((e) => setError(String(e)));

  useEffect(() => {
    refresh();
    api
      .listVersions()
      .then((list) => {
        const releases = list.filter((v) => v.type === "release");
        setVersions(releases);
        if (releases.length > 0) setVersion(releases[0].id);
      })
      .catch((e) => setError(`Impossible de charger les versions : ${String(e)}`))
      .finally(() => setLoadingVersions(false));
  }, []);

  const onCreate = async (e: FormEvent) => {
    e.preventDefault();
    const check = validateInstanceName(name);
    if (!check.ok) {
      setError(check.reason);
      return;
    }
    try {
      await api.createInstance(name.trim(), version);
      setName("");
      setError(null);
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  };

  const onDelete = async (id: number) => {
    try {
      await api.deleteInstance(id);
      setConfirmId(null);
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <>
      <h1 className="page-title">Instances</h1>
      {error && <div className="alert alert--error" role="alert">{error}</div>}

      <section className="card" aria-labelledby="create-title">
        <h2 id="create-title">Nouvelle instance</h2>
        <form className="row" onSubmit={onCreate}>
          <label className="muted" htmlFor="inst-name">Nom</label>
          <input
            id="inst-name"
            className="input"
            value={name}
            onChange={(e) => setName(e.target.value)}
            maxLength={60}
            placeholder="Ma survie PvP"
          />
          <label className="muted" htmlFor="inst-version">Version</label>
          <select
            id="inst-version"
            className="input"
            value={version}
            onChange={(e) => setVersion(e.target.value)}
            disabled={loadingVersions || versions.length === 0}
          >
            {versions.map((v) => (
              <option key={v.id} value={v.id}>{v.id}</option>
            ))}
          </select>
          <button className="btn" type="submit" disabled={!version}>Créer</button>
        </form>
      </section>

      <section className="card" aria-labelledby="list-title">
        <h2 id="list-title">Mes instances</h2>
        {instances.length === 0 ? (
          <p className="muted">Aucune instance. Créez-en une ci-dessus.</p>
        ) : (
          <ul className="list">
            {instances.map((i) => (
              <li className="list__item" key={i.id}>
                <span>
                  {i.name}
                  <span className="badge">{i.mcVersion}</span>
                  <span className="badge">{i.loader}</span>
                </span>
                {confirmId === i.id ? (
                  <span className="row">
                    <span className="muted">Supprime aussi les mondes.</span>
                    <button className="btn" onClick={() => onDelete(i.id)}>
                      Confirmer
                    </button>
                    <button className="btn btn--ghost" onClick={() => setConfirmId(null)}>
                      Annuler
                    </button>
                  </span>
                ) : (
                  <button
                    className="btn btn--ghost"
                    onClick={() => setConfirmId(i.id)}
                    aria-label={`Supprimer l'instance ${i.name}`}
                  >
                    <Trash2 size={16} aria-hidden="true" />
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}
