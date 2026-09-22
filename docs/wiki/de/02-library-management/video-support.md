# Video- & Animationsmedien-Unterstützung

Mit der rasanten Verbreitung generativer KI-Videomodelle (AnimateDiff, SVD, Wan2.1, HunyuanVideo, CogVideoX, LTX-Video) bietet Berry AI Studio erstklassige, native Unterstützung für animierte Kunstwerke und Videos in den Formaten **MP4** und **WebM**.

---

## 1. Unterstützte Videoformate & Container

Berry AI Studio analysiert Videocontainer direkt in nativem Rust (`berry-metadata`):

- **MP4 (`.mp4`)**: Liest die ISOBMFF-Box-Hierarchie (`ftyp`, `moov`, `trak`, `mdia`, `minf`, `stbl`).
  - Extrahiert automatisch Videodimensionen (`Breite × Höhe`), Bildwiederholrate (FPS), Spieldauer sowie den Codec (`H.264`, `H.265 / HEVC`, `AV1`).
  - Durchsucht `moov/udta`-Boxen nach eingebetteten ComfyUI-Ausführungsgraphen und Generierungsparametern.
- **WebM (`.webm`)**: Analysiert das EBML-Containerformat für VP8-, VP9- und AV1-Videostreams und ermittelt Dauer und Bildgröße direkt aus dem Stream-Header.

---

## 2. Video-Karten in der Galerie & Dynamische Vorschau

Im Galeriebereich unterscheiden sich Videodateien klar von statischen Bildern:

- **Dauer-Badge**: Zeigt die exakte Spieldauer in der Kartenecke (z. B. `00:05` oder `01:24`).
- **FPS- & Format-Badges**: Kennzeichnet Videos mit `MP4 · 24fps` oder `WEBP · 30fps`.
- **Vorschaubild-Generierung**:
  - Da Videodateien in Standard-Bildbibliotheken keine herkömmlichen Standbild-Decoder besitzen, erfasst das WebView-Frontend von Berry automatisch das erste Schlüsselbild über ein unsichtbares HTML5-`<video>`-Element, kodiert es in Base64 und das Rust-Backend speichert es als herunterskaliertes WebP-Vorschaubild via `save_video_thumbnail`.
- **Hover-Wiedergabe**: Wenn Sie den Mauszeiger über eine Videokarte bewegen, startet eine ressourcenschonende Videovorschau direkt im Galeriebereich, ohne dass ein separates Fenster geöffnet werden muss.

---

## 3. Lightbox-Videoplayer (`LightboxModal.vue`)

Das Drücken der `Leertaste` oder von `Enter` auf einer Videokarte öffnet die Vollbild-Vorschau mit dem integrierten **Lightbox-Player**:

```
┌────────────────────────────────────────────────────────────────────────┐
│ [✕]                                                       [★ Favorit]  │
│                                                                        │
│                      [ VIDEO-WIEDERGABEBEREICH ]                       │
│                                                                        │
│                                                                        │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ [▶ / ⏸] [⏪ 1f] [1f ⏩] [00:03 / 00:08] ──●───────── [1.0x ▾] [🔁] [🔊] │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ Filmstreifen: [Vorschau] [Vorschau] [● Video] [Vorschau] [Vorschau] │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### Steuerelemente des Lightbox-Players:
1. **Wiedergabe / Pause**: Klick in den Videobereich oder Drücken der `Leertaste`.
2. **Einzelbild-Weiterschaltung (Frame-by-Frame)**: Drücken Sie `←` oder `→` (oder die HUD-Schaltflächen), um das Video Einzelbild für Einzelbild vor- oder zurückzuspulen — ideal für präzise Bewegungsanalysen.
3. **Wiedergabegeschwindigkeit**: Regelbar zwischen **0.25x**, **0.5x**, **1.0x**, **1.5x** und **2.0x**.
4. **Schleife & Ton**: Nahtlose Endlosschleife (`🔁`) sowie Lautstärke- und Stummschalt-Regler (`🔊`).
5. **Filmstreifen-Navigation**: Blättern Sie über den unteren Vorschaustreifen schnell zu benachbarten Bildern oder Videos im aktuellen Verzeichnis.

---

## 4. Inspektion von ComfyUI-Video-Workflows

Viele KI-Videogeneratoren verwenden mehrstufige ComfyUI-Workflows (z. B. Text-Prompt → Initiales Latent-Bild → AnimateDiff-Bewegungsmodul → ControlNet OpenPose-Führung → Spatial-Upscaler).

Bei Betrachtung eines mit ComfyUI erzeugten Videos:
- Der **Eigenschafts-Inspektor** extrahiert und visualisiert die positiven und negativen Prompts, die für das Bewegungsmodell verwendet wurden.
- Das **Rohdaten-Akkordeon** rendert den gesamten Ausführungsgraphen inklusive Checkpoint-Modell, Bewegungs-LoRAs, Kontextfenster-Länge, Frame-Überlappung und VAE-Decodierungsoptionen.
- Mit einem Klick auf **„An ComfyUI senden“** können Sie den exakten Video-Workflow direkt zurück in Ihre laufende ComfyUI-Instanz laden, um ihn zu modifizieren oder neu zu rendern.
