# Gallery Modes & Display Options

Omera provides four dedicated gallery presentation modes designed to accommodate different curation workflows, ranging from rapid visual triage to detailed technical inspections.

---

## 1. Gallery View Modes

Use the top toolbar buttons or menu shortcuts (`View`) to toggle between view modes:

### 1. Uniform Grid View (`grid` — ⊞)
- **Concept**: Fixed-height, uniform aspect ratio cards arranged in responsive columns.
- **Responsive Architecture**: Card width stays stable when the window is resized. Omera dynamically adjusts the number of columns (`calculateGalleryColumns`) instead of stretching or squishing images.
- **Zoom Slider**: Drag the zoom slider or use `Ctrl + =` / `Ctrl + -` to scale card minimum width smoothly from **130 px** (overview thumbnail mode) to **360 px** (large detail mode).
- **Virtualization**: Only elements currently within the viewport (plus a small look-ahead overscan buffer) are rendered into the DOM. Scrolling through a 100,000-image library consumes no more memory than scrolling through 100 images.

### 2. Masonry Waterfall View (`masonry` — ▤)
- **Concept**: Fluid multi-column layout that preserves each image's native aspect ratio.
- **Uncropped Artwork**: Ideal for collections with mixed landscape wallpapers, tall portrait character concepts (9:16), and panoramic environments. Images are scaled cleanly using `object-fit: contain` without cropping edges.
- **Shortest-Column Distribution**: New items are placed in the column with the least vertical height, maintaining a balanced, visually appealing wall of art.

### 3. High-Density Table View (`table` — ☰)
- **Concept**: Virtualized spreadsheet layout with fixed 46px row height.
- **Information Density**: Displays micro-thumbnails (36px tier) alongside extensive tabular columns:
  - Checkbox selection state
  - Filename and relative directory path
  - Media container format (`PNG`, `WEBP`, `MP4`)
  - Dimensions (`Width × Height`)
  - File size (formatted KB/MB)
  - Date modified timestamp
  - Star rating (0–5 / 1–10)
  - Aesthetic prediction score

### 4. Visual Similarity Match View
- **Activation**: Click **"Find Similar"** in the right inspector or context menu on any image.
- **Top Match Banner**: Displays the source reference image, similarity threshold slider (**0% to 95%** in 5% increments), and a count limit selector (**20, 50, 100, 200** items).
- **Match Percentage Badge**: Every matched card displays a colored match score badge (e.g. `94% Match`) calculated from CLIP vector cosine distance.
- **Dismiss**: Press `Esc` or click the banner's close button (`✕`) to return to your normal gallery view.

---

## 2. Card Badges & Visual Overlays

In **Settings > Display & Safety**, you can toggle **Show Card Badges** on or off. When enabled, cards display clean informational chips:

- **Format / Container Badge**: Indicates `.png`, `.webp`, `.jpg`, `.mp4`, or `.webm`.
- **Dimensions Badge**: Native pixel resolution (e.g. `1024×1024` or `832×1216`).
- **Video Duration & FPS**: For animations and videos, shows playtime (e.g. `00:04`) and framerate (`24 fps`).
- **Generator Badge**: Identifies generator engine signatures (e.g. `WebUI`, `ComfyUI`, `NovelAI`, `Fooocus`).
- **Star Rating Overlay**: Displays active star rating (★ 1–5).
- **Favorite Flag**: Gold star icon in the upper-right corner for bookmarked images.

---

## 3. Sensitive Content (NSFW) Privacy Blur

To protect privacy during presentations or working in public spaces, Omera includes built-in content protection:

- **Automatic Blur (`blur_nsfw` setting)**: Any asset tagged as `is_nsfw` or identified with an adult content rating is masked with an aggressive CSS blur overlay.
- **Click-to-Reveal**: Clicking the eye icon (`👁`) or the card unblurs that specific item temporarily for review.
- **Global Sensitive Section**: The left sidebar includes a dedicated **Sensitive (18+) (🔞)** library filter to audit or recategorize flagged content in one place.
