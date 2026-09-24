# AIGC Metadata & Prompt Inspection

Omera features a lossless, multi-engine metadata parser written in Rust (`omera-metadata`). It automatically extracts prompts, negative prompts, models, seeds, and execution graphs across all major AI generation platforms.

---

## 1. Supported AI Generation Platforms

Omera natively understands metadata embedded in PNG chunks, WebP EXIF headers, and MP4 ISOBMFF boxes from:

| Platform / Tool | Extracted Metadata Fields | Source Container Location |
| :--- | :--- | :--- |
| **AUTOMATIC1111 / SD.Next / Forge** | Prompt, Negative Prompt, Steps, Sampler, CFG, Seed, Size, Model Hash, Model Name, Denoising, Hires Upscale | PNG `parameters` chunk / JPEG EXIF `UserComment` |
| **ComfyUI** | Full node execution graph, positive/negative CLIP text encode, KSampler seeds, steps, CFG, checkpoint loaders, LoRA loaders, latent upscale nodes | PNG `prompt` & `workflow` chunks / WebP ComfyUI chunks / MP4 `moov/udta` |
| **NovelAI** | Title, Description, Prompt, Negative Prompt, Seed, Sampler, Steps, Scale, Software version | PNG `Comment` & `Description` chunks |
| **Fooocus / Fooocus-MRE** | Base Model, Refiner, LoRA weights, Sharpness, Performance mode, Resolution, Prompt | PNG `parameters` text blocks |
| **InvokeAI** | Model name, VAE, Scheduler, Generation mode (txt2img/img2img), Seamless tiling | PNG `sd-metadata` & `invokeai_metadata` JSON |
| **EasyDiffusion / Stable Swarm** | Formatted JSON parameter blocks, Seed, Model name | PNG `sui_image_params` / sidecars |

---

## 2. The Property Inspector Pane (`InspectorPane.vue`)

When an image or video is selected, the right-hand panel (`I` hotkey) presents its metadata cleanly:

```
┌────────────────────────────────────────────────────────┐
│ PROPERTY INSPECTOR                                 [✕] │
├────────────────────────────────────────────────────────┤
│ [ Media Thumbnail Preview ]                            │
│ 1024 × 1024 · PNG · 3.4 MB · 2026-09-21                │
│ [ ★★★★★ ]  [ ★ Favorite ]  [ 🔞 NSFW ]                │
├────────────────────────────────────────────────────────┤
│ Positive Prompt                           [📋 Copy]    │
│ ┌────────────────────────────────────────────────────┐ │
│ │ [masterpiece] [1girl] [solo] [cyberpunk city]      │ │
│ │ [neon reflections] [rain] [volumetric lighting]    │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Negative Prompt                           [📋 Copy]    │
│ ┌────────────────────────────────────────────────────┐ │
│ │ worst quality, low quality, bad anatomy, bad hands │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Generation Parameters                                  │
│ Model:    animagine_xl_3.1.safetensors                 │
│ Hash:     31e35c80  [🔍 Find in Civitai]               │
│ Sampler:  DPM++ 2M Karras                              │
│ Steps:    28             CFG Scale: 7.0                │
│ Seed:     2849104812     [📋 Copy]                     │
├────────────────────────────────────────────────────────┤
│ Detected LoRAs (2)                                     │
│ • CyberpunkStyle (Weight: 0.85)           [+ In Prompt]│
│ • DetailedEyes (Weight: 0.6)              [+ In Prompt]│
├────────────────────────────────────────────────────────┤
│ ▼ Raw Metadata (ComfyUI Workflow JSON)                 │
└────────────────────────────────────────────────────────┘
```

---

## 3. Interactive Prompt Token Chips

Omera parses prompt strings into tokenized chips rather than displaying a wall of unformatted text:

- **1-Click Search**: Clicking any token chip (e.g. `[cyberpunk city]`) immediately executes a library search for that specific concept across all your artworks.
- **1-Click Copy**: Click the copy icon in the upper-right corner of the prompt box to copy the clean, unescaped prompt directly to your system clipboard.
- **Tag Discovery**: Tokenized chips make it easy to identify artist styles, lighting keywords, or quality tags that you want to reuse.

---

## 4. Checkpoint Models & Hash Identification

Generative tools often embed short model hashes (e.g. `31e35c80`) or full SHA256 hashes instead of human-readable filenames.

- Omera automatically queries its local **Model Cache** (`model_cache` SQLite table) to map cryptic hashes into friendly checkpoint names like `"Animagine XL 3.1"`.
- If an unknown hash is encountered, you can import an AUTOMATIC1111 `cache.json` file or look it up directly via Civitai (see [Models & LoRA Library](../04-intelligent-curation/models-and-loras.md)).

---

## 5. Raw Metadata & ComfyUI Workflow JSON

For advanced users and technical directors who need to examine node links:
- Expand the **Raw Metadata** accordion at the bottom of the inspector to inspect the unedited JSON payload.
- You can copy the entire JSON workflow block to paste directly into text editors or share with colleagues.
