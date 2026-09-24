# Maintenance de la base de données & du cache

Omera est conçu pour fonctionner en continu sans nécessiter d'intervention manuelle lourde. Cependant, à mesure que vous organisez, supprimez et modifiez des dizaines de milliers d'œuvres, effectuer périodiquement un compactage de la base de données et un entretien du cache garantit des performances optimales.

---

## 1. Fenêtre de maintenance de la base de données (`DatabaseManagerModal.vue`)

Ouvrez la fenêtre de gestion via **Fichier > Gestion de la base...** ou depuis le bouton dédié dans le pied de page de la barre latérale gauche.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Maintenance de la base de données & Métriques                      [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Métriques de stockage                                                  │
│ • Fichier de base :   omera.db (Mode WAL)                              │
│ • Fichiers indexés :  48 210 fichiers répartis sur 6 dossiers          │
│ • Espace disque :     128.4 Mo (Base) / 1.42 Go (Miniatures)           │
│ • Albums & Tags :     12 albums, 45 tags                               │
│ • Version de schéma : v14 (14 migrations appliquées)                   │
│ • Pages libres :      1 420 pages (~5.6 Mo récupérables)               │
├────────────────────────────────────────────────────────────────────────┤
│ Opérations de maintenance                                              │
│ [ 🧹 Exécuter VACUUM ]             [ 💾 Exporter la sauvegarde… ]      │
│ [ ↺ Restaurer depuis une sauvegarde ] [ 🗑 Vider le cache ]           │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Compactage de SQLite (`VACUUM`)

### Qu'est-ce que la fragmentation de base de données ?
Lorsque vous supprimez des fichiers, retirez des tags ou dissociez des piles, SQLite marque les pages de disque correspondantes comme « libres » (freelist) au lieu de réduire instantanément la taille physique du fichier `.db` sur votre disque.

### Exécuter `VACUUM` :
- Cliquer sur **« Exécuter VACUUM »** déclenche la procédure native de compactage de SQLite.
- Omera reconstruit le fichier de base de données dans une structure contiguë et saine, éliminant les pages libres et diminuant l'espace disque occupé.
- **Sécurité** : L'opération de compactage est entièrement transactionnelle. En cas de coupure de courant en plein traitement, SQLite effectue un retour arrière (rollback) sans aucun risque de corruption.

---

## 3. Sauvegarde & Restauration de la base de données

### Exporter une sauvegarde locale (`backup_database`)
- Cliquez sur **« Exporter la sauvegarde… »** pour générer une copie de sauvegarde certifiée sans interruption de service.
- Omera utilise l'API de sauvegarde en ligne de SQLite, vous permettant de réaliser des sauvegardes tout en continuant à utiliser l'application sans ralentissement.

### Restaurer depuis une sauvegarde (`restore_database`)
- Si vous souhaitez réinstaller votre bibliothèque sur un nouvel ordinateur ou annuler des modifications accidentelles, cliquez sur **« Restaurer… »**.
- Omera crée d'abord une copie de secours (rollback) de votre base active, charge le fichier de sauvegarde sélectionné, et recharge instantanément votre studio avec la bibliothèque restaurée.

---

## 4. Gestion du cache des miniatures & Éviction

Les miniatures sont stockées dans `<dossier_app_data>/thumbnails/` sous forme de fichiers WebP ultra-légers.

### Budget de cache disque configurable :
- Dans **Préférences > Galerie**, vous pouvez régler le **Budget de cache des miniatures** (par défaut : `2048 Mo` / 2 Go).
- Omera consigne l'horodatage d'accès de chaque fichier de miniature dans la table de base de données `thumbnail_cache_entries`.
- Dès que l'espace total alloué aux miniatures excède votre budget, Omera élimine automatiquement les fichiers les moins récemment consultés selon une politique **LRU (Least Recently Used)**.

### Vider le cache :
- Si vous souhaitez libérer immédiatement de l'espace disque, cliquez sur **« Vider le cache des miniatures »**.
- Omera supprime l'ensemble des fichiers WebP du disque et réinitialise la table de suivi. Les miniatures seront régénérées au fur et à mesure de votre navigation dans les dossiers.

---

## 5. Fenêtre de diagnostics des miniatures (`ThumbnailDiagnosticsModal.vue`)

Pour l'analyse des performances et le diagnostic technique :
- Ouvrez **Préférences > Galerie > Diagnostics** pour observer :
  - Le nombre de threads de calcul actifs dans le pool Rayon (ex. `4 workers en cours`).
  - La file d'attente des miniatures en attente de génération.
  - Le ratio de tâches terminées par rapport aux tâches annulées.
  - Le taux de succès en mémoire vive du cache LRU.
