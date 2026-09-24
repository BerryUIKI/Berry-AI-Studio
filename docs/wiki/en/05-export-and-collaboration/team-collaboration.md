# Multi-Database Team Studio

For design studios, gaming companies, and visual agencies with multiple artists working on shared network storage (NAS, SMB, NFS), Omera can scale beyond local SQLite into a **Multi-Database Team Collaboration Studio**.

---

## 1. The Multi-Database Architecture

Omera provides an asynchronous `StorageEngine` abstraction layer supporting three backends:

| Backend | Recommended Team Size | Concurrency Model | Performance Profile |
| :--- | :--- | :--- | :--- |
| **SQLite (Default)** | 1 user per library | Single-writer / Multi-reader WAL | <0.5 ms latency on local NVMe SSDs. |
| **MySQL 8.0+ / MariaDB** | 2 to 50+ concurrent users | Row-level locking & `ngram` full-text indexing | Sub-5ms queries on shared 500,000+ asset libraries. |
| **PostgreSQL 14+** | 2 to 100+ concurrent users | MVCC, `tsvector` GIN indexes & `LISTEN/NOTIFY` | Low-latency real-time collaboration with broadcast events. |

```mermaid
graph TD
    NAS[(Shared Studio NAS: SMB / NFS / WebDAV)]
    DB[(Central Studio DB: PostgreSQL 14+ / MySQL 8+)]

    subgraph Workstation A (Windows)
        A_UI[Omera UI]
        A_Thumb[Local NVMe Thumbnail Cache]
        A_UI --- A_Thumb
    end

    subgraph Workstation B (macOS)
        B_UI[Omera UI]
        B_Thumb[Local NVMe Thumbnail Cache]
        B_UI --- B_Thumb
    end

    A_UI -->|Z:\ai_vault| NAS
    B_UI -->|/Volumes/ai_vault| NAS

    A_UI <-->|OCC Version Checks & Sync| DB
    B_UI <-->|OCC Version Checks & Sync| DB
```

---

## 2. Cross-Platform Storage Root Mapping

A major challenge in cross-platform studios is that Windows, macOS, and Linux use different path formats for the same shared network folder:
- Windows: `Z:\ai_vault\2026\character_01.png`
- macOS: `/Volumes/ai_vault/2026/character_01.png`
- Linux: `/mnt/nas/ai_vault/2026/character_01.png`

### How Omera Solves This:
1. **Platform-Agnostic Root UUIDs**: Omera generates a universal UUID for the shared storage root (stored in the database table `storage_roots`).
2. **Normalized URIs**: In the database, paths are stored platform-agnostically:
   ```
   omera://550e8400-e29b-41d4-a716-446655440000/2026/character_01.png
   ```
3. **Client-Side Mount Mapping**: In **Settings > Team & Collaboration**, each artist maps the Root UUID to their local operating system mount path. Omera dynamically translates paths on the fly, so an artwork tagged by an artist on macOS is instantly accessible to an artist on Windows.

---

## 3. Optimistic Concurrency Control (OCC)

When multiple team members rate, tag, or curate the same collection simultaneously, Omera prevents data corruption using **Optimistic Concurrency Control (OCC)**:

- Every asset record includes a row-level `version` column.
- When an artist updates a rating, Omera submits `set_file_rating_occ(file_id, new_rating, expected_version)`.
- **Conflict Policies**:
  - **Ratings & Hero Covers**: Last-Write-Wins (LWW) with immediate UI update.
  - **Tags & Albums**: Set-Union merge (if Artist A adds tag `"Character"` and Artist B adds tag `"Concept"`, both tags are preserved).
  - **Deletions**: Soft-deletion status flags prevent phantom recreation.

---

## 4. Real-Time Collaboration Sync Tiers

Omera synchronizes state changes across the team using a 3-tier sync architecture:

1. **Tier 1: Change Log Polling (Default / Zero-DevOps)**:
   - Omera queries the `change_log` journal table every 3 seconds for new transaction IDs. Requires no special server setup.
2. **Tier 2: PostgreSQL `LISTEN / NOTIFY` (<50 ms Latency)**:
   - When using PostgreSQL, Omera establishes an asynchronous notification channel. Changes made by any workstation are broadcast to other clients in under 50 ms.
3. **Tier 3: Distributed WebSocket Hub**:
   - Optional lightweight broadcast service for larger enterprise studios.

---

## 5. Client-Side On-Demand Thumbnail Caching

Loading thumbnails across a 1Gbps or 10Gbps studio network can saturate shared bandwidth:
- Omera stores downscaled WebP thumbnails on each workstation's **local NVMe SSD** (`thumbnails/`).
- When an artist browses the shared library, thumbnails are generated and cached locally on demand.
- This keeps network traffic near zero during fast gallery scrolling, leaving shared NAS bandwidth free for high-resolution renders and model training.

---

## 6. Migration Wizard: SQLite to MySQL / PostgreSQL (`MigrationWizardModal.vue`)

If you started with a single-user SQLite library and want to upgrade to a shared team database:
1. Open **Settings > Team & Collaboration**.
2. Click **"Launch Central Database Migration Wizard"**.
3. Omera inspects your local SQLite database, lets you choose MySQL 8.0+ or PostgreSQL 14+, and generates dialect-optimized DDL schemas and transactional SQL batch migration files.
4. Execute the migration script on your database server to transition your studio's library.
