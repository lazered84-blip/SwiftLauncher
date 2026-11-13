# Guide d'installation et de test

Ce guide couvre trois usages : préparer son poste de développement, lancer le launcher en mode test, et le déployer sur un VPS (site de téléchargement et serveur de mises à jour).

---

## 1. Prérequis

| Outil | Version minimale | Vérification |
|---|---|---|
| Node.js | 20 LTS | `node -v` |
| npm | 10+ | `npm -v` |
| Rust (rustup) | stable récent | `rustc -V` |
| Cargo | fourni avec Rust | `cargo -V` |
| Git | récent | `git --version` |

Dépendances système selon l'OS.

**Windows** : installer Microsoft C++ Build Tools (charge de travail « Développement desktop en C++ ») et WebView2 (déjà présent sur Windows 10/11 à jour).

**macOS** : `xcode-select --install`.

**Linux (Debian/Ubuntu)** :
```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Installer Rust :
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Documentation officielle Tauri 2 : https://v2.tauri.app/start/prerequisites/

---

## 2. Installation du projet

```bash
git clone <url-du-depot> minecraft-launcher
cd minecraft-launcher
npm install
```

Vérifier le backend Rust :
```bash
cd src-tauri
cargo check
cd ..
```

---

## 2 bis. Voir l'interface sans Rust (mode démonstration)

Si Rust ne peut pas s'exécuter sur votre machine (par exemple, Windows bloque `cargo`), l'interface se lance quand même dans le navigateur, depuis la racine du projet (le dossier qui contient `package.json`) :

```bash
npm install        # sous PowerShell : npm.cmd install
npm run dev        # sous PowerShell : npm.cmd run dev
```

Ouvrez ensuite `http://localhost:1420`. Vous verrez un bandeau « Mode démonstration » : les données sont fictives, la connexion Microsoft est simulée et aucun jeu n'est lancé. Pour le vrai launcher, il faut la commande de la section suivante.

## 3. Lancer le launcher en mode développement

```bash
npm run tauri dev
```

La première compilation Rust peut prendre plusieurs minutes. Les suivantes sont beaucoup plus rapides.

Pour un rechargement à chaud du frontend, modifier un fichier dans `src/` suffit. Pour le backend, relancer la commande après une modification dans `src-tauri/`.

---

## 4. Tester

### Tests automatisés

```bash
npm run lint
npm run test
cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test && cd ..
```

### Tests manuels (checklist v0.1)

Utiliser un compte Microsoft qui possède Minecraft Java Edition. Ne jamais tester avec le compte d'une autre personne.

- [ ] Le launcher démarre sans erreur sur l'OS testé.
- [ ] La connexion Microsoft s'ouvre dans le navigateur, puis revient dans le launcher.
- [ ] Un compte sans licence Java est refusé avec un message clair.
- [ ] La déconnexion efface les tokens du trousseau système.
- [ ] Une instance Vanilla se crée, se télécharge et se lance.
- [ ] Le hash d'un fichier corrompu (modifier un `.jar` à la main) est détecté et le fichier retéléchargé.
- [ ] La RAM et la version Java sont modifiables par instance.
- [ ] Les logs sont écrits et ne contiennent ni token, ni mot de passe.
- [ ] Les textes sont en français, sans débordement, en thème clair.

### Où sont les logs

- Windows : `%APPDATA%\com.votre-nom.launcher\logs`
- macOS : `~/Library/Application Support/com.votre-nom.launcher/logs`
- Linux : `~/.local/share/com.votre-nom.launcher/logs`

