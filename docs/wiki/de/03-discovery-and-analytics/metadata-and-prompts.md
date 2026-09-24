# AIGC-Metadaten & Prompt-Inspektion

Omera verfügt über ein verlustfreies Metadaten-Extraktionsmodul in nativem Rust (`omera-metadata`). Es extrahiert vollautomatisch Prompts, negative Prompts, Modellnamen, Seeds und Ausführungsgraphen aller führenden Plattformen für generative KI.

---

## 1. Unterstützte KI-Generierungsplattformen

Omera versteht nativ Metadaten, die in PNG-Chunks, WebP-EXIF-Headern und MP4-ISOBMFF-Boxen eingebettet sind:

| Plattform / Werkzeug | Extrahierte Metadatenfelder | Container-Speicherort |
| :--- | :--- | :--- |
| **AUTOMATIC1111 / SD.Next / Forge** | Prompt, Negativer Prompt, Steps, Sampler, CFG, Seed, Dimensionen, Modell-Hash, Modellname, Denoising, Hires Upscale | PNG `parameters`-Chunk / JPEG EXIF `UserComment` |
| **ComfyUI** | Vollständiger Knoten-Ausführungsgraph, positive/negative CLIP-Texteinbettung, KSampler-Seeds, Steps, CFG, Checkpoint-Loader, LoRA-Loader, Latent-Upscale-Knoten | PNG `prompt`- & `workflow`-Chunks / WebP ComfyUI-Chunks / MP4 `moov/udta` |
| **NovelAI** | Titel, Beschreibung, Prompt, Negativer Prompt, Seed, Sampler, Steps, Skalierung, Softwareversion | PNG `Comment`- & `Description`-Chunks |
| **Fooocus / Fooocus-MRE** | Basis-Modell, Refiner, LoRA-Gewichte, Schärfe, Performance-Modus, Auflösung, Prompt | PNG `parameters`-Textblöcke |
| **InvokeAI** | Modellname, VAE, Scheduler, Generierungsmodus (txt2img/img2img), Nahtloses Kacheln | PNG `sd-metadata`- & `invokeai_metadata`-JSON |
| **EasyDiffusion / Stable Swarm** | Formatierte JSON-Parameterblöcke, Seed, Modellname | PNG `sui_image_params` / Sidecars |

---

## 2. Der Eigenschafts-Inspektor (`InspectorPane.vue`)

Wenn ein Bild oder Video ausgewählt ist, stellt das rechte Bedienfeld (`I`-Kürzel) dessen Metadaten übersichtlich dar:

```
┌────────────────────────────────────────────────────────┐
│ INSPEKTOR                                          [✕] │
├────────────────────────────────────────────────────────┤
│ [ Medien-Vorschau ]                                    │
│ 1024 × 1024 · PNG · 3.4 MB · 2026-09-21                │
│ [ ★★★★★ ]  [ ★ Favorit ]  [ 🔞 NSFW ]                  │
├────────────────────────────────────────────────────────┤
│ Prompt                                    [📋 Kopieren]│
│ ┌────────────────────────────────────────────────────┐ │
│ │ [masterpiece] [1girl] [solo] [cyberpunk city]      │ │
│ │ [neon reflections] [rain] [volumetric lighting]    │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Negativer Prompt                          [📋 Kopieren]│
│ ┌────────────────────────────────────────────────────┐ │
│ │ worst quality, low quality, bad anatomy, bad hands │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Generierungsparameter                                  │
│ Modell:      animagine_xl_3.1.safetensors              │
│ Hash:        31e35c80  [🔍 Auf Civitai suchen]         │
│ Sampler:     DPM++ 2M Karras                           │
│ Schritte:    28             CFG-Skala: 7.0             │
│ Seed:        2849104812     [📋 Kopieren]              │
├────────────────────────────────────────────────────────┤
│ Erkannte LoRAs (2)                                     │
│ • CyberpunkStyle (Gewichtung: 0.85)        [+ In Prompt]│
│ • DetailedEyes (Gewichtung: 0.6)           [+ In Prompt]│
├────────────────────────────────────────────────────────┤
│ ▼ Rohdaten (ComfyUI Workflow-JSON)                     │
└────────────────────────────────────────────────────────┘
```

---

## 3. Interaktive Prompt-Token-Chips

Omera zerlegt Prompt-Zeichenketten in interaktive Token-Chips, anstatt unformatierten Fließtext anzuzeigen:

- **1-Klick-Suche**: Ein Klick auf einen Token-Chip (z. B. `[cyberpunk city]`) führt sofort eine bibliotheksweite Suche nach diesem Begriff über all Ihre Kunstwerke aus.
- **1-Klick-Kopieren**: Klicken Sie auf das Kopiersymbol in der oberen rechten Ecke des Prompt-Bereichs, um den bereinigten Text direkt in die Zwischenablage zu übernehmen.
- **Entdeckung von Tags**: Tokenisierte Chips erleichtern das Identifizieren von Künstlerstilen, Beleuchtungs-Schlagwörtern oder Qualitäts-Tags zur Wiederverwendung in neuen Prompts.

---

## 4. Checkpoint-Modelle & Hash-Identifikation

Generative Werkzeuge betten häufig kurze Modell-Hashes (z. B. `31e35c80`) oder vollständige SHA256-Hashes anstelle von lesbaren Dateinamen ein.

- Omera fragt automatisch seinen lokalen **Modell-Cache** (SQLite-Tabelle `model_cache`) ab, um kryptische Hashes in lesbare Checkpoint-Namen wie `„Animagine XL 3.1“` aufzulösen.
- Bei unbekannten Hashes können Sie eine AUTOMATIC1111-`cache.json`-Datei importieren oder das Modell direkt über Civitai nachschlagen (siehe [Modelle & LoRA-Bibliothek](../04-intelligent-curation/models-and-loras.md)).

---

## 5. Rohdaten-Metadaten & ComfyUI-Workflow-JSON

Für fortgeschrittene Anwender und technische Direktoren, die Knoten-Verknüpfungen analysieren möchten:
- Klappen Sie das Akkordeon **Rohdaten** unten im Inspektor auf, um die unmodifizierten JSON-Daten zu prüfen.
- Sie können den gesamten JSON-Workflow-Block mit einem Klick kopieren, um ihn in Texteditoren einzufügen oder mit Teamkollegen zu teilen.
