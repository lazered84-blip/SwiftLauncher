//! Construction de la ligne de commande Java et démarrage de Minecraft.
//!
//! ⚠ Sécurité : le token d'accès est passé en argument (exigence du jeu). Il est donc visible
//! par les autres processus de la même session via la liste des processus. Il n'est jamais
//! écrit dans les logs. Piste d'amélioration : passer les arguments par un fichier temporaire
//! protégé ou via `--accessToken` depuis un canal privé (voir UPDATE.md, risques connus).

use crate::error::{LauncherError, Result};
use crate::modules::java_manager::JavaInstall;
use crate::modules::minecraft_core::install::Paths;
use crate::modules::minecraft_core::version::{expand_args, is_x86_32, os_name, select_libraries, substitute, VersionDetail};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Child, Command};

pub struct LaunchContext<'a> {
    pub detail: &'a VersionDetail,
    pub paths: &'a Paths,
    pub game_dir: &'a Path,
    pub player_name: &'a str,
    pub player_uuid: &'a str,
    pub access_token: &'a str,
    pub ram_mb: i64,
}

/// Choisit l'installation Java la plus proche de la version requise (sans descendre en dessous).
pub fn pick_java(installs: &[JavaInstall], required_major: u32) -> Result<&JavaInstall> {
    installs
        .iter()
        .filter(|j| j.major >= required_major)
        .min_by_key(|j| j.major)
        .ok_or_else(|| {
            LauncherError::Invalid(format!("Java {required_major} ou plus est requis."))
        })
}

pub fn build_args(ctx: &LaunchContext<'_>) -> Vec<String> {
    let detail = ctx.detail;
    let paths = ctx.paths;
    let os = os_name();
    let x86 = is_x86_32();
    let separator = if os == "windows" { ";" } else { ":" };

    let mut classpath: Vec<String> = select_libraries(detail, os, x86)
        .iter()
        .filter(|lib| lib.extract.is_none())
        .map(|lib| paths.libraries_dir().join(&lib.relative_path).to_string_lossy().into_owned())
        .collect();
    classpath.push(paths.client_jar(&detail.id).to_string_lossy().into_owned());

    let vars: HashMap<String, String> = [
        ("natives_directory", paths.natives_dir(&detail.id).to_string_lossy().into_owned()),
        ("launcher_name", "minecraft-launcher".to_string()),
        ("launcher_version", env!("CARGO_PKG_VERSION").to_string()),
        ("classpath", classpath.join(separator)),
        ("auth_player_name", ctx.player_name.to_string()),
        ("version_name", detail.id.clone()),
        ("game_directory", ctx.game_dir.to_string_lossy().into_owned()),
        ("assets_root", paths.assets_dir().to_string_lossy().into_owned()),
        ("assets_index_name", detail.asset_index.id.clone()),
        ("auth_uuid", ctx.player_uuid.to_string()),
        ("auth_access_token", ctx.access_token.to_string()),
        ("auth_xuid", String::new()),
        ("clientid", String::new()),
        ("user_type", "msa".to_string()),
        ("version_type", "release".to_string()),
        ("user_properties", "{}".to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    // Options JVM : mémoire et ramasse-miettes G1 avec pauses courtes (utile en PvP).
    let mut args: Vec<String> = vec![
        format!("-Xmx{}M", ctx.ram_mb),
        format!("-Xms{}M", ctx.ram_mb.min(1024)),
        "-XX:+UseG1GC".into(),
        "-XX:MaxGCPauseMillis=100".into(),
    ];
    match detail.arguments.as_ref() {
        Some(a) if !a.jvm.is_empty() => args.extend(expand_args(&a.jvm, os, x86, &vars)),
        // Anciennes versions sans liste JVM : arguments minimaux.
        _ => {
            args.push(substitute("-Djava.library.path=${natives_directory}", &vars));
            args.push("-cp".into());
            args.push(substitute("${classpath}", &vars));
        }
    }

    args.push(detail.main_class.clone());

    match (detail.arguments.as_ref(), detail.minecraft_arguments.as_ref()) {
        (Some(a), _) if !a.game.is_empty() => args.extend(expand_args(&a.game, os, x86, &vars)),
        (_, Some(legacy)) => args.extend(substitute(legacy, &vars).split_whitespace().map(str::to_string)),
        _ => {}
    }
    args
}

/// Démarre Java dans le dossier de l'instance. Le processus est détaché : on garde seulement le handle.
pub fn spawn(java: &Path, args: &[String], game_dir: &Path) -> Result<Child> {
    Ok(Command::new(java).args(args).current_dir(game_dir).spawn()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::minecraft_core::version::fixtures::DETAIL_JSON;
    use std::path::PathBuf;

    fn installs() -> Vec<JavaInstall> {
        vec![
            JavaInstall { path: "/j8".into(), version: "1.8.0_392".into(), major: 8 },
            JavaInstall { path: "/j21".into(), version: "21.0.2".into(), major: 21 },
            JavaInstall { path: "/j25".into(), version: "25".into(), major: 25 },
        ]
    }

    #[test]
    fn picks_smallest_sufficient_java() {
        assert_eq!(pick_java(&installs(), 21).unwrap().path, "/j21");
        assert_eq!(pick_java(&installs(), 8).unwrap().path, "/j8");
    }

    #[test]
    fn errors_only_when_nothing_is_recent_enough() {
        assert_eq!(pick_java(&installs(), 22).unwrap().path, "/j25", "Java 25 convient pour 22");
        assert!(pick_java(&installs(), 26).is_err());
        assert!(pick_java(&[], 21).is_err());
    }

    #[test]
    fn builds_expected_command_line() {
        let detail: VersionDetail = serde_json::from_str(DETAIL_JSON).unwrap();
        let paths = Paths::new(PathBuf::from("data"));
        let game_dir = paths.instance_dir(1);
        let ctx = LaunchContext {
            detail: &detail,
            paths: &paths,
            game_dir: &game_dir,
            player_name: "Alex",
            player_uuid: "0123456789abcdef0123456789abcdef",
            access_token: "token-test",
            ram_mb: 4096,
        };
        let args = build_args(&ctx);

        assert!(args.contains(&"-Xmx4096M".to_string()));
        assert!(args.contains(&"net.minecraft.client.main.Main".to_string()));
        let user_pos = args.iter().position(|a| a == "--username").expect("--username présent");
        assert_eq!(args[user_pos + 1], "Alex");
        assert!(!args.contains(&"--demo".to_string()));

        let cp_pos = args.iter().position(|a| a == "-cp").expect("-cp présent");
        assert!(args[cp_pos + 1].contains("1.21.1.jar"), "le client doit être dans le classpath");
        assert!(!args[cp_pos + 1].contains("lwjgl-natives-linux"), "les natives ne sont pas sur le classpath");
    }
}
