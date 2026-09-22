# 📦 Multi-Mode Folders, AIGC Ingestion Pipelines & Image Stacking Specification

This document defines the architectural specification, data schemas, interaction flows, and background processing models for **Multi-Mode Folders**, **AIGC Ingestion Pipelines**, and **Image Stacking** in Omera.

---

## 1. Multi-Mode Folder Architecture

Omera organizes local assets through three distinct folder operating modes while preserving existing core library abstractions (All Images, Favorites, Sensitive NSFW 18+, Smart Albums, and Color Tags).

```
▼ 📁 Folders (Multi-Mode)
   ├─ ▼ ⚡ Ingest Pipelines (Mode C)
   │     ├─ 🟢 WebUI Outputs (Active Watch · Grace Period: 24h)
   │     └─ 🟢 ComfyUI Outputs (Active Watch · Grace Period: 24h)
   ├─ ▼ 📂 Managed Project Vaults (Mode B)
   │     ├─ 🎨 Concept Art Project 2026
   │     └─ 🤖 Character Design Batch
   └─ ▼ 🔗 Linked External Folders (Mode A)
         └─ 📁 D:\My_Personal_Drawings (In-place Read-only)
```

### 1.1 Mode A: External Linked Folder (`link`)
- **Behavior**: In-place zero-copy indexing.
- **File Movement**: None. Files stay permanently in their original external directory.
- **Use Case**: Existing organized personal libraries, external hard drives, or network-attached storage (NAS).
- **Deletion**: Trashing an image from a linked folder triggers standard trash/recycle bin operations on the external file.

### 1.2 Mode B: Managed Project Vault (`managed`)
- **Behavior**: Managed storage repository inside the Berry library directory or user-specified vault.
- **Ingestion**: Supports dragging external files/folders directly into the folder in the UI:
  - **Copy Ingest** (`copy`): Copies files into vault, original files untouched.
  - **Move Ingest** (`move`): Moves files into vault, freeing external disk space.
- **Physical Layout**: Vaults use date-partitioned structures (`YYYY/MM/UUID_filename.ext`) to avoid filename collisions.

### 1.3 Mode C: AIGC Ingestion Pipeline (`pipeline`)
- **Behavior**: Active surveillance of AI generator output directories (e.g. Stable Diffusion WebUI `outputs/txt2img-images`, ComfyUI `output/`, Fooocus `outputs/`).
- **Harvesting Strategy**:
  - **Auto Real-Time**: Watches via filesystem events with write-lock debounce.
  - **Manual Harvest**: One-click "Harvest Now" batch ingestion.
- **Grace Period & Delayed Cleanup**:
  - Optional delayed cleanup of original source files to avoid WebUI UI broken images or ComfyUI reload conflicts.
  - Configurable retention: `immediate`, `1 hour`, `24 hours (default)`, `3 days`, `7 days`, or `never (copy-only)`.
  - Safety Guarantee: Cleanup moves files to the OS Recycle Bin / Trash rather than permanently deleting them.

---

## 2. Ingestion Pipeline Mechanics & Safety

### 2.1 Write-Lock Detection & Debounce
Generative AI tools frequently write large PNG files containing multi-megabyte ComfyUI workflow graphs or high-resolution images over several hundred milliseconds.

```
[FS Event: Create/Modify]
       │
       ▼
[Debounce Timer (500ms)] ──(File Size Still Growing?)──► [Wait another 500ms]
       │
       ▼ (Size Stable & File Read Lock Available)
[Extract Metadata & Compute Fingerprint]
       │
       ▼
[Atomic Ingest to Berry Vault (Copy First)]
       │
       ▼ (Verified Ingest Integrity: Size & Hash match)
[Schedule Delayed Cleanup (if Move mode enabled)]
```

### 2.2 Local Generator Output Autodetection
The system provides heuristic inspection of standard local AI installation directories on Windows, macOS, and Linux:
- **AUTOMATIC1111 / SD.Next**: `<drive>/stable-diffusion-webui/outputs/txt2img-images`, `.../img2img-images`
- **ComfyUI / ComfyUI Portable**: `<drive>/ComfyUI/output`, `<drive>/ComfyUI_windows_portable/ComfyUI/output`
- **Aki / NovelAI local packages**: `<drive>/sd-webui-aki/outputs`, `<drive>/ComfyUI-aki/ComfyUI/output`
- **Fooocus**: `<drive>/Fooocus/outputs`

---

## 3. Image Stacking (Burst & Prompt Similarity Grouping)

Image Stacking groups similar images (such as multi-seed batch generation, seed variations, or continuous prompt variations) into a single visual stack in the virtual grid.

