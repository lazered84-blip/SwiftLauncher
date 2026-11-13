# Fonctionnalités, PvP et pistes d'amélioration

Ce document précise comment le launcher peut être optimisé pour le PvP sans franchir la ligne de la triche, puis liste les fonctionnalités à ajouter et à améliorer.

---

## 1. Principe directeur

Le launcher peut rendre le jeu **plus fluide, plus stable et plus lisible**. Il ne doit pas donner d'avantage en combat qui n'est pas accessible à tous les joueurs.

Règle simple : si une fonctionnalité modifie ce que le joueur **voit de l'information du jeu** ou **ce que le serveur reçoit comme actions**, elle est exclue. Si elle améliore uniquement **les performances, la configuration et le diagnostic** du poste du joueur, elle est possible.

---

## 2. Ce qui est autorisé pour le PvP

### 2.1 Performances

- **Profils PvP** : réglages prêts à l'emploi (allocation RAM, Java recommandé, JVM flags éprouvés, priorité du processus).
- **Détection matérielle** : CPU, GPU, RAM, stockage, et proposition de réglages adaptés.
- **Gestion de la RAM** : suggestions selon le nombre de mods et l'historique de crash.
- **Mode performance rapide** : désactive les processus de fond inutiles du launcher pendant le jeu.
- **Limitation de l'usage du launcher** : le launcher se réduit en veille pendant la session de jeu.

### 2.2 Suivi (sans influence sur le jeu)

- **Affichage des FPS, du ping et du TPS** via un overlay optionnel, indépendant du jeu, qui ne lit que des données publiques (compteur de frames, mesure de latence réseau).
- **Historique de session** : ping moyen, FPS minimum, nombre de crashs par serveur.
- **Comparaison des performances** entre deux profils ou deux versions de Java, sur le même scénario.

### 2.3 Stabilité

- **Analyse des crashs** : lecture des rapports, identification du mod ou de la version responsable, suggestion de correctif.
- **Vérification des incompatibilités** avant lancement.
- **Sauvegarde automatique** avant chaque mise à jour de mod.
- **Restauration rapide** après une mise à jour problématique.

### 2.4 Serveurs

- **Favoris et ping** : liste des serveurs avec latence mesurée.
- **Profils par serveur** : un serveur PvP peut avoir son propre profil (mods autorisés, RAM, résolution).
- **Liste de mods autorisés par serveur** : fournie par l'administrateur du serveur, pour que le joueur sache ce qui est accepté. Le launcher n'applique pas de règle de jeu, il informe.

---

## 3. Ce qui est exclu

Ces fonctionnalités ne seront jamais implémentées dans le launcher, quelle que soit la demande :

- Modules de combat : reach, killaura, aimbot, triggerbot, auto-clic, velocity, anti-knockback.
- Modules de vision : xray, ESP, radar, wallhack, minimap de cartes cachées, affichage des joueurs à travers les murs.
- Modules d'information cachée : affichage de la vie ou de l'inventaire d'autres joueurs, prédiction de position.
- Automatisations : macros d'inventaire, automatisation des clics ou des touches, bots.
- Contournement : bypass d'anti-cheat, masquage de module, faux rapport de conformité.
- Tout module qui se présente comme « indétectable » ou « safe » pour un serveur.

Les interfaces qui mettent en avant ces fonctions, comme certains clients du marché avec un badge « Anti-Cheat Active » ou « Tournament Ready » accolé à des modules de triche, ne servent pas de référence pour ce projet. On retient de ces interfaces uniquement l'aspect visuel moderne, pas les fonctions.

---

## 4. Pistes de fonctionnalités à ajouter

Classées par impact et par effort estimé.

| Fonctionnalité | Intérêt | Version |
|---|---|---|
| Profils PvP prêts à l'emploi | Gain immédiat pour l'utilisateur | 0.5 |
| Détection matérielle et Java | Réglages adaptés sans expertise | 0.5 |
| Analyse de crash | Résout les problèmes les plus frustrants | 0.5 |
| Overlay FPS / ping léger | Visibilité sur ses performances | 0.5 |
| Favoris serveurs avec ping | Pratique pour les joueurs PvP | 0.5 |
| Historique de sessions | Suivi de progression | 0.5 |
| Sauvegarde avant mise à jour | Sécurité des données | 0.2 |
| Gestion multi-comptes | Usage courant | 0.1 |
| Partage d'instance par code | Entraide entre joueurs | 0.5 |
| Comparaison de performances | Choix éclairé des réglages | 1.0 |
| Synchronisation cloud facultative | Confort multi-appareils | 1.0 |
| Thème sombre | Demandé pour plus tard | 1.0 |
| Extensions communautaires | Écosystème à terme | 1.0+ |
| Assistant de diagnostic | Explication des erreurs en français | 0.5 |
| Mode hors ligne sans serveur obligatoire | Fidèle à l'objectif de départ | 0.1 |

---

## 5. Améliorations proposées sur l'existant

Idées pour que le launcher se démarque des concurrents (SKlauncher, Prism Launcher, ATLauncher, et autres) :

1. **Tableau de bord orienté action** : une seule carte « Jouer maintenant » avec l'instance, le serveur favori et l'état de santé du poste (vert, orange, rouge).
2. **Score de santé d'une instance** : une note simple combinant compatibilité, mises à jour disponibles, crashs récents et RAM.
3. **Assistant en français** : explication des erreurs de crash en langage clair, avec action proposée en un clic.
4. **Journal de sessions visuel** : graphique de FPS et de ping sur la dernière partie.
5. **Modpacks lisibles** : affichage des mods par catégorie (performance, visuel, utilitaire, PvP légal) avec les dépendances en clair.
6. **Import direct** depuis un lien Modrinth ou un fichier `.mrpack`.
7. **Mode simple / mode expert** : l'interface par défaut masque les réglages avancés, accessibles en un clic.
8. **Raccourcis clavier** : lancer, changer d'instance, ouvrir les logs.
9. **Notifications utiles uniquement** : mise à jour disponible, serveur favori en ligne, crash détecté. Pas de bruit.
10. **Accessibilité** : taille de texte réglable, contrastes renforcés, support du clavier complet.

---

## 6. Actualités et communauté

- Flux d'actualités Minecraft alimenté depuis une source officielle ou un flux RSS public, avec mise en cache sur le VPS.
- Pas de publicité intrusive. Si un modèle de financement est envisagé plus tard, il devra rester discret et transparent, sans rien vendre aux dépens de la sécurité.

---

## 7. Décisions à prendre

- Choisir la licence finale du code (MIT, GPL-3.0, Apache-2.0 ou autre).
- Décider si l'overlay FPS/ping est une fenêtre séparée ou une option du jeu. Dans les deux cas, il ne doit lire que des données publiques.
- Définir le nom final, le logo et l'identifiant d'application (`tauri.conf.json`).
- Choisir l'hébergeur VPS et le nom de domaine.
