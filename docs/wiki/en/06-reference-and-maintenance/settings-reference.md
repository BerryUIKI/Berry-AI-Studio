# Comprehensive Settings Reference

Omera's preferences window (`SettingsModal.vue`) is accessed via `File > Preferences...` or `Ctrl + ,` / `Cmd + ,`. All settings are persisted to `config.json` in your local app data directory.

---

## Tab 1: General Preferences

| Setting Field | Key in `config.json` | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **Application Language** | `locale` | `"auto"` | Choose from: `auto` (follows OS language), `en` (English), `zh-CN` (Simplified Chinese), `zh-TW` (Traditional Chinese), `ja` (Japanese), `de` (German), `fr` (French), `es` (Spanish). |
| **Default Gallery View** | `default_view` | `"grid"` | Initial view mode on startup: `"grid"` (Uniform Grid), `"masonry"` (Waterfall), or `"table"` (Detailed List). |
| **Auto-Scan on Startup** | `auto_scan` | `true` | Automatically checks indexed folders for new or modified files when Omera launches. |
| **Startup Scan Interval** | `startup_scan_interval_minutes`| `360` | Minimum cooldown in minutes between full directory reconciliations (`30`, `60`, `360`, `1440`). Prevents disk churn if you restart the app frequently. |
| **Auto-Check for Updates** | `auto_check_update` | `true` | Silently queries GitHub Releases on startup and displays an update badge if a newer version is available. |

---

## Tab 2: Display & Safety

| Setting Field | Key in `config.json` | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **Color Theme** | `theme` | `"system"` | UI color palette: `"system"`, `"midnight"` (OLED dark), `"graphite"` (studio neutral dark), `"violet"` (creative purple), or `"light"`. |
| **Blur Sensitive Content (NSFW)** | `blur_nsfw` | `true` | Applies a CSS blur mask over artworks flagged as adult or sensitive until clicked. |
| **Show Card Badges** | `show_card_badges` | `true` | Overlays format (`PNG`, `MP4`), resolution (`1024×1024`), generator badges, and ratings directly on gallery cards. |
| **Thumbnail Max Edge** | `thumbnail_max_edge` | `384` | Rendered thumbnail edge size (multiples of 64): `256` (Compact), `384` (Standard default), `448` (HD), `512` (Ultra). |
| **Thumbnail Cache Budget** | `thumbnail_cache_budget_mb` | `2048` | Maximum disk space (in megabytes) allocated for cached WebP thumbnails. Oldest accessed tiers are evicted automatically when exceeded. |
| **Clear Cache** | N/A | N/A | Action button to immediately purge all cached thumbnails from disk. |
| **Thumbnail Diagnostics** | N/A | N/A | Action button to open real-time Rayon thread pool and LRU cache metrics. |

---

## Tab 3: Stacks & Bursts

| Setting Field | Key in `config.json` | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **Enable Automatic Stacking** | `auto_stack` | `true` | Automatically clusters sequential generation variations into poker-deck cards. |
| **Prompt Similarity Threshold** | `stack_similarity_threshold` | `0.85` | Minimum tokenized Jaccard similarity score (0.0 to 1.0) required to group images. |
| **Max Time Window (Minutes)** | `stack_time_window_minutes` | `180` | Maximum time gap between consecutive generations to consider them part of the same burst run. |
| **Allow Multiple Open Stacks** | `allow_multiple_open_stacks` | `false` | When `false`, opening an image stack automatically collapses other open stacks. When `true`, multiple stacks can remain expanded simultaneously. |
| **Reset Suppressed Warnings** | N/A | N/A | Restores confirmation dialogs (e.g. stack merge warnings) if you previously checked "Do not show again". |

---

## Tab 4: Generation Interop

| Setting Field | Key in `config.json` | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **ComfyUI Base URL** | `comfyui_url` | `"http://127.0.0.1:8188"` | HTTP endpoint of your local ComfyUI server. Includes connection test button. |
| **SD WebUI Base URL** | `webui_url` | `"http://127.0.0.1:7860"` | HTTP endpoint of your AUTOMATIC1111 / Forge / SD.Next instance. Includes connection test button. |

---

## Tab 5: Team & Collaboration Database

| Setting Field | Key in `config.json` | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **Storage Backend** | `storage_backend` | `"sqlite"` | Active database engine: `"sqlite"`, `"mysql"`, or `"postgres"`. |
| **Remote Connection URL** | `remote_connection_url` | `""` | Database connection string (e.g. `postgres://user:pass@host:5432/omera_studio`). |
| **Workstation Client ID** | `client_identifier` | `""` | Unique machine name used for change logs and optimistic concurrency conflict tracking. |
| **Storage Root Mappings** | `root_mappings` | `{}` | Cross-platform directory mappings linking central Root UUIDs to local operating system mount paths. |
| **Test Connection** | N/A | N/A | Pings remote database server and displays round-trip network latency in milliseconds. |
| **Migration Wizard** | N/A | N/A | Launches the SQLite-to-MySQL/PostgreSQL schema and data exporter. |

---

## Tab 6: Cloud Backup & Media Mirroring

| Setting Field | Key in `config.json` | Default Value | Description |
| :--- | :--- | :--- | :--- |
| **Backup Provider** | `cloud_backup.provider` | `"local_path"` | Storage protocol: `"local_path"`, `"webdav"`, or `"s3"`. |
| **WebDAV URL / Credentials**| `cloud_backup.webdav_*` | `""` | Endpoint URL, username, and password for Nextcloud / Synology WebDAV. |
| **S3 Endpoint / Credentials**| `cloud_backup.s3_*` | `""` | Endpoint, bucket name, region, access key, and secret key for S3 / Cloudflare R2 / MinIO. |
| **Delta Sync Strategy** | `cloud_sync.strategy` | `"fingerprint"` | `"fingerprint"` (file size + ETag) or `"checksum"` (full SHA-256 verification). |
| **Max Transfer Threads** | `cloud_sync.threads` | `4` | Number of concurrent worker threads for cloud media uploads. |
| **Bandwidth Limit** | `cloud_sync.bandwidth_limit_kbs` | `0` | Maximum upload speed in KB/s (0 = unlimited). |

---

## Tab 7: Metadata Engines Overview

Displays real-time diagnostic status of built-in lossless metadata parsers:
- AUTOMATIC1111 / SD.Next PNG `parameters` chunk parser (Active 🟢)
- ComfyUI node execution graph and `workflow` JSON parser (Active 🟢)
- NovelAI `Comment` and `Description` parser (Active 🟢)
- Fooocus / Fooocus-MRE prompt block parser (Active 🟢)
- InvokeAI `sd-metadata` and `invokeai_metadata` parser (Active 🟢)
- MP4 ISOBMFF and WebM EBML video stream parser (Active 🟢)

---

## Tab 8: Storage & About

- **Version**: Displays the active application version (e.g. `v0.3.0`).
- **Database Schema**: Reports active SQLite schema migration level (e.g. `Schema Version 14`).
- **Active Database Location**: Absolute path to `omera.db`.
- **Quick-Open Folder Buttons**:
  - `Open Config`: Opens folder containing `config.json`.
  - `Open Database`: Opens directory containing `omera.db` and WAL journals.
  - `Open Thumbnails`: Opens the WebP thumbnail cache directory.
  - `Open Models`: Opens the ONNX AI weights directory.
