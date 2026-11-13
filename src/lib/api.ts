import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { createMockApi } from "./mock";

/** Types miroir des structures renvoyées par le backend Rust (src-tauri/src/commands.rs). */

export interface Instance {
  id: number;
  name: string;
  mcVersion: string;
  loader: string;
  ramMb: number;
  createdAt: string;
}

export interface Account {
  uuid: string;
  name: string;
}

export interface JavaInstall {
  path: string;
  version: string;
  major: number;
}

export interface VersionEntry {
  id: string;
  type: string;
}

export interface InstallProgress {
  stage: "java" | "client" | "index" | "assets" | "libraries";
  done: number;
  total: number;
}

const tauriApi = {
  listInstances: () => invoke<Instance[]>("list_instances"),

  createInstance: (name: string, mcVersion: string) =>
    invoke<Instance>("create_instance", { name, mcVersion }),

  deleteInstance: (id: number) => invoke<void>("delete_instance", { id }),

  listVersions: () => invoke<VersionEntry[]>("list_versions"),

  detectJava: () => invoke<JavaInstall[]>("detect_java"),

  loginMicrosoft: () => invoke<Account>("login_microsoft"),

  currentAccount: () => invoke<Account | null>("current_account"),

  signOut: () => invoke<void>("sign_out"),

  /** Installe si besoin et lance Minecraft. Renvoie le PID du processus Java. */
  launchInstance: (id: number) => invoke<number>("launch_instance", { id }),

  /** Écoute la progression d'installation. Renvoie la fonction de désabonnement. */
  onInstallProgress: (handler: (p: InstallProgress) => void): Promise<UnlistenFn> =>
    listen<InstallProgress>("install-progress", (event) => handler(event.payload)),
};

export type Api = typeof tauriApi;

/** Vrai dans l'application Tauri, faux dans un simple navigateur (npm run dev). */
const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Mode démonstration : données fictives, aucun lancement réel. */
export const isDemoMode = !inTauri;

export const api: Api = inTauri ? tauriApi : createMockApi();
