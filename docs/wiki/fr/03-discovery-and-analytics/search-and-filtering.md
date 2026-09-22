# Syntaxe de recherche & Filtres visuels

Berry AI Studio intègre un double moteur de recherche : la **Recherche syntaxique structurée** pour un filtrage technique de haute précision, et la **Recherche sémantique IA** pour les requêtes conceptuelles en langage naturel.

---

## 1. Recherche syntaxique & Langage de requête

Dans la barre de recherche supérieure (`/` ou `Ctrl + F`), vous pouvez saisir des mots-clés libres ou composer des requêtes clé-valeur structurées :

### Recherche textuelle libre de base
- Saisir des mots sans préfixe cible simultanément les noms de fichiers, les chemins relatifs, les prompts positifs, les prompts négatifs et les noms de modèles :
  ```
  cyberpunk neon rain
  ```
- Utilisez des guillemets doubles pour rechercher une expression exacte :
  ```
  "cyberpunk street" "rainy reflections"
  ```

---

## 2. Référence de la syntaxe clé-valeur

Berry analyse les jetons de recherche en une structure `SearchCriteria` en Rust natif, interrogeant les index SQLite avec une exécution inférieure à la milliseconde :

| Clé de jeton | Exemple de syntaxe | Description |
| :--- | :--- | :--- |
| `prompt:` | `prompt:"masterpiece, 1girl"` | Recherche des termes dans le prompt positif. |
| `neg:` | `neg:"bad hands, blurry"` | Recherche des termes dans le prompt négatif. |
| `model:` | `model:"animagine_xl"` | Filtre par nom de checkpoint de modèle. |
| `hash:` | `hash:31e35c80` | Recherche par hash court ou hash SHA256 complet. |
| `sampler:` | `sampler:"Euler a"` | Filtre par algorithme d'échantillonnage (Sampler). |
| `steps:` | `steps:30`, `steps:20..40`, `steps:>=25` | Filtre par nombre précis d'étapes ou par plage. |
| `cfg:` | `cfg:7`, `cfg:>=7.5`, `cfg:5..10` | Filtre par échelle de guidage CFG ou plage de valeurs. |
| `seed:` | `seed:12345678` | Recherche une graine (Seed) de génération spécifique. |
| `rating:` | `rating:5`, `rating:>=4`, `rating:1..3` | Filtre par note par étoiles attribuée. |
| `aesthetic:`| `aesthetic:>=7.0` | Filtre par score esthétique prédictif. |
| `fav:` | `fav:true`, `fav:false` | Filtre selon le statut de mise en favori. |
| `is:` | `is:nsfw`, `is:sfw` | Filtre par indicateur de contenu sensible. |
| `type:` | `type:image`, `type:video` | Filtre par type de conteneur média. |
| `duration:` | `duration:>=5`, `duration:5..30` | Filtre la durée vidéo en secondes. |
| `fps:` | `fps:>=30`, `fps:24..60` | Filtre la fréquence d'images par seconde (FPS). |

### Syntaxe des plages numériques
- **Plage comprise entre (`min..max`)** : `steps:20..35` (entre 20 et 35 étapes incluses).
- **Supérieur ou égal (`>=`)** : `cfg:>=7.0` (échelle de guidage de 7.0 ou plus).
- **Inférieur ou égal (`<=`)** : `rating:<=2` (2 étoiles ou moins).
- **Correspondance exacte** : `rating:5` (exactement 5 étoiles).

---

## 3. Tiroir de filtres visuels (`FilterDrawer.vue`)

Si vous préférez une interface graphique plutôt que de taper une syntaxe de requête, cliquez sur le bouton **Filtres (`☰ Filtres`)** à droite de la barre de recherche pour déployer le tiroir latéral :

```
┌────────────────────────────────────────────────────────┐
│ Recherche visuelle & Filtres        [Réinitialiser] [✕] │
├────────────────────────────────────────────────────────┤
│ Modèle Checkpoint                                      │
│ [ Tous les modèles ▾                                 ] │
│                                                        │
│ Sampler                                                │
│ [ Tous les samplers ▾                                ] │
│                                                        │
│ Note par étoiles                                       │
│ [ ★★★★★ (5 étoiles uniquement) ▾                      ] │
│                                                        │
│ Plage d'étapes (Steps)                                 │
│ Min : [ 20 ] ────────────●──────────── Max : [ 50 ]    │
│                                                        │
│ Plage d'échelle CFG                                    │
│ Min : [ 5.0 ] ───────────●──────────── Max : [ 12.0 ]  │
│                                                        │
│ Type de média                                          │
│ (●) Tous        ( ) Images           ( ) Vidéos        │
│                                                        │
│ Indicateurs de contenu                                 │
│ [✓] Favoris uniquement     [ ] Sensible (NSFW)         │
└────────────────────────────────────────────────────────┘
```

Le tiroir de filtres comporte un compteur de filtres actifs qui vous permet de constater d'un seul coup d'œil combien de critères restreignent vos résultats actuels.

---

## 4. Options de tri de la galerie (`SortBar.vue`)

À droite de l'en-tête du canevas de la galerie, vous pouvez trier vos résultats actifs :

### Champs de tri :
- **Date de modification (`modified_at`)** : Classement chronologique d'après l'horodatage des fichiers.
- **Nom de fichier / Chemin (`path`)** : Ordre alphabétique d'après le chemin sur le disque.
- **Taille de fichier (`size_bytes`)** : Classement selon l'espace disque occupé.
- **Note (`rating`)** : Ordre selon les notes par étoiles attribuées.
- **Score esthétique (`aesthetic_score`)** : Classement par le score prédictif esthétique neuronal.

### Sens du tri :
- Cliquez sur le bouton de direction pour basculer entre **Décroissant (`↓`)** (du plus grand / récent au plus petit / ancien) et **Croissant (`↑`)** (du plus petit / ancien au plus grand / récent).
