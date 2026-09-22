# Disposition de l'espace de travail & Anatomie de l'interface

Berry AI Studio propose un espace de travail de bureau à 3 volets inspiré d'**Eagle et Lightroom**, conçu pour la curation visuelle haute densité, la rapidité au clavier et une visualisation sans distraction.

---

## 1. Anatomie de la fenêtre & Structure de l'application

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Logo] Berry AI Studio   [ Fichier  Édition  Affichage  Outils  Aide ]   [ _ ] [ □ ] [ ✕ ] │  <- Barre de titre & Menus
├──────────────┬──────────────────────────────────────────────────────────┬──────────────┤
│              │ [🔍 Recherche : prompt, modèle, note... ] [🧠] [☰ Filtre] │              │
│  BARRE       ├──────────────────────────────────────────────────────────┤  INSPECTEUR  │
│  LATÉRALE DE │                                                          │  DE          │
│  NAVIGATION  │                  CANEVAS DE LA GALERIE CENTRALE          │  PROPRIÉTÉS  │
│              │                                                          │              │
│ - Tout       │  [ Carte ]  [ Carte ]  [ Pile de cartes (4) ]  [ Carte ] │ - Aperçu     │
│ - Favoris    │                                                          │ - Prompts    │
│ - Dossiers   │  [ Carte ]  [ Carte ]  [ Carte ]               [ Carte ] │ - Modèle/CFG │
│ - Albums     │                                                          │ - Tags LoRA  │
│ - Tags       │                                                          │ - Interop    │
│              │                                                          │              │
│ [Outils]     │                                                          │              │
├──────────────┴──────────────────────────────────────────────────────────┴──────────────┤
│ 1 248 / 8 920 éléments | 3 Sélectionnés | Schéma v14 | Prêt | [⚡ Activité]             │  <- Barre d'état
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Barre de titre & Menus natifs de l'application

La fenêtre adopte une conception sans cadre (frameless) avec une barre de titre personnalisée (`TitleBar.vue`) et un système de menus natifs de bureau (`MenuBar.vue`) :

### Référence du menu supérieur
- **Fichier** :
  - `Ajouter un dossier...` (`Ctrl + O` / `Cmd + O`) : Connecter un nouveau répertoire local ou réseau.
  - `Scanner le dossier actuel` : Réindexer le dossier actif pour détecter les fichiers nouveaux ou modifiés.
  - `Scanner tous les dossiers` : Forcer une analyse incrémentielle de tous les dossiers enregistrés.
  - `Gestion de la base...` : Ouvrir les outils de compactage, de statistiques de taille et de sauvegarde.
  - `Préférences / Paramètres...` (`Ctrl + ,` / `Cmd + ,`) : Ouvrir la fenêtre centrale de configuration à 8 onglets.
  - `Quitter` (`Alt + F4`) : Fermer proprement le studio.
- **Édition** :
  - `Tout sélectionner` (`Ctrl + A` / `Cmd + A`) : Sélectionner tous les éléments de la vue active.
  - `Désélectionner tout` (`Échap`) : Désélectionner tous les éléments mis en surbrillance.
  - `Ajouter des tags en lot...` : Assigner ou retirer des tags aux fichiers sélectionnés.
  - `Ajouter à l'album en lot...` : Assigner les fichiers sélectionnés à un album.
  - `Déplacer vers un dossier...` / `Copier vers un dossier...` : Transférer physiquement les fichiers vers un autre dossier indexé.
  - `Exporter...` (`Ctrl + E` / `Cmd + E`) : Ouvrir la fenêtre d'exportation, de transcodage et d'empaquetage.
  - `Mettre à la corbeille` (`Suppr` / `Retour arrière`) : Déplacer la sélection vers la corbeille du système d'exploitation.
