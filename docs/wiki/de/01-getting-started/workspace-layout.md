# Arbeitsbereich & UI-Aufbau

Berry AI Studio verfügt über einen von **Eagle / Lightroom** inspirierten 3-Spalten-Desktop-Arbeitsbereich, der für hochdichte visuelle Kuration, Tastaturgeschwindigkeit und ablenkungsfreies Arbeiten konzipiert ist.

---

## 1. Fensteraufbau & Desktop-Shell

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Logo] Berry AI Studio   [ Datei  Bearbeiten  Ansicht  Werkzeuge  Hilfe ]    [ _ ] [ □ ] [ ✕ ] │  <- Titelleiste & Menüleiste
├──────────────┬──────────────────────────────────────────────────────────┬──────────────┤
│              │ [🔍 Suche: Prompt, Modell, Bewertung... ] [🧠] [☰ Filter] │              │
│  NAVIGATION  ├──────────────────────────────────────────────────────────┤  EIGENSCHAFTEN│
│ SEITENLEISTE │                                                          │  INSPEKTOR   │
│              │                  ZENTRALER GALERIEBEREICH                │              │
│  - Alle      │                                                          │  - Vorschau  │
│  - Favoriten │  [ Karte ]  [ Karte ]  [ Poker-Stapel (4) ]  [ Karte ]   │  - Prompts   │
│  - Ordner    │                                                          │  - Modell/CFG│
│  - Alben     │  [ Karte ]  [ Karte ]  [ Karte ]             [ Karte ]   │  - LoRA-Tags │
│  - Tags      │                                                          │  - Interop   │
│              │                                                          │              │
│  [Werkzeuge] │                                                          │              │
├──────────────┴──────────────────────────────────────────────────────────┴──────────────┤
│ 1.248 / 8.920 Elemente | 3 Ausgewählt | Schema v14 | Bereit | [⚡ Aktivität]           │  <- Statusleiste
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Titelleiste & Native Anwendungsmenüs

Das Anwendungsfenster verwendet ein rahmenloses Design mit einer individuellen, nativ wirkenden Titelleiste (`TitleBar.vue`) und einem Desktop-Menüsystem (`MenuBar.vue`):

### Referenz des Hauptmenüs
- **Datei**:
  - `Ordner hinzufügen...` (`Strg + O` / `Cmd + O`): Ein neues lokales oder Netzwerkverzeichnis anbinden.
  - `Aktuellen Ordner scannen`: Den aktiven Ordner erneut auf geänderte oder neue Dateien prüfen.
  - `Alle Ordner erneut scannen`: Einen inkrementellen Scan über alle registrierten Ordner erzwingen.
  - `Datenbankverwaltung...`: Werkzeuge für Datenbankkomprimierung, Speicherstatistiken und Snapshots öffnen.
  - `Einstellungen...` (`Strg + ,` / `Cmd + ,`): Das zentrale Einstellungsfenster mit 8 Kategorien öffnen.
  - `Beenden` (`Alt + F4`): Berry AI Studio sauber schließen.
- **Bearbeiten**:
  - `Alles auswählen` (`Strg + A` / `Cmd + A`): Alle Assets in der aktuellen Galerieansicht markieren.
  - `Auswahl aufheben` (`Esc`): Alle Markierungen aufheben.
  - `Tags stapelweise hinzufügen...`: Tags für ausgewählte Dateien zuweisen oder entfernen.
  - `Zu Album hinzufügen...`: Ausgewählte Dateien einem Album zuordnen.
  - `In Ordner verschieben...` / `In Ordner kopieren...`: Dateien physisch in einen anderen indizierten Ordner übertragen.
  - `Exportieren...` (`Strg + E` / `Cmd + E`): Fenster für Stapelexport, Transkodierung und Bereinigung öffnen.
  - `In den Papierkorb` (`Entf` / `Rücktaste`): Auswahl sicher in den Papierkorb des Betriebssystems verschieben.
- **Ansicht**:
  - `Rasteransicht`: Zu gleichmäßigen responsiven Karten mit fester Höhe wechseln.
  - `Wasserfall (Masonry)`: Zu Spalten mit unbeschnittenem Original-Seitenverhältnis wechseln.
  - `Listenansicht (Tabelle)`: Zu einer kompakten tabellarischen Übersicht wechseln.
  - `Seitenleiste umschalten` (`B`): Linke Navigationsleiste ein-/ausblenden.
  - `Inspektor umschalten` (`I`): Rechten Metadaten-Inspektor ein-/ausblenden.
  - `Vollbild-Vorschau (Lightbox)` (`Leertaste` / `Enter`): Vollbild-Vorschaufenster öffnen.
  - `Vergrößern` (`Strg + =`), `Verkleinern` (`Strg + -`), `Zoom zurücksetzen` (`Strg + 0`).
