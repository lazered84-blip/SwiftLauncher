//! Fichier de version Mojang (ex. `1.21.1.json`) : modèle, règles de compatibilité,
//! chemins Maven et expansion des arguments. Aucun accès réseau ni disque : entièrement testable.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDetail {
    pub id: String,
    pub main_class: String,
    pub asset_index: AssetIndexRef,
    pub downloads: Downloads,
    #[serde(default)]
    pub libraries: Vec<Library>,
    pub arguments: Option<Arguments>,
    pub minecraft_arguments: Option<String>,
    pub java_version: Option<JavaVersion>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Downloads {
    pub client: Artifact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Présent pour les anciennes bibliothèques natives : OS → clé de classifier.
    #[serde(default)]
    pub natives: HashMap<String, String>,
    pub extract: Option<Extract>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: HashMap<String, Artifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Extract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgItem>,
    #[serde(default)]
    pub jvm: Vec<ArgItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgItem {
    Plain(String),
    Conditional { rules: Vec<Rule>, value: ArgValue },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ArgValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub major_version: u32,
}

/// Fichier à récupérer pour une bibliothèque, déjà filtré pour l'OS courant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryArtifact {
    /// Chemin relatif au dossier `libraries/` (venant du JSON : à valider avant écriture).
    pub relative_path: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    /// `Some(exclusions)` pour une archive native à extraire ; `None` pour un jar de classpath.
    pub extract: Option<Vec<String>>,
}

/// Nom de l'OS au format Mojang : `windows`, `osx` ou `linux`.
pub fn os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "osx",
        _ => "linux",
    }
}

/// Vrai si le binaire est compilé pour x86 32 bits (règle d'arch « x86 » de Mojang).
pub fn is_x86_32() -> bool {
    cfg!(target_arch = "x86")
}

/// Une règle s'applique si son OS correspond et si ses features sont toutes désactivées
/// (le launcher n'active aucune feature : démo, résolution personnalisée, etc.).
fn rule_applies(rule: &Rule, os: &str, x86: bool) -> bool {
    if let Some(os_rule) = &rule.os {
        if let Some(name) = &os_rule.name {
            if name != os {
                return false;
            }
        }
        if let Some(arch) = &os_rule.arch {
            if arch == "x86" && !x86 {
                return false;
            }
        }
    }
    if let Some(features) = &rule.features {
        if features.values().any(|enabled| *enabled) {
            return false;
        }
    }
    true
}

