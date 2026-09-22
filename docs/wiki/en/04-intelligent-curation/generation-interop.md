# Generation Tool Interoperability

Berry AI Studio acts as an active companion to your creative generation tools, offering bidirectional API communication with **ComfyUI** and **AUTOMATIC1111 / SD.Next**.

---

## 1. Configuring Generation Services

Configure connection endpoints in **Settings > Generation Interop**:

- **ComfyUI Base URL**: Default `http://127.0.0.1:8188`
- **AUTOMATIC1111 WebUI Base URL**: Default `http://127.0.0.1:7860`

Each configuration field includes a **"Test Connection"** button. Berry sends a lightweight healthcheck ping (`/system_stats` for ComfyUI or `/sdapi/v1/options` for WebUI) and displays an **Online (🟢)** or **Offline (🔴)** indicator badge.

---

## 2. ComfyUI Workflow Dispatch (`send_to_comfyui`)

When browsing an artwork or video generated in ComfyUI:
1. In the right-hand Property Inspector, locate the **Generation Interop Card**.
2. Click **"Send to ComfyUI"**.
3. Berry's backend connects to ComfyUI's `/prompt` HTTP endpoint and submits the exact node graph and latent parameters extracted from the image.
4. Your ComfyUI instance immediately loads the workflow and adds it to the generation queue—no need to drag and drop files across windows.

---

## 3. SD WebUI Prompt Dispatch (`send_to_webui`)

For images generated in AUTOMATIC1111, Forge, or SD.Next:
1. Click **"Send to SD WebUI"** in the Inspector.
2. Berry formats a txt2img generation payload containing the positive prompt, negative prompt, steps, sampler, CFG scale, seed, and dimensions.
3. The payload is sent to WebUI's `/sdapi/v1/txt2img` API, pre-populating your WebUI fields or queuing a new generation run.

---

## 4. Live Pipeline Surveillance Loop

When you combine **Generation Interop** with an **AIGC Pipeline Folder** (Mode C):
1. You dispatch a workflow to ComfyUI or WebUI from Berry.
2. The generator renders the image and writes it to its output directory.
3. Berry's background watcher detects the new file, debounces write locks (500 ms), extracts metadata, renders a WebP thumbnail, and displays the finished artwork at the top of your gallery canvas in real time.
4. This creates a tight, zero-interruption creative loop between generation and curation.
