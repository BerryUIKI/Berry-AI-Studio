# Vollständige Einstellungsreferenz

Das Einstellungsfenster von Omera (`SettingsModal.vue`) wird über `Datei > Einstellungen...` oder `Strg + ,` / `Cmd + ,` aufgerufen. Alle Einstellungen werden dauerhaft in der Datei `config.json` in Ihrem lokalen Anwendungsdaten-Verzeichnis gespeichert.

---

## Tab 1: Allgemein

| Einstellungsfeld | Schlüssel in `config.json` | Standardwert | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Anzeigesprache** | `locale` | `"auto"` | Verfügbare Optionen: `auto` (folgt der Systemsprache), `en` (Englisch), `zh-CN` (Vereinfachtes Chinesisch), `zh-TW` (Traditionelles Chinesisch), `ja` (Japanisch), `de` (Deutsch), `fr` (Französisch), `es` (Spanisch). |
| **Standard-Galerieansicht** | `default_view` | `"grid"` | Startansichtsmodus beim Öffnen der Anwendung: `"grid"` (Rasteransicht), `"masonry"` (Wasserfall) oder `"table"` (Detaillierte Liste). |
| **Automatisch beim Start scannen** | `auto_scan` | `true` | Sucht beim Start automatisch nach neuen oder geänderten Dateien in indizierten Ordnern. |
| **Wartezeit für Startscan** | `startup_scan_interval_minutes`| `360` | Mindestabstand in Minuten zwischen vollständigen Datenträger-Abgleichen (`30`, `60`, `360`, `1440`). Verhindert Festplattenlast bei häufigen Neustarts. |
| **Beim Start nach Updates suchen** | `auto_check_update` | `true` | Fragt beim Start im Hintergrund GitHub Releases ab und zeigt bei verfügbaren Aktualisierungen ein Benachrichtigungs-Badge an. |

---

## Tab 2: Galerie (Anzeige & Sicherheitsschutz)

| Einstellungsfeld | Schlüssel in `config.json` | Standardwert | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Design (Farbschema)** | `theme` | `"system"` | Benutzeroberflächen-Farbschema: `"system"`, `"midnight"` (OLED-Dunkel), `"graphite"` (Neutrales Studio-Grau), `"violet"` (Kreatives Lila) oder `"light"` (Hell). |
| **Sensible Inhalte weichzeichnen** | `blur_nsfw` | `true` | Legt eine CSS-Weichzeichnungsmaske über als sensibel/FSK18 markierte Bilder, bis sie angeklickt werden. |
| **Karten-Badges anzeigen** | `show_card_badges` | `true` | Blendet Format (`PNG`, `MP4`), Auflösung (`1024×1024`), Generator-Badges und Bewertungen direkt auf Bildkarten ein. |
| **Vorschaubild-Auflösung** | `thumbnail_max_edge` | `384` | Kantenlänge der WebP-Vorschaubilder (Vielfaches von 64): `256` (Kompakt), `384` (Standard / Empfohlen), `448` (HD), `512` (Ultra). |
| **Thumbnail Cache Budget** | `thumbnail_cache_budget_mb` | `2048` | Maximaler Festplattenspeicher (in MB) für WebP-Vorschaubilder. Älteste Einträge werden bei Überschreitung per LRU automatisch gelöscht. |
| **Cache leeren** | — | — | Aktionstaste zum sofortigen Löschen aller gecachten Vorschaubilder von der Festplatte. |
| **Diagnose** | — | — | Öffnet eine Echtzeit-Übersicht des Rayon-Thread-Pools und der LRU-Cache-Metriken. |

---

## Tab 3: Stapel (Bildstapel & Serien)

| Einstellungsfeld | Schlüssel in `config.json` | Standardwert | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Automatisches Stapeln aktivieren** | `auto_stack` | `true` | Fasst aufeinanderfolgende Generierungsvariationen automatisch zu Poker-Deck-Karten zusammen. |
| **Prompt-Ähnlichkeitsschwelle** | `stack_similarity_threshold` | `0.85` | Erforderlicher Jaccard-Token-Ähnlichkeitswert (0.0 bis 1.0) zur Zusammenfassung von Bildern. |
| **Maximales Zeitfenster (Minuten)** | `stack_time_window_minutes` | `180` | Maximaler zeitlicher Abstand zwischen Generationen, um als gemeinsame Serie gewertet zu werden. |
| **Mehrere Stapel gleichzeitig öffnen**| `allow_multiple_open_stacks` | `false` | Steuert, ob das Öffnen eines Stapels andere Stapel automatisch schließt (`false`) oder mehrere Stapel gleichzeitig ausgeklappt bleiben (`true`). |
| **Warnmeldungen zurücksetzen** | — | — | Aktiviert Bestätigungsdialoge (z. B. Stapel-Zusammenführungswarnungen) wieder, falls zuvor „Nicht mehr anzeigen“ gewählt wurde. |

