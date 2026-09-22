# Video & Motion Media Support

As AI generation expands into video (AnimateDiff, SVD, Wan2.1, HunyuanVideo, CogVideoX, LTX-Video), Berry AI Studio provides native, first-class support for animated and video artworks in **MP4** and **WebM** formats.

---

## 1. Supported Video Formats & Containers

Berry AI Studio parses video containers directly in native Rust (`berry-metadata`):

- **MP4 (`.mp4`)**: Parses the ISOBMFF box hierarchy (`ftyp`, `moov`, `trak`, `mdia`, `minf`, `stbl`).
  - Automatically extracts video dimensions (`Width × Height`), framerate (FPS), playtime duration, and video codec (`H.264`, `H.265 / HEVC`, `AV1`).
  - Sniffs `moov/udta` boxes for embedded ComfyUI execution graphs and generation parameters.
- **WebM (`.webm`)**: Parses the EBML container format for VP8, VP9, and AV1 video streams, reading duration and frame dimensions directly from the stream header.

---

## 2. Gallery Video Cards & Dynamic Previews

In the gallery canvas, video files are clearly distinguished from static images:

- **Duration Badge**: Displays exact playtime in the corner (e.g. `00:05` or `01:24`).
- **Framerate & Format Badges**: Shows `MP4 · 24fps` or `WEBP · 30fps`.
- **Thumbnail Generation**:
  - Because video files do not have traditional image decoders in standard image libraries, Berry's frontend WebView automatically captures the first keyframe from an offscreen HTML5 `<video>` canvas, encodes it to base64, and the Rust backend stores it as a downscaled WebP thumbnail via `save_video_thumbnail`.
- **Hover Playback**: Hovering over a video card initiates lightweight video previewing directly on the canvas without opening full playback.

---

## 3. Lightbox Video Player (`LightboxModal.vue`)

Pressing `Space` or `Enter` on any video card opens the fullscreen **Quick Look Lightbox Player**:

```
┌────────────────────────────────────────────────────────────────────────┐
│ [✕]                                                          [★ Fav]   │
│                                                                        │
│                      [ VIDEO PLAYBACK CANVAS ]                         │
│                                                                        │
│                                                                        │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ [▶ / ⏸] [⏪ 1f] [1f ⏩] [00:03 / 00:08] ──●───────── [1.0x ▾] [🔁] [🔊] │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ Filmstrip: [Thumb] [Thumb] [● Current Video] [Thumb] [Thumb]       │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### Lightbox Video Controls:
1. **Play / Pause**: Click canvas or press `Space`.
2. **Frame-by-Frame Stepping**: Press `←` or `→` (or click the HUD buttons) to advance or rewind video playback frame-by-frame for detailed motion inspection.
3. **Playback Speed**: Adjust speed selector between **0.25x**, **0.5x**, **1.0x**, **1.5x**, and **2.0x**.
4. **Loop & Audio**: Toggle continuous looping (`🔁`) and volume/mute controls (`🔊`).
5. **Filmstrip Scrubbing**: Browse through adjacent static images and videos in the active folder using the bottom thumbnail filmstrip.

---

## 4. ComfyUI Motion & Video Workflow Inspection

Many video generators use complex multi-stage ComfyUI workflows (e.g. text prompt → initial latent image → AnimateDiff motion module → ControlNet openpose guidance → spatial upscaler).

When viewing a video file generated with ComfyUI:
- The **Property Inspector** parses and displays the positive/negative text prompts used for the motion model.
- The **Raw Metadata Accordion** renders the entire execution graph, including model checkpoint, motion LoRAs, context window length, overlap frames, and VAE decode settings.
- You can click **"Send to ComfyUI"** to load the exact video workflow back into your running ComfyUI instance for re-rendering or tweaking.
