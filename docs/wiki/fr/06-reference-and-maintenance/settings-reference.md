# Référence complète des paramètres

La fenêtre des préférences de Berry AI Studio (`SettingsModal.vue`) s'ouvre via `Fichier > Préférences / Paramètres...` ou le raccourci `Ctrl + ,` / `Cmd + ,`. Tous les réglages sont conservés dans le fichier `config.json` de votre répertoire local de données d'application.

---

## Onglet 1 : Préférences générales

| Champ de réglage | Clé dans `config.json` | Valeur par défaut | Description |
| :--- | :--- | :--- | :--- |
| **Langue de l'application** | `locale` | `"auto"` | Choix parmi : `auto` (suit le système d'exploitation), `en` (anglais), `zh-CN` (chinois simplifié), `zh-TW` (chinois traditionnel), `ja` (japonais), `de` (allemand), `fr` (français), `es` (espagnol). |
| **Vue par défaut de la galerie** | `default_view` | `"grid"` | Mode d'affichage initial au démarrage : `"grid"` (Grille uniforme), `"masonry"` (Mosaïque Waterfall) ou `"table"` (Liste détaillée). |
| **Analyse automatique au démarrage** | `auto_scan` | `true` | Vérifie automatiquement les nouveaux fichiers ou modifications dans les dossiers indexés au lancement de Berry. |
| **Délai entre les analyses au démarrage** | `startup_scan_interval_minutes`| `360` | Intervalle minimal en minutes entre deux réconciliations complètes sur le disque (`30`, `60`, `360`, `1440`). Évite de surcharger les disques en cas de redémarrages fréquents. |
| **Vérifier les mises à jour au démarrage** | `auto_check_update` | `true` | Interroge silencieusement GitHub Releases au démarrage et affiche un badge si une version plus récente est disponible. |

---

## Onglet 2 : Affichage & Protection de sécurité

| Champ de réglage | Clé dans `config.json` | Valeur par défaut | Description |
| :--- | :--- | :--- | :--- |
| **Thème de couleur** | `theme` | `"system"` | Palette de l'interface : `"system"`, `"midnight"` (sombre OLED), `"graphite"` (sombre neutre studio), `"violet"` (créatif) ou `"light"` (clair). |
| **Flouter le contenu sensible (NSFW)** | `blur_nsfw` | `true` | Applique un flou CSS sur les créations classées comme adultes ou sensibles jusqu'au clic. |
| **Afficher les badges sur les cartes** | `show_card_badges` | `true` | Affiche le format (`PNG`, `MP4`), la résolution (`1024×1024`), le moteur de génération et les étoiles directement sur les cartes. |
| **Résolution des miniatures** | `thumbnail_max_edge` | `384` | Dimension du bord maximal des miniatures (multiples de 64) : `256` (Compact), `384` (Standard recommandé), `448` (HD), `512` (Ultra). |
| **Budget de cache des miniatures** | `thumbnail_cache_budget_mb` | `2048` | Espace disque maximal (en mégaoctets) alloué au cache WebP. Les paliers les moins récemment consultés (LRU) sont purgés automatiquement en cas de dépassement. |
| **Vider le cache des miniatures** | S/O | S/O | Bouton d'action pour purger immédiatement toutes les miniatures en cache sur le disque. |
| **Diagnostics des miniatures** | S/O | S/O | Ouvre les métriques en temps réel du pool de threads Rayon et du cache LRU. |

---

## Onglet 3 : Piles & Rafales

| Champ de réglage | Clé dans `config.json` | Valeur par défaut | Description |
| :--- | :--- | :--- | :--- |
| **Activer l'empilement automatique** | `auto_stack` | `true` | Regroupe automatiquement les rafales consécutives sous forme de cartes en paquet de jeu. |
| **Seuil de similarité des prompts** | `stack_similarity_threshold` | `0.85` | Score minimal de similarité tokenisée de Jaccard (de 0.0 à 1.0) requis pour grouper des images. |
| **Fenêtre temporelle maximale (Minutes)** | `stack_time_window_minutes` | `180` | Écart maximal de temps entre générations consécutives pour les considérer comme faisant partie de la même session. |
| **Autoriser plusieurs piles ouvertes** | `allow_multiple_open_stacks` | `false` | Si `false`, ouvrir une pile replie automatiquement les autres. Si `true`, plusieurs piles peuvent demeurer déployées en même temps. |
| **Réinitialiser les avertissements** | S/O | S/O | Rétablit les boîtes de confirmation (ex. avertissements de fusion de piles) si vous aviez coché « Ne plus afficher ». |

---

## Onglet 4 : Interopérabilité de génération

