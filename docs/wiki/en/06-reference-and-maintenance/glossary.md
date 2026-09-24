# Product Glossary

This glossary defines key domain terminology, architecture concepts, and features used throughout **Omera**.

---

## A
- **AIGC (AI-Generated Content)**: Digital media (images, animations, audio, and video) generated using neural network models such as Stable Diffusion, ComfyUI, Midjourney, or Flux.
- **AIGC Ingestion Pipeline (Mode C)**: An active folder monitoring mode that watches generator output directories, applies 500 ms write-lock debouncing, auto-harvests new files, and manages delayed cleanup queues.
- **Album**: A user-curated collection of artworks grouped without physically moving files on disk.
- **Authoritative Hero**: The primary cover image designated to represent an entire image stack in collapsed gallery view.

## B
- **Batch Action Bar**: A floating toolbar that appears at the bottom of the canvas when one or more cards are selected, offering fast bulk operations (rating, tagging, exporting, trashing).
- **Burst Stacking**: Automatically clustering sequential generation variations created with similar prompts and within a tight time window into a single stacked card.

## C
- **CFG Scale (Classifier-Free Guidance)**: A parameter in diffusion models controlling how closely the image generation adheres to the text prompt.
- **Checkpoint Model**: A base generative neural network model containing trained weights (typically `.safetensors` format), identified by name and SHA256 hash.
- **CLIP (Contrastive Language-Image Pre-training)**: A multi-modal neural network architecture used by Omera for natural language semantic search and visual similarity calculations.
- **ComfyUI**: A modular, node-based generative AI workflow engine. Omera parses its embedded workflow graphs and supports direct `/prompt` API dispatching.
- **Cull Drafts**: A batch cleanup action that keeps top-rated Hero artworks in a burst stack while moving lower-rated variations to the OS Recycle Bin.

## E
- **External Link (Mode A)**: A folder mode that indexes local or network storage in-place with zero-copy and leaves physical files untouched.

## J
- **Jaccard Similarity**: A statistical metric used to measure the overlap and similarity between two tokenized prompt strings for automatic burst clustering.

## K
- **Keyset Cursor Pagination**: A database query strategy that replaces slow `OFFSET N` queries with `WHERE (modified_at, id) < (?, ?)`, achieving sub-millisecond page traversal on 500,000+ records.

## L
- **Lightbox (Quick Look)**: A distraction-free, fullscreen viewer opened with `Space` or `Enter`, supporting mouse-wheel zoom, pan, and frame-by-frame video stepping.
- **LoRA (Low-Rank Adaptation)**: A small, efficient fine-tuning adapter applied on top of checkpoint models to introduce specific characters, styles, or concepts.

## M
- **Managed Vault (Mode B)**: A dedicated application repository that physically organizes imported files into a clean, date-partitioned folder structure (`YYYY/MM/UUID_filename.ext`).
- **Masonry Waterfall**: A gallery layout mode that arranges cards in dynamic columns while preserving each artwork's original aspect ratio without cropping.

## N
- **NSFW Blur**: A privacy feature that overlays a blur filter on adult or sensitive content until explicitly clicked by the user.

## O
- **OCC (Optimistic Concurrency Control)**: A concurrency management strategy that uses row-level `version` numbers to prevent data loss when multiple team members edit the same library simultaneously.

## P
- **Poker-Deck Card**: The visual representation of an image stack in the gallery, styled with layered edges behind the cover image and an interactive count badge.
- **Prompt Chips**: Interactive UI tags representing individual prompt tokens in the Property Inspector, allowing one-click searching and copying.

## S
- **Sampler / Scheduler**: The numerical integration algorithm (e.g. `Euler a`, `DPM++ 2M Karras`) used to denoise latents into finished images.
- **Seed**: An integer value that initializes the pseudorandom noise generator for reproducible AI generations.
- **Sidecar File**: A companion `.txt` or `.json` file stored beside an image containing generation parameters or Civitai metadata.
- **Storage Root**: A cross-platform abstract directory identifier (UUID) that maps network shares across different operating system mount paths.

## W
- **WAL (Write-Ahead Logging)**: A SQLite journaling mode that enables concurrent, non-blocking reads during background indexing and thumbnail generation.
- **WD14 Tagger**: A computer vision model (SmilingWolf) that automatically predicts Danbooru anime tags and content ratings from image pixels.
