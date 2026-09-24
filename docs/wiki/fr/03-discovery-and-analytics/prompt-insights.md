# Analyses de prompts & Statistiques

À mesure que votre collection s'enrichit, comprendre quels mots-clés de prompts, artistes et paramètres techniques produisent vos œuvres les mieux notées devient indispensable. Omera propose une fenêtre d'analyses dédiée (`PromptStatsModal.vue`) qui agrège les métadonnées de toute votre bibliothèque.

---

## 1. Ouvrir les analyses de prompts

Pour lancer le tableau de bord d'analyses :
- Cliquez sur **Outils > Statistiques des prompts...** dans la barre de menus supérieure, ou
- Cliquez sur le bouton **Statistiques** dans le pied de page des outils rapides de la barre latérale gauche.

---

## 2. Onglets d'analyses & Métriques visuelles

```
┌────────────────────────────────────────────────────────────────────────┐
│ Analyses de prompts & métadonnées                                  [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [ Mots positifs ]  [ Mots négatifs ]  [ Modèles ]  [ Samplers ]        │
├────────────────────────────────────────────────────────────────────────┤
│ Classé par : (●) Fréquence    ( ) Note moyenne                         │
├────────────────────────────────────────────────────────────────────────┤
│ Rang │ Jeton / Mot-clé              │ Occurrences │ Note moy.  │ Action│
├──────┼──────────────────────────────┼─────────────┼────────────┼───────┤
│ #1   │ masterpiece                  │ 4 120       │ ★ 4.2      │ [🔍]  │
│ #2   │ cinematic lighting           │ 2 845       │ ★ 4.7      │ [🔍]  │
│ #3   │ 1girl                        │ 2 410       │ ★ 3.9      │ [🔍]  │
│ #4   │ cyberpunk city               │ 1 890       │ ★ 4.8      │ [🔍]  │
│ #5   │ volumetric fog               │ 1 230       │ ★ 4.5      │ [🔍]  │
│ #6   │ photorealistic               │ 1 115       │ ★ 3.2      │ [🔍]  │
└────────────────────────────────────────────────────────────────────────┘
```

Le tableau de bord propose quatre vues analytiques distinctes :

### 1. Mots positifs fréquents
- Évalue chaque jeton de mot-clé individuel présent dans l'ensemble des prompts positifs de génération.
- Affiche le nombre total d'occurrences ainsi que la **Note moyenne par étoiles** des images utilisant ce jeton.
- Met en lumière vos termes clés fétiches — les mots-clés constamment corrélés à des notes de 4★ et 5★.

### 2. Mots négatifs fréquents
- Analyse les expressions de prompts négatifs les plus fréquemment employées dans votre flux de travail.
- Très utile pour repérer les embeddings négatifs superflus ou les mots-clés redondants qui n'améliorent pas la qualité du résultat.

### 3. Modèles populaires
- Classe chaque checkpoint de modèle de votre bibliothèque selon le volume total de générations et les notes attribuées.
- Aide à déterminer avec précision quels modèles affinés (fine-tuned) offrent vos meilleurs résultats.

### 4. Samplers populaires & Schedulers
- Classe les algorithmes d'échantillonnage (ex. `DPM++ 2M Karras`, `Euler a`, `UniPC`) par fréquence d'utilisation et note esthétique.

---

## 3. Intégration de la recherche interactive

Chaque ligne de mot-clé du tableau d'analyses intègre un bouton **Action (`🔍`)** :
- Cliquer sur l'icône de recherche ferme instantanément la fenêtre modale d'analyses, insère le jeton sélectionné dans la barre de recherche de la galerie principale (`prompt:"..."`) et filtre le canevas pour afficher toutes les créations contenant ce mot-clé.
- Cela vous permet de passer d'un simple clic des statistiques globales à l'examen concret des séries d'images correspondantes.
