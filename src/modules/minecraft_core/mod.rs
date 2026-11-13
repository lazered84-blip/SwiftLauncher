//! Cœur Minecraft : lecture du manifest officiel des versions.
//! Les fichiers du jeu sont téléchargés depuis les hôtes Mojang, jamais hébergés sur le VPS.

use crate::error::Result;
use serde::{Deserialize, Serialize};

pub mod install;
pub mod launch;
pub mod version;

pub const VERSION_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: String,
}

pub async fn fetch_manifest(http: &reqwest::Client) -> Result<VersionManifest> {
    let manifest = http
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<VersionManifest>()
        .await?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
        "latest": { "release": "1.21.1", "snapshot": "24w14a" },
        "versions": [
            { "id": "1.21.1", "type": "release", "url": "https://example.test/1.21.1.json", "sha1": "abc", "releaseTime": "x" },
            { "id": "24w14a", "type": "snapshot", "url": "https://example.test/24w14a.json", "sha1": "def" }
        ]
    }"#;

    #[test]
    fn parses_manifest_fixture() {
        let manifest: VersionManifest = serde_json::from_str(FIXTURE).unwrap();
        assert_eq!(manifest.latest.release, "1.21.1");
        assert_eq!(manifest.versions.len(), 2);
        assert_eq!(manifest.versions[0].kind, "release");
        assert_eq!(manifest.versions[1].sha1, "def");
    }
}
