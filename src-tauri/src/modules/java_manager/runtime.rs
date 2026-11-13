//! Téléchargement et installation d'un JRE officiel (Eclipse Temurin via l'API Adoptium).
//! Le JRE est installé dans `<données>/runtimes/java-<N>` : aucune modification du système.
//!
//! ⚠ Le format de l'API Adoptium n'a pas pu être vérifié depuis l'environnement de génération
//! (pas d'accès réseau). Les structures ci-dessous suivent la documentation publique et doivent
//! être validées par un premier téléchargement réel.

use crate::error::{LauncherError, Result};
use crate::modules::java_manager::{probe, JavaInstall};
use crate::modules::security;
use serde::Deserialize;
use std::path::{Path, PathBuf};

const API_BASE: &str = "https://api.adoptium.net/v3/assets/latest";

pub fn adoptium_os() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "mac",
        _ => "linux",
    }
}

pub fn adoptium_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86" => "x32",
        _ => "x64",
    }
}

pub fn assets_url(major: u32, os: &str, arch: &str) -> String {
    format!("{API_BASE}/{major}/hotspot?architecture={arch}&image_type=jre&os={os}&vendor=eclipse")
}

#[derive(Debug, Deserialize)]
struct Asset {
    binary: Binary,
}

#[derive(Debug, Deserialize)]
struct Binary {
    package: Package,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub link: String,
    pub name: String,
    pub checksum: String,
}

pub fn parse_package(json: &[u8]) -> Result<Package> {
    let assets: Vec<Asset> = serde_json::from_slice(json)?;
    assets
        .into_iter()
        .next()
        .map(|a| a.binary.package)
        .ok_or_else(|| LauncherError::Invalid("aucun JRE disponible pour cette plateforme".into()))
}

/// Refuse tout ce qui n'est pas HTTPS, tout nom de fichier suspect et tout hash mal formé.
pub fn validate_package(pkg: &Package) -> Result<()> {
    if !pkg.link.starts_with("https://") {
        return Err(LauncherError::Security("lien de JRE non HTTPS refusé".into()));
    }
    let name_ok = !pkg.name.contains('/')
        && !pkg.name.contains('\\')
        && (pkg.name.ends_with(".zip") || pkg.name.ends_with(".tar.gz"));
    if !name_ok {
        return Err(LauncherError::Security(format!("nom d'archive JRE refusé : {}", pkg.name)));
    }
    if pkg.checksum.len() != 64 || !pkg.checksum.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(LauncherError::Security("somme de contrôle SHA-256 invalide".into()));
    }
    Ok(())
}

/// Cherche `bin/java` dans l'arborescence (jusqu'à 4 niveaux : gère `Contents/Home` sous macOS).
pub fn find_java_exe(root: &Path) -> Option<PathBuf> {
    fn walk(dir: &Path, exe: &str, depth: u32) -> Option<PathBuf> {
        if depth == 0 {
            return None;
        }
        let direct = dir.join("bin").join(exe);
        if direct.is_file() {
            return Some(direct);
        }
        let mut subdirs: Vec<PathBuf> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        subdirs.sort();
        subdirs.iter().find_map(|p| walk(p, exe, depth - 1))
    }
    let exe = if cfg!(windows) { "java.exe" } else { "java" };
    walk(root, exe, 4)
}

/// JRE déjà installés par le launcher (bloquant : lance `java -version`).
pub fn detect_runtimes(runtimes_dir: &Path) -> Vec<JavaInstall> {
    let Ok(entries) = std::fs::read_dir(runtimes_dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|p| find_java_exe(&p))
        .filter_map(|exe| probe(&exe))
        .collect()
}

fn extract_tar_gz(bytes: &[u8], dest: &Path) -> Result<()> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    // `unpack` écarte les chemins absolus et les `..`.
    tar::Archive::new(decoder).unpack(dest)?;
    Ok(())
}

fn extract_zip(bytes: &[u8], dest: &Path) -> Result<()> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| LauncherError::Invalid(format!("archive JRE illisible : {e}")))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| LauncherError::Invalid(format!("entrée d'archive illisible : {e}")))?;
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let out = dest.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(&out)?;
        std::io::copy(&mut entry, &mut file)?;
    }
    Ok(())
}