- **Affichage** :
  - `Vue Grille` : Basculer vers des cartes responsives uniformes à hauteur fixe.
  - `Vue Mosaïque (Waterfall)` : Basculer vers des colonnes préservant les proportions d'origine sans recadrage.
  - `Vue Liste (Tableau)` : Basculer vers un mode tableur compact.
  - `Afficher/Masquer la barre latérale` (`B`) : Basculer la visibilité du panneau de navigation gauche.
  - `Afficher/Masquer l'inspecteur` (`I`) : Basculer la visibilité du panneau de métadonnées droit.
  - `Aperçu plein écran (Lightbox)` (`Espace` / `Entrée`) : Ouvrir la fenêtre modale de prévisualisation plein écran.
  - `Zoom avant` (`Ctrl + =`), `Zoom arrière` (`Ctrl + -`), `Réinitialiser le zoom` (`Ctrl + 0`).
- **Outils** :
  - `Statistiques des prompts...` : Fréquence visuelle des mots-clés et corrélation avec les notes.
  - `Gestionnaire de modèles...` : Résoudre les hashs de checkpoints en noms conviviaux.
  - `Index sémantique CLIP...` : Indexer les embeddings de la bibliothèque pour la recherche en langage naturel.
  - `Bibliothèque de déclencheurs LoRA` : Gérer les mots déclencheurs et les fichiers auxiliaires `.civitai.info`.
  - `Étiquetage automatique IA WD14` : Ouvrir l'étiqueteur d'anime/esthétique Danbooru local.
- **Aide** :
  - `Langue` : Basculer instantanément entre les 7 langues prises en charge.
  - `Raccourcis clavier` (`?`) : Afficher l'antisèche des raccourcis dans l'application.
  - `Vérifier les mises à jour...` : Interroger GitHub Releases pour détecter les nouvelles versions.
  - `À propos de Berry AI Studio` : Afficher les informations de version, d'auteur et de licence.

---

## 3. Volet de navigation gauche (`Sidebar.vue`)

La barre latérale gauche (`B` pour l'afficher ou la masquer) offre un accès rapide à vos collections :

1. **Bibliothèques système** :
   - **Toutes les images** : Vue exhaustive de la bibliothèque à travers tous les dossiers connectés avec un badge de décompte.
   - **Favoris (★)** : Filtre rapide pour toutes les images marquées d'une étoile ou mises en signet.
   - **Sensible (18+) (🔞)** : Accès rapide au contenu classé NSFW (affiché par défaut avec un voile de flou).
2. **Section Dossiers** :
   - Affiche tous les dossiers enregistrés avec des badges indicateurs de type :
     - `⚡` : Pipeline d'ingestion (surveillance active de la sortie d'un générateur).
     - `📦` : Coffre-fort de projet géré (stockage interne organisé).
     - `📁` : Lien externe (indexation sur place sans copie).
   - Clic droit ou survol pour déclencher des actions : **Récolter les images**, **Scanner le dossier**, **Reconstruire les métadonnées** ou **Supprimer le dossier**.
   - Agit comme une cible de glisser-déposer : déposez des cartes directement sur un dossier pour les déplacer ou les copier.
3. **Section Albums** :
   - Collections définies par l'utilisateur. Glissez-déposez des cartes sur les albums pour les y ajouter.
   - Comprend un bouton « + Nouveau ».
4. **Section Tags** :
   - Badges colorés de taxonomie (8 couleurs distinctes).
   - Glissez des éléments de la galerie sur les badges de tags pour les appliquer en lot.
5. **Pied de page Outils rapides** :
   - Boutons de raccourcis pour ouvrir les fenêtres modales Statistiques, Modèles, Maintenance de la base de données et Raccourcis clavier.

---

## 4. Canevas de la galerie centrale & Barre d'outils de recherche

L'espace de travail central est le lieu où vous explorez, sélectionnez et organisez vos médias :

- **Barre de recherche (`SearchBar.vue`)** :
  - **Mode Recherche syntaxique (`🔍`)** : Interrogez les métadonnées avec des mots-clés ou une syntaxe structurée (ex. `prompt:"cyberpunk" cfg:>7`).
  - **Mode Recherche sémantique IA (`🧠`)** : Requêtes texte-vers-image en langage naturel alimentées par les modèles locaux CLIP/SigLIP.
- **Sélecteur de mode d'affichage** :
  - Basculez entre les dispositions **Grille (⊞)**, **Mosaïque (▤)** et **Tableau (☰)**.
  - **Curseur de zoom** : Ajuste dynamiquement la largeur minimale des cartes entre **130px** et **360px**. Le nombre de colonnes s'adapte de manière responsive sans déformer les images.
