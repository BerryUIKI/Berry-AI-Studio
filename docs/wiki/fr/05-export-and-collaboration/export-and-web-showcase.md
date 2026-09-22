# Exportation par lots, Transcodage & Galerie Web

Berry AI Studio intègre un moteur d'exportation et d'empaquetage à haut débit (`ExportModal.vue`) propulsé par le multithreading **Rayon**. Il prend en charge le transcodage de formats, l'assainissement de la confidentialité des métadonnées, les modèles dynamiques de noms de fichiers et la génération de vitrines HTML interactives autonomes sans aucune dépendance.

---

## 1. Lancer l'exportation par lots

Pour ouvrir la fenêtre d'exportation :
- Sélectionnez une ou plusieurs images ou piles dans la galerie.
- Cliquez sur **« Exporter... »** dans la barre d'actions flottante, ou appuyez sur `Ctrl + E` / `Cmd + E` (ou choisissez `Édition > Exporter...`).

---

## 2. Transcodage de format & Compression

Berry convertit et réencode les images en parallèle sur l'ensemble des cœurs de votre processeur :

| Format cible | Options & Réglages | Cas d'usage idéal |
| :--- | :--- | :--- |
| **Format d'origine** | Préserve les octets et conteneurs sources exacts. | Sauvegarde ou archivage sans perte. |
| **WebP** | Curseur de qualité réglable (1 à 100 %, 85 % par défaut). Très haute compression. | Diffusion web, Discord, portfolios en ligne. |
| **JPEG** | Encodage JPEG progressif standard (qualité de 1 à 100 %). | Compatibilité universelle avec les visualiseurs photo anciens. |
| **PNG** | Compression pure sans perte. | Transfert studio, impression haute définition. |

### Contraintes de réduction de résolution :
Vous pouvez limiter les dimensions maximales pour éviter la diffusion involontaire d'images 4K/8K trop lourdes :
- **Résolution d'origine** (pas de réduction)
- **4K UHD** (bord maximal : 3840 px)
- **2K QHD** (bord maximal : 2048 px)
- **Full HD** (bord maximal : 1080 px)
- **Limite personnalisée** (seuil de pixels maximal défini par l'utilisateur)

---

## 3. Assainissement des métadonnées sur 4 niveaux

De nombreux créateurs souhaitent partager leurs créations en ligne tout en gardant secrets leurs prompts exclusifs, leurs embeddings négatifs ou leurs graines de génération. Berry propose **quatre niveaux d'assainissement de confidentialité** :

1. **Tout conserver (Prompts, workflows & EXIF)** :
   - Préserve l'intégralité des blocs de métadonnées intégrés (graphes ComfyUI, paramètres A1111, commentaires NovelAI et données EXIF).
2. **Supprimer les prompts uniquement (Conserver les paramètres techniques)** :
   - Supprime les chaînes de texte des prompts positifs et négatifs, mais conserve les réglages techniques (sampler, étapes, échelle CFG, seed, nom du modèle).
3. **Supprimer toutes les métadonnées et workflows IA** :
   - Supprime intégralement les graphes de nœuds ComfyUI, les blocs de paramètres A1111, les balises LoRA et les signatures de moteurs.
4. **Nettoyage complet (Pixels uniquement, pas d'EXIF/ICC)** :
   - Supprime absolument tout, y compris les en-têtes EXIF, les profils colorimétriques ICC et les signatures logicielles. Le fichier exporté ne contient que des données matricielles pures.

---

## 4. Modèles de noms de fichiers & Fichiers auxiliaires

Personnalisez les noms de fichiers générés à l'aide de variables dynamiques avec prévisualisation en temps réel :

### Jetons de modèles pris en charge :
- `{name}` : Nom de fichier d'origine sans extension.
- `{id}` : Identifiant unique dans la base de données ou UUID.
- `{index}` : Numéro d'index séquentiel (001, 002, 003...).
- `{date}` : Date de création (`AAAA-MM-JJ`).
- `{rating}` : Note par étoiles (ex. `5star`).
- `{model}` : Nom du modèle de checkpoint.
- `{seed}` : Valeur de la graine de génération.

*Exemple de modèle* : `{date}_{model}_{seed}_{name}` → `2026-09-22_animagine_xl_2849104812_cyberpunk_01.webp`

### Fichiers auxiliaires (Sidecars) :
Vous pouvez générer automatiquement des fichiers d'accompagnement à côté de chaque image exportée :
- **Aucun** : Aucun fichier auxiliaire.
- **Prompt texte (`.txt`)** : Exporte le prompt positif sous la forme d'un fichier texte compagnon.
- **Métadonnées JSON complètes (`.json`)** : Exporte l'intégralité des métadonnées structurées et les graphes de workflow ComfyUI.

---

## 5. Générateur d'album vitrine HTML autonome

Berry peut empaqueter vos créations exportées sous la forme d'une **vitrine web autonome en un fichier HTML unique** (`index.html`) :

- **Zéro dépendance** : Ne requiert aucun serveur web, Node.js ou bibliothèque JavaScript distante. Double-cliquez simplement pour l'ouvrir dans n'importe quel navigateur web.
- **Fonctionnalités intégrées** :
  - Esthétique sombre de studio assortie à Berry AI Studio.
  - Grille responsive avec chargement différé (lazy loading) des miniatures.
  - Visualiseur Lightbox plein écran avec zoom et panoramique à la molette.
  - Volet repliable d'inspection des prompts affichant les paramètres de génération.
  - Barre de recherche instantanée par mots-clés côté client.
- **Options d'empaquetage** : Exportez directement dans un dossier cible ou compilez l'ensemble dans une archive `.zip` unique pour livraison client ou hébergement web.
