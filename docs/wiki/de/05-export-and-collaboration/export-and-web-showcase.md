# Stapelexport, Transkodierung & Web-Showcase

Omera verfügt über eine hochperformante Export- und Paketierungs-Engine (`ExportModal.vue`), die durch Multi-Threading mit **Rayon** angetrieben wird. Sie unterstützt Formatkonvertierung, mehrstufige Metadatenbereinigung, Dateinamensmuster und die Erstellung vollkommen abhängigkeitsfreier HTML-Showcases.

---

## 1. Stapelexport aufrufen

So öffnen Sie das Exportfenster:
- Wählen Sie ein oder mehrere Bilder oder Stapel in der Galerie aus.
- Klicken Sie in der schwebenden Aktionsleiste auf **„Exportieren...“**, drücken Sie `Strg + E` / `Cmd + E` oder wählen Sie im Menü `Bearbeiten > Exportieren...`.

---

## 2. Format-Transkodierung & Kompression

Omera konvertiert und rekodiert Bilder parallel über alle verfügbaren CPU-Kerne:

| Zielformat | Optionen & Einstellungen | Bester Einsatzzweck |
| :--- | :--- | :--- |
| **Originalformat** | Behält exakte Quelldaten und Container bei. | Verlustfreie Archivierung oder Backups. |
| **WebP** | Einstellbare Qualität (1–100%, Standard 85%). Sehr kompakt. | Web-Veröffentlichung, Discord, Portfolio-Websites. |
| **JPEG** | Standardmäßiges progressives JPEG (Qualität 1–100%). | Universelle Kompatibilität mit älteren Bildbetrachtern. |
| **PNG** | Reine verlustfreie Komprimierung. | Druckvorstufe, Übergabe an Studios. |

### Auflösungsbegrenzung (Downscaling):
Sie können die maximale Kantenlänge begrenzen, um die Weitergabe unhandlich großer 4K-/8K-Originaldateien zu vermeiden:
- **Originalauflösung beibehalten** (keine Skalierung)
- **4K UHD** (maximale Kantenlänge: 3840 px)
- **2K QHD** (maximale Kantenlänge: 2048 px)
- **Full HD** (maximale Kantenlänge: 1080 px)
- **Benutzerdefinierte Kantenlänge** (individuelle Pixelgrenze)

---

## 3. 4-stufige Datenschutz-Metadatenbereinigung

Viele Urheber möchten Kunstwerke online präsentieren, eigene Prompts, negative Embeddings oder Seeds jedoch vertraulich behandeln. Omera bietet **vier diskrete Bereinigungsstufen**:

1. **Alles beibehalten (Keep All)**:
   - Behält alle eingebetteten Metadatenblöcke bei (ComfyUI-Workflow-Graphen, A1111-Parameter, NovelAI-Kommentare und EXIF-Daten).
2. **Nur Prompts entfernen (Strip Prompt Only)**:
   - Entfernt positive und negative Prompt-Texte, belässt jedoch technische Parameter (Sampler, Schritte, CFG-Skala, Seed, Modellname).
3. **Alle KI-Generierungs-Metadaten entfernen (Strip All AI Metadata)**:
   - Entfernt ComfyUI-Knotengraphen, A1111-Parameterblöcke, LoRA-Angaben und Generator-Signaturen vollständig.
4. **Vollständig bereinigen (Pixel-Only Sanitization)**:
   - Entfernt ausnahmslos alle Header, einschließlich EXIF-Tags, ICC-Farbprofilen und Software-Signaturen. Die exportierte Datei enthält ausschließlich reine Bildpixel.

---

## 4. Dateinamens-Muster & Sidecar-Dateien

Passen Sie die Namen exportierter Dateien über dynamische Variablen mit Echtzeit-Vorschau an:

### Unterstützte Token:
- `{name}`: Ursprünglicher Dateiname ohne Endung.
- `{id}`: Eindeutige Datenbank-ID oder UUID.
- `{index}`: Fortlaufende Ausgabenummer (001, 002, 003...).
- `{date}`: Erstellungsdatum (`JJJJ-MM-TT`).
- `{rating}`: Sterne-Bewertung (z. B. `5star`).
- `{model}`: Checkpoint-Modellname.
- `{seed}`: Generierungs-Seed.

*Beispielmuster*: `{date}_{model}_{seed}_{name}` → `2026-09-22_animagine_xl_2849104812_cyberpunk_01.webp`

### Metadaten-Sidecars:
Sie können begleitende Dateien parallel zu jedem Bild erzeugen lassen:
- **Keine**: Keine Begleitdateien.
- **Prompt-Text (`.txt`)**: Speichert den positiven Generierungs-Prompt als Textdatei ab.
- **Vollständige JSON-Metadaten (`.json`)**: Exportiert strukturierte Metadaten und vollständige ComfyUI-Workflows.

---

## 5. Eigenständiges interaktives HTML-Showcase-Album

Omera kann Ihre ausgewählten Werke zu einer **in sich geschlossenen Offline-Webgalerie** bündeln (`index.html`):

- **Keinerlei Abhängigkeiten**: Benötigt weder Webserver, Node.js noch externe JavaScript-Bibliotheken. Lässt sich per Doppelklick in jedem modernen Webbrowser öffnen.
- **Enthaltene Funktionen**:
  - Dunkles Studio-Design passend zu Omera.
  - Responsives Galerie-Raster mit Lazy-Loading für Vorschaubilder.
  - Vollbild-Lightbox mit stufenlosem Mausrad-Zoom und Verschieben.
  - Ausklappbarer Prompt-Inspektor mit Generierungsparametern.
  - Schnelle clientseitige Schlüsselwortsuche.
- **Verpackungsoptionen**: Exportieren Sie die Galerie direkt in einen Zielordner oder bündeln Sie alles in ein einzelnes `.zip`-Archiv für Kundenübergaben oder den Upload auf Webhosting-Plattformen.
