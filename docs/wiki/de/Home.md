# Berry AI Studio — Offizielles Wiki & Benutzerhandbuch

Willkommen bei der maßgeblichen Benutzerdokumentation und Wissensdatenbank für **Berry AI Studio** (`v0.3.0`).

Berry AI Studio ist ein quelloffener, lokaler („Local-First“) Asset-Manager und Prompt-Arbeitsbereich, der speziell für Kreative im Bereich der generativen KI, Prompt-Engineers und visuelle Designstudios entwickelt wurde. Entwickelt auf Basis von **Tauri v2**, **Rust** und **Vue 3**, verwaltet die Anwendung Bibliotheken von wenigen hundert Kunstwerken bis hin zu mehr als 500.000 Dateien mit Abfrage-Latenzen im Sub-Millisekunden-Bereich, völlig ohne Cloud-Abhängigkeiten und mit vollständiger Metadaten-Extraktion generativer Workflows.

---

## 🧭 Navigation & Inhaltsverzeichnis

### [Kapitel 1: Erste Schritte & Grundlagen](01-getting-started/installation.md)
- **[Installation & Systemanforderungen](01-getting-started/installation.md)**: Hardware-Voraussetzungen, Windows Setup- und portable Optionen, macOS Universal- und Apple Silicon-Builds, Linux AppImage/deb sowie der Einrichtungsassistent beim ersten Start.
- **[Arbeitsbereich & UI-Aufbau](01-getting-started/workspace-layout.md)**: Detaillierte Übersicht über das rahmenlose Fenster, die Anwendungsmenüleiste, das 3-Spalten-Layout (Seitenleiste, Galerie, Inspektor), die Statusleiste und die schwebende Stapelaktionsleiste.
- **[Tastaturkürzel-Übersicht](01-getting-started/keyboard-shortcuts.md)**: Globale Hotkeys, Auswahlanker, Blindbewertung mit Sternen, Schnellinspektion und Navigations-Kürzel.

### [Kapitel 2: Bibliotheksverwaltung & Medien](02-library-management/folder-modes-and-import.md)
- **[Medienimport & Ordnermodi](02-library-management/folder-modes-and-import.md)**: Vergleich von Modus A (Externer Link), Modus B (Verwalteter Projekt-Tresor) und Modus C (AIGC-Erfassungs-Pipeline mit verzögerter Quellbereinigung). Unterstützte Bild- (PNG, WebP, JPEG) und Videoformate (MP4, WebM).
- **[Galerieansichten & Anzeigeoptionen](02-library-management/gallery-views.md)**: Beherrschung von einheitlichem Raster (mit 130px–360px Zoom), Wasserfall-Ansicht (originales Seitenverhältnis), Tabellenansicht und visueller Ähnlichkeitsansicht. Karten-Badges und Weichzeichnung sensibler Inhalte (NSFW).
- **[Organisation, Bewertungen & Tags](02-library-management/organization-and-tags.md)**: 10-Sterne-Bewertungsskala, Favoriten, benutzerdefinierte Alben, 8-Farben-Tag-Taxonomie, Stapel-Drag-and-Drop und Stapelaktionsleiste.
- **[Video- & Animationsmedien-Unterstützung](02-library-management/video-support.md)**: Wiedergabe von AnimateDiff, Wan2.1, HunyuanVideo und SVD; Einzelbild-Weiterschaltung, Loop-/Geschwindigkeits-HUD und Inspektion eingebetteter Video-Workflows.

### [Kapitel 3: Suche, Metadaten & Analysen](03-discovery-and-analytics/search-and-filtering.md)
- **[Suchsyntax & visuelle Filter](03-discovery-and-analytics/search-and-filtering.md)**: Erweiterte Schlüssel-Wert-Abfragesprache (`prompt:`, `neg:`, `model:`, `cfg:>=7`, `steps:20..40`), numerische Bereiche und das ausklappbare Filter-Bedienfeld.
- **[AIGC-Metadaten & Prompt-Inspektion](03-discovery-and-analytics/metadata-and-prompts.md)**: Verlustfreies Parsen für AUTOMATIC1111, ComfyUI, NovelAI, Fooocus, InvokeAI; interaktive Prompt-Chips und Anzeige unformatierter Ausführungsgraphen.
- **[Prompt-Analysen & Einblicke](03-discovery-and-analytics/prompt-insights.md)**: Bibliotheksweite Worthäufigkeitsverteilungen, Ranglisten positiver/negativer Prompts und Korrelation mit Benutzerbewertungen.

