# Galerieansichten & Anzeigeoptionen

Berry AI Studio bietet vier spezialisierte Galerieansichten, die für unterschiedliche Kurationsanforderungen konzipiert sind — von der schnellen visuellen Sichtung bis hin zu detaillierten technischen Prüfungen.

---

## 1. Galerieansichtsmodi

Verwenden Sie die Schaltflächen in der oberen Symbolleiste oder das Menü (`Ansicht`), um zwischen den Modi zu wechseln:

### 1. Einheitliche Rasteransicht (`grid` — ⊞)
- **Konzept**: Karten mit fester Höhe und einheitlichem Seitenverhältnis, angeordnet in responsiven Spalten.
- **Responsive Architektur**: Die Kartenbreite bleibt bei Fenstergrößenänderungen stabil. Berry passt die Spaltenanzahl dynamisch an (`calculateGalleryColumns`), anstatt Bilder zu strecken oder zu stauchen.
- **Zoom-Schieberegler**: Ziehen Sie den Zoomregler oder nutzen Sie `Strg + =` / `Strg + -`, um die minimale Kartenbreite stufenlos von **130 px** (Übersichtsmodus) bis **360 px** (Detailmodus) zu skalieren.
- **Virtualisierung**: Nur Elemente, die sich aktuell im sichtbaren Bereich (Viewport) befinden (zuzüglich eines kleinen Vorauslade-Puffers), werden im DOM gerendert. Das Scrollen durch eine Bibliothek mit 100.000 Bildern verbraucht nicht mehr Arbeitsspeicher als das Betrachten von 100 Bildern.

### 2. Wasserfall-Ansicht (`masonry` — ▤)
- **Konzept**: Flexibles mehrspaltiges Layout, das das native Seitenverhältnis jedes Bildes vollständig bewahrt.
- **Unbeschnittene Kunstwerke**: Ideal für Sammlungen mit gemischten Querformat-Hintergründen, hochformatigen Charakterkonzepten (9:16) und Panorama-Landschaften. Bilder werden verlustfrei mit `object-fit: contain` skaliert, ohne Kanten abzuschneiden.
- **Verteilung nach kürzester Spalte**: Neue Elemente werden stets in die Spalte mit der geringsten vertikalen Gesamthöhe eingefügt, was eine ausgewogene, visuell ansprechende Bilderwand erzeugt.

### 3. Detaillierte Listenansicht / Tabelle (`table` — ☰)
- **Konzept**: Virtualisiertes Tabellenlayout mit fixer Zeilenhöhe von 46 px.
- **Informationsdichte**: Zeigt Mikro-Vorschaubilder (36-px-Stufe) neben umfangreichen Tabellenspalten an:
  - Kontrollkästchen-Auswahlstatus
  - Dateiname und relativer Ordnerpfad
  - Medien-Containerformat (`PNG`, `WEBP`, `MP4`)
  - Dimensionen (`Breite × Höhe`)
  - Dateigröße (formatiert in KB/MB)
  - Änderungszeitstempel
  - Sterne-Bewertung (0–5 Sterne)
  - Ästhetik-Score (Aesthetic Prediction)

### 4. Visuelle Ähnlichkeitsansicht
- **Aktivierung**: Klicken Sie im rechten Inspektor oder im Kontextmenü eines beliebigen Bildes auf **„Ähnliche Bilder suchen“**.
- **Ähnlichkeits-Banner**: Zeigt das Referenzquellbild, einen Schwellenwert-Schieberegler (**0% bis 95%** in 5%-Schritten) und eine Begrenzungsauswahl (**20, 50, 100, 200** Elemente) an.
- **Übereinstimmungs-Badge**: Jede passende Karte zeigt ein farbiges Badge mit dem Prozentsatz der Übereinstimmung (z. B. `94% Übereinstimmung`), berechnet aus der CLIP-Vektor-Kosinus-Distanz.
- **Beenden**: Drücken Sie `Esc` oder klicken Sie auf das Schließen-Symbol (`✕`) im Banner, um zur regulären Galerieansicht zurückzukehren.

---

## 2. Karten-Badges & Visuelle Overlays

Unter **Einstellungen > Galerie** können Sie die Option **Karten-Badges anzeigen** aktivieren oder deaktivieren. Wenn diese aktiv ist, zeigen die Karten übersichtliche Informations-Chips:

- **Format- / Container-Badge**: Kennzeichnet `.png`, `.webp`, `.jpg`, `.mp4` oder `.webm`.
- **Dimensions-Badge**: Native Pixelauflösung (z. B. `1024×1024` oder `832×1216`).
- **Video-Dauer & FPS**: Zeigt bei Animationen und Videos die Spieldauer (z. B. `00:04`) und Bildrate (`24 fps`) an.
- **Generator-Signatur**: Erkennt die Erzeuger-Signatur (z. B. `WebUI`, `ComfyUI`, `NovelAI`, `Fooocus`).
- **Sterne-Bewertungs-Overlay**: Zeigt die aktive Sternebewertung (★ 1–5).
- **Favoriten-Markierung**: Ein goldenes Sternsymbol in der oberen rechten Ecke für favorisierte Bilder.

---

## 3. Weichzeichnung sensibler Inhalte (NSFW)

Um Ihre Privatsphäre bei Präsentationen oder beim Arbeiten an öffentlichen Orten zu schützen, enthält Berry AI Studio einen integrierten Inhaltsschutz:

- **Automatischer Weichzeichner (`blur_nsfw`)**: Jedes Asset, das mit `is_nsfw` gekennzeichnet ist oder eine entsprechende Altersfreigabe aufweist, wird standardmäßig durch einen starken CSS-Weichzeichnungsfilter unkenntlich gemacht.
- **Klicken zum Anzeigen**: Ein Klick auf das Augensymbol (`👁`) oder auf die Bildkarte blendet das Bild zur Prüfung vorübergehend scharf ein.
- **Zentraler Bereich für sensible Inhalte**: Die linke Seitenleiste enthält die Filterkategorie **Sensibel (18+) (🔞)**, um markierte Inhalte gesammelt zu prüfen oder neu zu klassifizieren.
