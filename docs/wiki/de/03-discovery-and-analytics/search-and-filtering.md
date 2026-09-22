# Suchsyntax & visuelle Filter

Berry AI Studio verfügt über ein zweistufiges Suchsystem: Die **strukturierte Syntaxsuche** für präzise technische Filterungen und die **KI-Semantische Suche** für konzeptionelle Abfragen in natürlicher Sprache.

---

## 1. Syntaxsuche & Abfragesprache

In der oberen Suchleiste (`/` oder `Strg + F`) können Sie Freitext-Schlüsselwörter eingeben oder strukturierte Schlüssel-Wert-Abfragen zusammenstellen:

### Einfache Freitextsuche
- Die Eingabe von Wörtern ohne Präfix durchsucht Dateinamen, relative Verzeichnispfade, positive und negative Prompts sowie Modellnamen:
  ```
  cyberpunk neon regen
  ```
- Verwenden Sie doppelte Anführungszeichen für die exakte Phrasensuche:
  ```
  "cyberpunk street" "rainy reflections"
  ```

---

## 2. Referenz der Schlüssel-Wert-Syntax

Berry parst Suchbegriffe in nativem Rust in strukturierte `SearchCriteria` und fragt SQLite-Indizes mit Ausführungszeiten im Sub-Millisekunden-Bereich ab:

| Schlüssel | Beispielsyntax | Beschreibung |
| :--- | :--- | :--- |
| `prompt:` | `prompt:"masterpiece, 1girl"` | Sucht nach Wörtern im positiven Generierungs-Prompt. |
| `neg:` | `neg:"bad hands, blurry"` | Sucht nach Wörtern im negativen Prompt. |
| `model:` | `model:"animagine_xl"` | Filtert nach dem Namen des Checkpoint-Modells. |
| `hash:` | `hash:31e35c80` | Filtert nach dem kurzen Hash oder dem vollständigen SHA256-Modell-Hash. |
| `sampler:` | `sampler:"Euler a"` | Filtert nach dem Sampler-Algorithmus. |
| `steps:` | `steps:30`, `steps:20..40`, `steps:>=25` | Filtert nach exakter Schrittanzahl oder numerischen Wertebereichen. |
| `cfg:` | `cfg:7`, `cfg:>=7.5`, `cfg:5..10` | Filtert nach CFG-Skalawerten oder Wertebereichen. |
| `seed:` | `seed:12345678` | Sucht nach einem spezifischen Generierungs-Seed. |
| `rating:` | `rating:5`, `rating:>=4`, `rating:1..3` | Filtert nach Benutzer-Sternebewertung. |
| `aesthetic:`| `aesthetic:>=7.0` | Filtert nach dem Ästhetik-Vorhersagewert. |
| `fav:` | `fav:true`, `fav:false` | Filtert nach Favoriten-Status (Lesezeichen). |
| `is:` | `is:nsfw`, `is:sfw` | Filtert nach der Kennzeichnung für sensible Inhalte. |
| `type:` | `type:image`, `type:video` | Filtert nach dem Medientyp (Bild oder Video). |
| `duration:` | `duration:>=5`, `duration:5..30` | Filtert nach Videodauer in Sekunden. |
| `fps:` | `fps:>=30`, `fps:24..60` | Filtert nach der Bildwiederholrate (FPS). |

### Syntax für numerische Wertebereiche
- **Zwischen zwei Werten (`min..max`)**: `steps:20..35` (inklusive zwischen 20 und 35 Schritten).
- **Größer oder gleich (`>=`)**: `cfg:>=7.0` (CFG-Skala von 7.0 oder höher).
- **Kleiner oder gleich (`<=`)**: `rating:<=2` (2 Sterne oder weniger).
- **Exakte Übereinstimmung**: `rating:5` (genau 5 Sterne).

---

## 3. Das visuelle Filter-Bedienfeld (`FilterDrawer.vue`)

Wenn Sie eine grafische Oberfläche gegenüber der manuellen Syntaxeingabe bevorzugen, klicken Sie rechts neben der Suchleiste auf **„Filter“ (`☰`)**, um das Filter-Bedienfeld auszuklappen:

```
┌────────────────────────────────────────────────────────┐
│ Visuelle Suche & Filter                [Zurücksetzen] [✕] │
├────────────────────────────────────────────────────────┤
│ Modell-Checkpoint                                      │
│ [ Alle Modelle ▾                                     ] │
│                                                        │
│ Sampler                                                │
│ [ Alle Sampler ▾                                     ] │
│                                                        │
│ Sterne-Bewertung                                       │
│ [ ★★★★★ (Nur 5 Sterne) ▾                             ] │
│                                                        │
│ Schrittbereich (Steps)                                 │
│ Min: [ 20 ] ─────────────●──────────── Max: [ 50 ]     │
│                                                        │
│ CFG-Skalenbereich                                      │
│ Min: [ 5.0 ] ────────────●──────────── Max: [ 12.0 ]   │
│                                                        │
│ Medientyp                                              │
│ (●) Alle Medien    ( ) Nur Bilder    ( ) Nur Videos   │
│                                                        │
│ Inhaltsmerkmale                                        │
│ [✓] Nur Favoriten    [ ] Nur sensible Inhalte (NSFW)   │
└────────────────────────────────────────────────────────┘
```

Das Filter-Bedienfeld verfügt über einen Zähler für aktive Filterkriterien, sodass Sie auf einen Blick sehen, wie viele Filter Ihre aktuellen Ergebnisse einschränken.

---

## 4. Sortieroptionen der Galerie (`SortBar.vue`)

Rechts in der Kopfzeile des Galeriebereichs können Sie Ihre aktiven Ergebnisse sortieren:

### Sortierfelder:
- **Änderungsdatum (`modified_at`)**: Chronologische Reihenfolge nach Dateizeitstempel.
- **Dateiname / Pfad (`path`)**: Alphabetische Sortierung nach Dateipfad.
- **Dateigröße (`size_bytes`)**: Sortierung nach benötigtem Speicherplatz.
- **Bewertung (`rating`)**: Sortierung nach vergebenen Sternen.
- **Ästhetik-Score (`aesthetic_score`)**: Sortierung nach neuronaler Ästhetik-Bewertung.

### Sortierrichtung:
- Klicken Sie auf die Richtungsschaltfläche, um zwischen **Absteigend (`↓`)** (höchste / neueste zuerst) und **Aufsteigend (`↑`)** (niedrigste / älteste zuerst) zu wechseln.
