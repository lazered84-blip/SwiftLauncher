//! Téléchargements vérifiés : HTTPS obligatoire, hash contrôlé avant toute écriture définitive.

use crate::error::{LauncherError, Result};
use crate::modules::security;
use std::path::Path;

/// Télécharge `url` dans `dest` si son SHA-1 correspond à `expected_sha1`.
/// Écriture dans un fichier `.part` puis renommage atomique : pas de fichier partiel accepté.
pub async fn download_verified(
    http: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
) -> Result<()> {
    if !url.starts_with("https://") {
        return Err(LauncherError::Security("URL non HTTPS refusée".into()));
    }

    let bytes = http.get(url).send().await?.error_for_status()?.bytes().await?;
    security::verify_sha1(&bytes, expected_sha1)?;

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let tmp = dest.with_extension("part");
    tokio::fs::write(&tmp, &bytes).await?;
    tokio::fs::rename(&tmp, dest).await?;
    Ok(())
}
