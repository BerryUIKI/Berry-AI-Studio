# Dépannage & Foire Aux Questions (FAQ)

Ce guide répertorie les questions fréquentes, les situations particulières et les procédures de diagnostic et dépannage pour Berry AI Studio.

---

## 1. Résolution des problèmes fréquents

### Problème : Les nouvelles créations de mon générateur IA n'apparaissent pas dans la galerie
- **Origine** : Le dossier n'est peut-être pas configuré en tant que Pipeline d'ingestion actif, ou l'observateur du système de fichiers attend la fin complète du rendu (période d'inactivité anti-rebond).
- **Solution** :
  1. Vérifiez que le dossier est bien répertorié dans la barre latérale gauche.
  2. Si vous utilisez AUTOMATIC1111 ou ComfyUI, assurez-vous que le dossier est configuré en **Mode C : Pipeline** afin que l'anti-rebond de verrouillage en écriture soit actif.
  3. Faites un clic droit sur le dossier dans la barre latérale et cliquez sur **« Récolter les images »** ou **« Scanner le dossier »** pour forcer une réconciliation immédiate.

### Problème : Les miniatures mettent du temps à charger ou affichent des cadres d'attente
- **Origine** : Lors de grands imports initiaux, les threads de calcul d'arrière-plan Rayon réduisent les images par paquets successifs.
- **Solution** :
  1. Cliquez sur **⚡ Activité** dans la barre d'état inférieure pour vérifier si la file de génération des miniatures est active.
  2. Dans **Préférences > Galerie**, vérifiez que votre **Budget de cache des miniatures** est fixé à au moins `2048 Mo`.
  3. Évitez de lancer de lourds rendus vidéo ou des jeux vidéo exigeants simultanément lors du premier import volumineux.

### Problème : La recherche renvoie 0 résultat alors que le mot figure bien dans le prompt
- **Origine** : Vous êtes peut-être en **Mode sémantique (`🧠`)** au lieu du **Mode syntaxique (`🔍`)**, ou un filtre restrictif est resté appliqué.
- **Solution** :
  1. Vérifiez l'icône dans la barre de recherche : cliquez sur l'icône de cerveau pour repasser en **Recherche syntaxique (`🔍`)**.
  2. Si vous cherchez une expression exacte composée de plusieurs mots, entourez-la de guillemets : `prompt:"cyberpunk city"`.
  3. Ouvrez le **Tiroir de filtres (`☰ Filtres`)** et cliquez sur **« Réinitialiser »** pour vérifier qu'aucun filtre actif (comme un filtre 5 étoiles) ne masque vos résultats.

### Problème : Des piles apparaissent séparées ou des cartes semblent en double
- **Origine** : Les piles peuvent se fractionner si des images membres ont été renommées ou déplacées en dehors de Berry via l'explorateur de votre système d'exploitation.
- **Solution** : Sélectionnez les cartes concernées dans la galerie et appuyez sur `Ctrl + G` pour les réunir proprement en une seule pile consolidée.

### Problème : « Envoyer à ComfyUI » signale une connexion refusée
- **Origine** : ComfyUI n'est pas démarré localement, ou fonctionne sur un port d'écoute différent.
- **Solution** :
  1. Ouvrez **Préférences > Métadonnées** (ou la section d'interopérabilité).
  2. Vérifiez que l'**URL de base ComfyUI** concorde avec les indications de votre terminal (par défaut : `http://127.0.0.1:8188`).
  3. Cliquez sur **« Tester la connexion »** pour vous assurer que le service est accessible.

---

## 2. Foire Aux Questions (FAQ)

### Berry AI Studio est-il entièrement gratuit ?
Oui. Berry AI Studio est un logiciel libre et open-source sous licence **AGPL-3.0**. Il ne comporte aucun abonnement, aucune fonctionnalité payante bloquée ni mur payant.

### Berry AI Studio peut-il gérer des bibliothèques de 100 000 ou 500 000+ fichiers ?
Oui. Berry a été conçu dès le départ pour manipuler d'immenses collections. Il met en œuvre :
- **Une pagination profonde par curseur (Keyset Cursor)** (`search_files_cursor_page`) conservant une latence de requête inférieure à la milliseconde quelle que soit la taille de la base.
- **SQLite en mode Write-Ahead Logging (WAL)** garantissant des lectures simultanées sans blocage à très haut débit.
- **Une virtualisation dynamique du DOM** qui n'affiche dans l'interface que les éléments strictement visibles dans votre fenêtre.

### Berry AI Studio téléverse-t-il mes prompts ou mes images sur le cloud ?
Non. L'analyse des fichiers, l'extraction des métadonnées, le stockage en base de données et l'inférence par IA (CLIP et WD14) fonctionnent à 100 % en local sur votre machine. Aucune donnée d'utilisation ni télémétrie n'est collectée.

### Puis-je glisser-déposer des images depuis Berry directement vers ComfyUI ou Discord ?
Oui. Glisser une carte d'image depuis la galerie directement vers votre navigateur web ou une application de bureau externe transmet les données de fichier natives du système d'exploitation, préservant l'intégralité des métadonnées intégrées.

### Que se passe-t-il si je supprime un dossier de la barre latérale gauche ?
Supprimer un dossier dans Berry AI Studio retire le dossier et ses fiches d'indexation de la base de données de Berry. **Cela ne supprime ni ne déplace jamais vos fichiers médias physiques sur votre disque.**

### Plusieurs membres d'une même équipe peuvent-ils collaborer sur la même bibliothèque ?
Oui. En passant de SQLite à une base de données partagée **MySQL 8.0+** ou **PostgreSQL 14+** dans **Préférences > Équipe & Base de données**, plusieurs artistes peuvent se connecter simultanément à une bibliothèque réseau avec synchronisation des modifications en temps réel et mappage multiplateforme des chemins.