---

## Tab 4: Generierungs-Interop

| Einstellungsfeld | Schlüssel in `config.json` | Standardwert | Beschreibung |
| :--- | :--- | :--- | :--- |
| **ComfyUI Basis-URL** | `comfyui_url` | `"http://127.0.0.1:8188"` | HTTP-Endpunkt der lokalen ComfyUI-Instanz. Inklusive Verbindungstest-Schaltfläche. |
| **SD WebUI Basis-URL** | `webui_url` | `"http://127.0.0.1:7860"` | HTTP-Endpunkt der lokalen AUTOMATIC1111 / Forge / SD.Next-Instanz. Inklusive Verbindungstest. |

---

## Tab 5: Team-Kollaboration & Geteilte Datenbank

| Einstellungsfeld | Schlüssel in `config.json` | Standardwert | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Datenbank-Backend** | `storage_backend` | `"sqlite"` | Aktives Datenbanksystem: `"sqlite"`, `"mysql"` oder `"postgres"`. |
| **Remote-Verbindungs-URL** | `remote_connection_url` | `""` | Verbindungszeichenfolge (z. B. `postgres://user:pass@host:5432/omera_studio`). |
| **Workstation Client-ID** | `client_identifier` | `""` | Eindeutiger Rechnername zur Kennzeichnung im Änderungsprotokoll und zur OCC-Konfliktverfolgung. |
| **Geteilte Speicherpfade** | `root_mappings` | `{}` | Plattformübergreifende Pfadzuweisungen zwischen zentralen Root-UUIDs und lokalen Betriebssystem-Mounts. |
| **Verbindung testen** | — | — | Sendet einen Ping an den Datenbankserver und misst die Netzwerk-Roundtrip-Latenz in Millisekunden. |
| **Migrationsassistent** | — | — | Startet den Assistenten zum Exportieren von SQLite in MySQL/PostgreSQL-Schemata. |

---

## Tab 6: Cloud-Backup & Medienspiegelung

| Einstellungsfeld | Schlüssel in `config.json` | Standardwert | Beschreibung |
| :--- | :--- | :--- | :--- |
| **Backup-Speicheranbieter** | `cloud_backup.provider` | `"local_path"` | Protokoll: `"local_path"`, `"webdav"` oder `"s3"`. |
| **WebDAV-Zugangsdaten** | `cloud_backup.webdav_*` | `""` | Endpunkt-URL, Benutzername und Passwort für Nextcloud / Synology WebDAV. |
| **S3-Zugangsdaten** | `cloud_backup.s3_*` | `""` | Endpunkt, Bucket, Region, Access Key und Secret Key für S3 / Cloudflare R2 / MinIO. |
| **Delta-Synchronisations-Strategie**| `cloud_sync.strategy` | `"fingerprint"` | `"fingerprint"` (Dateigröße + ETag) oder `"checksum"` (vollständige SHA-256-Prüfsumme). |
| **Übertragungs-Threads** | `cloud_sync.threads` | `4` | Anzahl paralleler Worker-Threads für Medien-Uploads. |
| **Bandbreitenlimit** | `cloud_sync.bandwidth_limit_kbs` | `0` | Maximale Upload-Geschwindigkeit in KB/s (0 = unbegrenzt). |

---

## Tab 7: Metadaten-Extraktionsmodule

Zeigt den aktuellen Status der integrierten verlustfreien Metadaten-Parser an:
- AUTOMATIC1111 / SD.Next PNG `parameters`-Chunk-Parser (Aktiv 🟢)
- ComfyUI Knoten-Ausführungsgraph und `workflow`-JSON-Parser (Aktiv 🟢)
- NovelAI `Comment`- und `Description`-Parser (Aktiv 🟢)
- Fooocus / Fooocus-MRE Parameterblock-Parser (Aktiv 🟢)
- InvokeAI `sd-metadata`- und `invokeai_metadata`-Parser (Aktiv 🟢)
- MP4 ISOBMFF- und WebM EBML-Videostrom-Parser (Aktiv 🟢)

---

## Tab 8: Datenspeicher & Info

- **Version**: Zeigt die installierte Anwendungsversion an (z. B. `v0.3.0`).
- **Datenbankschema**: Meldet die aktive SQLite-Schema-Migrationsstufe (z. B. `Schema Version 14`).
- **Lokaler SQLite-Datenbankpfad**: Absoluter Dateipfad zu `omera.db`.
- **Schaltflächen zum schnellen Öffnen**:
  - `Konfigurationsdatei öffnen`: Öffnet den Ordner mit `config.json`.
  - `Datenbankordner öffnen`: Öffnet das Verzeichnis mit `omera.db` und den WAL-Dateien.
  - `Vorschaubild-Cache öffnen`: Öffnet das Verzeichnis des WebP-Thumbnail-Caches.
  - `Modellordner öffnen`: Öffnet das Verzeichnis der lokalen ONNX-Gewichte.
