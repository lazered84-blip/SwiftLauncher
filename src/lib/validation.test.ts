import { describe, expect, it } from "vitest";
import { validateInstanceName } from "./validation";

describe("validateInstanceName", () => {
  it("accepte un nom simple", () => {
    expect(validateInstanceName("Survival 1.21")).toEqual({ ok: true });
  });

  it("refuse un nom vide ou composé d'espaces", () => {
    expect(validateInstanceName("   ").ok).toBe(false);
  });

  it("refuse un nom trop long", () => {
    expect(validateInstanceName("a".repeat(41)).ok).toBe(false);
  });

  it("refuse les séparateurs de chemin (anti traversée de répertoire)", () => {
    expect(validateInstanceName("../etc").ok).toBe(false);
    expect(validateInstanceName("a/b").ok).toBe(false);
    expect(validateInstanceName("a\\b").ok).toBe(false);
  });

  it("refuse . et ..", () => {
    expect(validateInstanceName(".").ok).toBe(false);
    expect(validateInstanceName("..").ok).toBe(false);
  });
});
