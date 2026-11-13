# CLAUDE.md — Instructions pour l'agent sur ce projet

Tu travailles sur un launcher Minecraft indépendant, gratuit, multiplateforme (Windows, Linux, macOS). Lis ce fichier en entier avant toute modification.

## 1. Contexte

- Projet personnel, hébergé sur un VPS, distribué gratuitement.
- Langue du code : identifiants en anglais, commentaires et documentation en français.
- Langue de l'interface : français en priorité, anglais ensuite (i18n dès le départ).
- Le launcher doit être optimisé pour le PvP (latence, FPS, stabilité), sans jamais devenir un outil de triche.

## 2. Règles légales non négociables

1. **Authentification** : uniquement via le flux officiel Microsoft OAuth 2.0 (Xbox Live → XSTS → services Minecraft). Ne jamais demander ni stocker un mot de passe Microsoft.
2. **Propriété de Minecraft** : vérifier que le compte possède Minecraft Java Edition avant de lancer le jeu. Ne jamais contourner cette vérification, ne jamais proposer de « mode crack », ni d'émuler une licence.
3. **Aucune triche** : ne jamais implémenter de cheat, de hack, de module donnant un avantage en jeu (reach, killaura, aimbot, xray, bypass de keystrokes, faux FPS, etc.), ni de contournement d'un anti-cheat. Si une demande va dans ce sens, refuse et propose une alternative légitime (voir `docs/FONCTIONNALITES_PVP.md`).
4. **Marque** : ne pas utiliser le logo Minecraft comme logo du launcher. Le disclaimer « non affilié à Mojang/Microsoft » doit rester visible (README, à-propos, écran de connexion).
5. **Tiers** : Modrinth et CurseForge ont leurs propres conditions d'API. Respecter les User-Agent demandés, les limites de débit et les clés API. CurseForge exige une clé et une autorisation.
6. **Données personnelles** : minimiser les données, stocker les tokens uniquement via le coffre sécurisé (voir §5), ne jamais journaliser de token.
7. **Licences** : avant d'ajouter une dépendance, vérifier sa licence et l'ajouter à la liste dans `legal/CONFORMITE.md`.

Consulte `legal/CONFORMITE.md` avant toute fonctionnalité touchant aux comptes, aux téléchargements, aux mods ou au réseau.

## 3. Stack et architecture

- Frontend : React + TypeScript + Vite, dans `src/`.
- Backend : Tauri 2 + Rust, dans `src-tauri/`.
- Base : SQLite (accès via un module `database`, migrations versionnées).
- Modules Rust (un dossier par module dans `src-tauri/src/modules/`) :
  - `minecraft_core` : manifests Mojang, téléchargement des versions, arguments de lancement.
  - `instance_manager` : CRUD, duplication, import/export des instances.
  - `mod_manager` : Modrinth, CurseForge, dépendances, incompatibilités.
  - `loader_manager` : Fabric, Forge, NeoForge, Quilt.
  - `account_manager` : comptes multiples, rafraîchissement de session.
  - `download_manager` : file de téléchargement, reprise, vérification SHA-1/SHA-256.
  - `java_manager` : détection et installation des JRE.
  - `performance_engine` : détection matérielle, profils, analyse de crash.
  - `backup_engine` : sauvegardes, historique, restauration.
  - `database` : SQLite, migrations.
  - `security` : coffre des tokens, validation des fichiers et des chemins.
- Chaque module expose une API claire via `mod.rs`. Pas de dépendances circulaires entre modules.
- Les commandes Tauri sont fines : elles délèguent aux modules, sans logique métier.

## 4. Design system (à respecter côté React)

Tokens CSS à définir dans `src/styles/tokens.css` :

```css
:root {
  --color-primary: #60A5FA;
  --color-secondary: #93C5FD;
  --color-soft: #DBEAFE;
  --color-bg: #F5F9FF;
  --color-surface: #FFFFFF;
  --color-text: #172554;
  --color-text-muted: #64748B;
  --radius-card: 16px;
  --radius-btn: 12px;
  --shadow-soft: 0 1px 2px rgba(23,37,84,0.06), 0 4px 12px rgba(23,37,84,0.06);
}
```

- Le bleu clair est la couleur dominante. Pas de couleurs vives concurrentes sauf pour les états (succès, erreur, avertissement).
- Ombres discrètes, coins arrondis, animations de 150–250 ms maximum.
- Préparer le thème sombre : passer toutes les couleurs par des tokens, jamais en dur.
- Icônes : une seule famille (ex. Lucide).
- Accessibilité : contraste WCAG AA minimum, navigation clavier, labels ARIA.

## 5. Sécurité

- Tokens et refresh tokens : stockés via le trousseau système (Keychain macOS, Credential Manager Windows, Secret Service Linux), jamais en clair dans SQLite ni dans les logs.
- Téléchargements : vérifier le hash de chaque fichier avant utilisation. Refuser les fichiers sans hash connu quand la source en fournit un.
- Chemins : toute écriture disque passe par le module `security`, qui empêche la traversée de répertoire (`..`, chemins absolus issus d'un manifest ou d'un fichier de modpack).
- Permissions Tauri : au minimum nécessaire, par capability explicite. Pas de `fs:allow-all`.
- Mises à jour : sauvegarde automatique de l'instance avant une mise à jour de mod ou de loader.
- Logs : niveaux `info`, `warn`, `error`. Masquer les tokens, les e-mails et les UUID complets.

## 6. Qualité et tests

- Chaque module Rust a des tests unitaires. Les appels réseau sont mockés.
- Les flux critiques (auth, téléchargement, lancement) ont des tests d'intégration.
- Commandes de référence :
  - `npm run lint`, `npm run test`, `npm run tauri build`
  - `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
- Ne pas livrer de code qui ne compile pas. Ne pas désactiver un test pour « faire passer » la CI.

## 7. Règle du journal (UPDATE.md)

**À chaque modification du dépôt**, ajouter une entrée en haut de `UPDATE.md` :
- date (format AAAA-MM-JJ),
- version concernée,
- ce qui a changé (Ajouté / Modifié / Corrigé / Sécurité),
- fichiers touchés principaux,
- ce qui reste à faire.

Si une modification touche à la légalité, à la sécurité ou aux données personnelles, mettre à jour le fichier concerné dans `legal/` et le signaler dans l'entrée.

## 8. Feuille de route

- **v0.1** : interface, authentification Microsoft, installation Vanilla, gestion Java, lancement, gestion des instances.
- **v0.2** : Fabric, Forge, NeoForge, Quilt, gestionnaire de mods, Modrinth, import de modpacks.
- **v0.5** : optimisation, diagnostic intelligent, sauvegardes, historique, partage.
- **v1.0** : mise à jour automatique, tests Windows/Linux/macOS, sécurité renforcée, documentation, distribution publique.

Priorité actuelle : **v0.1**. Ne pas implémenter de fonctionnalité v0.2+ sans demande explicite.

## 9. Style de travail

- Avant une tâche importante, proposer un plan court et attendre une validation si la tâche touche à l'auth, aux téléchargements ou à la sécurité.
- Préférer de petits commits thématiques.
- Quand une information manque (version d'API, clé CurseForge, identifiant d'application Azure), le signaler plutôt qu'inventer une valeur.
- Ne jamais committer de secrets : clés API, client ID privés, certificats de signature.
