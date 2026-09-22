# Workspace & UI Anatomy

Berry AI Studio features an **Eagle / Lightroom** inspired 3-pane desktop workspace engineered for high-density visual curation, keyboard speed, and distraction-free viewing.

---

## 1. Window Anatomy & Desktop Shell

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Logo] Berry AI Studio   [ File  Edit  View  Tools  Help ]              [ _ ] [ □ ] [ ✕ ] │  <- TitleBar & MenuBar
├──────────────┬──────────────────────────────────────────────────────────┬──────────────┤
│              │ [🔍 Search: prompt, model, rating... ] [🧠] [☰ Filter]    │              │
│  NAVIGATION  ├──────────────────────────────────────────────────────────┤   PROPERTY   │
│   SIDEBAR    │                                                          │  INSPECTOR   │
│              │                  CENTER GALLERY CANVAS                   │              │
│  - All Items │                                                          │  - Preview   │
│  - Favorites │  [ Card ]  [ Card ]  [ Poker Stack (4) ]  [ Card ]       │  - Prompts   │
│  - Folders   │                                                          │  - Model/CFG │
│  - Albums    │  [ Card ]  [ Card ]  [ Card ]             [ Card ]       │  - LoRA tags │
│  - Tags      │                                                          │  - Interop   │
│              │                                                          │              │
│  [QuickTools]│                                                          │              │
├──────────────┴──────────────────────────────────────────────────────────┴──────────────┤
│ 1,248 / 8,920 items | 3 Selected | Schema v14 | Ready | [⚡ Activity]                  │  <- StatusBar
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Title Bar & Native Application Menus

The window uses a frameless design with custom native-like title bar (`TitleBar.vue`) and desktop menu system (`MenuBar.vue`):

### Top Menu Reference
- **File**:
  - `Add Folder...` (`Ctrl + O` / `Cmd + O`): Connect a new local or network directory.
  - `Scan Current Folder`: Re-index the active folder for modified or new files.
  - `Rescan All Folders`: Force an incremental scan across all registered folders.
  - `Database Management...`: Open database compaction, size statistics, and backup tools.
  - `Preferences...` (`Ctrl + ,` / `Cmd + ,`): Open the central 8-tab settings window.
  - `Exit` (`Alt + F4`): Gracefully close the studio.
- **Edit**:
  - `Select All` (`Ctrl + A` / `Cmd + A`): Select all assets in the active gallery view.
  - `Clear Selection` (`Esc`): Deselect all highlighted items.
  - `Batch Tag...`: Attach or remove tags from selected files.
  - `Batch Add to Album...`: Assign selected files to an album.
  - `Batch Move...` / `Batch Copy...`: Physically transfer files to another indexed folder.
  - `Batch Export...` (`Ctrl + E` / `Cmd + E`): Open export, transcoding, and packaging window.
  - `Move to Trash` (`Delete` / `Backspace`): Safely move selection to the OS Recycle Bin.
- **View**:
  - `Grid View`: Switch to uniform fixed-height responsive cards.
  - `Waterfall (Masonry) View`: Switch to uncropped aspect-ratio preserving columns.
  - `Table View`: Switch to compact tabular spreadsheet mode.
  - `Toggle Sidebar` (`B`): Show/hide the left navigation panel.
  - `Toggle Inspector` (`I`): Show/hide the right metadata panel.
  - `Quick Look (Lightbox)` (`Space` / `Enter`): Open fullscreen preview modal.
  - `Zoom In` (`Ctrl + =`), `Zoom Out` (`Ctrl + -`), `Reset Zoom` (`Ctrl + 0`).
- **Tools**:
  - `Prompt Keyword Insights`: Visual frequency distributions and ratings correlation.
  - `Model Manager & Cache`: Resolve checkpoint hashes to friendly names.
  - `CLIP Semantic Search Index`: Index library embeddings for natural-language search.
  - `LoRA Trigger Library`: Manage trigger words and `.civitai.info` sidecars.
  - `Auto-Tag with WD14 AI`: Open local Danbooru anime/aesthetic tagger.
- **Help**:
  - `Language`: Instantly switch between 7 supported languages.
  - `Keyboard Shortcuts` (`?`): Display in-app shortcut cheatsheet.
  - `Check for Updates...`: Query GitHub Releases for updates.
  - `About Berry AI Studio`: Display version, author, and license information.

---

## 3. Left Navigation Sidebar (`Sidebar.vue`)

The left sidebar (`B` to toggle) provides quick access to your asset collections:

1. **System Libraries**:
   - **All Images**: Comprehensive library view across all connected folders with total count badge.
   - **Favorites (★)**: Quick filter for all starred/bookmarked images.
   - **Sensitive (18+) (🔞)**: Fast access to NSFW-flagged content (rendered with blur overlay by default).
