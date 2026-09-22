# Modèles Checkpoint & Bibliothèque LoRA

Gérer des centaines de modèles de checkpoints Stable Diffusion et de LoRAs affinés représente un véritable défi au quotidien. Berry AI Studio intègre un système de catalogage, de résolution de hashs et de gestion des mots déclencheurs (trigger words).

---

## 1. Catalogue de modèles Checkpoint (`ModelManagerModal.vue`)

Berry suit automatiquement chaque modèle de checkpoint rencontré dans l'ensemble de votre bibliothèque.

### Découverte automatique des modèles :
- Lors de l'indexation des fichiers, Berry analyse les noms de checkpoints et les hashs de modèles à partir des métadonnées intégrées PNGInfo/EXIF.
- Ouvrez **Outils > Gestionnaire de modèles...** pour afficher le catalogue de tous les modèles, leurs hashs courts, leurs sommes de contrôle SHA256 complètes et le nombre total d'images associées.

### Résolution des hashs avec le `cache.json` d'AUTOMATIC1111 :
- Les modèles de checkpoints apparaissent fréquemment sous forme de hashs courts à 8 caractères (ex. `31e35c80`).
- Si vous disposez d'une installation d'AUTOMATIC1111 :
  1. Cliquez sur **« Importer A1111 cache.json »** dans le Gestionnaire de modèles.
  2. Sélectionnez votre fichier WebUI `cache.json` (situé généralement à la racine de votre installation WebUI).
  3. Berry importe ces correspondances dans sa table locale `model_cache`, résolvant instantanément les hashs cryptiques de toute votre bibliothèque en noms de modèles explicites.

### Résolution de hash SHA256 avec Civitai :
- Pour les modèles sans nom local identifié, vous pouvez cliquer sur le bouton de recherche Civitai dans l'Inspecteur pour interroger la base publique de Civitai au moyen du hash du modèle.

---

## 2. Bibliothèque de déclencheurs LoRA (`LoraManagerModal.vue`)

Les adaptations de bas rang (LoRAs) nécessitent des mots d'activation ou déclencheurs (trigger words) spécifiques dans vos prompts pour reproduire fidèlement les détails d'un personnage, un style artistique ou une tenue vestimentaire.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Bibliothèque de déclencheurs LoRA                                  [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [🔍 Rechercher LoRAs... ]              [+ Ajouter LoRA] [📁 Scanner]   │
├────────────────────────────────────────────────────────────────────────┤
│ Nom du LoRA             │ Poids rec. │ Mots déclencheurs      │ Actions │
├─────────────────────────┼────────────┼────────────────────────┼─────────┤
│ CyberpunkCityStyle      │ 0.80       │ cyberpunk, neon signs, │ [Copier]│
│                         │            │ futuristic alleys      │ [Inject]│
│ GenshinRaidenShogun     │ 0.85       │ raiden shogun, purple  │ [Copier]│
│                         │            │ braid, glowing katana  │ [Inject]│
│ StudioGhibliVintage     │ 0.70       │ ghibli style, vintage  │ [Copier]│
│                         │            │ watercolor, cel shaded │ [Inject]│
└────────────────────────────────────────────────────────────────────────┘
```

### Fonctionnalités clés des LoRAs :
1. **LoRAs détectés dans l'Inspecteur** :
   - Lors de la visualisation d'une création, l'Inspecteur de propriétés détecte automatiquement la syntaxe de prompt `<lora:nom:poids>` et les nœuds `LoraLoader` de ComfyUI.
   - Il liste les LoRAs détectés, les poids appliqués et les mots déclencheurs reconnus.
2. **Injection de prompt en un clic** :
   - Cliquez sur **« Copier avec <lora> »** pour copier la chaîne formatée `<lora:nom:0.8>` directement dans votre presse-papier.
   - Cliquez sur n'importe quel badge de mot déclencheur pour le copier et l'insérer immédiatement dans votre éditeur de prompt.
3. **Importation des métadonnées annexes Civitai (`.civitai.info`)** :
   - Si vous téléchargez des LoRAs accompagnés de fichiers d'information Civitai (`.civitai.info` ou `.json`), Berry extrait automatiquement les hashs de modèles, les architectures de base (SD 1.5, SDXL, Pony, Flux), les mots déclencheurs entraînés et les images d'aperçu.
4. **Analyse du répertoire local** :
   - Pointez Berry vers votre dossier local de LoRAs (`models/Lora/`).
   - Berry analyse tous les fichiers `.safetensors` et leurs images d'aperçu d'accompagnement, créant ainsi une bibliothèque de référence hors ligne complète et interrogeable.
