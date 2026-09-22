# Batch Export, Transcoding & Web Showcase

Berry AI Studio includes a high-throughput export and packaging engine (`ExportModal.vue`) powered by **Rayon** multi-threading. It supports format transcoding, privacy metadata stripping, filename templating, and zero-dependency HTML showcase generation.

---

## 1. Launching Batch Export

To open the export modal:
- Select one or more images or stacks in the gallery.
- Click **"Export..."** on the floating Batch Action Bar, or press `Ctrl + E` / `Cmd + E` (or select `Edit > Batch Export...`).

---

## 2. Format Transcoding & Compression

Berry converts and re-encodes images in parallel across all CPU cores:

| Target Format | Options & Settings | Best Use Case |
| :--- | :--- | :--- |
| **Original Format** | Preserves exact source bytes and containers. | Lossless archive or backup. |
| **WebP** | Adjustable quality slider (1–100%, default 85%). Highly compact. | Web distribution, Discord, portfolio websites. |
| **JPEG** | Standard progressive JPEG encoding (quality 1–100%). | Universal compatibility with older photo viewers. |
| **PNG** | Pure lossless compression. | Studio handoffs, high-end printing. |

### Downscaling Constraints:
You can constrain maximum output dimensions to prevent accidental distribution of huge 4K/8K uncompressed images:
- **Original** (no downscaling)
- **4K UHD** (max edge: 3840 px)
- **2K QHD** (max edge: 2048 px)
- **Full HD** (max edge: 1080 px)
- **Custom Bounding Edge** (user-defined max pixel limit)

---

## 3. 4-Tier Privacy Metadata Sanitization

Many creators want to share artworks online while keeping proprietary prompts, negative embeddings, or seeds private. Berry provides **four discrete privacy tiers**:

1. **Keep All**:
   - Preserves all embedded metadata chunks (ComfyUI workflow graphs, A1111 parameters, NovelAI comments, and EXIF camera data).
2. **Strip Prompt Only**:
   - Removes positive and negative prompt text strings, but keeps technical parameters (sampler, steps, CFG scale, seed, model name).
3. **Strip All AI Metadata**:
   - Completely removes all ComfyUI node graphs, A1111 parameter blocks, LoRA tags, and generator signatures.
4. **Full Clean (Pixel-Only Sanitization)**:
   - Strips everything, including EXIF headers, ICC color profiles, and software signatures. The exported file contains raw pixel data only.

---

## 4. Filename Pattern Templating & Sidecars

Customize output filenames using dynamic template variables with real-time preview:

### Supported Tokens:
- `{name}`: Original filename without extension.
- `{id}`: Unique database ID or UUID.
- `{index}`: Sequential output index (001, 002, 003...).
- `{date}`: Creation date (`YYYY-MM-DD`).
- `{rating}`: Star rating (e.g. `5star`).
- `{model}`: Checkpoint model name.
- `{seed}`: Generation seed value.

*Example Pattern*: `{date}_{model}_{seed}_{name}` → `2026-09-22_animagine_xl_2849104812_cyberpunk_01.webp`

### Metadata Sidecars:
You can automatically generate companion files alongside each exported image:
- **None**: No sidecar files.
- **Text Prompt (`.txt`)**: Exports the positive generation prompt as a companion text file.
- **Full JSON (`.json`)**: Exports complete structured metadata and ComfyUI workflow graphs.

---

## 5. Standalone Interactive HTML Showcase Generator

Berry can package your exported assets into a **self-contained, single-file HTML portfolio** (`index.html`):

- **Zero Dependencies**: Requires no web server, Node.js, or external JavaScript libraries. Double-click to open in any web browser.
- **Features Included**:
  - Dark studio aesthetic matching Berry AI Studio.
  - Responsive gallery grid with lazy-loaded thumbnails.
  - Fullscreen Lightbox viewer with mouse-wheel zoom and pan.
  - Collapsible prompt inspector showing generation parameters.
  - Instant client-side keyword search bar.
- **Packaging Options**: Export directly into a target folder, or bundle everything into a single `.zip` archive for client handoffs or website hosting.
