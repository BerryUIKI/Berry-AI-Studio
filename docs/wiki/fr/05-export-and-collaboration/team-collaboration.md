# Collaboration d'équipe & Base de données partagée

Pour les studios de design, les éditeurs de jeux vidéo et les agences visuelles regroupant plusieurs artistes travaillant sur un stockage réseau partagé (NAS, SMB, NFS), Omera peut évoluer au-delà de SQLite local pour devenir un **Studio de collaboration d'équipe multi-bases de données**.

---

## 1. Architecture multi-bases de données

Omera intègre une couche d'abstraction asynchrone `StorageEngine` prenant en charge trois moteurs :

| Moteur | Taille d'équipe recommandée | Modèle de concurrence | Profil de performance |
| :--- | :--- | :--- | :--- |
| **SQLite (Par défaut)** | 1 utilisateur par bibliothèque | Écrivain unique / Lecteurs multiples WAL | Latence <0,5 ms sur SSD NVMe locaux. |
| **MySQL 8.0+ / MariaDB** | 2 à plus de 50 utilisateurs simultanés | Verrouillage par ligne & indexation plein texte `ngram` | Requêtes <5 ms sur des bibliothèques partagées de plus de 500 000 actifs. |
| **PostgreSQL 14+** | 2 à plus de 100 utilisateurs simultanés | MVCC, index GIN `tsvector` & `LISTEN/NOTIFY` | Collaboration temps réel ultra-faible latence par diffusion d'événements. |

```mermaid
graph TD
    NAS[(NAS partagé du Studio : SMB / NFS / WebDAV)]
    DB[(Base centrale du Studio : PostgreSQL 14+ / MySQL 8+)]

    subgraph Poste de travail A (Windows)
        A_UI[Interface Omera]
        A_Thumb[Cache local de miniatures NVMe]
        A_UI --- A_Thumb
    end

    subgraph Poste de travail B (macOS)
        B_UI[Interface Omera]
        B_Thumb[Cache local de miniatures NVMe]
        B_UI --- B_Thumb
    end

    A_UI -->|Z:\ai_vault| NAS
    B_UI -->|/Volumes/ai_vault| NAS

    A_UI <-->|Vérifications de version OCC & Synchro| DB
    B_UI <-->|Vérifications de version OCC & Synchro| DB
```

---

## 2. Mappage des racines de stockage multiplateformes

Un défi majeur dans les environnements multiplateformes réside dans le fait que Windows, macOS et Linux utilisent des formats de chemins différents pour désigner le même dossier réseau partagé :
- Windows : `Z:\ai_vault\2026\character_01.png`
- macOS : `/Volumes/ai_vault/2026/character_01.png`
- Linux : `/mnt/nas/ai_vault/2026/character_01.png`

### Comment Omera résout cette difficulté :
1. **UUIDs de racines universels** : Omera génère un identifiant UUID universel pour la racine de stockage partagée (consigné dans la table de base de données `storage_roots`).
2. **URIs normalisés** : Dans la base de données centrale, les chemins sont stockés de façon agnostique vis-à-vis du système d'exploitation :
   ```
   omera://550e8400-e29b-41d4-a716-446655440000/2026/character_01.png
   ```
3. **Mappage des points de montage côté client** : Dans **Préférences > Équipe & Base de données**, chaque artiste associe l'UUID de la racine à son point de montage local. Omera traduit les chemins dynamiquement à la volée, de sorte qu'une œuvre étiquetée par un graphiste sous macOS est instantanément accessible à un collègue sous Windows.

---

## 3. Contrôle de concurrence optimiste (OCC)

Lorsque plusieurs membres de l'équipe notent, étiquettent ou trient simultanément la même collection, Omera élimine tout risque de corruption de données grâce au **Contrôle de concurrence optimiste (OCC)** :

- Chaque enregistrement d'actif inclut une colonne `version` au niveau de la ligne.
- Lorsqu'un artiste modifie une note, Omera exécute `set_file_rating_occ(file_id, new_rating, expected_version)`.
- **Politiques de résolution des conflits** :
  - **Notes & Images de couverture (Hero)** : Règle du dernier écrivain gagnant (Last-Write-Wins - LWW) avec mise à jour immédiate de l'interface.
  - **Tags & Albums** : Fusion par union d'ensembles (si l'artiste A ajoute le tag `"Personnage"` et l'artiste B le tag `"Concept"`, les deux tags sont conservés).
  - **Suppressions** : Des indicateurs d'état de suppression douce (soft-delete) empêchent les recréations fantômes.

---

## 4. Paliers de synchronisation collaborative en temps réel

Omera synchronise les changements d'état entre les membres de l'équipe selon une architecture à 3 paliers :

1. **Palier 1 : Scrutation du journal des modifications (Par défaut / Zéro-DevOps)** :
   - Omera interroge la table journal `change_log` toutes les 3 secondes pour détecter de nouveaux identifiants de transaction. Ne nécessite aucune configuration serveur particulière.
2. **Palier 2 : `LISTEN / NOTIFY` PostgreSQL (Latence <50 ms)** :
   - Avec PostgreSQL, Omera établit un canal de notification asynchrone. Les modifications appliquées par n'importe quel poste sont diffusées aux autres clients en moins de 50 ms.
   - **Palier 3 : Hub WebSocket distribué** :
   - Service de diffusion léger optionnel pour les studios et déploiements d'entreprise de grande envergure.

---

## 5. Cache de miniatures NVMe à la demande côté client

Charger des miniatures à travers un réseau d'entreprise à 1 Gbit/s ou 10 Gbit/s peut rapidement saturer la bande passante partagée :
- Omera stocke les miniatures WebP redimensionnées sur le **SSD NVMe local** de chaque poste de travail (`thumbnails/`).
- Lorsqu'un artiste explore la bibliothèque partagée, les miniatures sont générées et mises en cache localement à la demande.
- Cela maintient le trafic réseau proche de zéro lors des défilements rapides dans la galerie, libérant ainsi la bande passante du NAS pour les rendus haute résolution et l'entraînement de modèles.

---

## 6. Assistant de migration : de SQLite vers MySQL / PostgreSQL (`MigrationWizardModal.vue`)

Si vous avez débuté avec une bibliothèque locale SQLite mono-utilisateur et souhaitez évoluer vers une base d'équipe partagée :
1. Ouvrez **Préférences > Équipe & Base de données**.
2. Cliquez sur **« Ouvrir l'assistant de migration... »**.
3. Omera inspecte votre base locale SQLite, vous permet de choisir MySQL 8.0+ ou PostgreSQL 14+, et génère les schémas DDL optimisés ainsi que les scripts SQL de migration transactionnelle par lots.
4. Exécutez le script généré sur votre serveur de base de données pour basculer la bibliothèque de votre studio.
