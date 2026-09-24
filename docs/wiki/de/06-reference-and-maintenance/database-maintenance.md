# Datenbank- & Speicherwartung

Omera ist für einen dauerhaft wartungsarmen Betrieb ausgelegt. Dennoch empfiehlt es sich bei intensiver Nutzung — wenn zehntausende Kunstwerke kuratiert, gelöscht oder umorganisiert werden —, gelegentlich eine Datenbankkomprimierung und Cache-Bereinigung durchzuführen, um die maximale Arbeitsgeschwindigkeit zu sichern.

---

## 1. Das Datenbank-Verwaltungsfenster (`DatabaseManagerModal.vue`)

Öffnen Sie das Verwaltungsfenster über **Datei > Datenbankverwaltung...** oder über die Schaltfläche **Datenbank** in der Fußzeile der linken Seitenleiste.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Datenbank- & Speicherwartung                                       [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Datenbankspeicher & Tabellenmetriken                                   │
│ • Datenbankdatei:     omera.db (WAL-Modus)                             │
│ • Indizierte Bilder:  48.210 Dateien in 6 Ordnern                      │
│ • Speicherbelegung:   128,4 MB (Datenbank) / 1,42 GB (Vorschaubilder)  │
│ • Alben & Tags:       12 Alben, 45 Tags                                │
│ • Schemaversion:      v14 (14 angewendete Migrationen)                 │
│ • Freie Seiten:       1.420 Seiten (~5,6 MB freigebbar)                │
├────────────────────────────────────────────────────────────────────────┤
│ Wartungsoperationen                                                    │
│ [ 🧹 VACUUM ausführen ]             [ 💾 Backup exportieren… ]         │
│ [ ↺ Aus Backup wiederherstellen… ]  [ 🗑 Vorschaubild-Cache leeren ]   │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. SQLite-Komprimierung (`VACUUM`)

### Was ist Datenbank-Fragmentierung?
Beim Löschen von Bildern, Entfernen von Tags oder Auflösen von Bilderstapeln gibt SQLite den Speicherplatz auf dem Datenträger nicht sofort frei, sondern markiert betroffene Datenbankseiten lediglich als „frei“ (Freelist).

### Ausführen von `VACUUM`:
- Ein Klick auf **„VACUUM ausführen“** startet die native Optimierungsroutine von SQLite.
- Omera baut die Datenbankdatei vollständig neu auf, bereinigt ungenutzte Freelist-Seiten und reduziert die Dateigröße von `omera.db` auf der Festplatte.
- **Sicherheit**: Der Vorgang ist vollkommen transaktionssicher. Sollte während der Komprimierung der Strom ausfallen, führt SQLite beim nächsten Start ein automatisches Rollback durch, ohne Daten zu beschädigen.

---

## 3. Datenbank-Sicherung & Wiederherstellung

### Lokales Backup erstellen (`backup_database`)
- Klicken Sie auf **„Backup exportieren…“**, um einen geprüften, nicht-blockierenden Snapshot Ihrer Datenbank anzulegen.
- Omera nutzt die Online-Backup-API von SQLite, sodass Backups erstellt werden können, während Sie ohne Unterbrechung weiterarbeiten.

### Aus Backup wiederherstellen (`restore_database`)
- Wenn Sie Ihre Bibliothek auf einen neuen Rechner übertragen oder versehentliche Änderungen rückgängig machen möchten, wählen Sie **„Wiederherstellen…“**.
- Omera legt vor dem Überschreiben eine Sicherheitskopie der aktuellen Datenbank an (`omera.db.rollback`), ersetzt die Datenbank durch das Backup und lädt die Galerie direkt mit dem wiederhergestellten Stand neu.

---

## 4. Cache-Verwaltung für Vorschaubilder

Vorschaubilder werden in `<app_data_dir>/thumbnails/` als hochoptimierte WebP-Dateien gespeichert.

### Konfigurierbares Cache-Budget:
- Unter **Einstellungen > Galerie** können Sie das **Thumbnail Cache Budget** anpassen (Standard: `2048 MB` / 2 GB).
- Omera erfasst den letzten Zugriffszeitpunkt jedes Vorschaubilds in der Datenbanktabelle `thumbnail_cache_entries`.
- Überschreitet der Cache das gewählte Budget, werden nach dem **LRU-Prinzip (Least Recently Used)** automatisch die am längsten nicht aufgerufenen Vorschaubilder gelöscht.

### Cache manuell leeren:
- Um sofort Speicherplatz freizugeben, klicken Sie auf **„Cache leeren“**.
- Omera löscht alle gecachten WebP-Dateien von der Festplatte und setzt die Cache-Tabelle zurück. Beim erneuten Betrachten von Ordnern werden die Vorschaubilder bei Bedarf automatisch neu generiert.

---

## 5. Vorschaubild-Diagnose (`ThumbnailDiagnosticsModal.vue`)

Zur Leistungsanalyse und Fehlerprüfung:
- Öffnen Sie **Einstellungen > Galerie > Diagnose**, um detaillierte Systemmetriken einzusehen:
  - Anzahl aktiver Rayon-Hintergrund-Threads (z. B. `4 Worker aktiv`).
  - Ausstehende Generierungs-Warteschlangen.
  - Abgeschlossene vs. abgebrochene Rendering-Aufträge.
  - Trefferquote des speicherinternen LRU-Caches.