### [Kapitel 4: Intelligente Kuration & KI-Engines](04-intelligent-curation/stacks-and-bursts.md)
- **[Bildstapel, Serien & Bildvergleich](04-intelligent-curation/stacks-and-bursts.md)**: Automatische Serien-Clusterbildung (Jaccard-Prompt-Ähnlichkeit + Zeitfenster), Poker-Deck-Kartenstapel, Titelbilder, sicheres Auflösen von Stapeln, Aussortieren von Entwürfen und Side-by-Side-Bildvergleich (`C`).
- **[KI-Semantische Suche & Auto-Tagging](04-intelligent-curation/ai-semantic-and-tagger.md)**: Lokale ONNX CLIP/SigLIP Text-zu-Bild-Suche in natürlicher Sprache, visuelle Ähnlichkeitssuche Bild-zu-Bild und automatisches WD14 Danbooru-Anime-Tagging.
- **[Checkpoint-Modelle & LoRA-Bibliothek](04-intelligent-curation/models-and-loras.md)**: Automatische Katalogisierung von Checkpoints, A1111-`cache.json`-Hashauflösung, Civitai-Rückwärtssuche, LoRA-Trigger-Wort-Manager und 1-Klick-Prompt-Injektion.
- **[Generierungs-Interoperabilität](04-intelligent-curation/generation-interop.md)**: Direkte API-Kommunikation mit ComfyUI (`/prompt`) und AUTOMATIC1111 (`/sdapi/v1/txt2img`) mit Live-Verbindungsstatusprüfung.

### [Kapitel 5: Export, Cloud-Backup & Kollaboration](05-export-and-collaboration/export-and-web-showcase.md)
- **[Stapelexport, Transkodierung & Web-Showcase](05-export-and-collaboration/export-and-web-showcase.md)**: Multithreaded-Rayon-Transkodierung, 4-stufige Bereinigung sensibler Metadaten, dynamische Dateinamensvorlagen, ZIP-Archive und Generierung eigenständiger, interaktiver Offline-HTML-Showcases.
- **[Cloud-Snapshot-Backup & Medienspiegelung](05-export-and-collaboration/cloud-backup-and-sync.md)**: Konsistente SQLite `VACUUM INTO`-Snapshots auf AWS S3, Cloudflare R2, MinIO, WebDAV oder lokalen NAS-Speicher; inkrementeller Delta-Sync mit ETag/SHA-256-Erkennung und Bandbreitenbegrenzung.
- **[Multi-Datenbank-Team-Studio](05-export-and-collaboration/team-collaboration.md)**: Skalierung über SQLite hinaus auf gemeinsame MySQL 8.0+- oder PostgreSQL 14+-Server; plattformübergreifendes Speicher-Root-Mapping (Normalisierung von Windows-Laufwerksbuchstaben auf macOS/Linux-Pfade), optimistische Nebenläufigkeitssteuerung (OCC) und clientseitiges NVMe-Thumbnail-Caching.

### [Kapitel 6: Systemreferenz & Wartung](06-reference-and-maintenance/settings-reference.md)
- **[Vollständige Einstellungsreferenz](06-reference-and-maintenance/settings-reference.md)**: Vollständiger Parameterleitfaden für alle 8 Einstellungs-Tabs.
- **[Datenbank- & Speicherwartung](06-reference-and-maintenance/database-maintenance.md)**: SQLite-WAL-Komprimierung (`VACUUM`), Datenbank-Backup/-Wiederherstellung, Thumbnail-Cache-Budget (LRU-Bereinigung) und Warteschlangendiagnose.
- **[Updates & Lebenszyklus](06-reference-and-maintenance/updating.md)**: In-App-Auto-Updater, Datenerhaltungsgarantien über Versionswechsel hinweg und manuelle Release-Upgrades.
- **[Datenschutz- & Sicherheitsarchitektur](06-reference-and-maintenance/privacy-and-security.md)**: 100% Offline-First-Modell, keinerlei Telemetrie, isolierte lokale KI-Inferenz und AGPL-3.0-Lizenzierung.
- **[Fehlerbehebung & FAQ](06-reference-and-maintenance/troubleshooting-and-faq.md)**: Lösungen für häufige Probleme, Tipps zur Performance-Optimierung und Antworten auf häufig gestellte Fragen.
- **[Glossar](06-reference-and-maintenance/glossary.md)**: Verbindliche Definitionen zentraler Fachbegriffe (Titelbild/Hero, Erfassungs-Pipeline, Jaccard-Ähnlichkeit, Keyset-Cursor, OCC, Poker-Deck-Karte usw.).

---

## ⚡ Schnellstart-Tastenkürzel

| Aktion | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **Vollbild-Vorschau (Lightbox)** | `Leertaste` / `Enter` | `Leertaste` / `Return` |
| **Bildvergleich nebeneinander** | `C` | `C` |
| **In Stapel gruppieren** | `Strg + G` | `Cmd + G` |
| **Stapel auflösen** | `Strg + Umschalt + G` | `Cmd + Shift + G` |
| **Als Titelbild festlegen** | `Alt + S` | `Option + S` |
| **Bewertung 1–5 Sterne** | `1` – `5` (`0` zum Löschen) | `1` – `5` (`0` zum Löschen) |
| **Favorit umschalten** | `F` | `F` |
| **Suchleiste fokussieren** | `/` oder `Strg + F` | `/` oder `Cmd + F` |
| **Inspektor umschalten** | `I` | `I` |
| **Seitenleiste umschalten** | `B` | `B` |
| **Einstellungen öffnen** | `Strg + ,` | `Cmd + ,` |
