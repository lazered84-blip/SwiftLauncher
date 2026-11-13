// Mode démonstration : simule le backend Tauri dans un simple navigateur (npm run dev).
// Données fictives en mémoire. Aucun lancement réel, aucune connexion réelle.

import type { UnlistenFn } from "@tauri-apps/api/event";
import type { Account, InstallProgress, Instance, JavaInstall, VersionEntry } from "./api";
import { validateInstanceName } from "./validation";

const DEMO_VERSIONS: VersionEntry[] = [
  { id: "1.21.4", type: "release" },
  { id: "1.21.1", type: "release" },
  { id: "1.20.1", type: "release" },
  { id: "1.16.5", type: "release" },
  { id: "1.8.9", type: "release" },
  { id: "24w14a", type: "snapshot" },
];

const DEMO_JAVA: JavaInstall[] = [
  { path: "C:\\Program Files\\Java\\jdk-21\\bin\\java.exe (fictif)", version: "21.0.2", major: 21 },
];

export const DEMO_LAUNCH_MESSAGE =
  "Mode démonstration : aucun jeu n'est lancé. Utilisez l'application Tauri (npm run tauri dev) pour le vrai lancement.";

/** `delayMs` : durée de chaque étape de progression simulée (0 dans les tests). */
export function createMockApi(delayMs = 350) {
  let nextId = 1;
  let instances: Instance[] = [];
  let account: Account | null = null;
  const listeners = new Set<(p: InstallProgress) => void>();
  const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

  return {
    listInstances: async (): Promise<Instance[]> => [...instances],

    createInstance: async (name: string, mcVersion: string): Promise<Instance> => {
      const check = validateInstanceName(name);
      if (!check.ok) throw check.reason; // chaîne, comme les erreurs renvoyées par Tauri
      const trimmed = name.trim();
      if (instances.some((i) => i.name === trimmed)) throw "Une instance porte déjà ce nom.";
      const created: Instance = {
        id: nextId++,
        name: trimmed,
        mcVersion,
        loader: "vanilla",
        ramMb: 4096,
        createdAt: new Date().toISOString(),
      };
      instances = [...instances, created];
      return created;
    },

    deleteInstance: async (id: number): Promise<void> => {
      if (!instances.some((i) => i.id === id)) throw "Instance introuvable.";
      instances = instances.filter((i) => i.id !== id);
    },

    listVersions: async (): Promise<VersionEntry[]> => [...DEMO_VERSIONS],

    detectJava: async (): Promise<JavaInstall[]> => [...DEMO_JAVA],

    loginMicrosoft: async (): Promise<Account> => {
      await sleep(delayMs);
      account = { uuid: "00000000000000000000000000000000", name: "Joueur démo" };
      return account;
    },

    currentAccount: async (): Promise<Account | null> => account,

    signOut: async (): Promise<void> => {
      account = null;
    },

    /** Simule la progression d'installation, puis refuse de lancer : rien n'est réellement démarré. */
    launchInstance: async (id: number): Promise<number> => {
      if (!instances.some((i) => i.id === id)) throw "Instance introuvable.";
      const stages: InstallProgress["stage"][] = ["java", "client", "assets", "libraries"];
      for (const stage of stages) {
        for (const done of [0, 1]) {
          listeners.forEach((fn) => fn({ stage, done, total: 1 }));
          await sleep(delayMs);
        }
      }
      throw DEMO_LAUNCH_MESSAGE;
    },

    onInstallProgress: (handler: (p: InstallProgress) => void): Promise<UnlistenFn> => {
      listeners.add(handler);
      return Promise.resolve(() => {
        listeners.delete(handler);
      });
    },
  };
}
