# UPDATE.md — Journal des modifications

Format : une entrée par modification, la plus récente en haut.
Chaque entrée indique la date, la version, les changements (Ajouté / Modifié / Corrigé / Sécurité), les fichiers principaux et ce qui reste à faire.

---

## [0.1.0-alpha.5] — 2026-11-13 — Correctifs CI (icônes, vérifications non bloquantes)

### Contexte
Premier run de la CI GitHub : échec. Journal non encore consulté ; deux causes très probables corrigées d'avance.

### Ajouté
- **Icônes provisoires** dans `src-tauri/icons/` (carré arrondi bleu clair et triangle « lecture », créés pour le projet, sans élément Minecraft) : `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, `icon.ico`, `icon.icns`. À remplacer par le vrai logo avec `npm run tauri icon <logo.png>`.

### Modifié
- `src-tauri/tauri.conf.json` : `bundle.icon` référence les icônes (la compilation Tauri les exige, en particulier `icon.ico` sous Windows).
- `.github/workflows/ci.yml` : `cargo fmt --check` et `cargo clippy` passent en `continue-on-error` (le code n'a pas pu être formaté avec rustfmt dans l'environnement de génération). La compilation et `cargo test` restent bloquants. À rétablir en bloquant une fois `cargo fmt` appliqué et les avertissements clippy corrigés.

### Risques connus
- Sous Windows, `cargo test` sur un crate Tauri peut échouer au lancement des tests (`STATUS_ENTRYPOINT_NOT_FOUND`, manifeste Windows absent du binaire de test). Non confirmé ici ; à traiter seulement si la CI le montre.
- Le code Rust reste non compilé : d'autres erreurs sont probables.

### Reste à faire
- Lire le journal du job en échec et corriger les erreurs de compilation.
- Puis : `cargo fmt`, corrections clippy, remise en bloquant de ces deux étapes.

---

## [0.1.0-alpha.4] — 2026-11-13 — Mode démonstration dans le navigateur

### Ajouté
- **Mode démonstration** (`src/lib/mock.ts`) : quand l'interface tourne dans un simple navigateur (`npm run dev`), un faux backend en mémoire remplace Tauri : instances, versions, Java et compte fictifs, progression d'installation simulée. Permet de voir et tester l'interface sans Rust.
- Bandeau « Mode démonstration » affiché en haut de l'application dans ce mode.
- 4 tests Vitest pour le mode démonstration (instances, règles de nom identiques au backend, compte fictif, lancement refusé).

### Sécurité / honnêteté
- Le mode démonstration **ne lance jamais le jeu** : `launchInstance` simule la progression puis refuse avec un message explicite. La connexion Microsoft y est fictive (« Joueur démo ») et n'envoie rien à Microsoft.
- Le bascule se fait uniquement sur la présence de `__TAURI_INTERNALS__` : dans l'application Tauri, le vrai backend est toujours utilisé.

### Vérifications effectuées
- Frontend : `tsc --noEmit` OK, `vitest` 9/9 OK, `vite build` OK. Le rendu visuel dans un navigateur n'a pas pu être vérifié dans l'environnement de génération.
- Backend : toujours non compilé (voir 0.1.0-alpha.3).

### Reste à faire
- Inchangé : compiler le backend (`cargo check`, `cargo test` ou CI), icônes, enregistrement de l'application Microsoft Entra, demande d'approbation Mojang, test réel de lancement.

---

## [0.1.0-alpha.3] — 2026-11-13 — JRE automatique et suppression sûre des instances

### Ajouté
- **`java_manager/runtime.rs`** : téléchargement d'un JRE Eclipse Temurin via l'API Adoptium quand aucune version de Java suffisante n'est trouvée. Installation dans `<données>/runtimes/java-<N>` (aucune modification du système), extraction `.zip` (Windows) ou `.tar.gz` (Linux, macOS), détection des JRE déjà installés par le launcher.
- **`security::verify_sha256`** (avec test sur un digest connu).
- **Suppression d'instance** : les fichiers de l'instance (mondes compris) sont supprimés avec elle. L'interface demande une confirmation en deux temps (« Supprime aussi les mondes. »).
- Étape « Java » dans la progression d'installation affichée au lancement.
- Dépendances : `flate2`, `tar`.

### Modifié
- `launch_instance` : cherche Java dans le système puis dans les JRE du launcher, et télécharge un JRE si nécessaire.
- Message d'erreur Java simplifié (l'installation automatique existe maintenant).
- `Paths::runtimes_dir`.

### Sécurité
- Lien du JRE refusé s'il n'est pas en HTTPS, nom d'archive refusé s'il contient un séparateur de chemin ou une extension inattendue, somme SHA-256 exigée (64 caractères hexadécimaux) et vérifiée avant toute extraction.
- Extraction `.zip` via `enclosed_name` (refuse `..` et chemins absolus), extraction `.tar.gz` via `Archive::unpack` (même protection).

### Risques connus
- **Format de l'API Adoptium non vérifié** : écrit d'après la documentation publique, sans accès réseau pour le tester. À valider par un premier téléchargement réel ; les tests ne couvrent que l'analyse d'un exemple de réponse.
- **Suppression irréversible** : supprimer une instance efface ses mondes. Une corbeille ou une sauvegarde automatique avant suppression est à prévoir (module `backup_engine`, v0.5).
- Les risques de `0.1.0-alpha.2` (jeton en ligne de commande, hash des fichiers existants, architectures ARM) restent ouverts.

### Vérifications effectuées
- Frontend : `tsc --noEmit` OK, `vitest` 5/5 OK, `vite build` OK.
- Backend : **toujours non compilé** (pas de toolchain Rust dans l'environnement de génération). Seul un contrôle superficiel a été fait : équilibre des accolades, parenthèses et crochets sur les 20 fichiers Rust, et correspondance entre `modules/mod.rs` et les dossiers. Cela ne remplace pas `cargo check`.

### Reste à faire
- **Compiler et tester le backend** (`cargo check`, `cargo test`, ou CI).
- Tester un lancement réel de bout en bout (compte avec Minecraft Java requis).
- Valider les endpoints Minecraft Services et celui d'Adoptium.
- Générer les icônes (`npm run tauri icon`).
- Prochaine étape de la feuille de route : v0.2 (loaders Fabric/Forge/NeoForge/Quilt, Modrinth).

---

## [0.1.0-alpha.2] — 2026-11-13 — Installation Vanilla et lancement du jeu

### Ajouté
- **`minecraft_core/version.rs`** : modèle du fichier de version Mojang, règles OS et features (dernière règle applicable décide), chemins Maven, sélection des bibliothèques (classpath et natives), expansion des arguments JVM et jeu, substitution des `${...}`. Fonctions pures, testées sans réseau.
- **`minecraft_core/install.rs`** : arborescence `Paths`, téléchargements parallèles bornés (`download_items`), ignorance des fichiers déjà présents à la bonne taille, index et objets d'assets, extraction des archives natives (zip, sans `META-INF` ni chemins dangereux), progression émise sur l'événement `install-progress`.
- **`minecraft_core/launch.rs`** : construction de la ligne de commande Java (mémoire, G1 avec pauses courtes), choix de la version Java requise, démarrage du processus dans le dossier de l'instance.
- **`account_manager::launch_session`** : renouvellement du jeton Microsoft et obtention d'un jeton Minecraft frais à chaque lancement (rien n'est persisté).
- **`instance_manager::get`**.
- **Commande `launch_instance`** : installe si besoin, vérifie Java, prépare la session, lance le jeu et renvoie le PID.
- **Frontend** : sélection d'instance sur le tableau de bord, bouton « Jouer » fonctionnel, progression d'installation en direct, messages d'erreur lisibles.
- **Tests Rust** : règles d'OS, chemins Maven, sélection des bibliothèques, arguments, substitution, choix de Java, hash d'assets, validation des chemins.

### Sécurité
- Tous les chemins venant d'un JSON Mojang passent par `security::safe_join`. Les hash d'assets sont validés (40 caractères hexadécimaux) avant d'être utilisés comme chemins.
- Téléchargements uniquement en HTTPS, vérifiés par SHA-1 avant écriture.
- Le jeton Minecraft n'est jamais écrit sur disque ni dans les logs.

### Risques connus (à traiter)
- **Token sur la ligne de commande** : l'argument `--accessToken` (ou `auth_access_token`) est visible par les autres processus de la session (liste des processus). C'est le fonctionnement attendu du jeu, mais c'est une fuite à réduire. Piste : canal privé ou fichier temporaire protégé.
- **Vérification des fichiers existants** : seule la taille est contrôlée pour les fichiers déjà présents (pour éviter de tout re-hasher à chaque lancement). Une corruption de même taille ne serait pas détectée. Piste : vérification SHA-1 périodique ou à l'ouverture de l'instance.
- **Règle d'architecture** : les règles `arch` sont approximées (seule la règle `x86` 32 bits est prise en compte). Les natives ARM ne sont pas vérifiées.
- **Java** : le launcher ne télécharge pas encore de JRE. Il utilise une installation déjà présente et échoue proprement si la version requise manque.

### Modifié
- `Cargo.toml` : dépendance `zip` (extraction des natives, sans backends inutiles).
- `AppState` : ajout de `data_dir`.
- `minecraft_core/mod.rs` : déclaration des sous-modules.

### Vérifications effectuées
- Frontend : `tsc --noEmit` OK, `vitest` 5/5 OK.
- Backend : **toujours non compilé** dans l'environnement de génération (toolchain Rust et domaines Mojang/rustup inaccessibles). Le code a été relu, mais une compilation réelle est indispensable.

### Reste à faire
- **Compiler et tester le backend** (`cargo check`, `cargo test`, puis CI).
- **Tester un lancement réel** sur un compte qui possède Minecraft Java : installation Vanilla complète, puis lancement jusqu'au menu principal.
- Valider les endpoints Minecraft Services et le contrôle de licence sur un compte réel.
- Télécharger un JRE officiel par version Java requise (`java_manager`), ou au minimum guider l'utilisateur.
- Persister les dossiers d'instance (duplication, suppression des fichiers à la suppression de l'instance).
- Générer les icônes (`npm run tauri icon`).

---

## [0.1.0-alpha.1] — 2026-11-12 — Scaffold du projet et premier code v0.1

### Ajouté
- **Frontend** (React + TypeScript + Vite) : navigation latérale, pages Accueil, Instances, Compte et Réglages, design system bleu clair (tokens CSS), disclaimer légal affiché en permanence.
- **Validation** des noms d'instance côté interface, avec tests Vitest.
- **Backend** (Tauri 2 + Rust) :
  - `security` : `safe_join` (anti traversée de répertoire), `verify_sha1`, validation des noms et versions, tests unitaires.
  - `database` : SQLite avec migrations versionnées (`PRAGMA user_version`), WAL, clés étrangères.
  - `instance_manager` : création, liste, suppression, doublon refusé, tests.
  - `java_manager` : détection via `JAVA_HOME` et PATH, parsing de `java -version` (Java 8 et modernes), tests.
  - `minecraft_core` : lecture du manifest officiel Mojang, tests sur fixture.
  - `download_manager` : HTTPS obligatoire, vérification SHA-1 avant écriture, écriture atomique (`.part` puis renommage).
  - `account_manager` : OAuth 2.0 avec PKCE, redirection loopback sur le port 53682 avec contrôle du `state`, chaîne Xbox Live → XSTS → Minecraft Services, vérification de la licence Java, refresh token dans le trousseau système, renouvellement silencieux au démarrage, déconnexion. Tests du PKCE, de l'URL d'autorisation et du parsing du callback.
- **Permissions Tauri** minimales (`core:default`, `opener:default`), CSP restrictive.
- **Stubs** documentés pour `loader_manager`, `mod_manager`, `performance_engine`, `backup_engine` (v0.2 et v0.5).
- **CI** : `.github/workflows/ci.yml` (frontend, et backend sur Linux, Windows, macOS).
- **SECURITY.md** : politique de signalement des vulnérabilités.
- `.gitignore`, `.env.example`.

### Modifié
- `docs/GUIDE_INSTALLATION.md` : port de redirection, injection de `MS_CLIENT_ID` à la compilation, étape des icônes, section CI.

### Sécurité
- Aucun mot de passe Microsoft manipulé ; aucun token en base SQLite ni dans les logs.
- Messages d'erreur réseau ne contiennent ni corps de réponse ni token.
- Vérification de la licence Java avant tout accès au profil.

### Fichiers principaux
- `src/` (App, pages, lib/api.ts, lib/validation.ts)
- `src-tauri/src/` (lib.rs, commands.rs, error.rs, modules/*)
- `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- `.github/workflows/ci.yml`, `SECURITY.md`

