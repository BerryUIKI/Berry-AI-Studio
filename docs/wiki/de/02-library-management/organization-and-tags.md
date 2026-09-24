# Organisation, Bewertungen & Tags

Omera bietet leistungsfähige Kurationsmechanismen, um zehntausende Kunstwerke schnell zu sortieren, zu kategorisieren und zu priorisieren, ohne Ihr Dateisystem zu überfrachten.

---

## 1. Sterne-Bewertungen & Bewertungssystem

Omera nutzt ein Bewertungssystem mit doppelter Genauigkeit, das direkt in SQLite gespeichert wird (`files.rating` und `files.aesthetic_score`):

### Sterne-Bewertungen (0 bis 5 Sterne)
- **Tastaturkürzel**: Wählen Sie ein oder mehrere Bilder aus und drücken Sie:
  - `1` bis `5`: Weist die entsprechende Anzahl an Sternen sofort zu.
  - `0`: Löscht die Bewertung.
- **Inspektor / Aktionsleiste**: Unterstützt über Auswahlmenüs auch feinere Abstufungen.
- **Datenbankindizierung**: Die Spalte `rating` ist über SQLite-B-Trees indiziert, was Abfragen wie `rating:>=4` oder `rating:5` selbst in Bibliotheken mit 500.000 Elementen in Sekundenbruchteilen ausführt.

### Favoriten (★ Lesezeichen)
- **Umschalten**: Drücken Sie die Taste `F` oder klicken Sie auf das Sternsymbol im Inspektor.
- **Visuelle Kennzeichnung**: Ein goldenes Stern-Badge erscheint in der oberen rechten Ecke der Karte.
- **Zugriff über die Seitenleiste**: Alle favorisierten Bilder sind in der linken Seitenleiste unter **Favoriten (★)** mit einem Klick erreichbar.

---

## 2. Kuratierte Alben (`AlbumModal.vue`)

Alben ermöglichen es, zusammengehörige Kunstwerke aus unterschiedlichen Ordnern zu gruppieren, ohne physische Dateien auf dem Datenträger verschieben zu müssen.

### Alben erstellen & verwalten
1. Klicken Sie in der linken Seitenleiste unter **Alben** auf **„+ Neu“**.
2. Vergeben Sie einen Albumnamen (z. B. `„Cyberpunk Charaktere“`, `„Portfolio 2026“`) und optional eine Beschreibung.
3. Benennen Sie Alben um oder löschen Sie sie per Rechtsklick in der Seitenleiste. Das Löschen eines Albums entfernt lediglich die Gruppierungszuordnung, löscht jedoch niemals Ihre Originaldateien.

### Bilder zu Alben hinzufügen
- **Drag & Drop**: Wählen Sie Karten in der Galerie aus und ziehen Sie diese direkt auf ein Album in der linken Seitenleiste.
- **Stapelaktionsleiste**: Klicken Sie in der schwebenden Leiste auf **„Album“**.
- **Rechter Inspektor**: Alben können auch direkt im Eigenschafts-Inspektor gesucht und zugewiesen werden.

---

## 3. Farbcodierte Tag-Taxonomie (`TagModal.vue`)

Tags ermöglichen eine feingliedrige Kategorisierung und visuelle Ordnung:

### Vordefinierte Farb-Badges
Omera bietet 8 sofort erkennbare Farbvarianten:
- 🔴 Rot
- 🟠 Orange
- 🟡 Gelb
- 🟢 Grün
- 🔵 Blau
- 🟣 Violett
- 🌸 Rosa
- ⚪ Schiefergrau

### Arbeiten mit Tags
- **Tags erstellen**: Klicken Sie in der Seitenleiste auf **„+ Neu“**, wählen Sie eine Farbpalette und geben Sie eine Bezeichnung ein (z. B. `Titel-Charakter`, `Entwurf`, `Freigegeben`).
- **Drag & Drop zum Verschlagworten**: Ziehen Sie ausgewählte Karten direkt auf einen Tag-Chip in der Seitenleiste.
- **Nach Tags filtern**: Klicken Sie auf ein beliebiges Tag in der Seitenleiste, um die Galerie danach zu filtern.
- **Mehrfach-Verschlagwortung**: Jedes Kunstwerk kann beliebig vielen Tags zugeordnet werden.

---

## 4. Schwebende Stapelaktionsleiste (`BatchActionBar.vue`)

Sobald mehrere Elemente markiert werden (über `Strg+Klick`, `Umschalt+Klick` oder `Strg+A`), blendet sich am unteren Bildschirmrand die **Stapelaktionsleiste** ein:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  [✓ 14 von 120 ausgewählt]  [Alle auswählen]  [Auswahl aufheben]                       │
│  [★ Bewertung ▾]  [🏷 Tag]  [📁 Album]  [🏷️ Auto-Tag]  [★ Favorit]  [🔞 NSFW]          │
│  [📋 Pfade kopieren]  [💬 Prompts kopieren]  [📂 Verschieben]  [📄 Kopieren]           │
│  [🧹 Entwürfe aussortieren]  [📤 Exportieren...]  [🗑 Papierkorb]                       │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Zentrale Stapelaktionen:
- **Bewertung setzen**: Eine einheitliche Sternebewertung für alle ausgewählten Bilder vergeben.
- **Tag / Album**: Den Dialog öffnen, um Tags und Alben für alle markierten Dateien zuzuweisen oder zu entfernen.
- **Prompts kopieren**: Kopiert die positiven Prompts aller markierten Bilder in die Zwischenablage, getrennt durch saubere `---`-Trennlinien.
- **Pfade kopieren**: Kopiert vollständige absolute Dateipfade (einer pro Zeile) zum Einfügen in Skripte oder Terminals.
- **Verschieben & Kopieren**: Physisches Verschieben oder Kopieren der Dateien in ein anderes von Omera indiziertes Verzeichnis.
- **Niedrig bewertete Entwürfe aussortieren**: Aktiv, wenn ausgewählte Karten Bilderstapel enthalten (siehe [Stapel & Serien](../04-intelligent-curation/stacks-and-bursts.md)).
- **Papierkorb**: Verschiebt alle ausgewählten Dateien sicher in den Papierkorb des Betriebssystems.
