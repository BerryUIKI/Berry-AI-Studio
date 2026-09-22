# Interopérabilité des outils de génération

Berry AI Studio agit comme un compagnon actif de vos outils de génération créative, offrant une communication bidirectionnelle par API avec **ComfyUI** et **AUTOMATIC1111 / SD.Next**.

---

## 1. Configurer les services de génération

Configurez les points de terminaison de connexion dans **Préférences > Métadonnées** ou la section dédiée :

- **URL de base ComfyUI** : Par défaut `http://127.0.0.1:8188`
- **URL de base SD WebUI** : Par défaut `http://127.0.0.1:7860`

Chaque champ de configuration intègre un bouton **« Tester la connexion »**. Berry envoie un test d'état de fonctionnement léger (`/system_stats` pour ComfyUI ou `/sdapi/v1/options` pour WebUI) et affiche un indicateur visuel **En ligne (🟢)** ou **Hors ligne (🔴)**.

---

## 2. Envoi de workflows ComfyUI (`send_to_comfyui`)

Lors de la consultation d'une œuvre ou d'une vidéo générée avec ComfyUI :
1. Dans l'Inspecteur de propriétés à droite, repérez la **Carte d'interopérabilité de génération**.
2. Cliquez sur **« Envoyer à ComfyUI »**.
3. Le backend de Berry se connecte au point de terminaison HTTP `/prompt` de ComfyUI et soumet le graphe de nœuds exact ainsi que les paramètres d'espace latent extraits de l'image.
4. Votre instance de ComfyUI charge immédiatement le flux de travail et l'ajoute à la file d'attente de génération — sans nécessiter le moindre glisser-déposer de fichiers entre fenêtres.

---

## 3. Envoi de prompts à SD WebUI (`send_to_webui`)

Pour les images produites dans AUTOMATIC1111, Forge ou SD.Next :
1. Cliquez sur **« Envoyer à SD WebUI »** dans l'Inspecteur.
2. Berry formule une charge utile de génération txt2img contenant le prompt positif, le prompt négatif, les étapes, le sampler, l'échelle CFG, la seed et les dimensions.
3. Cette charge est transmise à l'API `/sdapi/v1/txt2img` de WebUI, préremplissant les champs de votre interface ou planifiant une nouvelle session de calcul.

---

## 4. Boucle de surveillance active en pipeline

En associant l'**Interopérabilité de génération** à un dossier en **Mode Pipeline AIGC** (Mode C) :
1. Vous envoyez un flux de travail vers ComfyUI ou WebUI depuis Berry.
2. Le générateur calcule l'image et l'enregistre dans son répertoire de sortie.
3. L'observateur d'arrière-plan de Berry détecte le nouveau fichier, attend la libération des verrous d'écriture (anti-rebond de 500 ms), extrait les métadonnées, génère une miniature WebP et affiche la création au sommet de votre galerie en temps réel.
4. Cela crée une boucle créative continue et sans friction entre génération et curation.