### Vérifications effectuées
- Frontend : `tsc --noEmit` OK, `vitest` 5/5 OK, `vite build` OK.
- Backend : **non compilé dans l'environnement de génération** (toolchain Rust indisponible). Il doit être compilé et testé localement ou via la CI avant toute utilisation.

### Reste à faire
- Compiler le backend (`cargo check`, `cargo test`) et corriger les éventuelles erreurs de compilation.
- Générer les icônes (`npm run tauri icon`).
- Valider la chaîne d'authentification Minecraft Services sur un compte réel (endpoints et contrôle de licence).
- Enregistrer l'application Microsoft Entra et obtenir l'accès aux services Minecraft.
- Implémenter le téléchargement complet d'une version Vanilla (manifest de version, bibliothèques, assets) avec `download_manager`.
- Implémenter le lancement du jeu (arguments JVM, classpath, natives).
- Ajouter la persistance des dossiers d'instance sur disque.
- Compléter les modèles `legal/` (éditeur, adresse, contact, hébergeur) et `SECURITY.md` (e-mail).
- Choisir définitivement la licence.

---

## [0.0.1] — 2026-11-12 — Documentation et cadre légal

### Ajouté
- `README.md` : présentation, fonctionnalités, direction artistique, stack, structure.
- `CLAUDE.md` : instructions pour les agents IA qui travaillent sur le code (règles légales, architecture, design system, sécurité, règle du journal).
- `UPDATE.md` : ce journal.
- `docs/GUIDE_INSTALLATION.md` : installation de l'environnement, lancement en développement, tests, déploiement sur VPS.
- `docs/FONCTIONNALITES_PVP.md` : optimisation PvP légitime, pistes de fonctionnalités à ajouter ou améliorer, limites à ne pas franchir.
- `legal/CONFORMITE.md` : checklist de conformité (Mojang/Microsoft, Modrinth, CurseForge, licences, RGPD, sécurité).
- `legal/MENTIONS_LEGALES.md`, `legal/CGU.md`, `legal/CONFIDENTIALITE.md` : modèles à compléter.
- `LICENSE` : modèle MIT à valider.

### Sécurité
- Interdiction explicite de tout cheat et de tout contournement de licence ou d'anti-cheat dans les consignes de l'agent.

### Fichiers principaux
- Racine du dépôt, `docs/`, `legal/`

### Reste à faire (v0.1)
- Initialiser le projet Tauri 2 + React + TypeScript + Vite (`src-tauri/`, `src/`).
- Créer les tokens CSS du design system.
- Enregistrer l'application auprès de Microsoft Entra (Azure) et faire la demande d'accès aux services Minecraft.
- Implémenter le module `security` (coffre des tokens) et `account_manager`.
- Implémenter `minecraft_core` (manifests Mojang, téléchargement Vanilla).
- Implémenter `java_manager` et le lancement du jeu.
- Mettre en place la base SQLite et les migrations.
- Compléter les modèles `legal/` avec les informations réelles (éditeur, adresse, contact, hébergeur VPS).
- Choisir définitivement la licence.
