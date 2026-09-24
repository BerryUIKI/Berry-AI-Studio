# Modes de galerie & Options d'affichage

Omera propose quatre modes de présentation dédiés de la galerie, pensés pour s'adapter aux différents besoins de curation, du tri visuel rapide à l'inspection technique approfondie.

---

## 1. Modes d'affichage de la galerie

Utilisez les boutons de la barre d'outils supérieure ou les raccourcis du menu (`Affichage`) pour basculer entre les modes :

### 1. Vue Grille uniforme (`grid` — ⊞)
- **Concept** : Cartes responsives à hauteur fixe et proportions uniformes, disposées en colonnes.
- **Architecture responsive** : La largeur des cartes reste stable lors du redimensionnement de la fenêtre. Omera ajuste dynamiquement le nombre de colonnes (`calculateGalleryColumns`) au lieu d'étirer ou de déformer les images.
- **Curseur de zoom** : Faites glisser le curseur de zoom ou utilisez `Ctrl + =` / `Ctrl + -` pour faire varier la largeur minimale des cartes de **130 px** (mode vignettes pour vue d'ensemble) à **360 px** (mode grand format pour observer les détails).
- **Virtualisation** : Seuls les éléments présents dans la zone d'affichage (plus une petite mémoire tampon d'anticipation) sont générés dans le DOM. Faire défiler une bibliothèque de 100 000 images ne consomme pas plus de mémoire vive que faire défiler 100 images.

### 2. Vue Mosaïque fluide (Waterfall) (`masonry` — ▤)
- **Concept** : Disposition fluide en plusieurs colonnes préservant les proportions natives de chaque image.
- **Œuvres sans recadrage** : Idéal pour les collections mélangeant fonds d'écran paysages, concepts de personnages verticaux (9:16) et décors panoramiques. Les images sont redimensionnées proprement via `object-fit: contain` sans perte de bords.
- **Distribution par colonne la plus courte** : Chaque nouvel élément est inséré dans la colonne ayant la hauteur verticale cumulée la plus faible, maintenant un mur d'art visuellement harmonieux et équilibré.

### 3. Vue Tableau haute densité (`table` — ☰)
- **Concept** : Disposition virtualisée sous forme de feuille de calcul avec une hauteur de ligne fixe de 46 px.
- **Densité d'informations** : Affiche des micro-miniatures (palier 36 px) aux côtés de colonnes détaillées :
  - Case à cocher d'état de sélection
  - Nom de fichier et chemin relatif du répertoire
  - Format du conteneur multimédia (`PNG`, `WEBP`, `MP4`)
  - Dimensions (`Largeur × Hauteur`)
  - Taille du fichier (formatée en Ko/Mo)
  - Date et heure de dernière modification
  - Note par étoiles (0–5 / 1–10)
  - Score esthétique prédictif

### 4. Vue de correspondance par similarité visuelle
- **Activation** : Cliquez sur **« Trouver des images similaires »** dans l'inspecteur droit ou via le menu contextuel d'une image.
- **Bannière supérieure de correspondance** : Affiche l'image de référence source, un curseur de seuil de similarité (**de 0 % à 95 %** par paliers de 5 %) et un sélecteur de limite de résultats (**20, 50, 100, 200** éléments).
- **Badge de pourcentage de correspondance** : Chaque carte correspondante affiche un badge coloré de score (ex. `Correspondance : 94 %`) calculé d'après la distance cosinus des vecteurs CLIP.
- **Fermeture** : Appuyez sur `Échap` ou cliquez sur le bouton de fermeture (`✕`) de la bannière pour revenir à votre affichage normal.

---

## 2. Badges sur les cartes & Incrustations visuelles

Dans **Préférences > Galerie**, vous pouvez activer ou désactiver l'option **Afficher les badges sur les cartes**. Lorsqu'elle est activée, des badges épurés s'affichent sur chaque carte :

- **Badge de format / conteneur** : Indique `.png`, `.webp`, `.jpg`, `.mp4` ou `.webm`.
- **Badge de dimensions** : Résolution native en pixels (ex. `1024×1024` ou `832×1216`).
- **Durée & FPS vidéo** : Pour les animations et vidéos, indique le temps de lecture (ex. `00:04`) et la fréquence d'images (`24 fps`).
- **Signature du générateur** : Identifie les signatures de moteurs (ex. `WebUI`, `ComfyUI`, `NovelAI`, `Fooocus`).
- **Incrustation de la note** : Affiche la note par étoiles active (★ 1–5).
- **Indicateur Favori** : Icône d'étoile dorée dans le coin supérieur droit pour les images mises en signet.

---

## 3. Flou de confidentialité pour contenu sensible (NSFW)

Afin de préserver votre confidentialité lors de présentations professionnelles ou en travaillant dans des espaces publics, Omera intègre une protection du contenu :

- **Flou automatique (paramètre `blur_nsfw`)** : Tout actif marqué comme `is_nsfw` ou identifié avec une classification pour adultes est masqué par un filtre CSS de flou prononcé.
- **Cliquer pour afficher** : Un clic sur l'icône de l'œil (`👁`) ou directement sur la carte défloute temporairement cet élément précis pour examen.
- **Section Sensible globale** : La barre latérale gauche inclut un filtre de bibliothèque dédié **Sensible (18+) (🔞)** permettant d'auditer et de reclasser tout le contenu signalé en un seul endroit.
