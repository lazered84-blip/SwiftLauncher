//! Backend du launcher Minecraft (Tauri 2).
//!
//! Architecture : les commandes (`commands.rs`) restent fines et délèguent
//! à des modules métier (`modules/`). Voir CLAUDE.md §3.

pub mod commands;
pub mod error;
pub mod modules;

use modules::database::Database;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

/// État partagé injecté dans les commandes Tauri.
pub struct AppState {
    pub db: Database,
    pub http: reqwest::Client,
    pub data_dir: std::path::PathBuf,
}

const USER_AGENT: &str = concat!(
    "MinecraftLauncher/",
    env!("CARGO_PKG_VERSION"),
    " (+https://votre-domaine.fr)"
);

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let db = Database::open(&data_dir.join("launcher.sqlite"))?;
            let http = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
            app.manage(AppState { db, http, data_dir: data_dir.clone() });
            tracing::info!("launcher démarré, données : {}", data_dir.display());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_instances,
            commands::create_instance,
            commands::delete_instance,
            commands::list_versions,
            commands::detect_java,
            commands::login_microsoft,
            commands::current_account,
            commands::sign_out,
            commands::launch_instance,
        ])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer l'application");
}
