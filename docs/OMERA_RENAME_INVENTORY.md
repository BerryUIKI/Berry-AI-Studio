# Omera R0 Baseline and Compatibility Inventory

Updated: 2026-09-24. Prepared by Lead Engineer/CTO for [#134](https://github.com/BerryUIKI/Omera/issues/134).
Reference baseline: `dev` commit `05715da44bcf9c50144fd5f58a2624866176f934`.
Governing contracts: [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md), [OMERA_MIGRATION.md](OMERA_MIGRATION.md), [API_CONTRACTS.md](API_CONTRACTS.md).

---

## 1. Released Versions & Installer Matrix

| Release Tag | Product Name | Tauri Identifier | Executable / Assets | Installer Scope & Formats |
| :--- | :--- | :--- | :--- | :--- |
| `v0.1.0` | Berry AIGC Toolbox | `com.berryuiki.berryaigctoolbox` | `Berry-AIGC-Toolbox` | Windows NSIS (`.exe`), MSI (`.msi`); macOS DMG (`.dmg`), App bundle (`.app.tar.gz`); Linux AppImage, DEB (`.deb`), RPM (`.rpm`) |
| `v0.1.1` | Berry-AIGC-Toolbox | `com.berryuiki.berryaigctoolbox` | `Berry-AIGC-Toolbox` | Same formats as v0.1.0 |
| `v0.1.2` | Berry-AIGC-Toolbox | `com.berryuiki.berryaigctoolbox` | `Berry-AIGC-Toolbox` | Same formats as v0.1.0 |
| `v0.1.3` | Berry AI Studio | `com.berryuiki.berryaistudio` | `Berry AI Studio` | Windows NSIS (`x64-setup.exe`), MSI; macOS DMG (`aarch64`, `x64`), `.app.tar.gz`; Linux AppImage, DEB, RPM |
| `v0.2.0` | Berry AI Studio | `com.berryuiki.berryaistudio` | `Berry AI Studio` | Same formats as v0.1.3 |
| `v0.2.1` | Berry AI Studio | `com.berryuiki.berryaistudio` | `Berry AI Studio` | Same formats as v0.1.3 |
| `v0.3.0` | Berry AI Studio | `com.berryuiki.berryaistudio` | `Berry AI Studio` | Same formats as v0.1.3 |
| **Target (R4)** | **Omera** | **`com.berryuiki.omera`** | **`omera`** | Windows NSIS/MSI, macOS DMG/App bundle, Linux AppImage/DEB/RPM |

---

## 2. Platform Storage Roots & Path Mapping

Tauri resolves application directories using OS-standard conventions based on the bundle identifier.

### A. Root Directories by Platform

| Platform | Target Root (`com.berryuiki.omera`) | Primary Legacy Root (`com.berryuiki.berryaistudio`) | Secondary Legacy Root (`com.berryuiki.berryaigctoolbox`) |
| :--- | :--- | :--- | :--- |
| **macOS** | `~/Library/Application Support/com.berryuiki.omera` | `~/Library/Application Support/com.berryuiki.berryaistudio` | `~/Library/Application Support/com.berryuiki.berryaigctoolbox` |
| **Windows** | `%APPDATA%\com.berryuiki.omera` | `%APPDATA%\com.berryuiki.berryaistudio` | `%APPDATA%\com.berryuiki.berryaigctoolbox` |
| **Linux** | `~/.config/com.berryuiki.omera` | `~/.config/com.berryuiki.berryaistudio` | `~/.config/com.berryuiki.berryaigctoolbox` |

### B. Ownership Classification: App-Owned vs. Protected User Assets

During discovery, migration, and any eventual user-approved cleanup, paths are strictly partitioned:

1. **Eligible App-Owned Data** (can be migrated and, upon separate explicit confirmation, moved to system Trash):
   - SQLite Database & WAL: `berry.db`, `berry.db-wal`, `berry.db-shm`
   - Configuration files: `config.json`, `config.json.bak`
   - Thumbnail cache directory: `thumbnails/`
   - Downloaded ONNX model weights: `models/`
   - Downloaded staging installers: `updates/`
2. **Protected User Assets** (STRICTLY EXCLUDED from deletion/cleanup):
   - User media files and directories (scanned folders)
   - Managed library vaults selected by the user
   - External linked folders and mounts
   - Backup snapshots (`berry_snapshot_*.zip`)
   - Any file/directory outside the verified application data root

---

## 3. Credential Store (Keyring) Services

Secrets persisted in OS keychains (Apple Keychain, Windows Credential Manager, Linux Secret Service):

| Secret Description | Config Property Key | Legacy Service (`Berry-AI-Studio`) | Target Service (`Omera`) |
| :--- | :--- | :--- | :--- |
| WebDAV Password | `cloud_backup.webdav_password` | `Berry-AI-Studio` | `Omera` |
| S3 Secret Key | `cloud_backup.s3_secret_key` | `Berry-AI-Studio` | `Omera` |
| S3 Access Key | `cloud_backup.s3_access_key` | `Berry-AI-Studio` | `Omera` |
| Remote Connection URL | `remote_connection_url` | `Berry-AI-Studio` | `Omera` |

*Note:* Accounts are identified by UUID with `keyring:<uuid>` prefix in `config.json`. The migration coordinator must read existing secrets from legacy services, write them to `Omera`, and verify read-back before recording the credential migration as complete in the receipt.

---

## 4. Frontend LocalStorage Preference Key Mappings

| Legacy Key (`berry_*`) | Target Key (`omera_*`) | Type | Default Value | Security Sensitive |
| :--- | :--- | :--- | :--- | :--- |
| `berry_locale` | `omera_locale` | String | `"en"` | No |
| `berry_theme` | `omera_theme` | String | `"dark"` | No |
| `berry_autoscan` | `omera_autoscan` | Boolean | `true` | No |
| `berry_blur_nsfw` | `omera_blur_nsfw` | Boolean | `true` | No |
| `berry_card_badges` | `omera_card_badges` | Boolean | `true` | No |
| `berry_default_view` | `omera_default_view` | String | `"grid"` | No |
| `berry_thumbnail_max_edge` | `omera_thumbnail_max_edge` | Number | `768` | No |
| `berry_thumbnail_cache_budget_mb` | `omera_thumbnail_cache_budget_mb` | Number | `2048` | No |
| `berry_similarity_limit` | `omera_similarity_limit` | Number | `50` | No |
| `berry_auto_check_update` | `omera_auto_check_update` | Boolean | `true` | No |
| `berry_silent_install` | `omera_silent_install` | Boolean | `false` | No |
| `berry_comfyui_url` | `omera_comfyui_url` | String | `"http://127.0.0.1:8188"` | No |
| `berry_webui_url` | `omera_webui_url` | String | `"http://127.0.0.1:7860"` | No |

**Precedence Invariant:**
If a setting exists in the new Omera store, that value wins unconditionally over any legacy value, even if the Omera value is `false`, `0`, or `""`.

---

## 5. Database & Schema Invariants

1. **Current Schema Version:** `12` (`LATEST_VERSION` in `crates/berry-storage/src/migrations.rs`).
2. **Database Filenames:**
   - Active / Legacy source: `berry.db`
   - Migrated target: `omera.db`
3. **Validation Requirements:**
   - `PRAGMA integrity_check` == `"ok"`
   - `PRAGMA foreign_key_check` returns 0 rows
   - `PRAGMA user_version` in `1..=LATEST_VERSION`
4. **WAL Safety:**
   - Never copy a live SQLite `.db` directly with filesystem `copy()`.
   - SQLite Online Backup API (`rusqlite::backup::Backup`) is mandatory to safely capture committed WAL transactions into the staged replica.
   - Staged replica must checkpoint and truncate WAL before atomic publication.
