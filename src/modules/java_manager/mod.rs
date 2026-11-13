//! Détection des installations Java (JAVA_HOME puis PATH).
//! Installation automatique des JRE : prévue dans une version ultérieure.

pub mod runtime;

use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstall {
    pub path: String,
    pub version: String,
    pub major: u32,
}

/// Lit la sortie de `java -version` (écrite sur stderr).
/// Exemples : `openjdk version "21.0.2" ...` → (21.0.2, 21), `java version "1.8.0_392"` → (1.8.0_392, 8).
pub fn parse_version(output: &str) -> Option<(String, u32)> {
    let start = output.find('"')? + 1;
    let end = output[start..].find('"')? + start;
    let version = output[start..end].to_string();

    let mut parts = version.split(&['.', '_', '-', '+'][..]);
    let first: u32 = parts.next()?.parse().ok()?;
    let major = if first == 1 {
        parts.next()?.parse().ok()?
    } else {
        first
    };
    Some((version, major))
}

pub(crate) fn probe(path: &Path) -> Option<JavaInstall> {
    let output = Command::new(path).arg("-version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stderr).into_owned();
    let (version, major) = parse_version(&text)?;
    Some(JavaInstall {
        path: path.to_string_lossy().into_owned(),
        version,
        major,
    })
}

/// Bloquant : à appeler depuis `spawn_blocking`.
pub fn detect() -> Vec<JavaInstall> {
    let exe = if cfg!(windows) { "java.exe" } else { "java" };
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(home) = std::env::var("JAVA_HOME") {
        candidates.push(PathBuf::from(home).join("bin").join(exe));
    }
    candidates.push(PathBuf::from(exe));

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|p| seen.insert(p.to_string_lossy().into_owned()))
        .filter_map(|p| probe(&p))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modern_openjdk_version() {
        let out = "openjdk version \"21.0.2\" 2024-01-16\nOpenJDK Runtime Environment";
        assert_eq!(parse_version(out), Some(("21.0.2".to_string(), 21)));
    }

    #[test]
    fn parses_legacy_java_8_version() {
        let out = "java version \"1.8.0_392\"";
        assert_eq!(parse_version(out), Some(("1.8.0_392".to_string(), 8)));
    }

    #[test]
    fn rejects_unknown_output() {
        assert_eq!(parse_version("aucune information"), None);
    }
}