### 3.1 Grouping Criteria & Thresholds
- **Auto-Stacking Engine**:
  - **Prompt Equality**: Exact match on normalized positive prompt, negative prompt, and checkpoint model.
  - **Prompt Similarity**: Configurable tokenized Jaccard similarity threshold (e.g. `80% ~ 100%`, default `90%`).
  - **Time Window**: Batch clustering constraint ensuring images generated within a time window (e.g. 30 minutes) are grouped together, preventing false clustering with historical outputs from months prior.
- **Manual Stacking**:
  - Select multiple images in the gallery and press **`Ctrl+G`** (`Cmd+G`) to stack them immediately.
  - Stacks cannot be nested. If the selection contains stacks, Berry warns before
    flattening every source stack and selected standalone image into the first selected
    stack. The target retains its cover and existing member order.
  - Select a stack and press **`Ctrl+Shift+G`** (`Cmd+Shift+G`) to dissolve the stack back into individual items.

### 3.2 Stack Navigation & Hero Selection
- **Poker Deck Display**:
  - Collapsed state displays the **Hero Cover image** above two offset card surfaces with a compact stack icon and numeric count.
  - A single click on the collapsed card or its count badge toggles **Inline Expansion**, presenting all stack members sequentially.
- **Hero Cover Selection**:
  - Select any member inside a stack and press **`Alt+S`** or choose **"Set as Stack Cover"** from the context menu to designate it as the primary cover image.
- **Side-by-Side Compare Mode (`C`)**:
  - Launch a split-screen 1-to-1 comparison viewer with synchronized zoom and pan to rapidly evaluate faces, details, and seed candidates.
- **Batch Cull Tool**:
  - One-click action: "Keep Hero & 3★+ images, move remaining stack drafts to Trash".

---

## 4. Database Schema Evolution (Schema v9)

```sql
-- Schema Migration v9: Multi-Mode Folders & Image Stacking

-- Add mode and pipeline configuration to folders
ALTER TABLE folders ADD COLUMN folder_type TEXT NOT NULL DEFAULT 'link'; -- 'link', 'managed', 'pipeline'
ALTER TABLE folders ADD COLUMN source_path TEXT;                         -- For pipeline: source output path to watch
ALTER TABLE folders ADD COLUMN ingest_action TEXT DEFAULT 'copy';        -- 'copy' or 'move'
ALTER TABLE folders ADD COLUMN grace_period_hours INTEGER DEFAULT 24;    -- Delayed deletion window
ALTER TABLE folders ADD COLUMN auto_harvest INTEGER DEFAULT 1;          -- 1 for active watch, 0 for manual

-- Image Stacking columns on images table
ALTER TABLE images ADD COLUMN stack_id TEXT;                             -- UUID shared by all items in the stack
ALTER TABLE images ADD COLUMN stack_order INTEGER DEFAULT 0;             -- 0 = Hero cover, 1..N = member order

-- Indexes for lightning-fast queries
CREATE INDEX IF NOT EXISTS idx_images_stack_id ON images(stack_id);
CREATE INDEX IF NOT EXISTS idx_folders_type ON folders(folder_type);

-- Table for delayed deletion queue
CREATE TABLE IF NOT EXISTS pipeline_cleanup_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_file_path TEXT NOT NULL UNIQUE,
    target_image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    scheduled_delete_at INTEGER NOT NULL, -- Unix epoch timestamp
    created_at INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' -- 'pending', 'deleted', 'cancelled', 'failed'
);
CREATE INDEX IF NOT EXISTS idx_cleanup_schedule ON pipeline_cleanup_queue(scheduled_delete_at, status);
```

---

## 5. UI/UX Workflow Specifications

### 5.1 Add Folder Wizard (`AddFolderModal.vue`)
Triggered via `Ctrl+O` or the `+` button in the sidebar:
1. **Mode Selection Step**:
   - Card 1: 🔗 External Link (In-Place Reference).
   - Card 2: 📂 Managed Project Vault (Berry Repository).
   - Card 3: ⚡ AIGC Ingestion Pipeline (Watch & Harvest).
2. **Path Configuration Step**:
   - For Pipeline: "🔍 Autodetect Local AI Tools" button + manual path browser.
   - For Ingest Action: Copy vs. Move with Grace Period selector (`Immediately`, `1 hour`, `24 hours (Recommended)`, `3 days`, `7 days`).

### 5.2 First-Run Onboarding Wizard (`OnboardingModal.vue`)
Displayed once upon initial software launch:
1. **Welcome & Language**: Select UI language and dark theme preview.
2. **AI Tool Autodetection**: Automatically scans drives for WebUI, ComfyUI, and Fooocus installations; presents checkable list of detected output paths.
3. **Pipeline Ingest Preferences**: Select default harvest action and safe delayed cleanup grace period.
4. **Complete**: Instant transition into the main studio workspace.
