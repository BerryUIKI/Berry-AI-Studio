# Glossar

Dieses Glossar definiert zentrale Fachbegriffe, Architekturkonzepte und Funktionen von **Berry AI Studio**.

---

## A
- **AIGC (AI-Generated Content)**: Digitale Medieninhalte (Bilder, Animationen, Audio und Video), die mithilfe neuronaler Netze wie Stable Diffusion, ComfyUI, Midjourney oder Flux generiert werden.
- **AIGC-Erfassungs-Pipeline (Modus C)**: Ein aktiver Ordnerüberwachungsmodus, der Ausgabeverzeichnisse von Generatoren überwacht, Schreibsperren mit 500 ms entprellt, neue Dateien automatisch übernimmt und verzögerte Aufräumwarteschlangen verwaltet.
- **Album**: Eine benutzerdefinierte Sammlung von Kunstwerken, die gruppiert werden, ohne die Originaldateien auf der Festplatte zu bewegen.
- **Autoratives Titelbild (Hero)**: Das primäre Deckblatt, das einen gesamten Bilderstapel in der eingeklappten Galerieansicht repräsentiert.

## B
- **Bilderstapel (Image Stack)**: Eine zusammengehörige Gruppe von Bildvariationen, die auf der Galeriefläche als einzelne, aufklappbare Karte dargestellt wird.
- **Bildvergleich nebeneinander (Side-by-Side Compare)**: 1:1-Vergleichsansicht (`C`) mit synchronisiertem Zoom und Pan zur genauen Inspektion feinster Detailunterschiede.

## C
- **CFG-Skala (Classifier-Free Guidance)**: Ein Steuerungsparameter bei Diffusionsmodellen, der festlegt, wie streng sich der Generierungsprozess am Text-Prompt orientieren soll.
- **Checkpoint-Modell**: Ein trainiertes Basis-Neuronalnetz (üblicherweise im `.safetensors`-Format), das über seinen Modellnamen und SHA256-Hash identifiziert wird.
- **CLIP (Contrastive Language-Image Pre-training)**: Eine multimodale neuronale Netzwerkarchitektur, die in Berry AI Studio für die semantische Suche in natürlicher Sprache und visuelle Ähnlichkeitsberechnungen eingesetzt wird.
- **ComfyUI**: Eine modulare, knotenbasierte Oberfläche für generative KI-Workflows. Berry parst eingebettete Ausführungsgraphen und unterstützt die direkte Übergabe an die `/prompt`-API.

## E
- **Entwürfe aussortieren (Cull Drafts)**: Eine Stapel-Bereinigungsaktion, die hochbewertete Titelbilder in Bilderserien behält, während geringer bewertete Entwürfe mit einem Klick in den Papierkorb verschoben werden.
- **Externer Link & Überwachung (Modus A)**: Ein Ordnermodus, der lokale oder Netzwerkverzeichnisse vor Ort („In-Place“) ohne Kopieren indiziert und die Originaldateien unverändert lässt.

## J
- **Jaccard-Ähnlichkeit**: Ein mathematisches Ähnlichkeitsmaß für Mengen, das von Berry genutzt wird, um aufeinanderfolgende Prompts anhand gemeinsamer Token für die automatische Stapelbildung zu vergleichen.

## K
- **Keyset-Cursor-Paginierung**: Eine Datenbankabfrage-Strategie, die langsame `OFFSET N`-Befehle durch `WHERE (modified_at, id) < (?, ?)` ersetzt und selbst bei mehr als 500.000 Datensätzen Abfragezeiten im Sub-Millisekunden-Bereich liefert.

## L
- **Lightbox (Vollbild-Vorschau)**: Ein ablenkungsfreier Vollbildbetrachter, der über die `Leertaste` oder `Enter` geöffnet wird und Mausrad-Zoom, Pan sowie Einzelbild-Weiterschaltung bei Videos unterstützt.
- **LoRA (Low-Rank Adaptation)**: Ein kompaktes Feinabstimmungsmodell, das auf ein Checkpoint-Modell aufgesetzt wird, um spezifische Charaktere, Kleidungsstücke oder Stile zu erzeugen.

## M
- **Masonry (Wasserfall-Ansicht)**: Ein dynamisches Galerielayout, das Bilder in Spalten unter voller Beibehaltung ihres nativen Seitenverhältnisses ohne Beschnitt darstellt.

## N
- **NSFW-Weichzeichner (Privacy Blur)**: Eine Schutzfunktion, die sensible Inhalte automatisch unscharf maskiert, bis sie durch Anklicken bewusst freigelegt werden.

## O
- **OCC (Optimistische Nebenläufigkeitssteuerung)**: Eine Strategie zur Vermeidung von Schreibkonflikten bei mehreren zeitgleichen Nutzern im Team, basierend auf zeilenbasierten Versionsnummern.

## P
- **Poker-Deck-Karte**: Die visuelle Aufmachung eines Bilderstapels in der Galerie, dargestellt mit dezent versetzten Kartenebenen im Hintergrund und einem interaktiven Zähler-Badge.
- **Prompt-Chips**: Interaktive UI-Elemente im Inspektor, die einzelne Prompt-Token repräsentieren und 1-Klick-Suchen sowie -Kopieren ermöglichen.

## S
- **Sampler / Scheduler**: Der numerische Integrationsalgorithmus (z. B. `Euler a`, `DPM++ 2M Karras`), der aus latentem Rauschen Schritt für Schritt das fertige Bild errechnet.
- **Seed**: Ein ganzzahliger Startwert für den Pseudozufallsgenerator, der für reproduzierbare Ergebnisse bei der Bildgenerierung sorgt.
- **Sidecar-Datei**: Eine Begleitdatei (`.txt` oder `.json`), die neben dem Bild liegt und Generierungsparameter oder Civitai-Metadaten bereitstellt.
- **Speicher-Root (Storage Root)**: Eine plattformneutrale Verzeichnis-UUID, die geteilte Netzwerkpfade verschiedenen Betriebssystem-Mounts zuordnet.
- **Stapelaktionsleiste (Batch Action Bar)**: Eine schwebende Werkzeugleiste am unteren Bildschirmrand, die erscheint, sobald mehrere Bildkarten markiert sind.

## V
- **Verwalteter Projekt-Tresor (Modus B)**: Ein intern verwalteter Speicherbereich, der importierte Bilder automatisch in einer sauberen, nach Datum unterteilten Verzeichnisstruktur (`JJJJ/MM/UUID_Dateiname.ext`) ablegt.

## W
- **WAL (Write-Ahead Logging)**: Ein SQLite-Journalmodus, der parallele, blockierungsfreie Lesezugriffe während Hintergrundindizierungen und Thumbnail-Generierungen ermöglicht.
- **WD14-Tagger**: Ein Computer-Vision-Modell (SmilingWolf), das automatisch Danbooru-Anime-Tags und Altersfreigaben aus den Bildpixeln vorhersagt.
