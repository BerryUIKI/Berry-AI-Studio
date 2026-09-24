# Omera — Wiki officiel & Guide d'utilisation

Bienvenue dans la documentation officielle et la base de connaissances de **Omera** (`v0.3.0`).

Omera est un gestionnaire d'actifs multimédias et un studio de prompts open-source, axé sur le local (« local-first »), spécialement conçu pour les créateurs d'IA générative, les ingénieurs de prompts et les studios de design visuel. Conçu sur **Tauri v2**, **Rust** et **Vue 3**, il gère des bibliothèques allant de quelques centaines d'œuvres à plus de 500 000 fichiers avec une latence de requête inférieure à la milliseconde, zéro dépendance cloud et une extraction exhaustive des métadonnées de génération.

---

## 🧭 Navigation & Table des matières

### [Chapitre 1 : Prise en main & Fondamentaux](01-getting-started/installation.md)
- **[Installation & Prérequis système](01-getting-started/installation.md)** : Prérequis matériels, options d'installation et versions portables Windows, builds macOS Universal & Apple Silicon, paquets Linux AppImage/deb, et l'assistant de bienvenue au premier démarrage.
- **[Espace de travail & Anatomie de l'interface](01-getting-started/workspace-layout.md)** : Analyse détaillée de la fenêtre sans cadre, de la barre de menus, de la disposition en 3 volets (Barre latérale, Galerie, Inspecteur), de la barre d'état et de la barre d'actions par lots flottante.
- **[Guide des raccourcis clavier](01-getting-started/keyboard-shortcuts.md)** : Raccourcis globaux, ancres de sélection, notation aveugle par étoiles, inspection rapide et touches de navigation.

### [Chapitre 2 : Gestion de la bibliothèque & Navigation](02-library-management/folder-modes-and-import.md)
- **[Importation de médias & Modes de dossiers](02-library-management/folder-modes-and-import.md)** : Comparaison entre le Mode A (Lien externe), le Mode B (Coffre géré) et le Mode C (Pipeline AIGC avec surveillance anti-rebond et recyclage différé). Formats d'images (PNG, WebP, JPEG) et de vidéos (MP4, WebM) pris en charge.
- **[Modes de galerie & Options d'affichage](02-library-management/gallery-views.md)** : Maîtrise de la Grille uniforme (zoom de 130px à 360px), de la Mosaïque (proportions d'origine), de la Vue Tableau et de la vue de similarité visuelle. Badges sur les cartes et flou de confidentialité NSFW.
- **[Organisation, Notes & Tags](02-library-management/organization-and-tags.md)** : Échelle de notation sur 10 étoiles, favoris, albums personnalisés, taxonomie de tags en 8 couleurs, glisser-déposer par lots et barre d'outils d'actions groupées.
- **[Prise en charge des vidéos & médias animés](02-library-management/video-support.md)** : Lecture AnimateDiff, Wan2.1, HunyuanVideo et SVD ; saut d'image par image (frame stepping), affichage tête haute boucle/vitesse, et inspection des workflows vidéo intégrés.

### [Chapitre 3 : Découverte, Recherche & Analyses](03-discovery-and-analytics/search-and-filtering.md)
- **[Syntaxe de recherche & Filtres visuels](03-discovery-and-analytics/search-and-filtering.md)** : Langage de requête clé-valeur avancé (`prompt:`, `neg:`, `model:`, `cfg:>=7`, `steps:20..40`), plages numériques et tiroir de filtres latéral.
- **[Métadonnées AIGC & Inspection de prompts](03-discovery-and-analytics/metadata-and-prompts.md)** : Analyse sans perte pour AUTOMATIC1111, ComfyUI, NovelAI, Fooocus, InvokeAI ; étiquettes de prompts interactives tokenisées et graphes d'exécution bruts.
- **[Analyses de prompts & Statistiques](03-discovery-and-analytics/prompt-insights.md)** : Distributions de fréquence des tokens à l'échelle de la bibliothèque, classements des prompts positifs/négatifs et corrélation avec les notes attribuées par l'utilisateur.

### [Chapitre 4 : Curation intelligente & Moteurs IA](04-intelligent-curation/stacks-and-bursts.md)
- **[Piles d'images, Rafales & Comparaison](04-intelligent-curation/stacks-and-bursts.md)** : Regroupement automatique en rafales (similarité de prompt Jaccard + fenêtres temporelles), cartes en paquet de jeu, images de couverture, aplatissement sécurisé de piles, outil Élaguer les brouillons et mode de comparaison côte à côte (`C`).
- **[Recherche sémantique IA & Étiquetage automatique](04-intelligent-curation/ai-semantic-and-tagger.md)** : Requêtes en langage naturel texte-vers-image locales ONNX CLIP/SigLIP, plus proches voisins visuels image-vers-image, et étiquetage automatique anime WD14 Danbooru.
- **[Modèles Checkpoint & Bibliothèque LoRA](04-intelligent-curation/models-and-loras.md)** : Catalogage automatique des checkpoints, résolution des hashs via le `cache.json` d'A1111, recherche inversée Civitai, gestionnaire de mots déclencheurs LoRA et injection de prompts en 1 clic.
- **[Interopérabilité des outils de génération](04-intelligent-curation/generation-interop.md)** : Communication directe par API avec ComfyUI (`/prompt`) et AUTOMATIC1111 (`/sdapi/v1/txt2img`), avec vérification d'état de connexion en direct.

### [Chapitre 5 : Exportation, Sauvegarde Cloud & Collaboration](05-export-and-collaboration/export-and-web-showcase.md)
- **[Exportation par lots, Transcodage & Galerie Web](05-export-and-collaboration/export-and-web-showcase.md)** : Transcodage multithread avec Rayon, assainissement des métadonnées de confidentialité sur 4 niveaux, modèles dynamiques de noms de fichiers, archives ZIP et génération de vitrines HTML autonomes interactives en fichier unique.
- **[Sauvegarde d'instantanés Cloud & Miroir multimédia](05-export-and-collaboration/cloud-backup-and-sync.md)** : Instantanés SQLite à chaud via `VACUUM INTO` vers AWS S3, Cloudflare R2, MinIO, WebDAV ou NAS local ; synchronisation différentielle incrémentielle avec détection ETag/SHA-256 et limitation de bande passante.
- **[Studio d'équipe multi-bases de données](05-export-and-collaboration/team-collaboration.md)** : Évolution au-delà de SQLite vers des serveurs partagés MySQL 8.0+ ou PostgreSQL 14+ ; mappage des racines de stockage multiplateformes (normalisation des lettres de lecteurs Windows en chemins macOS/Linux), contrôle de concurrence optimiste (OCC) et mise en cache des miniatures NVMe côté client.

### [Chapitre 6 : Référence système & Maintenance](06-reference-and-maintenance/settings-reference.md)
- **[Référence complète des paramètres](06-reference-and-maintenance/settings-reference.md)** : Guide exhaustif des paramètres à travers les 8 onglets de préférences.
- **[Maintenance de la base de données & du cache](06-reference-and-maintenance/database-maintenance.md)** : Compactage SQLite WAL (`VACUUM`), sauvegarde/restauration de la base de données, budget de cache des miniatures (éviction LRU) et diagnostics de la file d'attente.
- **[Mises à jour & Cycle de vie](06-reference-and-maintenance/updating.md)** : Système de mise à jour automatique intégré, garanties de préservation des données entre les versions et mises à niveau manuelles.
- **[Architecture de confidentialité & Sécurité](06-reference-and-maintenance/privacy-and-security.md)** : Modèle 100 % local-first hors ligne, zéro télémétrie, isolation de l'inférence IA locale et licence AGPL-3.0.
- **[Dépannage & FAQ](06-reference-and-maintenance/troubleshooting-and-faq.md)** : Solutions aux problèmes courants, conseils d'optimisation des performances et questions fréquemment posées.
- **[Glossaire du produit](06-reference-and-maintenance/glossary.md)** : Définitions de référence des termes métier (Image de couverture/Hero, Pipeline d'ingestion, Similarité de Jaccard, Curseur Keyset, OCC, Carte en paquet de jeu, etc.).

---

## ⚡ Raccourcis de démarrage rapide

| Action | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **Ouvrir l'aperçu Lightbox (Aperçu rapide)** | `Espace` / `Entrée` | `Espace` / `Retour` |
| **Comparaison côte à côte** | `C` | `C` |
| **Grouper dans une pile** | `Ctrl + G` | `Cmd + G` |
| **Dissocier la pile** | `Ctrl + Maj + G` | `Cmd + Shift + G` |
| **Définir comme couverture de la pile** | `Alt + S` | `Option + S` |
| **Noter la sélection de 1 à 5 étoiles** | `1` – `5` (`0` pour effacer) | `1` – `5` (`0` pour effacer) |
| **Basculer le statut Favori** | `F` | `F` |
| **Activer la barre de recherche** | `/` ou `Ctrl + F` | `/` ou `Cmd + F` |
| **Afficher/Masquer l'inspecteur** | `I` | `I` |
| **Afficher/Masquer la barre latérale** | `B` | `B` |
| **Ouvrir les Préférences** | `Ctrl + ,` | `Cmd + ,` |
