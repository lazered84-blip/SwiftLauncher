use serde::{Serialize, Serializer};
use thiserror::Error;

/// Erreur unique du launcher. Le message est renvoyé tel quel à l'interface :
/// ne jamais y inclure de token, de secret ni de corps de réponse brut.
#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("base de données : {0}")]
    Database(#[from] rusqlite::Error),

    #[error("réseau : {0}")]
    Network(#[from] reqwest::Error),

    #[error("fichier : {0}")]
    Io(#[from] std::io::Error),

    #[error("données JSON invalides : {0}")]
    Json(#[from] serde_json::Error),

    #[error("trousseau système : {0}")]
    Keyring(#[from] keyring::Error),

    #[error("sécurité : {0}")]
    Security(String),

    #[error("authentification : {0}")]
    Auth(String),

    #[error("ce compte ne possède pas Minecraft Java Edition")]
    NoJavaLicense,

    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, LauncherError>;

impl Serialize for LauncherError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
