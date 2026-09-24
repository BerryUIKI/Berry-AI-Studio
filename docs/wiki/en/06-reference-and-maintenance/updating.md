# Updates & Application Lifecycle

Omera features a silent, in-place auto-updater designed to deliver enhancements and bug fixes without interrupting your workflow or risking your data.

---

## 1. Checking for Updates

### Automatic Check on Startup:
By default, Omera queries the GitHub Releases API when the application launches:
- If a newer version is available, an update indicator badge appears in the **Help** menu.
- You can enable or disable this behavior in **Settings > General > Auto-Check for Updates on Startup**.

### Manual Check:
You can check for updates manually at any time:
- Click **Help > Check for Updates...** from the top menu bar.

---

## 2. In-Place Update Modal (`UpdateModal.vue`)

When an update is detected, the **Update Window** appears:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Update Available: v0.3.1                                           [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ A new release of Omera is available (Current: v0.3.0).       │
│                                                                        │
│ What's New:                                                            │
│ • Optimized ComfyUI video metadata parser for HunyuanVideo.            │
│ • Improved keyset deep pagination query latency on 100k+ libraries.    │
│ • Added 10-star rating selector to the floating batch toolbar.         │
├────────────────────────────────────────────────────────────────────────┤
│ Download Progress:                                                     │
│ [██████████████████████████░░░░░░░░░░] 68% (12.4 MB / 18.2 MB · 4 MB/s)│
├────────────────────────────────────────────────────────────────────────┤
│ [ Cancel ]                                 [ Download & Install Now ]  │
└────────────────────────────────────────────────────────────────────────┘
```

### Installation Flow:
1. Click **"Download & Install Now"**.
2. Omera streams the platform-specific release package directly from GitHub Releases into a temporary update staging folder.
3. Once the download finishes, Omera prompts you to restart the application.
4. The native installer executes silently in-place and re-launches the application.

---

## 3. Data Preservation Guarantees

Upgrading Omera **never touches your user data**:

- **Database Safety**: Your `omera.db` database, custom albums, color tags, ratings, and burst stack relationships are stored in your operating system's user AppData directory (`%APPDATA%`, `~/Library/Application Support`, or `~/.config`), completely separate from the application binaries.
- **Append-Only Schema Migrations**: When a new version of Omera includes database changes, Omera's Rust backend runs **append-only schema migrations** on startup via `PRAGMA user_version`. Migrations update tables incrementally without rewriting or deleting existing records.
- **Persistent Settings**: Your `config.json` preferences, thumbnail cache budget, and folder paths are preserved across all updates.
