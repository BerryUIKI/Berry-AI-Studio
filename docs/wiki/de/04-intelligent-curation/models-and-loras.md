# Checkpoint-Modelle & LoRA-Bibliothek

Die Verwaltung hunderter Stable-Diffusion-Checkpoints und spezialisierter LoRAs ist eine der zentralen Herausforderungen in der generativen Bildgestaltung. Berry AI Studio bietet integrierte Werkzeuge zur Modellkatalogisierung, Hash-Auflösung und Trigger-Wort-Verwaltung.

---

## 1. Checkpoint-Modellkatalog (`ModelManagerModal.vue`)

Berry erfasst automatisch jedes Checkpoint-Modell, das in Ihren Bildmetadaten auftaucht.

### Automatische Modellerkennung:
- Beim Indizieren von Dateien parst Berry Modellnamen und Modell-Hashes aus den eingebetteten PNGInfo-/EXIF-Metadaten.
- Öffnen Sie **Werkzeuge > Modell-Manager...**, um eine Übersicht aller Modelle, ihrer Kurz-Hashes, vollständigen SHA256-Prüfsummen und der zugehörigen Bildanzahl anzuzeigen.

### Hashes auflösen mit der AUTOMATIC1111 `cache.json`:
- Checkpoints erscheinen in Metadaten häufig als 8-stellige Hashes (z. B. `31e35c80`).
- Wenn Sie eine bestehende AUTOMATIC1111-Installation besitzen:
  1. Klicken Sie im Modell-Manager auf **„A1111 cache.json importieren“**.
  2. Wählen Sie Ihre WebUI-Datei `cache.json` aus (üblicherweise unter `<webui_root>/cache.json`).
  3. Berry übernimmt die Zuordnungen in die lokale Tabelle `model_cache` und löst kryptische Hashes in Ihrer gesamten Bibliothek sofort in lesbare Modellnamen auf.

### Civitai SHA256-Hash-Auflösung:
- Für Modelle ohne lokalen Klarnamen können Sie im Inspektor auf die Civitai-Schaltfläche klicken, um die öffentliche Civitai-Datenbank anhand des SHA256-Hashes abzufragen.

---

## 2. LoRA-Trigger-Bibliothek (`LoraManagerModal.vue`)

Low-Rank Adaptations (LoRAs) erfordern bestimmte Aktivierungs- oder Trigger-Wörter im Prompt, um gewünschte Charakterdetails, Kunststile oder Kleidungsmerkmale präzise darzustellen.

```
┌────────────────────────────────────────────────────────────────────────┐
│ LoRA-Trigger-Bibliothek                                            [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [🔍 LoRAs suchen... ]                 [+ LoRA hinzufügen] [📁 Ordner]  │
├────────────────────────────────────────────────────────────────────────┤
│ LoRA-Modellname         │ Gewichtung │ Trigger-Wörter         │Aktionen│
├─────────────────────────┼────────────┼────────────────────────┼────────┤
│ CyberpunkCityStyle      │ 0.80       │ cyberpunk, neon signs, │ [Kopie]│
│                         │            │ futuristic alleys      │[Prompt]│
│ GenshinRaidenShogun     │ 0.85       │ raiden shogun, purple  │ [Kopie]│
│                         │            │ braid, glowing katana  │[Prompt]│
│ StudioGhibliVintage     │ 0.70       │ ghibli style, vintage  │ [Kopie]│
│                         │            │ watercolor, cel shaded │[Prompt]│
└────────────────────────────────────────────────────────────────────────┘
```

### Wichtige LoRA-Funktionen:
1. **Erkannte LoRAs im Inspektor**:
   - Beim Betrachten eines Kunstwerks erkennt der Inspektor automatisch die Syntax `<lora:name:weight>` sowie ComfyUI-`LoraLoader`-Knoten.
   - Er führt erkannte LoRAs samt Gewichtung und hinterlegten Trigger-Wörtern übersichtlich auf.
2. **1-Klick-Prompt-Injektion**:
   - Klicken Sie auf **„Mit <lora> kopieren“**, um die formatierte Zeichenkette `<lora:name:0.8>` direkt in die Zwischenablage zu übernehmen.
   - Klicken Sie auf einen Trigger-Wort-Chip, um ihn sofort in Ihren Prompt-Editor einzufügen.
3. **Import von Civitai-Sidecars (`.civitai.info`)**:
   - Wenn Sie LoRAs zusammen mit Civitai-Helper-Dateien (`.civitai.info` oder `.json`) herunterladen, liest Berry Modell-Hashes, Basis-Architekturen (SD 1.5, SDXL, Pony, Flux), trainierte Aktivierungsbegriffe und Vorschaubilder automatisch ein.
4. **Lokale Verzeichnisscans**:
   - Verweisen Sie Berry auf Ihren lokalen LoRA-Ordner (`models/Lora/`).
   - Berry scannt alle `.safetensors`-Dateien und Begleitbilder und baut eine durchsuchbare Offline-Referenzbibliothek auf.