- **Werkzeuge**:
  - `Prompt-Statistiken...`: Worthäufigkeitsverteilungen und Korrelation mit Bewertungen analysieren.
  - `Modell-Manager...`: Checkpoint-Hashes in lesbare Modellnamen auflösen.
  - `CLIP-Semantiksuchindex...`: Vektor-Einbettungen für die Suche in natürlicher Sprache indizieren.
  - `LoRA-Trigger-Bibliothek`: Trigger-Wörter und `.civitai.info`-Sidecars verwalten.
  - `Auto-Tag (WD14)`: Lokalen Danbooru-Anime-/Ästhetik-Tagger öffnen.
- **Hilfe**:
  - `Sprache`: Direkt zwischen 7 unterstützten Sprachen wechseln.
  - `Tastaturkürzel` (`?`): Interaktive Tastaturkürzel-Übersicht anzeigen.
  - `Nach Updates suchen...`: GitHub Releases auf neue Versionen prüfen.
  - `Über Berry AI Studio`: Versionsnummer, Entwickler- und Lizenzinformationen anzeigen.

---

## 3. Linke Navigations-Seitenleiste (`Sidebar.vue`)

Die linke Seitenleiste (`B` zum Umschalten) bietet schnellen Zugriff auf Ihre Sammlungen:

1. **Systembibliotheken**:
   - **Alle Bilder**: Übergreifende Bibliotheksansicht über alle angebundenen Ordner mit Zähler-Badge.
   - **Favoriten (★)**: Schnellfilter für alle mit Stern markierten/gespeicherten Bilder.
   - **Sensibel (18+) (🔞)**: Schneller Zugriff auf als jugendgefährdend/NSFW markierte Inhalte (standardmäßig mit Weichzeichnungs-Overlay geschützt).
2. **Ordnerbereich**:
   - Zeigt alle registrierten Verzeichnisse mit Typ-Symbolen an:
     - `⚡`: AIGC-Erfassungs-Pipeline (aktive Überwachung von Generator-Ausgaben).
     - `📦`: Verwalteter Projekt-Tresor (intern strukturierter Speicherbereich).
     - `📁`: Externer Link (In-Place Zero-Copy-Index).
   - Rechtsklick oder Kontextmenü für Schnellaktionen: **Neue Bilder erfassen**, **Ordner scannen**, **Neu aufbauen** oder **Entfernen**.
   - Dient als Drag-and-Drop-Ziel: Ziehen Sie Karten aus der Galerie direkt auf einen Ordner, um sie zu verschieben oder zu kopieren.
3. **Albenbereich**:
   - Benutzerdefinierte Kollektionen. Ziehen Sie Bilder per Drag-and-Drop auf Alben, um sie hinzuzufügen.
   - Mit Schaltfläche „+ Neu“ für neue Alben.
4. **Tag-Bereich**:
   - Farbcodierte Schlagwörter (8 unterscheidbare Farb-Badges).
   - Ziehen Sie Galerie-Elemente auf Tag-Chips, um sie stapelweise zu verschlagworten.
5. **Schnellwerkzeuge in der Fußzeile**:
   - Direktschaltflächen für Einblicke, Modelle, Datenbankwartung und Tastaturkürzel.

---

## 4. Zentraler Galeriebereich & Suchleiste

Der zentrale Arbeitsbereich dient zum Durchsuchen, Auswählen und Kuratieren von Beständen:

- **Suchleiste (`SearchBar.vue`)**:
  - **Syntaxsuche (`🔍`)**: Durchsuchen von Metadaten mit Schlüsselwörtern oder strukturierter Syntax (z. B. `prompt:"cyberpunk" cfg:>7`).
  - **KI-Semantische Suche (`🧠`)**: Bildsuche in natürlicher Sprache auf Basis lokaler CLIP/SigLIP-Modelle.
- **Ansichtsumschalter**:
  - Wechseln zwischen **Raster (⊞)**, **Wasserfall (▤)** und **Tabelle (☰)**.
  - **Zoom-Schieberegler**: Passt die Mindestkartenbreite dynamisch zwischen **130px** und **360px** an. Die Spaltenanzahl passt sich responsiv an, ohne Bilder zu verzerren.
