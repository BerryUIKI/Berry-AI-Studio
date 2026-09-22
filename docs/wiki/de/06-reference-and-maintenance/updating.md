# Aktualisierung & Anwendungslebenszyklus

Berry AI Studio verfügt über eine integrierte, leise Aktualisierungsfunktion, die Verbesserungen und Fehlerbehebungen bereitstellt, ohne Ihren Arbeitsablauf zu unterbrechen oder Ihre Daten zu gefährden.

---

## 1. Nach Updates suchen

### Automatische Prüfung beim Start:
Standardmäßig fragt Berry beim Start der Anwendung die GitHub-Releases-API ab:
- Ist eine neuere Version verfügbar, erscheint ein Benachrichtigungs-Badge im Menü **Hilfe**.
- Dieses Verhalten lässt sich unter **Einstellungen > Allgemein > Beim Start nach Updates suchen** an- oder abschalten.

### Manuelle Prüfung:
Sie können jederzeit manuell nach neuen Versionen suchen:
- Wählen Sie in der oberen Menüleiste **Hilfe > Nach Updates suchen...**.

---

## 2. Das Aktualisierungsfenster (`UpdateModal.vue`)

Wird eine neue Version gefunden, öffnet sich das Update-Fenster:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Neue Version verfügbar: v0.3.1                                     [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Eine neue Version von Berry AI Studio ist verfügbar (Aktuell: v0.3.0). │
│                                                                        │
│ Versionshinweise:                                                      │
│ • Optimierter ComfyUI-Videometadaten-Parser für HunyuanVideo.          │
│ • Verbesserte Latenz bei Keyset-Cursor-Seitenaufrufen (100k+ Dateien). │
│ • 10-Sterne-Bewertungsauswahl in der Stapelaktionsleiste ergänzt.      │
├────────────────────────────────────────────────────────────────────────┤
│ Download-Fortschritt:                                                  │
│ [██████████████████████████░░░░░░░░░░] 68% (12,4 MB / 18,2 MB · 4 MB/s)│
├────────────────────────────────────────────────────────────────────────┤
│ [ Abbrechen ]                        [ 🚀 Automatisch aktualisieren ]  │
└────────────────────────────────────────────────────────────────────────┘
```

### Installationsablauf:
1. Klicken Sie auf **„🚀 Automatisch laden & aktualisieren“**.
2. Berry lädt das passende Release-Paket direkt von GitHub Releases in einen temporären Staging-Ordner.
3. Nach Abschluss des Downloads bittet Berry um einen Neustart der Anwendung.
4. Das Installationsprogramm führt eine nahtlose stille Aktualisierung durch und startet Berry AI Studio direkt in der neuen Version.

---

## 3. Garantien zur Datenerhaltung

Ein Upgrade von Berry AI Studio **berührt niemals Ihre persönlichen Daten**:

- **Sichere Datenbank**: Ihre SQLite-Datenbank `berry.db`, eigene Alben, Farb-Tags, Bewertungen und Stapelzuordnungen liegen im AppData-Verzeichnis Ihres Betriebssystems (`%APPDATA%`, `~/Library/Application Support` oder `~/.config`) — vollständig getrennt von den ausführbaren Anwendungsdateien.
- **Inkrementelle Schema-Migrationen**: Bringt eine neue Version strukturelle Datenbankänderungen mit, führt das Rust-Backend beim ersten Start automatisch **abwärtskompatible Schema-Migrationen** über `PRAGMA user_version` aus. Tabellen werden schrittweise erweitert, bestehende Einträge bleiben unangetastet.
- **Dauerhafte Einstellungen**: Ihre Konfiguration in `config.json`, das Vorschaubild-Budget und alle Ordnerpfade bleiben über alle Aktualisierungen hinweg zuverlässig erhalten.