/// Sans règle : autorisé. Avec des règles : la dernière règle applicable décide, par défaut refusé.
pub fn rules_allow(rules: &[Rule], os: &str, x86: bool) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        if rule_applies(rule, os, x86) {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

/// `group:artifact:version[:classifier]` → `group/artifact/version/artifact-version[-classifier].jar`.
pub fn maven_path(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let (group, artifact, version) = (parts[0], parts[1], parts[2]);
    let file = match parts.get(3) {
        Some(classifier) => format!("{artifact}-{version}-{classifier}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    Some(format!("{}/{artifact}/{version}/{file}", group.replace('.', "/")))
}

/// Sélectionne les fichiers de bibliothèques utiles pour l'OS et l'architecture courants.
pub fn select_libraries(detail: &VersionDetail, os: &str, x86: bool) -> Vec<LibraryArtifact> {
    let mut out = Vec::new();
    for lib in &detail.libraries {
        if !rules_allow(&lib.rules, os, x86) {
            continue;
        }
        let Some(downloads) = &lib.downloads else {
            continue;
        };
        let artifact = match lib.natives.get(os) {
            Some(key) => {
                let key = key.replace("${arch}", if x86 { "32" } else { "64" });
                match downloads.classifiers.get(&key) {
                    Some(a) => a,
                    None => continue,
                }
            }
            None => match &downloads.artifact {
                Some(a) => a,
                None => continue,
            },
        };
        let relative_path = match &artifact.path {
            Some(path) => path.clone(),
            None => match maven_path(&lib.name) {
                Some(path) => path,
                None => continue,
            },
        };
        let extract = if lib.natives.contains_key(os) {
            Some(lib.extract.as_ref().map(|e| e.exclude.clone()).unwrap_or_default())
        } else {
            None
        };
        out.push(LibraryArtifact {
            relative_path,
            url: artifact.url.clone(),
            sha1: artifact.sha1.clone(),
            size: artifact.size,
            extract,
        });
    }
    out
}

/// Remplace chaque `${clé}` par sa valeur.
pub fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        out = out.replace(&format!("${{{key}}}"), value);
    }
    out
}

/// Développe une liste d'arguments Mojang (chaînes simples et entrées conditionnelles).
pub fn expand_args(items: &[ArgItem], os: &str, x86: bool, vars: &HashMap<String, String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        match item {
            ArgItem::Plain(s) => out.push(substitute(s, vars)),
            ArgItem::Conditional { rules, value } => {
                if rules_allow(rules, os, x86) {
                    match value {
                        ArgValue::One(s) => out.push(substitute(s, vars)),
                        ArgValue::Many(list) => out.extend(list.iter().map(|s| substitute(s, vars))),
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod fixtures {
    /// Version factice avec : une bibliothèque commune, une réservée à Windows,
    /// une archive native Linux à extraire, des arguments conditionnels et une Java 21.
    pub const DETAIL_JSON: &str = r#"{
        "id": "1.21.1",
        "mainClass": "net.minecraft.client.main.Main",
        "assetIndex": { "id": "17", "url": "https://example.test/17.json", "sha1": "aaaa", "size": 1 },
        "downloads": { "client": { "sha1": "bbbb", "size": 10, "url": "https://example.test/client.jar" } },
        "libraries": [
            { "name": "com.google:gson:2.10",
              "downloads": { "artifact": { "path": "com/google/gson/2.10/gson-2.10.jar", "sha1": "c1", "size": 5, "url": "https://example.test/gson.jar" } } },
            { "name": "org.win:only:1.0",
              "downloads": { "artifact": { "path": "org/win/only/1.0/only-1.0.jar", "sha1": "c2", "size": 5, "url": "https://example.test/win.jar" } },
              "rules": [ { "action": "allow", "os": { "name": "windows" } } ] },
            { "name": "org.lwjgl:lwjgl:3.3.3",
              "downloads": {
                "artifact": { "path": "x", "sha1": "c3", "size": 5, "url": "https://example.test/x.jar" },
                "classifiers": { "natives-linux": { "path": "n/lwjgl-natives-linux.jar", "sha1": "n1", "size": 7, "url": "https://example.test/nat.jar" } }
              },
              "natives": { "linux": "natives-linux", "windows": "natives-windows" },
              "extract": { "exclude": ["META-INF/"] } }
        ],
        "arguments": {
            "game": [ "--username", "${auth_player_name}",
                      { "rules": [ { "action": "allow", "features": { "is_demo_user": true } } ], "value": "--demo" } ],
            "jvm": [ "-Djava.library.path=${natives_directory}", "-cp", "${classpath}" ]
        },
        "javaVersion": { "component": "java-runtime-delta", "majorVersion": 21 }
    }"#;
}

#[cfg(test)]
mod tests {
    use super::fixtures::DETAIL_JSON;
    use super::*;

    fn detail() -> VersionDetail {
        serde_json::from_str(DETAIL_JSON).unwrap()
    }

    #[test]
    fn maven_path_without_and_with_classifier() {
        assert_eq!(
            maven_path("com.google:gson:2.10").as_deref(),
            Some("com/google/gson/2.10/gson-2.10.jar")
        );
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3:natives-linux").as_deref(),
            Some("org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-linux.jar")
        );
        assert_eq!(maven_path("invalide"), None);
    }

    #[test]
    fn os_rules_filter_by_platform() {
        let rules = vec![Rule {
            action: "allow".into(),
            os: Some(OsRule { name: Some("windows".into()), arch: None }),
            features: None,
        }];
        assert!(rules_allow(&rules, "windows", false));
        assert!(!rules_allow(&rules, "linux", false));
        assert!(rules_allow(&[], "linux", false), "sans règle = autorisé");
    }

    #[test]
    fn feature_rules_do_not_apply_to_launcher() {
        // La règle de démo ne s'applique pas : le launcher n'active aucune feature.
        let d = detail();
        let game = &d.arguments.as_ref().unwrap().game;
        let vars = HashMap::new();
        let expanded = expand_args(game, "linux", false, &vars);
        assert!(!expanded.contains(&"--demo".to_string()));
    }

    #[test]
    fn selects_libraries_for_linux() {
        let libs = select_libraries(&detail(), "linux", false);
        // gson (classpath) + lwjgl natives (extraction). La lib Windows est exclue.
        assert_eq!(libs.len(), 2);
        assert_eq!(libs[0].relative_path, "com/google/gson/2.10/gson-2.10.jar");
        assert_eq!(libs[0].extract, None);
        assert_eq!(libs[1].relative_path, "n/lwjgl-natives-linux.jar");
        assert_eq!(libs[1].extract, Some(vec!["META-INF/".to_string()]));
    }

    #[test]
    fn selects_windows_library_on_windows() {
        let libs = select_libraries(&detail(), "windows", false);
        assert!(libs.iter().any(|l| l.relative_path == "org/win/only/1.0/only-1.0.jar"));
    }

    #[test]
    fn substitute_replaces_placeholders() {
        let mut vars = HashMap::new();
        vars.insert("auth_player_name".to_string(), "Alex".to_string());
        assert_eq!(substitute("--username ${auth_player_name}", &vars), "--username Alex");
    }
}
