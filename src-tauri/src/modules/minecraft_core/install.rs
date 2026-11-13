//! Installation d'une version Vanilla : client, index et objets d'assets, bibliothèques, natives.
//! Chaque fichier est vérifié par hash (téléchargement) ; les fichiers déjà présents
//! avec la bonne taille sont conservés. Les chemins venant du JSON passent par `security::safe_join`.

use crate::error::{LauncherError, Result};
use crate::modules::download_manager::download_verified;
use crate::modules::minecraft_core::version::{is_x86_32, os_name, select_libraries, LibraryArtifact, VersionDetail};
use crate::modules::minecraft_core::fetch_manifest;
use crate::modules::security;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

const ASSET_BASE_URL: &str = "https://resources.download.minecraft.net";

/// Progression émise vers l'interface (événement `install-progress`).
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub stage: &'static str,
    pub done: usize,
    pub total: usize,
}

pub type ProgressFn = dyn Fn(Progress) + Send + Sync;

/// Arborescence des données du launcher. Tout chemin de jeu passe par ce type.
#[derive(Debug, Clone)]
pub struct Paths {
    root: PathBuf,
}

impl Paths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn libraries_dir(&self) -> PathBuf {
        self.root.join("libraries")
    }

    pub fn runtimes_dir(&self) -> PathBuf {
        self.root.join("runtimes")
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    pub fn version_dir(&self, id: &str) -> PathBuf {
        self.root.join("versions").join(id)
    }

    pub fn version_json(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(format!("{id}.json"))
    }

    pub fn client_jar(&self, id: &str) -> PathBuf {
        self.version_dir(id).join(format!("{id}.jar"))
    }

    pub fn natives_dir(&self, id: &str) -> PathBuf {
        self.version_dir(id).join("natives")
    }

    pub fn asset_index(&self, id: &str) -> PathBuf {
        self.assets_dir().join("indexes").join(format!("{id}.json"))
    }

    /// Un objet d'asset est rangé par les deux premiers caractères de son hash.
    pub fn asset_object(&self, hash: &str) -> PathBuf {
        self.assets_dir().join("objects").join(&hash[..2]).join(hash)
    }

    pub fn instance_dir(&self, id: i64) -> PathBuf {
        self.root.join("instances").join(id.to_string())
    }
}

/// Un fichier à télécharger vers `dest`.
#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub url: String,
    pub dest: PathBuf,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
struct AssetObject {
    hash: String,
    size: u64,
}

pub fn asset_url(hash: &str) -> String {
    format!("{ASSET_BASE_URL}/{}/{hash}", &hash[..2])
}

fn validate_hash(hash: &str) -> Result<()> {
    if hash.len() == 40 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(LauncherError::Security(format!("hash d'asset invalide : {hash}")))
    }
}

/// Transforme l'index d'assets en liste de téléchargements (hash validés).
pub fn asset_items(index_json: &[u8], paths: &Paths) -> Result<Vec<DownloadItem>> {
    let index: AssetIndex = serde_json::from_slice(index_json)?;
    let mut items = Vec::with_capacity(index.objects.len());
    for object in index.objects.values() {
        validate_hash(&object.hash)?;
        items.push(DownloadItem {
            url: asset_url(&object.hash),
            dest: paths.asset_object(&object.hash),
            sha1: object.hash.clone(),
            size: object.size,
        });
    }
    Ok(items)
}

fn is_present(item: &DownloadItem) -> bool {
    item.dest.metadata().map(|m| m.len() == item.size).unwrap_or(false)
}

/// Télécharge en parallèle (borné par `concurrency`), en ignorant les fichiers déjà présents.
pub async fn download_items(
    http: &reqwest::Client,
    items: Vec<DownloadItem>,
    concurrency: usize,
    stage: &'static str,
    progress: &ProgressFn,
) -> Result<()> {
    let total = items.len();
    let pending: Vec<DownloadItem> = items.into_iter().filter(|i| !is_present(i)).collect();
    let skipped = total - pending.len();
    progress(Progress { stage, done: skipped, total });

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut set = JoinSet::new();
    for item in pending {
        let permit = semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| LauncherError::Invalid(format!("file de téléchargement fermée : {e}")))?;
        let http = http.clone();
        set.spawn(async move {
            let _permit = permit;
            download_verified(&http, &item.url, &item.dest, &item.sha1).await
        });
    }

    let mut done = skipped;
    while let Some(joined) = set.join_next().await {
        joined.map_err(|e| LauncherError::Invalid(format!("téléchargement interrompu : {e}")))??;
        done += 1;
        progress(Progress { stage, done, total });
    }
    Ok(())
}

