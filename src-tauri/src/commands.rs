//! Commandes exposées au frontend. Aucune logique métier ici : uniquement délégation.

use crate::error::{LauncherError, Result};
use crate::modules::account_manager::{self, Account};
use crate::modules::instance_manager::{self, Instance};
use crate::modules::java_manager::{self, runtime, JavaInstall};
use crate::modules::minecraft_core::install::{self, Paths, Progress};
use crate::modules::minecraft_core::launch::{self, LaunchContext};
use crate::modules::minecraft_core::{self, VersionEntry};
use crate::AppState;
use std::path::Path;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn list_instances(state: State<'_, AppState>) -> Result<Vec<Instance>> {
    instance_manager::list(&state.db)
}

#[tauri::command]
pub fn create_instance(
    state: State<'_, AppState>,
    name: String,
    mc_version: String,
) -> Result<Instance> {
    instance_manager::create(&state.db, &name, &mc_version)
}

#[tauri::command]
pub fn delete_instance(state: State<'_, AppState>, id: i64) -> Result<()> {
    instance_manager::delete(&state.db, id)?;
    // Supprime aussi les fichiers (mondes compris) : l'interface demande une confirmation avant.
    let dir = Paths::new(state.data_dir.clone()).instance_dir(id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<Vec<VersionEntry>> {
    Ok(minecraft_core::fetch_manifest(&state.http).await?.versions)
}

#[tauri::command]
pub async fn detect_java() -> Result<Vec<JavaInstall>> {
    tokio::task::spawn_blocking(java_manager::detect)
        .await
        .map_err(|e| LauncherError::Invalid(format!("détection Java interrompue : {e}")))
}

#[tauri::command]
pub async fn login_microsoft(app: AppHandle, state: State<'_, AppState>) -> Result<Account> {
    account_manager::login(&state.http, &state.db, |url| {
        app.opener()
            .open_url(url, None::<&str>)
            .map_err(|e| LauncherError::Auth(format!("impossible d'ouvrir le navigateur ({e})")))
    })
    .await
}

#[tauri::command]
pub async fn current_account(state: State<'_, AppState>) -> Result<Option<Account>> {
    account_manager::restore_session(&state.http, &state.db).await
}

#[tauri::command]
pub fn sign_out(state: State<'_, AppState>) -> Result<()> {
    account_manager::sign_out(&state.db)
}

/// Installe si besoin la version de l'instance, prépare la session et lance Minecraft.
/// Renvoie le PID du processus Java. La progression est émise sur `install-progress`.
#[tauri::command]
pub async fn launch_instance(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<u32> {
    let instance = instance_manager::get(&state.db, id)?;
    let paths = Paths::new(state.data_dir.clone());

    let emit = |p: Progress| {
        let _ = app.emit("install-progress", p);
    };
    let detail = install::ensure_installed(&state.http, &paths, &instance.mc_version, &emit).await?;

    let required = detail.java_version.as_ref().map(|j| j.major_version).unwrap_or(8);
    let runtimes_dir = paths.runtimes_dir();
    let scan_dir = runtimes_dir.clone();
    let installs = tokio::task::spawn_blocking(move || {
        let mut found = java_manager::detect();
        found.extend(runtime::detect_runtimes(&scan_dir));
        found
    })
    .await
    .map_err(|e| LauncherError::Invalid(format!("détection Java interrompue : {e}")))?;
    let java = match launch::pick_java(&installs, required) {
        Ok(found) => found.path.clone(),
        Err(_) => {
            // Aucune version suffisante : on télécharge un JRE officiel dans le dossier du launcher.
            emit(Progress { stage: "java", done: 0, total: 1 });
            let downloaded = runtime::ensure_runtime(&state.http, &runtimes_dir, required).await?;
            emit(Progress { stage: "java", done: 1, total: 1 });
            downloaded.path
        }
    };

    let (account, token) = account_manager::launch_session(&state.http, &state.db).await?;

    let game_dir = paths.instance_dir(id);
    tokio::fs::create_dir_all(&game_dir).await?;
    let args = launch::build_args(&LaunchContext {
        detail: &detail,
        paths: &paths,
        game_dir: &game_dir,
        player_name: &account.name,
        player_uuid: &account.uuid,
        access_token: &token,
        ram_mb: instance.ram_mb,
    });

    let mut child = launch::spawn(Path::new(&java), &args, &game_dir)?;
    let pid = child.id();
    tracing::info!("Minecraft lancé (pid {pid}, instance {id})");
    std::thread::spawn(move || match child.wait() {
        Ok(status) => tracing::info!("Minecraft terminé : {status}"),
        Err(e) => tracing::warn!("suivi du processus impossible : {e}"),
    });
    Ok(pid)
}
