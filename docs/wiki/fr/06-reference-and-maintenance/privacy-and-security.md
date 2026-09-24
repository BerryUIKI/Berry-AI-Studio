# Architecture de confidentialité & Sécurité

Omera est conçu selon une philosophie fondamentale : **100 % axé sur le local (« Local-First »), zéro télémétrie**. À une époque où les flux de travail en IA générative manipulent des styles graphiques exclusifs, des concepts de personnages confidentiels et des actifs clients sensibles, Omera garantit que vos œuvres restent strictement confinées sur votre machine.

---

## 1. Zéro télémétrie & Fonctionnement hors ligne

### Aucune communication réseau cachée
- Omera ne contient **aucun pixel espion, aucun kit d'analyse comportementale (SDK) et aucun traqueur d'erreurs externe** (pas de Google Analytics, Sentry, Mixpanel ou PostHog).
- Vous pouvez faire fonctionner Omera de manière totalement déconnectée d'Internet ou au sein d'environnements d'entreprise hautement sécurisés en réseau isolé (air-gapped) sans la moindre dégradation des fonctionnalités.

### Requêtes réseau sortantes autorisées
Omera n'établit de connexions réseau que dans **trois cas explicites** :
1. **Vérification des mises à jour** : Si l'option de vérification automatique est active (ou déclenchée manuellement via `Aide > Vérifier les mises à jour...`), Omera interroge l'API publique de GitHub Releases (`https://api.github.com/repos/BerryUIKI/Omera/releases/latest`).
2. **Résolution de hash Civitai** : Lorsque vous cliquez explicitement sur le bouton de recherche Civitai pour un hash de modèle inconnu, Omera envoie une requête HTTP unique vers le point de terminaison public de l'API Civitai.
3. **Sauvegarde Cloud & Synchronisation d'équipe** : Uniquement si vous avez configuré un compartiment AWS S3, un serveur WebDAV ou une base de données centrale PostgreSQL/MySQL dans les Préférences.

---

## 2. Inférence d'intelligence artificielle exclusivement locale

Toutes les fonctionnalités d'apprentissage automatique intégrées à Omera s'exécutent entièrement sur votre CPU ou GPU local via des sessions **ONNX Runtime** intégrées (`ort`) :

- **Embeddings vectoriels CLIP / SigLIP** : Le prétraitement des images et l'encodage des vecteurs s'opèrent localement. Aucun prompt, aucune requête textuelle et aucun pixel d'image ne sont jamais transmis à des serveurs distants.
- **Étiquetage automatique anime WD14** : L'inférence neuronale s'exécute directement sur les fichiers de poids locaux (`models/`). Les prédictions de balises sont écrites immédiatement dans votre base de données SQLite locale.

---

## 3. Exportation et assainissement respectueux de la vie privée

Lors de l'exportation ou de la préparation d'œuvres destinées à une diffusion publique, Omera intègre un **moteur d'assainissement des métadonnées sur 4 niveaux** :

- Vous pouvez retirer d'un seul clic les graphes de nœuds ComfyUI, les prompts positifs/négatifs, les graines et les LoRAs avant de publier vos images sur les réseaux sociaux ou Discord (voir [Exportation & Vitrine Web](../05-export-and-collaboration/export-and-web-showcase.md)).
- Le palier **Nettoyage complet** élimine tous les segments EXIF et ICC, ne conservant que les pixels purs.

---

## 4. Transparence Open-Source & Licence

Omera est un logiciel libre et open-source publié sous la licence **GNU Affero General Public License v3.0 (AGPL-3.0)** :

- Chaque ligne de code du backend en Rust, des commandes de passerelle Tauri et des composants d'interface en Vue 3 est publiquement vérifiable sur [GitHub](https://github.com/BerryUIKI/Omera).
- Vous disposez d'une totale liberté pour auditer, compiler, créer un embranchement (fork) ou déployer Omera dans votre environnement personnel ou au sein d'un studio professionnel.