- **Bouton du tiroir de filtres (`FilterDrawer.vue`)** :
  - Déployez le volet latéral multicritère pour filtrer par modèle de checkpoint, sampler, ratio d'aspect, note par étoiles et propriétés vidéo.

---

## 5. Inspecteur de propriétés droit (`InspectorPane.vue`)

L'inspecteur droit (`I` pour l'afficher ou le masquer) dévoile les métadonnées de génération détaillées et sans perte de l'élément sélectionné :

- **Aperçu du média** : Miniature haute résolution avec bouton de défloutage NSFW et déclencheur d'aperçu Lightbox.
- **Contrôles de curation** : Notation par étoiles (0 à 5 étoiles), bascule Favori (`F`) et indicateur de contenu sensible NSFW.
- **Prompt positif** : Vue tokenisée du prompt. Chaque jeton de prompt s'affiche sous la forme d'un badge interactif :
  - Cliquez sur un jeton pour le rechercher dans toute la bibliothèque.
  - Cliquez sur le bouton copier pour copier l'intégralité du prompt positif.
- **Prompt négatif** : Texte complet du prompt négatif avec bouton de copie en 1 clic.
- **Paramètres techniques de génération** :
  - Nom du modèle & Hash SHA256 du checkpoint.
  - Algorithmes de Sampler et de Scheduler.
  - Nombre d'étapes (Steps), échelle CFG, Seed (avec bouton de copie).
  - Dimensions natives de génération (`Largeur × Hauteur`).
- **LoRAs détectés** :
  - Liste l'ensemble des balises `<lora:nom:poids>` ou des nœuds chargeurs de LoRA ComfyUI détectés dans le fichier.
  - Indique les mots déclencheurs, les valeurs de poids et offre un bouton de copie en 1 clic de la balise de prompt formatée.
- **Carte d'interopérabilité de génération** :
  - Affiche l'état de connectivité en direct avec **ComfyUI** et **AUTOMATIC1111** locaux.
  - Boutons en 1 clic « Envoyer à ComfyUI » (remet en file d'attente le graphe du workflow) ou « Envoyer à SD WebUI ».
- **Accordéon des métadonnées brutes** : Affiche le bloc complet des paramètres JSON non formaté ou le graphe de nœuds ComfyUI brut.

---

## 6. Barre d'état inférieure & Popover Activité

Située tout en bas de la fenêtre :
- **Compteurs Filtré / Total** : Affiche les éléments correspondants visibles par rapport à la taille totale de la bibliothèque (ex. `Filtré : 420 / 12 500 éléments`).
- **Compteur de sélection** : Affiche le nombre d'éléments sélectionnés (`Sélectionné : 5 éléments`).
- **État de la base de données** : Affiche le nom du fichier de base de données actif et la version du schéma.
- **Popover Activité (`⚡ Activité`)** :
  - Ouvre un moniteur en temps réel indiquant l'état des observateurs du système de fichiers (racines actives, arriéré du journal, état d'erreur) et les files d'attente de décodage des miniatures en arrière-plan.

---

## 7. Barre d'actions par lots flottante (`BatchActionBar.vue`)

Lorsqu'une ou plusieurs cartes sont sélectionnées, une barre d'outils flottante apparaît au centre inférieur du canevas :

- Indicateur de sélection multiple (`X sur Y sélectionnés`) avec boutons Tout sélectionner / Désélectionner.
- Menu déroulant Définir la note (0 à 10 étoiles / demi-étoiles).
- Déclencheurs de modales « Ajouter à l'album » et « Ajouter un tag ».
- Déclencheur « Étiquetage auto (WD14) ».
- Bascule « Favori ».
- « Copier chemins » et « Copier prompts » dans le presse-papier.
- Boîtes de dialogue de destination « Déplacer » et « Copier ».
- « Élaguer les brouillons » (actif automatiquement lorsque des piles sont sélectionnées).
- « Exporter... » (transcodage par lots, suppression des métadonnées privées, vitrine HTML).
- « Corbeille » (déplace les éléments sélectionnés vers la corbeille du système).
