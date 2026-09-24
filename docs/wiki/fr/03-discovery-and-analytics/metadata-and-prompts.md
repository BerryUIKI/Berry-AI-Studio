# Métadonnées AIGC & Inspection de prompts

Omera intègre un moteur d'analyse de métadonnées sans perte multi-plateformes écrit en Rust (`omera-metadata`). Il extrait automatiquement les prompts, les prompts négatifs, les modèles, les graines (seeds) et les graphes d'exécution de toutes les principales plateformes de génération par IA.

---

## 1. Plateformes de génération IA prises en charge

Omera interprète nativement les métadonnées intégrées dans les chunks PNG, les en-têtes EXIF WebP et les boîtes ISOBMFF MP4 provenant de :

| Plateforme / Outil | Champs de métadonnées extraits | Emplacement dans le conteneur |
| :--- | :--- | :--- |
| **AUTOMATIC1111 / SD.Next / Forge** | Prompt, Prompt négatif, Étapes (Steps), Sampler, CFG, Seed, Dimensions, Hash du modèle, Nom du modèle, Denoising, Sur-échantillonnage Hires | Chunk PNG `parameters` / EXIF JPEG `UserComment` |
| **ComfyUI** | Graphe complet d'exécution des nœuds, encodage texte CLIP positif/négatif, seeds KSampler, étapes, CFG, chargeurs de checkpoints, chargeurs de LoRA, nœuds d'upscale latent | Chunks PNG `prompt` & `workflow` / Chunks WebP ComfyUI / MP4 `moov/udta` |
| **NovelAI** | Titre, Description, Prompt, Prompt négatif, Seed, Sampler, Étapes, Échelle, Version logicielle | Chunks PNG `Comment` & `Description` |
| **Fooocus / Fooocus-MRE** | Modèle de base, Raffineur (Refiner), Poids LoRA, Netteté, Mode de performance, Résolution, Prompt | Blocs de texte PNG `parameters` |
| **InvokeAI** | Nom du modèle, VAE, Scheduler, Mode de génération (txt2img/img2img), Répétition sans raccord | JSON PNG `sd-metadata` & `invokeai_metadata` |
| **EasyDiffusion / Stable Swarm** | Blocs formatés de paramètres JSON, Seed, Nom du modèle | PNG `sui_image_params` / Fichiers auxiliaires |

---

## 2. Le volet d'inspection des propriétés (`InspectorPane.vue`)

Lorsqu'une image ou une vidéo est sélectionnée, le panneau de droite (raccourci `I`) présente clairement ses métadonnées :

```
┌────────────────────────────────────────────────────────┐
│ INSPECTEUR DE PROPRIÉTÉS                           [✕] │
├────────────────────────────────────────────────────────┤
│ [ Aperçu miniature du média ]                          │
│ 1024 × 1024 · PNG · 3.4 Mo · 2026-09-21                │
│ [ ★★★★★ ]  [ ★ Favori ]  [ 🔞 Sensible ]               │
├────────────────────────────────────────────────────────┤
│ Prompt positif                            [📋 Copier]  │
│ ┌────────────────────────────────────────────────────┐ │
│ │ [masterpiece] [1girl] [solo] [cyberpunk city]      │ │
│ │ [neon reflections] [rain] [volumetric lighting]    │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Prompt négatif                            [📋 Copier]  │
│ ┌────────────────────────────────────────────────────┐ │
│ │ worst quality, low quality, bad anatomy, bad hands │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Paramètres de génération                               │
│ Modèle :   animagine_xl_3.1.safetensors                │
│ Hash :     31e35c80  [🔍 Trouver sur Civitai]          │
│ Sampler :  DPM++ 2M Karras                             │
│ Étapes :   28             Échelle CFG : 7.0            │
│ Seed :     2849104812     [📋 Copier]                  │
├────────────────────────────────────────────────────────┤
│ LoRAs détectés (2)                                     │
│ • CyberpunkStyle (Poids : 0.85)           [+ Au prompt]│
│ • DetailedEyes (Poids : 0.6)              [+ Au prompt]│
├────────────────────────────────────────────────────────┤
│ ▼ Métadonnées brutes (JSON du workflow ComfyUI)        │
└────────────────────────────────────────────────────────┘
```

---

## 3. Jetons de prompt interactifs (Chips)

Omera segmente les chaînes de prompts sous forme de jetons interactifs au lieu d'afficher un bloc de texte brut difficile à lire :

- **Recherche en 1 clic** : Cliquez sur n'importe quel jeton (ex. `[cyberpunk city]`) pour lancer immédiatement une recherche de ce concept sur l'ensemble de votre bibliothèque.
- **Copie en 1 clic** : Cliquez sur l'icône de copie dans le coin supérieur droit du cadre de prompt pour copier le texte propre directement dans votre presse-papier.
- **Découverte de styles** : Ces jetons interactifs facilitent l'identification des styles d'artistes, des termes d'éclairage ou des mots-clés de qualité que vous souhaitez réutiliser.

---

## 4. Modèles Checkpoint & Résolution de hash

Les outils de génération intègrent fréquemment des hashs courts (ex. `31e35c80`) ou des hashs SHA256 complets plutôt que des noms de fichiers lisibles.

- Omera consulte automatiquement son **Cache de modèles** local (table SQLite `model_cache`) pour convertir les hashs cryptiques en noms conviviaux tels que `"Animagine XL 3.1"`.
- Si un hash inconnu est rencontré, vous pouvez importer un fichier `cache.json` d'AUTOMATIC1111 ou le résoudre directement via Civitai (voir [Modèles Checkpoint & Bibliothèque LoRA](../04-intelligent-curation/models-and-loras.md)).

---

## 5. Métadonnées brutes & JSON de workflow ComfyUI

Pour les utilisateurs avancés et les directeurs techniques souhaitant examiner les connexions de nœuds :
- Déployez l'accordéon **Métadonnées brutes** situé tout en bas de l'inspecteur pour consulter le contenu JSON d'origine sans altération.
- Vous pouvez copier l'intégralité du bloc JSON de workflow pour le coller directement dans un éditeur de texte ou le partager avec vos collègues.
