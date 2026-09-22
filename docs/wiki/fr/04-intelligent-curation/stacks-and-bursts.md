# Piles d'images, Rafales & Comparaison

Les créateurs d'IA générative produisent fréquemment des séries de 10 à 50 variations avec des prompts identiques ou légèrement modifiés pour trouver la composition idéale. Sans outils de curation adaptés, cela submerge rapidement votre bibliothèque d'ébauches quasi identiques.

Berry AI Studio résout ce problème grâce à l'**Empilement intelligent en rafales**, aux **Cartes façon paquet de jeu**, à la **Sélection d'image de couverture (Hero)** et au **Mode comparateur côte à côte**.

---

## 1. Fonctionnement des piles

Une **Pile d'images** est un ensemble de variations d'images regroupées et représentées sur le canevas de la galerie par une image de couverture unique appelée **Hero** (Image vedette).

```mermaid
flowchart LR
    subgraph Rafale de génération
        A[Variation 1 - Seed 101]
        B[Variation 2 - Seed 102 (Hero 5★)]
        C[Variation 3 - Seed 103]
        D[Variation 4 - Seed 104]
    end

    subgraph Représentation de la pile
        E["Carte en paquet de jeu [Badge: 4] (Hero : Variation 2)"]
    end

    A & B & C & D -->|Regroupement auto ou manuel| E
```

### Piles repliées vs. Piles déployées
- **Repliée (par défaut)** : S'affiche sous forme d'une carte unique simulant un paquet de cartes superposées avec un badge numérique de décompte (ex. `[ 4 ]`).
- **Déployée** : Un clic sur le badge de décompte déploie directement toutes les variations de la pile dans la grille de la galerie, permettant de les noter, de les inspecter ou de les supprimer individuellement.

---

## 2. Empilement automatique en rafales (`auto_stack_images`)

Berry peut détecter et regrouper automatiquement les rafales de génération consécutives en arrière-plan :

### Critères de regroupement :
1. **Similarité des jetons de prompt** : Calcule la similarité de Jaccard tokenisée sur les prompts positifs. Vous pouvez ajuster le seuil requis dans **Préférences > Piles** (par défaut : `0.85` / 85 % de correspondance).
2. **Fenêtre temporelle maximale** : Les rafales génératives se produisent généralement dans un intervalle rapproché. Berry regroupe les variations créées dans un intervalle de temps configurable (par défaut : `180 minutes`).
3. **Exécution** : Vous pouvez lancer l'empilement automatique à la demande via **Outils > Organiser la bibliothèque par prompt (Tous les dossiers ou Dossier actuel)**, ou laisser Berry regrouper les images au fur et à mesure de leur ingestion.

---

## 3. Opérations manuelles d'empilement

Vous pouvez créer, dissocier et ajuster les piles à l'aide des raccourcis clavier :

| Action | Raccourci | Description |
| :--- | :--- | :--- |
| **Grouper dans une pile** | `Ctrl + G` / `Cmd + G` | Regroupe toutes les images indépendantes ou piles sélectionnées en une seule pile. |
| **Dissocier la pile** | `Ctrl + Maj + G` / `Cmd + Shift + G` | Dissout la pile sélectionnée pour rétablir les cartes individuelles indépendantes. |
| **Définir comme couverture** | `Alt + S` / `Option + S` | Définit l'image active comme couverture principale (`stack_order = 0`). |

### Sécurité et aplatissement de la fusion de piles
Dans Berry AI Studio, les piles **ne peuvent pas être imbriquées** (une pile ne peut pas contenir une autre pile). Lorsque vous sélectionnez plusieurs piles et appuyez sur `Ctrl + G` :
- Berry aplatit automatiquement toutes les piles sources au sein de la pile cible.
- Une boîte de dialogue de confirmation (`StackMergeWarningModal.vue`) s'affiche pour éviter tout regroupement accidentel.
- Vous pouvez cocher *« Ne plus afficher cet avertissement »* (réinitialisable dans **Préférences > Piles > Réinitialiser les avertissements**).

---

## 4. Arbitrage des gestes : Clic simple vs Double-clic

Pour garantir une interaction fluide sans conflit de gestes :
- **Clic simple sur le badge de décompte** : Déploie ou replie la pile directement sur place.
- **Clic simple sur le corps de la carte** : Sélectionne la pile (temporisation de 240 ms).
- **Double-clic sur le corps de la carte** : Annule immédiatement la temporisation de déploiement et ouvre l'image **Hero** dans l'**Aperçu Lightbox plein écran**.

---

## 5. Comparaison synchronisée côte à côte (`CompareModal.vue`)

Lorsqu'il s'agit de trancher entre des variations subtiles (ex. rendu des yeux, anatomie des mains, éclairage) :
1. Sélectionnez deux images dans la galerie.
2. Appuyez sur `C` (ou cliquez sur **Comparer**).
3. La fenêtre modale de **Comparaison d'images côte à côte** s'ouvre :
   - L'**Emplacement A (Gauche)** et l'**Emplacement B (Droite)** affichent les deux images côte à côte.
   - **Zoom & Panoramique synchronisés** : Glisser ou utiliser la molette de la souris sur l'une des images déplace et zoome les deux simultanément, facilitant une comparaison pixel à pixel à 100 %.
   - **Comparaison des métadonnées en affichage tête haute** : Souligne les différences de Seed, de Modèle, d'Étapes et d'échelle CFG.
   - **Bouton « ★ Définir comme couverture »** : Cliquez pour définir l'image gagnante comme couverture de sa pile.

---

## 6. Outil Élaguer les brouillons (`CullDraftsModal.vue`)

Une fois votre image de couverture sélectionnée et vos favorites notées 3★ et plus :
1. Sélectionnez la pile et cliquez sur **« Élaguer les brouillons »** dans la barre d'actions flottante.
2. La fenêtre présente toutes les variations non vedettes notées en dessous de votre seuil.
3. Cliquez sur **« Déplacer vers la corbeille »** pour envoyer les ébauches indésirables dans la corbeille de votre système en un seul clic, récupérant ainsi de l'espace disque tout en préservant vos créations de premier ordre.
