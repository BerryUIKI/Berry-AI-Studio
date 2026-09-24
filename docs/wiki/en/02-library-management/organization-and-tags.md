# Organization, Ratings & Tags

Omera provides rich curation mechanisms designed to sort, categorize, and prioritize tens of thousands of artworks quickly without cluttering your filesystem.

---

## 1. Star Ratings & Scoring System

Omera uses a dual-precision rating model stored directly in SQLite (`files.rating` and `files.aesthetic_score`):

### Star Ratings (0 to 5 Stars or 1 to 10 Scale)
- **Keyboard Shortcuts**: Select one or more images and press:
  - `1` through `5`: Assign that star rating immediately.
  - `0`: Clear star rating.
- **Inspector / Batch Bar**: Supports the extended 10-star rating scale (e.g. 8/10 or 9/10) via dropdowns.
- **Database Indexing**: The `rating` column is indexed with SQLite B-trees, enabling instant queries like `rating:>=4` or `rating:5` on 500,000-item libraries.

### Favorites (★ Bookmarking)
- **Toggle**: Press `F` or click the star icon in the Inspector.
- **Visuals**: A gold star badge appears in the top-right corner of the card.
- **Sidebar Access**: All favorited items are instantly accessible in the left navigation sidebar under **Favorites (★)**.

---

## 2. Curated Albums (`AlbumModal.vue`)

Albums provide a way to group related artworks across different folders without moving physical files.

### Creating & Managing Albums
1. In the left sidebar under **Albums**, click **"+ New Album"**.
2. Enter an album name (e.g., `"Cyberpunk Characters"`, `"2026 Portfolio"`) and an optional description.
3. Rename or delete albums at any time by right-clicking them in the sidebar. Deleting an album removes the grouping associations but never deletes your physical files.

### Adding Assets to Albums
- **Drag & Drop**: Select one or more cards from the gallery and drag them directly onto an album in the left sidebar.
- **Batch Action Bar**: Click **"Add to Album"** in the floating batch toolbar.
- **Right Inspector**: Search and attach albums directly from the Inspector pane.

---

## 3. Color-Coded Tag Taxonomy (`TagModal.vue`)

Tags provide granular categorization and visual triage:

### Preset Color Badges
Omera includes 8 distinct visual color presets:
- 🔴 Red
- 🟠 Orange
- 🟡 Yellow
- 🟢 Green
- 🔵 Blue
- 🟣 Purple
- 🌸 Pink
- ⚪ Slate / Gray

### Working with Tags
- **Creating Tags**: Click **"+ New Tag"** in the sidebar, pick a color preset, and enter a name (e.g. `Hero Character`, `Draft`, `Client Approved`).
- **Drag to Tag**: Drag selected cards from the canvas directly onto a tag chip in the left sidebar to attach it.
- **Filtering by Tag**: Click any tag in the sidebar to filter the gallery.
- **Multi-Tagging**: You can assign an unlimited number of tags to each artwork.

---

## 4. Floating Batch Action Bar (`BatchActionBar.vue`)

Whenever you select multiple items (via `Ctrl+Click`, `Shift+Click`, or `Ctrl+A`), the **Batch Action Bar** slides in at the bottom of the screen:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  [✓ 14 of 120 selected]  [Select All]  [Deselect]                                      │
│  [★ Rate ▾]  [🏷 Tag]  [📁 Album]  [🧠 Auto-Tag]  [★ Fav]  [🔞 NSFW]                    │
│  [📋 Copy Paths]  [📋 Copy Prompts]  [⇄ Move]  [⧉ Copy]  [🧹 Cull Drafts]  [📤 Export] │
│  [🗑 Trash]                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Key Batch Operations:
- **Batch Rate**: Assign a uniform star rating across all selected images.
- **Batch Tag / Album**: Open the modal to apply or remove tags and albums in bulk.
- **Copy Prompts**: Copies the positive prompts of all selected images to your clipboard, formatted with clean `---` dividers.
- **Copy Paths**: Copies full absolute filesystem paths (one per line) for pasting into command lines or scripts.
- **Move & Copy**: Move or copy physical files into another folder indexed in Omera.
- **Cull Drafts**: Active when selected cards contain burst stacks (see [Stacks & Bursts](../04-intelligent-curation/stacks-and-bursts.md)).
- **Trash**: Safely moves all selected files to your OS Recycle Bin/Trash.
