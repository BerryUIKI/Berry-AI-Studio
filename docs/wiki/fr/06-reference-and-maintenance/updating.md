# Mises à jour & Cycle de vie

Berry AI Studio intègre un module de mise à jour automatique silencieuse sur place, conçu pour vous apporter des améliorations et des correctifs sans interrompre votre travail ni mettre vos données en danger.

---

## 1. Vérification des mises à jour

### Recherche automatique au démarrage :
Par défaut, Berry interroge l'API GitHub Releases au lancement de l'application :
- Si une nouvelle version est disponible, un badge indicateur de mise à jour apparaît dans le menu **Aide**.
- Vous pouvez activer ou désactiver cette option dans **Préférences > Général > Vérifier les mises à jour au démarrage**.

### Recherche manuelle :
Vous pouvez lancer une vérification manuelle à tout moment :
- Cliquez sur **Aide > Vérifier les mises à jour...** dans la barre de menus supérieure.

---

## 2. Fenêtre de mise à jour sur place (`UpdateModal.vue`)

Lorsqu'une nouvelle version est détectée, la **Fenêtre de mise à jour** s'affiche :

```
┌────────────────────────────────────────────────────────────────────────┐
│ Nouvelle version disponible : v0.3.1                               [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Une nouvelle version de Berry AI Studio est disponible (Actuelle: v0.3.0). │
│                                                                        │
│ Notes de version :                                                     │
│ • Optimisation de l'analyseur ComfyUI vidéo pour HunyuanVideo.         │
│ • Amélioration de la pagination par curseur sur les bases de +100k.   │
│ • Ajout du sélecteur de note à 10 étoiles dans la barre d'actions.     │
├────────────────────────────────────────────────────────────────────────┤
│ Progression du téléchargement :                                        │
│ [██████████████████████████░░░░░░░░░░] 68% (12.4 Mo / 18.2 Mo · 4 Mo/s)│
├────────────────────────────────────────────────────────────────────────┤
│ [ Annuler ]                         [ 🚀 Mettre à jour automatiquement ]│
└────────────────────────────────────────────────────────────────────────┘
```

### Déroulement de l'installation :
1. Cliquez sur **« 🚀 Télécharger & mettre à jour automatiquement »**.
2. Berry télécharge le paquet de mise à jour adapté à votre plateforme directement depuis GitHub Releases dans un dossier temporaire.
3. Une fois le téléchargement achevé, Berry vous invite à redémarrer l'application.
4. L'installateur natif s'exécute silencieusement sur place et relance automatiquement le studio.

---

## 3. Garanties de préservation des données

Mettre à niveau Berry AI Studio **ne modifie ni n'efface jamais vos données personnelles** :

- **Sécurité de la base de données** : Votre base SQLite `berry.db`, vos albums, vos tags de couleur, vos notes et les relations d'empilement sont conservés dans le répertoire AppData de votre profil utilisateur (`%APPDATA%`, `~/Library/Application Support` ou `~/.config`), indépendamment de l'emplacement des binaires du logiciel.
- **Migrations de schéma additives** : Lorsqu'une mise à jour logicielle intègre des évolutions de base de données, le backend Rust de Berry exécute des **migrations de schéma additives** au démarrage en s'appuyant sur `PRAGMA user_version`. Les migrations adaptent les tables de façon incrémentielle sans jamais réécrire ni altérer vos enregistrements existants.
- **Préférences durables** : Votre fichier de réglages `config.json`, votre budget de cache de miniatures et l'ensemble de vos dossiers enregistrés demeurent intacts à chaque mise à jour.