| Champ de réglage | Clé dans `config.json` | Valeur par défaut | Description |
| :--- | :--- | :--- | :--- |
| **URL de base ComfyUI** | `comfyui_url` | `"http://127.0.0.1:8188"` | Point de terminaison HTTP de votre instance locale ComfyUI. Inclut un bouton de test de connexion. |
| **URL de base SD WebUI** | `webui_url` | `"http://127.0.0.1:7860"` | Point de terminaison HTTP de votre serveur AUTOMATIC1111 / Forge / SD.Next. Inclut un bouton de test de connexion. |

---

## Onglet 5 : Collaboration d'équipe & Base de données

| Champ de réglage | Clé dans `config.json` | Valeur par défaut | Description |
| :--- | :--- | :--- | :--- |
| **Moteur de base de données** | `storage_backend` | `"sqlite"` | Moteur actif : `"sqlite"`, `"mysql"` ou `"postgres"`. |
| **URL de connexion distante** | `remote_connection_url` | `""` | Chaîne de connexion à la base de données (ex. `postgres://user:pass@host:5432/berry_studio`). |
| **Identifiant du poste client** | `client_identifier` | `""` | Nom unique de la machine consigné dans les journaux de modifications et pour la résolution des conflits OCC. |
| **Mappage des racines de stockage** | `root_mappings` | `{}` | Associations liant les UUIDs universels des racines aux chemins de montage locaux du système d'exploitation. |
| **Tester la connexion** | S/O | S/O | Envoie un ping au serveur de base de données distant et affiche la latence aller-retour en millisecondes. |
| **Assistant de migration** | S/O | S/O | Ouvre l'exportateur de schéma et de données SQLite vers MySQL/PostgreSQL. |

---

## Onglet 6 : Sauvegarde Cloud & Miroir multimédia

| Champ de réglage | Clé dans `config.json` | Valeur par défaut | Description |
| :--- | :--- | :--- | :--- |
| **Fournisseur de stockage** | `cloud_backup.provider` | `"local_path"` | Protocole : `"local_path"`, `"webdav"` ou `"s3"`. |
| **URL / Identifiants WebDAV** | `cloud_backup.webdav_*` | `""` | URL du point de terminaison, nom d'utilisateur et mot de passe pour WebDAV Nextcloud / Synology. |
| **Paramètres S3 / Identifiants** | `cloud_backup.s3_*` | `""` | Point de terminaison, nom du compartiment (bucket), région, clé d'accès et clé secrète pour S3 / Cloudflare R2 / MinIO. |
| **Stratégie delta** | `cloud_sync.strategy` | `"fingerprint"` | `"fingerprint"` (taille + ETag) ou `"checksum"` (vérification SHA-256 intégrale). |
| **Threads de transfert** | `cloud_sync.threads` | `4` | Nombre de threads d'exécution simultanés pour les transferts cloud. |
| **Limite de bande passante** | `cloud_sync.bandwidth_limit_kbs` | `0` | Vitesse maximale d'envoi en Ko/s (0 = illimitée). |

---

## Onglet 7 : Moteurs d'extraction de métadonnées intégrés

Présente l'état de fonctionnement en temps réel des analyseurs de métadonnées sans perte intégrés :
- Analyseur de blocs PNG `parameters` d'AUTOMATIC1111 / SD.Next (Actif 🟢)
- Analyseur de graphes d'exécution et JSON `workflow` de ComfyUI (Actif 🟢)
- Analyseur de blocs `Comment` et `Description` de NovelAI (Actif 🟢)
- Analyseur de paramètres de prompts de Fooocus / Fooocus-MRE (Actif 🟢)
- Analyseur de structures JSON `sd-metadata` et `invokeai_metadata` d'InvokeAI (Actif 🟢)
- Analyseur de conteneurs vidéo MP4 ISOBMFF et WebM EBML (Actif 🟢)

---

## Onglet 8 : Stockage de données & À propos

- **Version** : Affiche la version actuelle de l'application (ex. `v0.3.0`).
- **Schéma de base de données** : Indique le niveau de migration actif du schéma SQLite (ex. `Schéma version 14`).
- **Emplacement de la base active** : Chemin absolu vers `berry.db`.
- **Boutons d'accès direct aux dossiers** :
  - `Fichier de configuration` : Ouvre le dossier contenant `config.json`.
  - `Base de données SQLite` : Ouvre le dossier contenant `berry.db` et les fichiers journaux WAL.
  - `Cache des miniatures` : Ouvre le répertoire de cache des miniatures WebP.
  - `Dossier des modèles` : Ouvre le répertoire contenant les poids de modèles d'IA ONNX.
