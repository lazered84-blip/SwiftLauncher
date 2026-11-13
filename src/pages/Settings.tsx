import { useEffect, useState } from "react";
import { api, JavaInstall } from "../lib/api";

export default function Settings() {
  const [javas, setJavas] = useState<JavaInstall[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.detectJava().then(setJavas).catch((e) => setError(String(e)));
  }, []);

  return (
    <>
      <h1 className="page-title">Réglages</h1>
      {error && <div className="alert alert--error" role="alert">{error}</div>}

      <section className="card" aria-labelledby="java-title">
        <h2 id="java-title">Java détecté</h2>
        {javas.length === 0 ? (
          <p className="muted">
            Aucune installation Java trouvée. Installez un JDK 21 (Minecraft récent) ou 17 (versions anciennes).
          </p>
        ) : (
          <ul className="list">
            {javas.map((j) => (
              <li className="list__item" key={j.path}>
                <span>Java {j.version}<span className="badge">major {j.major}</span></span>
                <span className="muted">{j.path}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="card" aria-labelledby="theme-title">
        <h2 id="theme-title">Apparence</h2>
        <p className="muted">Thème clair (par défaut). Le thème sombre arrivera dans une version ultérieure.</p>
      </section>
    </>
  );
}
