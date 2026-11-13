# Politique de sécurité

## Signaler une vulnérabilité

Merci de ne **pas** ouvrir d'issue publique pour une faille de sécurité.

Envoyez un message privé à : [E-MAIL DE SÉCURITÉ]

Précisez :
- la version concernée et le système d'exploitation,
- les étapes pour reproduire le problème,
- l'impact que vous estimez.

Délais visés : accusé de réception sous 72 heures, première évaluation sous 14 jours. Une correction est publiée dans une mise à jour signée, puis le problème est documenté dans `UPDATE.md`.

## Périmètre

Dans le périmètre :
- authentification et gestion des jetons,
- téléchargements et vérification des hash,
- écriture de fichiers et traversée de chemins,
- permissions Tauri et communication avec l'interface,
- mécanisme de mise à jour.

Hors périmètre :
- les services de Mojang, Microsoft, Modrinth et CurseForge (signalez-les à leurs éditeurs),
- les mods tiers installés par l'utilisateur.

## Engagements du projet

- Aucun mot de passe Microsoft n'est collecté ni stocké.
- Les tokens sont dans le trousseau système, jamais en clair.
- Les téléchargements sont vérifiés par hash avant utilisation.
- Les mises à jour sont signées ; une mise à jour non signée est refusée.

## Recherche responsable

Ne testez que vos propres comptes et vos propres installations. Ne tentez pas d'accéder aux comptes d'autres personnes ni de perturber les services de tiers.
