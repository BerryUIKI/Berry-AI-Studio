# Privacy & Security Architecture

Omera is engineered around a **100% Local-First, Zero-Telemetry** design philosophy. In an era where generative AI workflows often involve proprietary art styles, private character concepts, and sensitive client assets, Omera ensures your creative work stays strictly on your machine.

---

## 1. Zero Telemetry & Offline Operation

### No Phone-Home Behavior
- Omera contains **zero tracking pixels, zero analytics SDKs, and zero crash-reporting services** (no Google Analytics, Sentry, Mixpanel, or PostHog).
- You can run Omera completely disconnected from the internet or behind strict corporate air-gapped firewalls without degraded functionality.

### Outbound Network Requests
Omera makes network connections **only** under three explicit circumstances:
1. **Application Updates**: When **Auto-Check for Updates** is enabled (or triggered manually via `Help > Check for Updates...`), Omera queries the public GitHub Releases API (`https://api.github.com/repos/BerryUIKI/Omera/releases/latest`).
2. **Civitai Hash Resolution**: When you explicitly click the "Lookup on Civitai" button for an unrecognized checkpoint model hash, Omera sends a single HTTP query to Civitai's public model endpoint.
3. **Cloud Backup & Team Sync**: When you configure an AWS S3, WebDAV, or central PostgreSQL/MySQL database server in Settings.

---

## 2. Local-Only AI Inference

All machine learning features in Omera run entirely on your local CPU/GPU using embedded **ONNX Runtime** sessions (`ort`):

- **CLIP / SigLIP Embeddings**: Image preprocessing and vector encoding occur locally. No prompts, text queries, or image pixels are transmitted to external servers.
- **WD14 Anime Auto-Tagging**: Neural network inference executes on local weights (`models/`). Tag predictions are written directly to your local SQLite database.

---

## 3. Privacy-First Export & Sanitization

When exporting or packaging artworks for public distribution, Omera includes a dedicated **4-tier metadata sanitization engine**:

- You can completely scrub embedded ComfyUI node graphs, positive/negative prompts, seeds, and LoRA tags with a single click before uploading images to social media or Discord (see [Export & Web Showcase](../05-export-and-collaboration/export-and-web-showcase.md)).
- The **Full Clean** tier removes all EXIF and ICC chunks, producing a pure pixel-only file.

---

## 4. Open-Source Transparency & License

Omera is free, open-source software licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**:

- Every line of Rust backend code, Tauri bridge commands, and Vue 3 frontend components is publicly auditable on [GitHub](https://github.com/BerryUIKI/Omera).
- You have complete freedom to audit, compile, fork, or deploy Omera within your personal workspace or commercial studio environment.
