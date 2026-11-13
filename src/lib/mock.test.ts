import { describe, expect, it } from "vitest";
import { createMockApi, DEMO_LAUNCH_MESSAGE } from "./mock";
import type { InstallProgress } from "./api";

describe("mode démonstration", () => {
  it("crée, liste et supprime des instances", async () => {
    const api = createMockApi(0);
    const created = await api.createInstance("  Survie  ", "1.21.1");
    expect(created.name).toBe("Survie");
    expect(await api.listInstances()).toHaveLength(1);
    await api.deleteInstance(created.id);
    expect(await api.listInstances()).toHaveLength(0);
  });

  it("applique les mêmes règles de nom que le vrai backend", async () => {
    const api = createMockApi(0);
    await expect(api.createInstance("../evil", "1.21.1")).rejects.toBeTruthy();
    await api.createInstance("A", "1.21.1");
    await expect(api.createInstance("A", "1.21.1")).rejects.toBe("Une instance porte déjà ce nom.");
  });

  it("ne lance jamais réellement le jeu", async () => {
    const api = createMockApi(0);
    const inst = await api.createInstance("Test", "1.21.1");
    const seen: InstallProgress["stage"][] = [];
    const off = await api.onInstallProgress((p) => seen.push(p.stage));
    await expect(api.launchInstance(inst.id)).rejects.toBe(DEMO_LAUNCH_MESSAGE);
    off();
    expect(seen).toContain("assets");
  });

  it("gère la connexion fictive et la déconnexion", async () => {
    const api = createMockApi(0);
    expect(await api.currentAccount()).toBeNull();
    expect((await api.loginMicrosoft()).name).toBe("Joueur démo");
    await api.signOut();
    expect(await api.currentAccount()).toBeNull();
  });
});