(Le nom exact dépend de l'identifiant défini dans `tauri.conf.json`.)

---

## 4 bis. Icônes (obligatoire avant le premier build)

Le bundle Tauri attend des icônes. Préparer une image carrée d'au moins 1024×1024 px, sans logo Minecraft, puis :
```bash
npm run tauri icon chemin/vers/logo.png
```
Cela génère `src-tauri/icons/`. Commiter ce dossier une fois généré.

## 5. Compiler un binaire de test

```bash
npm run tauri build
```

Les installeurs se trouvent dans `src-tauri/target/release/bundle/`. Tester l'installeur sur une machine propre (ou une VM) avant chaque diffusion.

---

## 4 quater. Java

Le launcher cherche Java dans `JAVA_HOME` et le PATH, puis dans son propre dossier `runtimes/`. Si la version requise par Minecraft manque (Java 21 pour les versions récentes, 17 ou 8 pour les anciennes), il télécharge automatiquement un JRE Eclipse Temurin depuis Adoptium (connexion Internet requise, aucun droit administrateur nécessaire). Le dossier est dans les données de l'application, à côté de `launcher.sqlite`.

## 4 ter. Intégration continue

Le fichier `.github/workflows/ci.yml` lance à chaque push : lint, tests et build TypeScript, puis `cargo fmt`, `clippy` et `cargo test` sur Linux, Windows et macOS. Une PR qui casse l'un de ces contrôles ne doit pas être fusionnée.

## 6. Enregistrer l'application Microsoft (obligatoire pour la connexion)

Sans cette étape, la connexion Microsoft ne fonctionne pas.

1. Créer une application dans le portail Microsoft Entra (Azure AD) : https://entra.microsoft.com
2. Type de compte : comptes personnels Microsoft.
3. Ajouter une URI de redirection de type « application mobile et de bureau » : `http://localhost:53682` (port défini par `REDIRECT_PORT` dans `src-tauri/src/modules/account_manager/mod.rs`). Le launcher écoute sur ce port pendant la connexion uniquement.
4. Noter l'identifiant client (Client ID). Il n'est pas secret pour une application publique, mais ne le publier qu'après vérification.
5. Demander l'accès aux services Minecraft (API Minecraft Services) via le formulaire prévu par Mojang/Microsoft. Cette étape demande une validation et peut prendre du temps : la lancer tôt.
6. Renseigner le Client ID au moment de la compilation, sans le committer :
```bash
MS_CLIENT_ID=votre-client-id npm run tauri dev
MS_CLIENT_ID=votre-client-id npm run tauri build
```
Sous Windows PowerShell : `$env:MS_CLIENT_ID="votre-client-id"; npm run tauri dev`.

Référence : documentation Microsoft sur l'authentification Minecraft et les flux OAuth 2.0 pour applications de bureau.

---

## 7. Clés d'API tierces

| Service | Usage | Action |
|---|---|---|
| Modrinth | Recherche et installation de mods | Respecter leur User-Agent et leurs limites de débit. Aucune clé requise pour la lecture publique. |
| CurseForge | Mods CurseForge | Demander une clé API via leur console développeur. Clé à placer dans un fichier `.env` local, non versionné. |

Fichier `.env` local (ne jamais le committer) :
```
CURSEFORGE_API_KEY=
MS_CLIENT_ID=
```

Vérifier que `.env` figure dans `.gitignore`.

---

## 8. Déployer sur un VPS

Le VPS sert à trois choses : le site de téléchargement, le serveur de mises à jour du launcher et, plus tard, les actualités et les statistiques. Le launcher lui-même tourne sur les postes des utilisateurs.

### 8.1 Choisir et préparer le serveur

- Distribution recommandée : Ubuntu 24.04 LTS ou Debian 12.
- Ressources de départ : 1 vCPU, 1 Go de RAM, 20 Go de disque suffisent pour les téléchargements et le site.
- Hébergeur en Union européenne si le projet s'adresse à des utilisateurs européens (RGPD).

Connexion et mise à jour :
```bash
ssh root@IP_DU_VPS
apt update && apt upgrade -y
adduser deploy
usermod -aG sudo deploy
```

Pare-feu (UFW) :
```bash
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw enable
```

Désactiver la connexion root par mot de passe et utiliser une clé SSH.

### 8.2 Nom de domaine et HTTPS

1. Acheter un nom de domaine et pointer un enregistrement A vers l'IP du VPS.
2. Installer nginx et Certbot :
```bash
apt install -y nginx certbot python3-certbot-nginx
certbot --nginx -d votre-domaine.fr -d www.votre-domaine.fr
```

### 8.3 Structure sur le serveur

```
/var/www/launcher/
├── downloads/          # Installeurs signés (.msi, .dmg, .AppImage, .deb)
├── updates/            # Fichier latest.json et signatures de mise à jour
└── site/               # Page de téléchargement statique
```

Donner les droits à l'utilisateur `deploy` et à nginx en lecture.

### 8.4 Configuration nginx minimale

Fichier `/etc/nginx/sites-available/launcher` :
```nginx
server {
    listen 80;
    server_name votre-domaine.fr www.votre-domaine.fr;
    root /var/www/launcher/site;
    index index.html;

    location /updates/ {
        alias /var/www/launcher/updates/;
        add_header Cache-Control "no-cache";
    }

    location /downloads/ {
        alias /var/www/launcher/downloads/;
    }

    location / {
        try_files $uri $uri/ =404;
    }
}
```

Activer et tester :
```bash
ln -s /etc/nginx/sites-available/launcher /etc/nginx/sites-enabled/
nginx -t && systemctl reload nginx
```

### 8.5 Mises à jour automatiques (Tauri Updater)

- Générer une paire de clés de signature Tauri : `npm run tauri signer generate -- -w ~/.tauri/launcher.key`.
- Garder la clé privée hors du dépôt et hors du VPS si possible (sur le poste de build).
- Le fichier `updates/latest.json` doit contenir la version, la date, les notes de version, et pour chaque plateforme l'URL de l'installeur et sa signature.
- Ne jamais publier une mise à jour sans signature valide : le launcher doit la refuser.

Documentation : https://v2.tauri.app/plugin/updater/

### 8.6 Sauvegardes du serveur

- Snapshot hebdomadaire du VPS chez l'hébergeur.
- Sauvegarde quotidienne de `/var/www/launcher` et de la configuration nginx.
- Tester une restauration au moins une fois.

### 8.7 Mises à jour de sécurité

```bash
apt install -y unattended-upgrades
dpkg-reconfigure -plow unattended-upgrades
```

---

## 9. Problèmes fréquents

| Symptôme | Cause probable | Solution |
|---|---|---|
| `webkit2gtk` introuvable (Linux) | Dépendances système manquantes | Réinstaller les paquets listés en §1 |
| Fenêtre blanche au lancement | Erreur frontend | Ouvrir les outils de développement (clic droit, Inspecter) et lire la console |
| Connexion Microsoft bloquée | Client ID absent ou URI de redirection incorrecte | Vérifier §6 |
| Compte refusé « licence absente » | Le compte n'a pas Minecraft Java | Normal : utiliser un compte qui possède le jeu |
| Téléchargement qui échoue | Pare-feu ou proxy | Vérifier la connexion aux domaines Mojang et Modrinth |
| Certificat HTTPS invalide | Certbot non renouvelé | `certbot renew --dry-run` |