- **Filter-Bedienfeld (`FilterDrawer.vue`)**:
  - Seitliches Bedienfeld für Mehrkriterienfilter nach Checkpoint-Modell, Sampler, Seitenverhältnis, Sternebewertung und Video-Eigenschaften.

---

## 5. Rechter Eigenschafts-Inspektor (`InspectorPane.vue`)

Der rechte Inspektor (`I` zum Umschalten) liefert tiefgehende, verlustfreie Generierungs-Metadaten für das aktive Element:

- **Medienvorschau**: Hochauflösende Vorschau mit Entschärfungsschalter für sensible Inhalte und Vollbild-Vorschau.
- **Kurationssteuerelemente**: Sterne-Bewertung (0–5 Sterne), Favoriten-Umschalter (`F`) und NSFW-Kennzeichnung.
- **Positiver Prompt**: Tokenisierte Prompt-Ansicht. Jedes Prompt-Token wird als interaktiver Chip dargestellt:
  - Klicken Sie auf einen Chip, um in der gesamten Bibliothek danach zu suchen.
  - Klicken Sie auf die Kopieren-Schaltfläche, um den gesamten positiven Prompt in die Zwischenablage zu übernehmen.
- **Negativer Prompt**: Vollständiger Text des negativen Prompts mit 1-Klick-Kopierfunktion.
- **Technische Generierungsparameter**:
  - Modellname & Checkpoint-SHA256-Hash.
  - Sampler- & Scheduler-Algorithmen.
  - Schrittanzahl (Steps), CFG-Skala, Seed (mit Kopierschaltfläche).
  - Native Bilddimensionen (`Breite × Höhe`).
- **Erkannte LoRAs**:
  - Listet alle in der Datei erkannten `<lora:name:weight>`-Tags oder ComfyUI-LoRA-Loader-Knoten auf.
  - Zeigt Trigger-Wörter, Gewichtung und 1-Klick-Kopieren des formatierten Prompt-Tags an.
- **Generierungs-Interop**:
  - Zeigt den Live-Verbindungsstatus zu lokalem **ComfyUI** und **AUTOMATIC1111** an.
  - 1-Klick-Aktionen: „An ComfyUI senden“ (stellt den Workflow-Graphen erneut in die Warteschlange) oder „An SD WebUI senden“.
- **Rohdaten-Akkordeon**: Anzeige des vollständigen, unformatierten JSON-Parameterblocks oder ComfyUI-Knotengraphen.

---

## 6. Untere Statusleiste & Aktivitäts-Popover

Am unteren Fensterrand platziert:
- **Gefiltert / Gesamt-Zähler**: Zeigt die sichtbaren Treffer im Verhältnis zur Gesamtbibliothek an (z. B. `Gefiltert: 420 / 12.500 Elemente`).
- **Auswahlzähler**: Zeigt die Anzahl an, wenn Elemente markiert sind (`Ausgewählt: 5 Elemente`).
- **Datenbankstatus**: Zeigt den Namen der aktiven Datenbankdatei und die Schemaversion an.
- **Aktivitäts-Popover (`⚡ Aktivität`)**:
  - Öffnet einen Echtzeit-Monitor mit Status des Dateisystem-Wächters (überwachte Pfade, Journal-Rückstand, Fehlerstatus) und Hintergrundwarteschlangen zur WebP-Vorschaubild-Generierung.

---

## 7. Schwebende Stapelaktionsleiste (`BatchActionBar.vue`)

Sobald eine oder mehrere Karten ausgewählt sind, erscheint am unteren Rand eine schwebende Aktionsleiste:

- Mehrfachauswahl-Zähler (`X von Y ausgewählt`) mit Schaltflächen für „Alle auswählen“ und „Auswahl aufheben“.
- Stapel-Bewertungs-Dropdown (0 bis 5 Sterne).
- Dialoge für „Zu Album hinzufügen“ und „Tag hinzufügen“.
- „Auto-Tag (WD14)“-Aktion.
- „Favorit“-Umschalter.
- „Pfade kopieren“ und „Prompts kopieren“ in die Zwischenablage.
- Dialoge zum Verschieben und Kopieren in Zielordner.
- „Niedrig bewertete Entwürfe aussortieren“ (aktiv bei ausgewählten Bildstapeln).
- „Exportieren...“ (Stapel-Transkodierung, Metadatenbereinigung, HTML-Showcase).
- „Papierkorb“ (verschiebt ausgewählte Elemente in den System-Papierkorb).
