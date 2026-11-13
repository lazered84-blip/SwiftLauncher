//! Sécurité : validation des chemins, des noms et des hash de fichiers.
//! Toute écriture disque issue d'une source externe (manifest, modpack) doit passer ici.

use crate::error::{LauncherError, Result};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::path::{Component, Path, PathBuf};

const INSTANCE_NAME_MAX: usize = 40;
const MC_VERSION_MAX: usize = 32;
const FORBIDDEN_NAME_CHARS: &str = "\\/:*?\"<>|";

fn security_error(msg: impl Into<String>) -> LauncherError {
    LauncherError::Security(msg.into())
}

/// Joint `relative` à `base` en refusant toute traversée (`..`, racine, préfixe Windows).
pub fn safe_join(base: &Path, relative: &str) -> Result<PathBuf> {
    if relative.is_empty() {
        return Err(security_error("chemin vide"));
    }
    for component in Path::new(relative).components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(security_error(format!("chemin refusé : {relative}")));
        }
    }
    Ok(base.join(relative))
}

/// Vérifie le hash SHA-1 des données. Utilisé pour les fichiers Mojang et les mods Modrinth.
pub fn verify_sha1(data: &[u8], expected_hex: &str) -> Result<()> {
    let actual = hex::encode(Sha1::digest(data));
    if actual.eq_ignore_ascii_case(expected_hex.trim()) {
        Ok(())
    } else {
        Err(security_error(format!(
            "hash SHA-1 invalide (attendu {expected_hex}, obtenu {actual})"
        )))
    }
}

/// Vérifie le hash SHA-256 des données (JRE Adoptium).
pub fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<()> {
    let actual = hex::encode(Sha256::digest(data));
    if actual.eq_ignore_ascii_case(expected_hex.trim()) {
        Ok(())
    } else {
        Err(security_error("hash SHA-256 invalide"))
    }
}

/// Même règles que la validation côté interface (src/lib/validation.ts).
pub fn validate_instance_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(LauncherError::Invalid("Le nom ne peut pas être vide.".into()));
    }
    if trimmed.chars().count() > INSTANCE_NAME_MAX {
        return Err(LauncherError::Invalid(format!(
            "Le nom ne doit pas dépasser {INSTANCE_NAME_MAX} caractères."
        )));
    }
    if trimmed.chars().any(|c| FORBIDDEN_NAME_CHARS.contains(c) || c.is_control()) {
        return Err(LauncherError::Invalid("Le nom contient des caractères non autorisés.".into()));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(LauncherError::Invalid("Ce nom n'est pas autorisé.".into()));
    }
    Ok(())
}

/// Un identifiant de version Minecraft ne contient que lettres, chiffres, '.', '-' et '_'.
pub fn validate_mc_version(version: &str) -> Result<()> {
    let valid = !version.is_empty()
        && version.len() <= MC_VERSION_MAX
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
    if valid {
        Ok(())
    } else {
        Err(LauncherError::Invalid("Version Minecraft invalide.".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_accepts_plain_relative_paths() {
        let base = Path::new("/data");
        let joined = safe_join(base, "libraries/net/fabric/x.jar").unwrap();
        assert!(joined.starts_with(base));
    }

    #[test]
    fn safe_join_rejects_traversal_and_absolute_paths() {
        let base = Path::new("/data");
        assert!(safe_join(base, "../etc/passwd").is_err());
        assert!(safe_join(base, "a/../../b").is_err());
        assert!(safe_join(base, "/etc/passwd").is_err());
        assert!(safe_join(base, "").is_err());
    }

    #[test]
    fn verify_sha1_matches_known_digest() {
        // SHA-1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d
        let expected = "a9993e364706816aba3e25717850c26c9cd0d89d";
        assert!(verify_sha1(b"abc", expected).is_ok());
        assert!(verify_sha1(b"abd", expected).is_err());
    }

    #[test]
    fn verify_sha256_matches_known_digest() {
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify_sha256(b"abc", expected).is_ok());
        assert!(verify_sha256(b"abd", expected).is_err());
    }

    #[test]
    fn instance_name_rules() {
        assert!(validate_instance_name("Survie PvP").is_ok());
        assert!(validate_instance_name("  ").is_err());
        assert!(validate_instance_name("../x").is_err());
        assert!(validate_instance_name(&"a".repeat(41)).is_err());
    }

    #[test]
    fn mc_version_rules() {
        assert!(validate_mc_version("1.21.1").is_ok());
        assert!(validate_mc_version("24w14a").is_ok());
        assert!(validate_mc_version("1.21/../x").is_err());
    }
}
