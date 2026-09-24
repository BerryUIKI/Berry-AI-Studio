# Datenschutz- & Sicherheitsarchitektur

Omera basiert auf dem Grundsatz **100% Local-First und keinerlei Telemetrie**. In einer Zeit, in der generative KI-Workflows oft geschützte Kunststile, vertrauliche Charakterkonzepte und sensible Kundenprojekte umfassen, stellt Omera sicher, dass Ihre kreativen Arbeiten ausnahmslos auf Ihrem lokalen Rechner verbleiben.

---

## 1. Null Telemetrie & Vollständiger Offline-Betrieb

### Kein Nach-Hause-Telefonieren
- Omera enthält **keine Tracking-Pixel, keine Analyse-SDKs und keine externen Absturzmeldedienste** (weder Google Analytics noch Sentry, Mixpanel oder PostHog).
- Sie können Omera vollständig ohne Internetverbindung oder hinter restriktiven Unternehmens-Firewalls betreiben, ohne dass Funktionen eingeschränkt werden.

### Ausgehende Netzwerkverbindungen
Omera baut Netzwerkverbindungen **ausschließlich** in drei explizit vom Nutzer gesteuerten Fällen auf:
1. **Anwendungs-Aktualisierungen**: Wenn die Option **Beim Start nach Updates suchen** aktiv ist (oder manuell über `Hilfe > Nach Updates suchen...` ausgelöst wird), fragt Omera die offizielle GitHub-Releases-API ab (`https://api.github.com/repos/BerryUIKI/Omera/releases/latest`).
2. **Civitai-Hash-Abfrage**: Wenn Sie im Inspektor gezielt auf die Schaltfläche zum Nachschlagen eines unbekannten Modell-Hashes klicken, sendet Omera eine einzelne HTTP-Anfrage an die öffentliche Civitai-API.
3. **Cloud-Backup & Team-Synchronisation**: Wenn Sie in den Einstellungen explizit einen AWS S3-, WebDAV- oder zentralen PostgreSQL/MySQL-Server konfiguriert haben.

---

## 2. Ausschließlich lokale KI-Inferenz

Alle Machine-Learning-Funktionen in Omera laufen vollständig auf Ihrer lokalen CPU/GPU über integrierte **ONNX Runtime**-Sitzungen (`ort`):

- **CLIP- / SigLIP-Einbettungen**: Bildvorverarbeitung und Vektorisierung finden lokal statt. Weder Prompts, Textsuchanfragen noch Bildpixel verlassen Ihren Computer.
- **WD14 Anime-Auto-Tagging**: Die neuronale Inferenz nutzt ausschließlich lokale Modelldateien (`models/`). Erkannte Tags werden direkt in Ihre lokale SQLite-Datenbank geschrieben.

---

## 3. Datenschutzorientierter Export & Metadatenbereinigung

Für die Veröffentlichung von Kunstwerken bietet Omera ein spezialisiertes **4-stufiges Bereinigungsmodul**:

- Sie können eingebettete ComfyUI-Knotengraphen, positive und negative Prompts, Seeds und LoRA-Tags vor dem Hochladen auf soziale Plattformen oder Discord mit einem Klick entfernen (siehe [Export & Web-Showcase](../05-export-and-collaboration/export-and-web-showcase.md)).
- Die Stufe **Vollständig bereinigen** entfernt ausnahmslos alle EXIF- und ICC-Daten, sodass lediglich reine Bildpixel exportiert werden.

---

## 4. Open-Source-Transparenz & Lizenz

Omera ist freie Open-Source-Software unter der **GNU Affero General Public License v3.0 (AGPL-3.0)**:

- Jede Zeile des Rust-Backends, der Tauri-Bridge-Befehle und der Vue 3-Frontend-Komponenten ist auf [GitHub](https://github.com/BerryUIKI/Omera) öffentlich einsehbar und prüfbar.
- Sie haben die volle Freiheit, Omera für private Zwecke oder im kommerziellen Studioumfeld zu auditieren, zu kompilieren, anzupassen und einzusetzen.
