# Conformité légale

Ce document recense ce qui doit être vrai avant chaque publication. Il ne remplace pas l'avis d'un professionnel du droit : pour une diffusion publique, faire relire les documents par un avocat, surtout les CGU et la politique de confidentialité.

---

## 1. Propriété intellectuelle de Minecraft

- [ ] Le launcher ne contient aucun actif officiel de Minecraft (logo, textures, sons, musiques, fichiers du jeu). Les fichiers du jeu sont téléchargés depuis les serveurs officiels de Mojang à la demande de l'utilisateur.
- [ ] Le nom du launcher n'utilise pas « Minecraft » seul ni de façon qui laisse croire à une affiliation. Formulation acceptée : « Launcher compatible avec Minecraft » ou « pour Minecraft ».
- [ ] Le disclaimer suivant figure dans le README, les mentions légales, l'écran « À propos » et la page de téléchargement :
  > Ce projet n'est pas affilié à Mojang AB, Mojang Synergies AB ni à Microsoft. Minecraft est une marque déposée de Mojang Synergies AB.
- [ ] Le logo du launcher est original et ne reprend pas le bloc, la police ou le logo Minecraft.
- [ ] Respecter les règles d'utilisation de la marque Minecraft publiées par Mojang, qui évoluent : les relire avant chaque version majeure.
- [ ] Le contrat de licence utilisateur final de Minecraft (EULA) est respecté : le launcher ne modifie pas le jeu de façon à le redistribuer, et ne donne pas accès à un jeu non licencié.

## 2. Authentification et licence du jeu

- [ ] La connexion passe uniquement par le flux officiel Microsoft OAuth 2.0 et les services Minecraft.
- [ ] Le launcher vérifie que le compte possède Minecraft Java Edition avant de lancer le jeu.
- [ ] Aucun mot de passe Microsoft n'est saisi dans le launcher ni stocké.
- [ ] Les tokens sont stockés dans le trousseau système, jamais en clair.
- [ ] La demande d'accès aux services Minecraft (application enregistrée) a été faite et acceptée avant toute diffusion publique.
- [ ] Aucune fonction de contournement de la vérification de licence n'existe, ni en code, ni en option, ni en documentation.

## 3. Interdiction de la triche

- [ ] Aucun module de triche (voir `docs/FONCTIONNALITES_PVP.md`, section 3).
- [ ] Aucune fonction de contournement d'anti-cheat.
- [ ] Les textes marketing ne promettent aucun avantage en combat et n'affirment pas qu'un module est indétectable.
- [ ] Les règles des serveurs sont respectées : le launcher n'impose pas de mod, il en informe seulement.

## 4. Services tiers

| Service | Conditions à respecter | Action |
|---|---|---|
| Modrinth | Conditions d'utilisation de l'API, User-Agent identifiant l'application, limites de débit | Vérifier la page des conditions avant publication. Citer Modrinth et ne jamais redistribuer leurs données hors des règles. |
| CurseForge | Clé API obligatoire, autorisation nécessaire pour distribuer les téléchargements, règles de marque | Demander l'autorisation avant d'activer l'intégration. Sans autorisation, la fonction reste désactivée. |
| Mojang (manifests, assets, bibliothèques) | Téléchargement depuis les hôtes officiels, vérification des hash | Ne pas héberger les fichiers du jeu sur le VPS. |
| Microsoft Entra / Minecraft Services | Enregistrement de l'application, acceptation des conditions | Suivre la procédure d'enregistrement. |
| Actualités Minecraft | Source publique, mention de l'auteur, lien vers la source | Ne pas copier les articles : afficher un résumé et un lien. |

## 5. Licences logicielles

- [ ] Inventaire des dépendances tenu à jour (npm et Cargo), avec leur licence.
- [ ] Aucune dépendance à licence incompatible avec la licence choisie pour le projet.
- [ ] Les mentions obligatoires des licences tierces figurent dans un fichier `THIRD_PARTY_LICENSES` généré avant chaque release.
- [ ] Outils recommandés : `cargo-about` pour Rust, `license-checker` pour npm.

Dépendances à vérifier en priorité : Tauri (MIT ou Apache-2.0), React (MIT), SQLite (domaine public), et tout SDK d'authentification ajouté.

## 6. Données personnelles (RGPD)

- [ ] Données collectées limitées au strict nécessaire : aucune donnée obligatoire pour utiliser le launcher hors connexion Microsoft.
- [ ] Données locales (instances, préférences) restent sur le poste de l'utilisateur.
- [ ] Télémétrie désactivée par défaut. Si elle existe, elle est opt-in, anonymisée, et décrite dans `legal/CONFIDENTIALITE.md`.
- [ ] Le VPS est hébergé dans l'Union européenne, ou les transferts hors UE sont encadrés.
- [ ] Journaux serveur (nginx) conservés une durée limitée et documentée.
- [ ] Droits des personnes (accès, rectification, effacement, opposition) accessibles par un contact indiqué dans les mentions légales.
- [ ] Un registre simplifié des traitements est tenu.

## 7. Diffusion et édition

- [ ] Mentions légales complètes : éditeur (nom, adresse, contact), hébergeur du VPS, directeur de publication si requis.
- [ ] CGU acceptées ou consultables avant la première utilisation.
- [ ] Signature des installeurs (Windows et macOS) pour éviter les alertes de sécurité. Certificat de signature hors du dépôt.
- [ ] Les mises à jour sont signées (Tauri Updater) et le launcher refuse toute mise à jour non signée.
- [ ] Accessibilité : la page de téléchargement et l'interface respectent les critères de base WCAG AA.

## 8. Avant chaque release publique

- [ ] Relecture de cette checklist.
- [ ] Mise à jour de `UPDATE.md`.
- [ ] Vérification des dates des documents légaux.
- [ ] Test de l'installeur sur une machine propre.
- [ ] Vérification que le disclaimer est présent.
