# Checkpoint Models & LoRA Library

Managing hundreds of Stable Diffusion checkpoint models and fine-tuned LoRAs is a challenge in generative AI workflows. Berry AI Studio provides built-in cataloging, hash resolution, and trigger word management.

---

## 1. Checkpoint Model Catalog (`ModelManagerModal.vue`)

Berry automatically tracks every checkpoint model encountered across your library.

### Automatic Model Discovery:
- When indexing files, Berry parses checkpoint names and model hashes from embedded PNGInfo/EXIF metadata.
- Open **Tools > Model Manager & Cache** to view a catalog of all models, their short hashes, full SHA256 checksums, and total image counts.

### Resolving Hashes with AUTOMATIC1111 `cache.json`:
- Checkpoint models often appear as 8-character hashes (e.g. `31e35c80`).
- If you have an existing AUTOMATIC1111 installation:
  1. Click **"Import Model Cache"** in the Model Manager.
  2. Select your WebUI `cache.json` file (typically located in `<webui_root>/cache.json`).
  3. Berry imports the mapping into its local `model_cache` table, instantly resolving cryptic hashes across your entire library into friendly model titles.

### Civitai SHA256 Hash Resolution:
- For models without local names, you can click the Civitai lookup button in the Inspector to query Civitai's public model database via model hash.

---

## 2. LoRA Trigger Words Library (`LoraManagerModal.vue`)

Low-Rank Adaptations (LoRAs) require specific activation or trigger words in generation prompts to produce intended character details, art styles, or clothing.

```
┌────────────────────────────────────────────────────────────────────────┐
│ LoRA Trigger Words Library & Manager                               [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [🔍 Search LoRAs... ]                   [+ Add LoRA] [📁 Scan Directory]│
├────────────────────────────────────────────────────────────────────────┤
│ LoRA Name               │ Default Wt │ Trigger Words          │ Actions │
├─────────────────────────┼────────────┼────────────────────────┼─────────┤
│ CyberpunkCityStyle      │ 0.80       │ cyberpunk, neon signs, │ [Copy]  │
│                         │            │ futuristic alleys      │ [Inject]│
│ GenshinRaidenShogun     │ 0.85       │ raiden shogun, purple  │ [Copy]  │
│                         │            │ braid, glowing katana  │ [Inject]│
│ StudioGhibliVintage     │ 0.70       │ ghibli style, vintage  │ [Copy]  │
│                         │            │ watercolor, cel shaded │ [Inject]│
└────────────────────────────────────────────────────────────────────────┘
```

### Key LoRA Features:
1. **Detected LoRAs in Inspector**:
   - When viewing an artwork, the Property Inspector automatically sniffs `<lora:name:weight>` prompt syntax and ComfyUI `LoraLoader` nodes.
   - It lists detected LoRAs, their applied weights, and recognized trigger words.
2. **One-Click Prompt Injection**:
   - Click **"+ In Prompt"** to copy the formatted `<lora:name:0.8>` string directly to your clipboard.
   - Click any trigger word chip to copy it for immediate use in your prompt editor.
3. **Importing Civitai Sidecars (`.civitai.info`)**:
   - If you download LoRAs with Civitai helper sidecars (`.civitai.info` or `.json`), Berry extracts model hashes, base model architectures (SD 1.5, SDXL, Pony, Flux), trained trigger words, and preview images automatically.
4. **Local Directory Scanning**:
   - Point Berry to your local LoRA folder (`models/Lora/`).
   - Berry scans all `.safetensors` files and companion preview images, building a comprehensive searchable offline reference library.
