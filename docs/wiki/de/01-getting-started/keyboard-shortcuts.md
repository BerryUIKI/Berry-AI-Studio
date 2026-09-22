# Tastaturkürzel-Übersicht

Berry AI Studio ist für einen **Tastatur-fokussierten Arbeitsablauf** optimiert. Sie können umfangreiche Bibliotheken durchsuchen, bewerten, gruppieren, untersuchen und organisieren, ohne die Hand von der Tastatur nehmen zu müssen.

---

## 1. Vorschau & Galerienavigation

| Tastenkürzel | Aktion | Bereich | Beschreibung |
| :--- | :--- | :--- | :--- |
| `Leertaste` oder `Enter` | **Vollbild-Vorschau (Lightbox)** | Galerieauswahl | Öffnet die hochauflösende Vorschau für das ausgewählte Bild. |
| `Esc` | **Schließen / Zurücksetzen** | Global | Schließt die Lightbox oder Modalfenster, leert die Suche oder hebt die Kartenauswahl auf. |
| `←` / `→` | **Vorheriges / Nächstes Element** | Galerie & Lightbox | Bewegt die Auswahl zum benachbarten Element (oder 1 Frame weiter bei Videowiedergabe). |
| `↑` / `↓` | **Zeile nach oben / unten** | Rasteransicht | Verschiebt die Auswahl visuell um eine Zeile nach oben oder unten. |
| `Strg + A` / `Cmd + A` | **Alles auswählen** | Galeriebereich | Wählt alle Karten aus, die dem aktuellen Filter/Suchkriterium entsprechen. |
| `Entf` oder `Rücktaste` | **In den Papierkorb** | Galerieauswahl | Verschiebt ausgewählte Elemente sicher in den Papierkorb des Betriebssystems. |

---

## 2. Schnelle Kuration & Bewertung

| Tastenkürzel | Aktion | Beschreibung |
| :--- | :--- | :--- |
| `1` | **1 Stern vergeben** | Weist dem ausgewählten Element 1 Stern zu. |
| `2` | **2 Sterne vergeben** | Weist dem ausgewählten Element 2 Sterne zu. |
| `3` | **3 Sterne vergeben** | Weist dem ausgewählten Element 3 Sterne zu. |
| `4` | **4 Sterne vergeben** | Weist dem ausgewählten Element 4 Sterne zu. |
| `5` | **5 Sterne vergeben** | Weist dem ausgewählten Element 5 Sterne zu. |
| `0` | **Bewertung löschen** | Entfernt die Sternebewertung von ausgewählten Elementen. |
| `F` | **Favorit umschalten** | Markiert ausgewählte Elemente als Favorit oder hebt die Markierung auf. |

> [!TIP]
> **Blindbewertung**: Halten Sie eine Hand auf den Pfeiltasten (`←`, `→`), um durch Bilder zu blättern, während Sie mit der anderen Hand die Zifferntasten `1`–`5` bedienen. Bewertungen werden sofort in die SQLite-Datenbank geschrieben, ohne das Weiterschalten zu verzögern.

---

## 3. Stapelbildung & Bildvergleich

| Tastenkürzel | Aktion | Bereich | Beschreibung |
| :--- | :--- | :--- | :--- |
| `Strg + G` / `Cmd + G` | **In Stapel gruppieren** | Galerieauswahl | Fasst ausgewählte Bilder zu einem Poker-Deck-Serienstapel zusammen. |
| `Strg + Umschalt + G` / `Cmd + Shift + G` | **Stapel auflösen** | Galerieauswahl | Löst den ausgewählten Stapel wieder in Einzelbilder auf. |
| `Alt + S` / `Option + S` | **Als Titelbild festlegen** | Stapelauswahl | Legt das aktive Bild als primäres Titelbild (Hero) des Stapels fest. |
| `C` | **Bildvergleich nebeneinander** | Galerieauswahl | Öffnet die 1:1-Vergleichsansicht mit synchronisiertem Zoom und Verschieben. |

---

## 4. Arbeitsbereich-Bedienfelder & Zoom

| Tastenkürzel | Aktion | Beschreibung |
| :--- | :--- | :--- |
| `B` | **Seitenleiste umschalten** | Blendet das linke Navigationspanel ein oder aus (Bibliothek, Ordner, Tags). |
| `I` | **Inspektor umschalten** | Blendet das rechte Metadaten-Inspektorpanel ein oder aus. |
| `/` oder `Strg + F` / `Cmd + F` | **Suchleiste fokussieren** | Aktiviert das Suchfeld und markiert den vorhandenen Abfragetext. |
| `Strg + =` / `Cmd + =` | **Vergrößern** | Vergrößert die Kartengröße in der Galerieansicht. |
| `Strg + -` / `Cmd + -` | **Verkleinern** | Verkleinert die Kartengröße in der Galerieansicht. |
| `Strg + 0` / `Cmd + 0` | **Zoom zurücksetzen** | Setzt die Kartengröße auf die Standardbreite (256px) zurück. |
| `F11` | **Vollbild** | Schaltet den rahmenlosen Vollbildmodus des Fensters um. |

---

## 5. Bibliotheksaktionen & Dialoge

| Tastenkürzel | Aktion | Beschreibung |
| :--- | :--- | :--- |
| `Strg + O` / `Cmd + O` | **Ordner hinzufügen** | Öffnet den Assistenten zum Hinzufügen von Ordnern und Pipelines. |
| `Strg + E` / `Cmd + E` | **Exportieren...** | Öffnet das Modal für Stapelexport, Transkodierung und HTML-Showcases. |
| `Strg + ,` / `Cmd + ,` | **Einstellungen...** | Öffnet das zentrale Einstellungsfenster der Anwendung. |
| `?` oder `Umschalt + /` | **Tastaturkürzel** | Öffnet die Übersicht aller Tastenkürzel. |
| `Alt + F4` | **Beenden** | Beendet Berry AI Studio ordnungsgemäß. |

---

## 6. Auswahlerweiterungen mit Maustasten

- **Einfacher Klick**: Wählt eine einzelne Karte aus und setzt sie als **Auswahlanker**.
- **Umschalt + Klick**: Erweitert die Auswahl vom Anker bis zur angeklickten Karte als **zusammenhängenden Bereich**.
- **Strg + Klick** (bzw. `Cmd + Klick` unter macOS): **Schaltet die Auswahl** für eine einzelne Karte um, ohne andere Auswahlen aufzuheben, und aktualisiert den Anker.
- **Doppelklick auf Titelbild eines Stapels**: Öffnet direkt die Vollbild-Vorschau (Lightbox) des Titelbilds (eine Gestenerkennung verhindert versehentliches Aufklappen).
- **Einfacher Klick auf Stapelzähler-Badge**: Klappt den Bildstapel direkt im Raster auf oder zu.
