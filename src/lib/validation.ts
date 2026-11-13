export const INSTANCE_NAME_MAX = 40;

/** Caractères interdits dans un nom d'instance (utilisé aussi comme nom de dossier côté Rust). */
const FORBIDDEN = /[\\/:*?"<>|\u0000-\u001f]/;

export type ValidationResult = { ok: true } | { ok: false; reason: string };

export function validateInstanceName(raw: string): ValidationResult {
  const name = raw.trim();
  if (name.length === 0) {
    return { ok: false, reason: "Le nom ne peut pas être vide." };
  }
  if (name.length > INSTANCE_NAME_MAX) {
    return { ok: false, reason: `Le nom ne doit pas dépasser ${INSTANCE_NAME_MAX} caractères.` };
  }
  if (FORBIDDEN.test(name)) {
    return { ok: false, reason: "Le nom contient des caractères non autorisés." };
  }
  if (name === "." || name === "..") {
    return { ok: false, reason: "Ce nom n'est pas autorisé." };
  }
  return { ok: true };
}
