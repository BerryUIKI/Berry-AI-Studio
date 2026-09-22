# Glossaire du produit

Ce glossaire définit les principaux termes métier, les concepts architecturaux et les fonctionnalités employés au sein de **Berry AI Studio**.

---

## A
- **AIGC (Contenu généré par IA / AI-Generated Content)** : Médias numériques (images, animations, audio et vidéos) générés à l'aide de modèles neuronaux comme Stable Diffusion, ComfyUI, Midjourney ou Flux.
- **Album** : Collection personnalisée d'œuvres regroupées logiquement par l'utilisateur sans déplacement physique des fichiers sur le disque.
- **Authoritative Hero (Image de couverture / vedette)** : L'image de référence choisie pour représenter l'ensemble d'une pile d'images lorsque celle-ci est repliée dans la galerie.

## B
- **Barre d'actions par lots (Batch Action Bar)** : Barre d'outils flottante apparaissant au bas du canevas dès qu'une ou plusieurs cartes sont sélectionnées, offrant des opérations de masse rapides (notation, étiquetage, exportation, suppression).

## C
- **Carte en paquet de jeu (Poker-Deck Card)** : Représentation visuelle d'une pile d'images dans la galerie, stylisée avec des bordures étagées simulant des cartes superposées et un badge interactif de décompte.
- **CFG Scale (Classifier-Free Guidance)** : Paramètre des modèles de diffusion contrôlant le niveau de fidélité et d'adhérence du rendu au prompt textuel d'origine.
- **Checkpoint (Modèle de base)** : Modèle neuronal génératif fondamental contenant l'ensemble des poids entraînés (généralement au format `.safetensors`), identifié par son nom et son hash SHA256.
- **CLIP (Contrastive Language-Image Pre-training)** : Architecture de réseau neuronal multimodal employée par Berry AI Studio pour la recherche sémantique en langage naturel et les calculs de similarité visuelle.
- **Coffre-fort de projet géré (Mode B / Managed Vault)** : Répertoire d'application dédié qui organise physiquement les fichiers importés dans une arborescence ordonnée et découpée par date (`AAAA/MM/UUID_nomfichier.ext`).
- **ComfyUI** : Moteur de génération par IA modulaire basé sur des graphes de nœuds. Berry analyse ses flux de travail intégrés et prend en charge l'envoi direct via l'API `/prompt`.
- **Curseur Keyset (Pagination par curseur)** : Stratégie de requête de base de données remplaçant les requêtes lentes `OFFSET N` par `WHERE (modified_at, id) < (?, ?)`, garantissant une traversée de page en moins d'une milliseconde même sur plus de 500 000 enregistrements.

## E
- **Élaguer les brouillons (Cull Drafts)** : Opération de nettoyage groupé conservant l'image vedette (Hero) au sein d'une pile de rafale tout en déplaçant les ébauches et variations moins bien notées vers la corbeille du système.
- **Empilement en rafales (Burst Stacking)** : Regroupement automatique des variations génératives consécutives partageant des prompts similaires et produites dans une fenêtre temporelle étroite sous la forme d'une carte de pile unique.

## F
- **Fichier auxiliaire (Sidecar file)** : Fichier compagnon `.txt` ou `.json` stocké à côté d'une image, contenant des paramètres de génération ou des métadonnées Civitai.
- **Flou NSFW (NSFW Blur)** : Fonctionnalité de confidentialité appliquant un filtre de flou sur le contenu adulte ou sensible jusqu'à ce que l'utilisateur clique expressément dessus.

## G
- **Graine (Seed)** : Valeur entière initialisant le générateur de bruit pseudo-aléatoire, permettant de reproduire fidèlement une génération IA.

## J
- **Jetons de prompt (Prompt Chips)** : Badges d'interface interactifs représentant chaque terme individuel d'un prompt dans l'Inspecteur de propriétés, permettant la recherche et la copie en 1 clic.

## L
- **Lien externe (Mode A / External Link)** : Mode de dossier qui indexe un stockage local ou un partage réseau sur place sans copie et sans modifier les fichiers physiques originaux.
- **Lightbox (Aperçu rapide / Quick Look)** : Visualiseur plein écran immersif ouvert avec `Espace` ou `Entrée`, permettant le zoom, le panoramique et le défilement vidéo image par image.
- **LoRA (Low-Rank Adaptation)** : Adaptateur d'affinage léger appliqué sur un modèle de checkpoint afin d'introduire des personnages, des styles artistiques ou des concepts spécifiques.

## M
- **Mosaïque fluide (Waterfall / Masonry)** : Mode de disposition de la galerie agençant les cartes en colonnes dynamiques tout en préservant le ratio d'aspect d'origine de chaque œuvre sans recadrage.

## O
- **OCC (Contrôle de concurrence optimiste / Optimistic Concurrency Control)** : Stratégie de gestion de la concurrence s'appuyant sur des numéros de `version` par ligne pour éliminer les pertes de données lorsque plusieurs membres d'une équipe éditent la même bibliothèque simultanément.

## P
- **Pipeline d'ingestion AIGC (Mode C)** : Mode de surveillance active de dossiers qui scrute les répertoires de sortie de générateurs d'IA, applique un anti-rebond d'écriture de 500 ms, récolte automatiquement les créations et gère des délais de grâce de nettoyage.

## R
- **Racine de stockage (Storage Root)** : Identifiant abstrait universel (UUID) reliant un partage réseau aux différents chemins de montage locaux selon le système d'exploitation.

## S
- **Sampler / Scheduler (Échantillonneur / Planificateur)** : Algorithme d'intégration numérique (ex. `Euler a`, `DPM++ 2M Karras`) utilisé pour débruiter les représentations latentes jusqu'à l'image finale.
- **Similarité de Jaccard** : Métrique statistique mesurant le degré de recouvrement entre deux chaînes de prompts tokenisées pour l'empilement automatique en rafales.

## W
- **WAL (Write-Ahead Logging)** : Mode de journalisation de SQLite permettant des lectures simultanées sans aucun verrouillage durant l'indexation d'arrière-plan et la génération des miniatures.
- **WD14 Tagger (Étiqueteur WD14)** : Modèle de vision par ordinateur (SmilingWolf) prédisant automatiquement les tags anime Danbooru et les niveaux de classification de contenu directement à partir des pixels de l'image.
