# Guide des raccourcis clavier

Berry AI Studio est pensé pour un **flux de travail axé sur le clavier**. Vous pouvez parcourir, noter, regrouper, inspecter et organiser des bibliothèques colossales sans jamais avoir à toucher la souris.

---

## 1. Aperçu rapide & Navigation dans la galerie

| Raccourci | Action | Portée | Description |
| :--- | :--- | :--- | :--- |
| `Espace` ou `Entrée` | **Ouvrir l'aperçu Lightbox** | Sélection dans la galerie | Ouvre l'aperçu haute résolution plein écran de l'élément sélectionné. |
| `Échap` | **Fermer / Effacer** | Global | Ferme l'aperçu Lightbox ou une fenêtre modale, annule la recherche ou désélectionne les cartes. |
| `←` / `→` | **Élément précédent / suivant** | Galerie & Lightbox | Déplace la sélection vers l'élément adjacent (ou avance/recule d'1 image lors de la lecture vidéo). |
| `↑` / `↓` | **Ligne supérieure / inférieure** | Vue Grille | Déplace la sélection vers le haut ou le bas d'une rangée visuelle. |
| `Ctrl + A` / `Cmd + A` | **Tout sélectionner** | Canevas de la galerie | Sélectionne toutes les cartes correspondant actuellement à la recherche/au filtre actif. |
| `Suppr` ou `Retour arrière` | **Mettre à la corbeille** | Sélection dans la galerie | Envoie les éléments sélectionnés dans la corbeille du système d'exploitation. |

---

## 2. Curation rapide & Notation

| Raccourci | Action | Description |
| :--- | :--- | :--- |
| `1` | **Noter 1 étoile** | Attribue une note de 1 étoile aux éléments sélectionnés. |
| `2` | **Noter 2 étoiles** | Attribue une note de 2 étoiles aux éléments sélectionnés. |
| `3` | **Noter 3 étoiles** | Attribue une note de 3 étoiles aux éléments sélectionnés. |
| `4` | **Noter 4 étoiles** | Attribue une note de 4 étoiles aux éléments sélectionnés. |
| `5` | **Noter 5 étoiles** | Attribue une note de 5 étoiles aux éléments sélectionnés. |
| `0` | **Effacer la note** | Supprime la note par étoiles des éléments sélectionnés. |
| `F` | **Basculer Favori** | Ajoute ou retire les éléments sélectionnés des favoris. |

> [!TIP]
> **Notation à l'aveugle** : Vous pouvez garder une main sur les touches fléchées (`←`, `→`) pour faire défiler les images tout en appuyant sur `1`–`5` de l'autre main. Les notes sont écrites instantanément dans SQLite sans interrompre l'affichage.

---

## 3. Empilement & Comparaison

| Raccourci | Action | Portée | Description |
| :--- | :--- | :--- | :--- |
| `Ctrl + G` / `Cmd + G` | **Grouper dans une pile** | Sélection dans la galerie | Regroupe les éléments sélectionnés sous forme de pile de rafale façon jeu de cartes. |
| `Ctrl + Maj + G` / `Cmd + Shift + G` | **Dissocier la pile** | Sélection dans la galerie | Dissout la pile sélectionnée pour rétablir les images indépendantes. |
| `Alt + S` / `Option + S` | **Définir comme couverture** | Sélection dans une pile | Définit l'image active comme couverture principale (Hero) de sa pile. |
| `C` | **Comparaison côte à côte** | Sélection dans la galerie | Ouvre la vue comparative scindée 1-contre-1 avec zoom et panoramique synchronisés. |

---

## 4. Panneaux de l'espace de travail & Zoom

| Raccourci | Action | Description |
| :--- | :--- | :--- |
| `B` | **Afficher/Masquer la barre latérale** | Affiche ou masque le volet de navigation gauche (Bibliothèque, Dossiers, Tags). |
| `I` | **Afficher/Masquer l'inspecteur** | Affiche ou masque le volet d'inspection des propriétés à droite. |
| `/` ou `Ctrl + F` / `Cmd + F` | **Activer la barre de recherche** | Met en surbrillance la barre de recherche et sélectionne l'ensemble du texte. |
| `Ctrl + =` / `Cmd + =` | **Zoom avant** | Augmente la taille des miniatures des cartes de la galerie. |
| `Ctrl + -` / `Cmd + -` | **Zoom arrière** | Diminue la taille des miniatures des cartes de la galerie. |
| `Ctrl + 0` / `Cmd + 0` | **Réinitialiser le zoom** | Réinitialise la taille des cartes à la largeur standard par défaut (256px). |
| `F11` | **Plein écran** | Bascule le mode plein écran sans bordure de la fenêtre. |

---

## 5. Opérations de bibliothèque & Fenêtres modales

| Raccourci | Action | Description |
| :--- | :--- | :--- |
| `Ctrl + O` / `Cmd + O` | **Ajouter un dossier** | Ouvre l'assistant de sélection du mode de dossier. |
| `Ctrl + E` / `Cmd + E` | **Exporter par lots** | Ouvre la fenêtre de transcodage groupé, d'assainissement de confidentialité et d'empaquetage. |
| `Ctrl + ,` / `Cmd + ,` | **Préférences** | Ouvre la fenêtre centrale des paramètres de l'application. |
| `?` ou `Maj + /` | **Guide des raccourcis** | Ouvre l'antisèche interactive des raccourcis clavier. |
| `Alt + F4` | **Quitter l'application** | Ferme proprement Berry AI Studio. |

---

## 6. Modificateurs de sélection

- **Clic simple** : Sélectionne une seule carte et la définit comme **ancre de sélection**.
- **Maj + Clic** : Étend la sélection depuis l'ancre jusqu'à la carte cliquée sous la forme d'une **plage contiguë inclusive**.
- **Ctrl + Clic** (ou `Cmd + Clic` sur macOS) : **Bascule** l'état de sélection d'une carte individuelle sans désélectionner les autres éléments, et met à jour l'ancre de sélection.
- **Double-clic sur la couverture d'une pile** : Ouvre instantanément le zoom Lightbox pour l'image de couverture (un arbitrage de geste évite tout déploiement involontaire).
- **Clic simple sur le badge de décompte d'une pile** : Déploie ou replie la pile de rafale directement sur place.
