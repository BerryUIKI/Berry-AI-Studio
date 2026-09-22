# Prise en charge des vidéos & médias animés

Alors que la génération par IA s'étend rapidement à la vidéo (AnimateDiff, SVD, Wan2.1, HunyuanVideo, CogVideoX, LTX-Video), Berry AI Studio intègre une prise en charge native de premier ordre pour les créations animées et les vidéos aux formats **MP4** et **WebM**.

---

## 1. Formats vidéo & Conteneurs pris en charge

Berry AI Studio analyse les conteneurs vidéo directement en Rust natif (`berry-metadata`) :

- **MP4 (`.mp4`)** : Analyse l'arborescence des boîtes ISOBMFF (`ftyp`, `moov`, `trak`, `mdia`, `minf`, `stbl`).
  - Extrait automatiquement les dimensions de la vidéo (`Largeur × Hauteur`), la fréquence d'images (FPS), la durée de lecture et le codec vidéo (`H.264`, `H.265 / HEVC`, `AV1`).
  - Inspecte les boîtes `moov/udta` à la recherche des graphes d'exécution ComfyUI et des paramètres de génération intégrés.
- **WebM (`.webm`)** : Analyse le format de conteneur EBML pour les flux vidéo VP8, VP9 et AV1, lisant la durée et les dimensions des images directement depuis l'en-tête du flux.

---

## 2. Cartes vidéo dans la galerie & Prévisualisations dynamiques

Dans le canevas de la galerie, les fichiers vidéo se distinguent clairement des images statiques :

- **Badge de durée** : Indique le temps de lecture exact dans le coin de la carte (ex. `00:05` ou `01:24`).
- **Badges de fréquence d'images & format** : Affiche `MP4 · 24fps` ou `WEBP · 30fps`.
- **Génération des miniatures** :
  - Comme les fichiers vidéo ne disposent pas de décodeurs d'images standard dans les bibliothèques logicielles habituelles, la vue Web (WebView) de Berry capture automatiquement la première image clé depuis un canevas HTML5 `<video>` en arrière-plan, l'encode en base64, puis le backend Rust l'enregistre sous forme de miniature WebP optimisée via `save_video_thumbnail`.
- **Lecture au survol** : Survoler une carte vidéo avec le curseur déclenche un aperçu vidéo léger directement sur le canevas sans avoir à ouvrir le lecteur plein écran.

---

## 3. Lecteur vidéo Lightbox (`LightboxModal.vue`)

Appuyer sur `Espace` ou `Entrée` sur n'importe quelle carte vidéo ouvre le **Lecteur Lightbox plein écran (Aperçu rapide)** :

```
┌────────────────────────────────────────────────────────────────────────┐
│ [✕]                                                          [★ Fav]   │
│                                                                        │
│                      [ ZONE DE LECTURE VIDÉO ]                         │
│                                                                        │
│                                                                        │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ [▶ / ⏸] [⏪ 1i] [1i ⏩] [00:03 / 00:08] ──●───────── [1.0x ▾] [🔁] [🔊] │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ Pellicule : [Miniature] [Miniature] [● Vidéo] [Miniature]          │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### Contrôles vidéo de la Lightbox :
1. **Lecture / Pause** : Cliquez sur la vidéo ou appuyez sur `Espace`.
2. **Saut d'image par image (Frame stepping)** : Appuyez sur `←` ou `→` (ou cliquez sur les boutons de l'affichage tête haute) pour avancer ou reculer image par image afin d'analyser le mouvement en détail.
3. **Vitesse de lecture** : Ajustez le sélecteur de vitesse entre **0.25x**, **0.5x**, **1.0x**, **1.5x** et **2.0x**.
4. **Boucle & Audio** : Activez ou désactivez la lecture en boucle continue (`🔁`) et le volume/muet (`🔊`).
5. **Défilement par pellicule (Filmstrip)** : Parcourez les images statiques et vidéos adjacentes du dossier actif en utilisant la bande de miniatures inférieure.

---

## 4. Inspection des workflows ComfyUI vidéo & mouvement

De nombreux générateurs vidéo s'appuient sur des graphes ComfyUI multi-étapes complexes (ex. prompt textuel → image latente initiale → module de mouvement AnimateDiff → guidage ControlNet openpose → upscaler spatial).

Lors de la visualisation d'un fichier vidéo produit par ComfyUI :
- L'**Inspecteur de propriétés** extrait et affiche les prompts textuels positifs/négatifs appliqués au modèle de mouvement.
- L'**Accordéon des métadonnées brutes** affiche le graphe d'exécution intégral, incluant le checkpoint du modèle, les LoRAs de mouvement, la longueur de la fenêtre de contexte, les images de chevauchement et les réglages de décodage VAE.
- Vous pouvez cliquer sur **« Envoyer à ComfyUI »** pour recharger le flux de travail vidéo exact dans votre instance locale en cours d'exécution pour des ajustements ou un nouveau rendu.