/// Extrait une archive native (zip) dans `dest`, en ignorant les exclusions et les chemins dangereux.
pub fn extract_natives(archive_path: &Path, dest: &Path, exclude: &[String]) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| LauncherError::Invalid(format!("archive native illisible : {e}")))?;
    std::fs::create_dir_all(dest)?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| LauncherError::Invalid(format!("entrée d'archive illisible : {e}")))?;
        // enclosed_name refuse les chemins absolus et les `..`.
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        if entry.is_dir() || relative.starts_with("META-INF") {
            continue;
        }
        let name = relative.to_string_lossy().into_owned();
        if exclude.iter().any(|prefix| name.starts_with(prefix.as_str())) {
            continue;
        }
        let out = dest.join(&relative);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut out_file = std::fs::File::create(&out)?;
        std::io::copy(&mut entry, &mut out_file)?;
    }
    Ok(())
}

fn library_dest(paths: &Paths, lib: &LibraryArtifact) -> Result<PathBuf> {
    security::safe_join(&paths.libraries_dir(), &lib.relative_path)
}

/// Installe (ou complète) une version. Renvoie son descripteur pour le lancement.
pub async fn ensure_installed(
    http: &reqwest::Client,
    paths: &Paths,
    version_id: &str,
    progress: &ProgressFn,
) -> Result<VersionDetail> {
    security::validate_mc_version(version_id)?;

    let json_path = paths.version_json(version_id);
    if !json_path.exists() {
        let manifest = fetch_manifest(http).await?;
        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| LauncherError::Invalid(format!("version inconnue : {version_id}")))?;
        download_verified(http, &entry.url, &json_path, &entry.sha1).await?;
    }
    let detail: VersionDetail = serde_json::from_slice(&tokio::fs::read(&json_path).await?)?;

    // 1. Client et index d'assets.
    let client = &detail.downloads.client;
    download_items(
        http,
        vec![DownloadItem {
            url: client.url.clone(),
            dest: paths.client_jar(version_id),
            sha1: client.sha1.clone(),
            size: client.size,
        }],
        1,
        "client",
        progress,
    )
    .await?;

    security::validate_mc_version(&detail.asset_index.id)?;
    let index_path = paths.asset_index(&detail.asset_index.id);
    download_items(
        http,
        vec![DownloadItem {
            url: detail.asset_index.url.clone(),
            dest: index_path.clone(),
            sha1: detail.asset_index.sha1.clone(),
            size: detail.asset_index.size,
        }],
        1,
        "index",
        progress,
    )
    .await?;

    // 2. Objets d'assets.
    let index_bytes = tokio::fs::read(&index_path).await?;
    let objects = asset_items(&index_bytes, paths)?;
    download_items(http, objects, 16, "assets", progress).await?;

    // 3. Bibliothèques (classpath et archives natives).
    let libs = select_libraries(&detail, os_name(), is_x86_32());
    let mut lib_items = Vec::with_capacity(libs.len());
    let mut to_extract: Vec<(PathBuf, Vec<String>)> = Vec::new();
    for lib in &libs {
        let dest = library_dest(paths, lib)?;
        if let Some(exclude) = &lib.extract {
            to_extract.push((dest.clone(), exclude.clone()));
        }
        lib_items.push(DownloadItem {
            url: lib.url.clone(),
            dest,
            sha1: lib.sha1.clone(),
            size: lib.size,
        });
    }
    download_items(http, lib_items, 8, "libraries", progress).await?;

    // 4. Extraction des natives (blocage disque : hors du runtime async).
    let natives_dir = paths.natives_dir(version_id);
    tokio::task::spawn_blocking(move || -> Result<()> {
        for (archive, exclude) in to_extract {
            extract_natives(&archive, &natives_dir, &exclude)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| LauncherError::Invalid(format!("extraction interrompue : {e}")))??;

    Ok(detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> Paths {
        Paths::new(PathBuf::from("data"))
    }

    #[test]
    fn asset_url_uses_two_char_prefix() {
        let hash = "abcdef0123456789abcdef0123456789abcdef01";
        assert_eq!(
            asset_url(hash),
            "https://resources.download.minecraft.net/ab/abcdef0123456789abcdef0123456789abcdef01"
        );
    }

    #[test]
    fn asset_items_are_built_from_index() {
        let json = br#"{"objects":{"minecraft/x.png":{"hash":"abcdef0123456789abcdef0123456789abcdef01","size":3}}}"#;
        let items = asset_items(json, &paths()).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].dest.ends_with("ab/abcdef0123456789abcdef0123456789abcdef01"));
        assert_eq!(items[0].size, 3);
    }

    #[test]
    fn asset_index_rejects_hash_that_could_escape_directory() {
        let json = br#"{"objects":{"x":{"hash":"../../../../etc/passwd/../../../../../xxxx","size":1}}}"#;
        assert!(asset_items(json, &paths()).is_err());
    }

    #[test]
    fn version_paths_are_under_root() {
        let p = paths();
        assert!(p.client_jar("1.21.1").starts_with("data"));
        assert!(p.instance_dir(3).ends_with("instances/3") || p.instance_dir(3).ends_with("instances\\3"));
    }
}
