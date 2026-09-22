# Troubleshooting & Frequently Asked Questions (FAQ)

This guide covers common questions, edge cases, and troubleshooting steps for Berry AI Studio.

---

## 1. Troubleshooting Common Issues

### Issue: New images from my AI generator are not showing up in the gallery
- **Cause**: The folder may not be configured as an active Ingestion Pipeline, or the filesystem watcher quiet period is waiting for rendering to complete.
- **Solution**:
  1. Check that the folder is registered in the left sidebar.
  2. If using AUTOMATIC1111 or ComfyUI, ensure the folder mode is set to **Mode C: Pipeline** so write-lock debouncing is active.
  3. Right-click the folder in the sidebar and click **"Harvest Pipeline Outputs"** or **"Rescan Folder"** to trigger an immediate check.

### Issue: Thumbnails are loading slowly or showing placeholder boxes
- **Cause**: On large initial imports, background Rayon threads downscale images in batches.
- **Solution**:
  1. Click **⚡ Activity** in the bottom status bar to check if the thumbnail worker pool is still indexing.
  2. In **Settings > Display & Safety**, ensure your **Thumbnail Cache Budget** is set to at least `2048 MB`.
  3. Avoid running heavy video renders or gaming simultaneously during the first large folder import.

### Issue: Search returns 0 results even though I know the prompt contains the word
- **Cause**: You may be in **Semantic Mode (`🧠`)** instead of **Syntax Mode (`🔍`)**, or filtering on an unindexed field.
- **Solution**:
  1. Check the icon in the search bar: click the brain icon to toggle back to **Syntax Search (`🔍`)**.
  2. If searching for exact phrases with spaces, wrap your query in quotes: `prompt:"cyberpunk city"`.
  3. Open the **Filter Drawer (`☰ Filter`)** and click **"Reset All"** to ensure no active filters (such as a 5-star rating filter) are hiding matching results.

### Issue: Stacks appear separated or duplicate cards are visible
- **Cause**: Stacks can become fragmented if member images were renamed or moved externally via your OS file explorer.
- **Solution**: Select the affected cards in the gallery and press `Ctrl + G` to re-group them cleanly into a single unified stack.

### Issue: ComfyUI "Send to ComfyUI" reports connection refused
- **Cause**: ComfyUI is not running locally, or is listening on a different port.
- **Solution**:
  1. Open **Settings > Generation Interop**.
  2. Verify that the **ComfyUI Base URL** matches your terminal output (default: `http://127.0.0.1:8188`).
  3. Click **"Test Connection"** to verify the port is open and reachable.

---

## 2. Frequently Asked Questions (FAQ)

### Is Berry AI Studio completely free to use?
Yes. Berry AI Studio is free, open-source software licensed under **AGPL-3.0**. There are no subscriptions, paywalls, or locked features.

### Can Berry AI Studio handle libraries with 100,000+ or 500,000+ files?
Yes. Berry was engineered from the ground up to support massive collections. It uses:
- **Keyset cursor deep pagination** (`search_files_cursor_page`) that maintains sub-millisecond query performance regardless of library size.
- **SQLite Write-Ahead Logging (WAL)** for high-throughput non-blocking reads.
- **Dynamic DOM virtualization** that only renders elements visible in your viewport.

### Does Berry AI Studio upload my prompts or images to the cloud?
No. All scanning, metadata extraction, database storage, and AI inference (CLIP and WD14) run 100% locally on your machine. No telemetry or analytics are collected.

### Can I drag and drop images from Berry directly into ComfyUI or Discord?
Yes. Dragging an image card from the gallery directly into your web browser or external desktop application transmits standard OS file drop payloads, preserving embedded metadata chunks.

### What happens if I delete a folder from the left sidebar?
Removing a folder from Berry AI Studio removes the folder and its index records from Berry's database. **It never deletes or moves your physical media files on disk.**

### Can multiple team members collaborate on the same library?
Yes. By switching from SQLite to a shared **MySQL 8.0+** or **PostgreSQL 14+** database in **Settings > Team & Collaboration**, multiple artists can connect to a shared network library with real-time change synchronization and cross-platform path mapping.
