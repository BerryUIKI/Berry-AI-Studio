# Multi-Datenbank-Team-Studio

Für Designstudios, Spieleentwickler und Kreativagenturen, bei denen mehrere Künstler gemeinsam auf geteiltem Netzwerkspeicher (NAS, SMB, NFS) arbeiten, kann Berry AI Studio über lokales SQLite hinaus zu einem **Multi-Datenbank-Studio für Team-Kollaboration** skaliert werden.

---

## 1. Die Multi-Datenbank-Architektur

Berry stellt eine asynchrone Abstraktionsschicht (`StorageEngine`) bereit, die drei Datenbank-Backends unterstützt:

| Backend | Empfohlene Teamgröße | Nebenläufigkeitsmodell | Performance-Profil |
| :--- | :--- | :--- | :--- |
| **SQLite (Standard)** | 1 Benutzer pro Bibliothek | Einzelner Schreiber / Mehrere Leser (WAL) | <0.5 ms Latenz auf lokalen NVMe-SSDs. |
| **MySQL 8.0+ / MariaDB** | 2 bis 50+ gleichzeitige Benutzer | Zeilenebenen-Sperren & `ngram`-Volltextindizierung | Sub-5ms Abfragen auf geteilten Bibliotheken mit 500.000+ Assets. |
| **PostgreSQL 14+** | 2 bis 100+ gleichzeitige Benutzer | MVCC, `tsvector` GIN-Indizes & `LISTEN/NOTIFY` | Extrem latenzarme Echtzeit-Kollaboration mit Broadcast-Ereignissen. |

```mermaid
graph TD
    NAS[("Geteiltes Studio-NAS: SMB / NFS / WebDAV")]
    DB[("Zentrale Studio-DB: PostgreSQL 14+ / MySQL 8+")]

    subgraph Workstation A (Windows)
        A_UI["Berry UI"]
        A_Thumb["Lokaler NVMe-Vorschaubild-Cache"]
        A_UI --- A_Thumb
    end

    subgraph Workstation B (macOS)
        B_UI["Berry UI"]
        B_Thumb["Lokaler NVMe-Vorschaubild-Cache"]
        B_UI --- B_Thumb
    end

    A_UI -->|Z:\ai_vault| NAS
    B_UI -->|/Volumes/ai_vault| NAS

    A_UI <-->|OCC-Versionsprüfung & Sync| DB
    B_UI <-->|OCC-Versionsprüfung & Sync| DB
```

---

## 2. Plattformübergreifendes Speicher-Root-Mapping

Eine zentrale Hürde in gemischten Studio-Umgebungen besteht darin, dass Windows, macOS und Linux unterschiedliche Pfadformate für denselben Netzwerkordner nutzen:
- Windows: `Z:\ai_vault\2026\character_01.png`
- macOS: `/Volumes/ai_vault/2026/character_01.png`
- Linux: `/mnt/nas/ai_vault/2026/character_01.png`

### Wie Berry dieses Problem löst:
1. **Plattformunabhängige Root-UUIDs**: Berry generiert eine universelle UUID für das geteilte Speicher-Root-Verzeichnis (gespeichert in der Tabelle `storage_roots`).
2. **Normalisierte URIs**: In der Datenbank werden Pfade plattformneutral verwaltet:
   ```
   berry://550e8400-e29b-41d4-a716-446655440000/2026/character_01.png
   ```
3. **Clientseitiges Mount-Mapping**: Unter **Einstellungen > Team & Datenbank** weist jede Workstation der Root-UUID ihren lokalen Dateisystem-Mount-Pfad zu. Berry übersetzt Pfade dynamisch zur Laufzeit, sodass ein von macOS aus getaggtes Bild für Windows-Nutzer ohne Pfadkonflikte sofort erreichbar ist.

---

## 3. Optimistische Nebenläufigkeitssteuerung (OCC)

Wenn mehrere Teammitglieder dieselbe Kollektion gleichzeitig bewerten, verschlagworten oder kuratieren, verhindert Berry Datenverluste mittels **Optimistischer Nebenläufigkeitssteuerung (OCC)**:

- Jeder Asset-Datensatz enthält eine `version`-Spalte auf Zeilenebene.
- Aktualisiert ein Künstler eine Bewertung, sendet Berry `set_file_rating_occ(file_id, new_rating, expected_version)`.
- **Konflikt-Richtlinien**:
  - **Bewertungen & Titelbilder**: Last-Write-Wins (LWW) mit sofortiger Oberflächen-Aktualisierung.
  - **Tags & Alben**: Vereinigungsmengen-Zusammenführung (Set-Union: Fügt Künstler A das Tag `„Charakter“` und Künstler B das Tag `„Konzept“` hinzu, bleiben beide Tags erhalten).
  - **Löschungen**: Soft-Delete-Statusflags verhindern versehentliches Wiederauferstehen gelöschter Einträge.

---

## 4. Mehrstufige Echtzeit-Synchronisation

Berry hält alle Workstations im Team über drei Synchronisationsstufen auf dem aktuellen Stand:

1. **Stufe 1: Abfrage des Änderungsprotokolls (Standard / Zero-DevOps)**:
   - Berry prüft die Journal-Tabelle `change_log` alle 3 Sekunden auf neue Transaktions-IDs. Erfordert keine zusätzliche Serverinfrastruktur.
2. **Stufe 2: PostgreSQL `LISTEN / NOTIFY` (<50 ms Latenz)**:
   - Bei Nutzung von PostgreSQL öffnet Berry einen asynchronen Benachrichtigungskanal. Änderungen einer Workstation werden in unter 50 ms an alle verbundenen Clients übertragen.
3. **Stufe 3: Verteilter WebSocket-Hub**:
   - Optionaler leichtgewichtiger Broadcast-Dienst für größere Enterprise-Studios.

---

## 5. Clientseitiges On-Demand-Vorschaubild-Caching

Das Laden zehntausender Vorschaubilder über das 1-Gbps- oder 10-Gbps-Studionetzwerk kann die Bandbreite stark belasten:
- Berry speichert herunterskalierte WebP-Vorschaubilder auf der **lokalen NVMe-SSD** jeder Workstation (`thumbnails/`).
- Beim Durchstöbern der geteilten Bibliothek werden Vorschaubilder bei Bedarf lokal generiert und vorgehalten.
- Dadurch bleibt der Netzwerkverkehr beim schnellen Scrollen in der Galerie minimal, sodass die NAS-Bandbreite für hochauflösende Renderings und Modell-Trainings frei bleibt.

---

## 6. Migrationsassistent: SQLite zu MySQL / PostgreSQL (`MigrationWizardModal.vue`)

Wenn Sie mit einer lokalen SQLite-Einzelplatz-Bibliothek begonnen haben und in eine zentrale Team-Datenbank wechseln möchten:
1. Öffnen Sie **Einstellungen > Team & Datenbank**.
2. Klicken Sie auf **„Migrationsassistent öffnen...“**.
3. Berry analysiert Ihre SQLite-Datenbank, lässt Sie zwischen MySQL 8.0+ und PostgreSQL 14+ wählen und generiert dialektoptimierte DDL-Schemata sowie transaktionale SQL-Batch-Migrationsskripte.
4. Führen Sie das generierte Skript auf Ihrem Datenbankserver aus, um die Team-Bibliothek sofort bereitzustellen.