2. **Folders Section**:
   - Displays all registered folders with type badges:
     - `⚡`: Ingestion Pipeline (active generator surveillance).
     - `📦`: Managed Vault (internal organized storage).
     - `📁`: External Link (zero-copy in-place index).
   - Right-click or hover to trigger actions: **Harvest Pipeline**, **Rescan Folder**, **Rebuild Metadata**, or **Remove Folder**.
   - Acts as a drop target: drag cards from the gallery directly onto a folder to move or copy them.
3. **Albums Section**:
   - User-defined collections. Drag and drop cards onto albums to add them.
   - Includes a "+ New Album" button.
4. **Tags Section**:
   - Color-coded taxonomy chips (8 distinct color badges).
   - Drag items from gallery onto tag chips to apply them in bulk.
5. **Quick Tools Footer**:
   - Shortcut buttons to open Insights, Models, Database Maintenance, and Shortcuts Help modals.

---

## 4. Center Gallery Canvas & Search Toolbar

The central workspace is where you browse, select, and curate assets:

- **Search Bar (`SearchBar.vue`)**:
  - **Syntax Search Mode (`🔍`)**: Query metadata with keywords or structured syntax (e.g., `prompt:"cyberpunk" cfg:>7`).
  - **AI Semantic Search Mode (`🧠`)**: Natural language text-to-image queries powered by local CLIP/SigLIP models.
- **View Mode Switcher**:
  - Toggle between **Grid (⊞)**, **Waterfall (▤)**, and **Table (☰)** layouts.
  - **Zoom Slider**: Dynamically adjusts card minimum width between **130px** and **360px**. The column count adjusts responsively without stretching images.
- **Filter Drawer Toggle (`FilterDrawer.vue`)**:
  - Slide out the multi-criteria filter panel to filter by model checkpoint, sampler, aspect ratio, star ratings, and video properties.

---

## 5. Right Property Inspector (`InspectorPane.vue`)

The right inspector (`I` to toggle) reveals deep lossless generation metadata for the active selection:

- **Media Preview**: High-res thumbnail with NSFW unblur toggle and Quick Look trigger.
- **Curation Controls**: Star rating (0–5 stars), Favorite toggle (`F`), and NSFW flag.
- **Positive Prompt**: Tokenized prompt view. Each prompt token is rendered as an interactive chip:
  - Click any token to search for it across your library.
  - Click the copy button to copy the entire positive prompt.
- **Negative Prompt**: Full negative prompt text with 1-click copy button.
- **Technical Generation Parameters**:
  - Model Name & Checkpoint SHA256 Hash.
  - Sampler & Scheduler algorithms.
  - Step count, CFG Guidance Scale, Seed (with copy button).
  - Native generation dimensions (`Width × Height`).
- **Detected LoRAs**:
  - Lists all `<lora:name:weight>` or ComfyUI LoRA loader nodes detected in the file.
  - Shows trigger words, weight values, and 1-click copy for the formatted prompt tag.
- **Generation Interop Card**:
  - Displays live connectivity to local **ComfyUI** and **AUTOMATIC1111**.
  - One-click "Send to ComfyUI" (re-queues workflow graph) or "Send to SD WebUI".
- **Raw Metadata Accordion**: View the entire unformatted JSON parameter block or ComfyUI node graph.

---

## 6. Bottom Status Bar & Activity Popover

Located at the bottom of the window:
- **Filtered / Total Counters**: Displays visible matching items versus total library size (e.g., `Filtered: 420 / 12,500 items`).
- **Selection Count**: Displays count when items are selected (`Selected: 5 items`).
- **Database Status**: Displays active database file name and schema version.
- **Activity Popover (`⚡ Activity`)**:
  - Opens a real-time monitor showing filesystem watcher status (active roots, journal backlog, error health) and background thumbnail decode queues.

---

## 7. Floating Batch Action Bar (`BatchActionBar.vue`)

When one or more cards are selected, a floating toolbar appears at the bottom-center of the canvas:

- Multi-selection count indicator (`X of Y selected`) with Select All / Deselect buttons.
- Batch Star Rating dropdown (0 to 10 stars).
- "Add to Album" and "Add Tag" modal triggers.
- "Auto-Tag (WD14)" trigger.
- "Favorite" toggle.
- "Copy Paths" and "Copy Prompts" to clipboard.
- "Move" and "Copy" destination dialogs.
- "Cull Drafts" (automatically active when stacks are selected).
- "Export..." (batch transcode, privacy strip, HTML showcase).
- "Trash" (moves selected items to OS Recycle Bin).
