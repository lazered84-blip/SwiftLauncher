# Launcher Minecraft

Launcher Minecraft indépendant, gratuit et multiplateforme (Windows, Linux, macOS), conçu pour la simplicité d'usage, la gestion de modpacks et une expérience optimisée pour le **PvP**.

> **Avertissement** : ce projet n'est pas affilié à Mojang AB, Mojang Synergies AB ni à Microsoft. Minecraft est une marque déposée de Mojang Synergies AB. Le launcher ne fonctionne que pour les utilisateurs possédant une licence officielle de Minecraft Java Edition.

## Fonctionnalités

- **Dashboard** : lancement rapide, instance récente, actualités, notifications, téléchargements en cours.
- **Instances** : création, duplication, import/export, configurations isolées, versions Minecraft, Java et RAM.
- **Mods** : intégration Modrinth (et CurseForge sous réserve d'autorisation), recherche, installation, mises à jour, vérification des dépendances et des incompatibilités.
- **Loaders** : Vanilla, Fabric, Forge, NeoForge, Quilt.
- **Comptes** : connexion Microsoft officielle (OAuth 2.0), vérification de la licence Java, plusieurs comptes, sessions renouvelées de façon sécurisée. Aucun mot de passe Microsoft n'est stocké.
- **Performances** : détection matérielle et Java, profils de performance, analyse des crashs, recommandations.
- **Sauvegardes** : mondes et instances, historique, restauration, export.
- **PvP** : profils de performance orientés combat, suivi de latence et de FPS, mode « compétition » légitime (voir `docs/FONCTIONNALITES_PVP.md`).

Ce que le launcher **ne fait pas** : aucun cheat, aucun module qui donne un avantage illégitime en jeu, aucune contournement de l'anti-cheat ou des règles d'un serveur.

## Direction artistique

Thème bleu clair dominant, coins arrondis, ombres discrètes, animations légères.

| Rôle | Couleur |
|---|---|
| Bleu principal | `#60A5FA` |
| Bleu secondaire | `#93C5FD` |
| Bleu très clair | `#DBEAFE` |
| Fond principal | `#F5F9FF` |
| Fond des cartes | `#FFFFFF` |
| Texte principal | `#172554` |
| Texte secondaire | `#64748B` |

Un thème sombre sera ajouté plus tard.

## Stack technique

- **Frontend** : React, TypeScript, Vite
- **Backend** : Tauri 2, Rust
- **Base de données** : SQLite
- **Tests** : Vitest (front), `cargo test` (Rust)

## Structure du dépôt

```
minecraft-launcher/
├── src/                  # Frontend React + TypeScript
├── src-tauri/            # Backend Rust (Tauri 2)
│   └── src/modules/      # minecraft_core, instance_manager, mod_manager, ...
├── docs/                 # Guides et spécifications
├── legal/                # CGU, confidentialité, mentions, conformité
├── README.md
├── CLAUDE.md             # Instructions pour les agents IA qui travaillent sur le code
├── UPDATE.md             # Journal des modifications (à mettre à jour à chaque changement)
└── LICENSE
```

## Démarrage rapide

Voir le guide complet : [`docs/GUIDE_INSTALLATION.md`](docs/GUIDE_INSTALLATION.md).

```bash
npm install
npm run tauri dev
```

## Documentation

- [Guide d'installation et de test](docs/GUIDE_INSTALLATION.md)
- [Fonctionnalités, PvP et pistes d'amélioration](docs/FONCTIONNALITES_PVP.md)
- [Conformité légale](legal/CONFORMITE.md)
- [Mentions légales](legal/MENTIONS_LEGALES.md) · [CGU](legal/CGU.md) · [Confidentialité](legal/CONFIDENTIALITE.md)
- [Journal des modifications](UPDATE.md)

## Licence

Voir [LICENSE](LICENSE). Le code du launcher est distinct de Minecraft, dont les droits appartiennent à Mojang Synergies AB et Microsoft.
