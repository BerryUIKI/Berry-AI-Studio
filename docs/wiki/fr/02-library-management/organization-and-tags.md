# Organisation, Notes & Tags

Berry AI Studio propose de riches mécanismes de curation conçus pour trier, catégoriser et hiérarchiser des dizaines de milliers d'œuvres rapidement sans encombrer votre système de fichiers.

---

## 1. Système de notation par étoiles & Évaluation

Berry utilise un modèle de notation à double précision stocké directement dans SQLite (`files.rating` et `files.aesthetic_score`) :

### Notes par étoiles (0 à 5 étoiles ou échelle de 1 à 10)
- **Raccourcis clavier** : Sélectionnez une ou plusieurs images et appuyez sur :
  - `1` à `5` : Attribue immédiatement la note correspondante par étoiles.
  - `0` : Efface la note.
- **Inspecteur / Barre d'actions groupées** : Prend en charge l'échelle étendue à 10 étoiles (ex. 8/10 ou 9/10) via des menus déroulants.
- **Indexation dans la base de données** : La colonne `rating` est indexée par des B-trees SQLite, permettant des requêtes instantanées comme `rating:>=4` ou `rating:5` sur des bibliothèques de 500 000 éléments.

### Favoris (★ Mise en signet)
- **Bascule** : Appuyez sur `F` ou cliquez sur l'icône d'étoile dans l'Inspecteur.
- **Indicateur visuel** : Un badge d'étoile dorée apparaît dans le coin supérieur droit de la carte.
- **Accès dans la barre latérale** : Tous les éléments marqués comme favoris sont accessibles instantanément dans le volet de navigation sous **Favoris (★)**.

---

## 2. Albums personnalisés (`AlbumModal.vue`)

Les albums permettent de regrouper des créations artistiques connexes dispersées dans différents dossiers, sans déplacer les fichiers physiques.

### Créer & Gérer des albums
1. Dans la barre latérale gauche, sous **Albums**, cliquez sur **« + Nouveau »**.
2. Saisissez un nom d'album (ex. `"Personnages Cyberpunk"`, `"Portfolio 2026"`) et une description facultative.
3. Renommez ou supprimez des albums à tout moment d'un clic droit dans la barre latérale. La suppression d'un album retire les associations de regroupement mais ne supprime jamais vos fichiers physiques.

### Ajouter des médias aux albums
- **Glisser-Déposer** : Sélectionnez une ou plusieurs cartes dans la galerie et glissez-les directement sur un album dans la barre latérale gauche.
- **Barre d'actions groupées** : Cliquez sur **« Album »** dans la barre d'outils flottante.
- **Inspecteur droit** : Recherchez et associez des albums directement depuis le volet de l'Inspecteur.

---

## 3. Taxonomie de tags avec code couleur (`TagModal.vue`)

Les tags offrent une catégorisation granulaire et un tri visuel efficace :

### Palettes de couleurs prédéfinies
Berry intègre 8 couleurs visuelles distinctes :
- 🔴 Rouge
- 🟠 Orange
- 🟡 Jaune
- 🟢 Vert
- 🔵 Bleu
- 🟣 Violet
- 🌸 Rose
- ⚪ Ardoise / Gris

### Utilisation des tags
- **Créer des tags** : Cliquez sur **« + Nouveau »** dans la section Tags de la barre latérale, choisissez une couleur et entrez un nom (ex. `Personnage principal`, `Brouillon`, `Validé client`).
- **Glisser pour étiqueter** : Glissez les cartes sélectionnées depuis le canevas directement sur un tag dans la barre latérale gauche pour l'appliquer.
- **Filtrer par tag** : Cliquez sur n'importe quel tag dans la barre latérale pour filtrer la galerie.
- **Étiquetage multiple** : Vous pouvez attribuer un nombre illimité de tags à chaque création.

---

## 4. Barre d'actions par lots flottante (`BatchActionBar.vue`)

Dès que vous sélectionnez plusieurs éléments (via `Ctrl+Clic`, `Maj+Clic` ou `Ctrl+A`), la **Barre d'actions par lots** se déploie dans le bas de l'écran :

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  [✓ 14 sur 120 sélectionnés]  [Tout sélectionner]  [Désélectionner]                    │
│  [★ Définir la note ▾]  [🏷 Tag]  [📁 Album]  [🧠 Étiquetage auto]  [★ Fav]  [🔞 NSFW]   │
│  [📋 Copier chemins]  [💬 Copier prompts]  [📂 Déplacer]  [📄 Copier]  [🧹 Élaguer]    │
│  [📤 Exporter...]  [🗑 Corbeille]                                                      │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Principales opérations par lots :
- **Définir la note par lots** : Applique une note par étoiles uniforme à toutes les images sélectionnées.
- **Tags & Albums par lots** : Ouvre la boîte de dialogue pour appliquer ou retirer des tags et des albums en masse.
- **Copier prompts** : Copie les prompts positifs de toutes les images sélectionnées dans votre presse-papier, séparés par des délimiteurs `---` clairs.
- **Copier chemins** : Copie les chemins absolus complets du système de fichiers (une ligne par fichier) pour les coller dans un terminal ou un script.
- **Déplacer & Copier** : Déplace ou copie les fichiers physiques vers un autre dossier indexé dans Berry.
- **Élaguer les brouillons (Cull Drafts)** : Actif lorsque les cartes sélectionnées contiennent des piles de rafales (voir [Piles & Rafales](../04-intelligent-curation/stacks-and-bursts.md)).
- **Corbeille** : Déplace en toute sécurité tous les fichiers sélectionnés vers la corbeille de votre système d'exploitation.