fn existing_runtime(dest: &Path) -> Option<JavaInstall> {
    find_java_exe(dest).and_then(|exe| probe(&exe))
}

/// Garantit la présence d'un JRE de la version majeure demandée et renvoie son installation.
pub async fn ensure_runtime(http: &reqwest::Client, runtimes_dir: &Path, major: u32) -> Result<JavaInstall> {
    let dest = runtimes_dir.join(format!("java-{major}"));

    let probe_dest = dest.clone();
    if let Some(found) = tokio::task::spawn_blocking(move || existing_runtime(&probe_dest))
        .await
        .map_err(|e| LauncherError::Invalid(format!("détection interrompue : {e}")))?
    {
        return Ok(found);
    }

    let url = assets_url(major, adoptium_os(), adoptium_arch());
    let listing = http.get(&url).send().await?.error_for_status()?.bytes().await?;
    let package = parse_package(&listing)?;
    validate_package(&package)?;

    let archive = http.get(&package.link).send().await?.error_for_status()?.bytes().await?;
    security::verify_sha256(&archive, &package.checksum)?;

    let name = package.name.clone();
    tokio::task::spawn_blocking(move || -> Result<JavaInstall> {
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        std::fs::create_dir_all(&dest)?;
        if name.ends_with(".zip") {
            extract_zip(&archive, &dest)?;
        } else {
            extract_tar_gz(&archive, &dest)?;
        }
        let exe = find_java_exe(&dest)
            .ok_or_else(|| LauncherError::Invalid("JRE téléchargé, mais java est introuvable".into()))?;
        probe(&exe).ok_or_else(|| LauncherError::Invalid("le JRE téléchargé ne démarre pas".into()))
    })
    .await
    .map_err(|e| LauncherError::Invalid(format!("installation du JRE interrompue : {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTING: &str = r#"[{
        "binary": { "package": {
            "link": "https://github.com/adoptium/temurin21-binaries/releases/download/x/OpenJDK21U-jre_x64_linux.tar.gz",
            "name": "OpenJDK21U-jre_x64_linux.tar.gz",
            "checksum": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "size": 50000000 } },
        "release_name": "jdk-21.0.2+13"
    }]"#;

    #[test]
    fn builds_assets_url() {
        let url = assets_url(21, "linux", "x64");
        assert!(url.starts_with("https://api.adoptium.net/v3/assets/latest/21/hotspot"));
        assert!(url.contains("image_type=jre") && url.contains("os=linux") && url.contains("architecture=x64"));
    }

    #[test]
    fn parses_and_validates_listing() {
        let pkg = parse_package(LISTING.as_bytes()).unwrap();
        assert!(pkg.name.ends_with(".tar.gz"));
        assert!(validate_package(&pkg).is_ok());
    }

    #[test]
    fn empty_listing_is_an_error() {
        assert!(parse_package(b"[]").is_err());
    }

    #[test]
    fn rejects_unsafe_packages() {
        let good = parse_package(LISTING.as_bytes()).unwrap();

        let mut http = good.clone();
        http.link = "http://example.test/x.tar.gz".into();
        assert!(validate_package(&http).is_err());

        let mut traversal = good.clone();
        traversal.name = "../evil.tar.gz".into();
        assert!(validate_package(&traversal).is_err());

        let mut bad_hash = good;
        bad_hash.checksum = "xyz".into();
        assert!(validate_package(&bad_hash).is_err());
    }

    #[test]
    fn finds_java_inside_nested_macos_layout() {
        let root = std::env::temp_dir().join(format!("mcl-runtime-test-{}", std::process::id()));
        let exe_name = if cfg!(windows) { "java.exe" } else { "java" };
        let bin = root.join("jdk-21.jre").join("Contents").join("Home").join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join(exe_name), b"").unwrap();

        let found = find_java_exe(&root).expect("java doit être trouvé");
        assert!(found.ends_with(Path::new("bin").join(exe_name)));
        assert!(find_java_exe(&root.join("absent")).is_none());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
