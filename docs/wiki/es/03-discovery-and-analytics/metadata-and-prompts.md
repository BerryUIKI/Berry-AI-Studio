# Metadatos AIGC e inspección de prompts

Omera cuenta con un analizador de metadatos multimodelo sin pérdidas desarrollado en Rust (`omera-metadata`). Extrae de forma automática prompts, prompts negativos, modelos, semillas y grafos de ejecución de las principales plataformas de generación con IA.

---

## 1. Plataformas de generación con IA compatibles

Omera reconoce de forma nativa los metadatos incrustados en fragmentos PNG, cabeceras EXIF de WebP y cajas ISOBMFF de MP4 procedentes de:

| Plataforma / Herramienta | Campos de metadatos extraídos | Ubicación en el contenedor |
| :--- | :--- | :--- |
| **AUTOMATIC1111 / SD.Next / Forge** | Prompt, Prompt negativo, Pasos, Sampler, CFG, Semilla, Dimensiones, Hash del modelo, Nombre del modelo, Denoising, Reescalado Hires | Bloque PNG `parameters` / EXIF JPEG `UserComment` |
| **ComfyUI** | Grafo completo de nodos, codificación de texto CLIP positivo/negativo, semillas KSampler, pasos, CFG, cargadores de checkpoint, cargadores de LoRA, nodos de reescalado latente | Fragmentos PNG `prompt` y `workflow` / bloques WebP ComfyUI / MP4 `moov/udta` |
| **NovelAI** | Título, Descripción, Prompt, Prompt negativo, Semilla, Sampler, Pasos, Escala, Versión de software | Bloques PNG `Comment` y `Description` |
| **Fooocus / Fooocus-MRE** | Modelo base, Refinador, Pesos de LoRA, Nitidez, Modo de rendimiento, Resolución, Prompt | Bloques de texto PNG `parameters` |
| **InvokeAI** | Nombre de modelo, VAE, Scheduler, Modo de generación (txt2img/img2img), Patrón continuo (seamless) | Bloques JSON PNG `sd-metadata` e `invokeai_metadata` |
| **EasyDiffusion / Stable Swarm** | Bloques de parámetros JSON estructurados, Semilla, Nombre de modelo | Bloques PNG `sui_image_params` / archivos auxiliares |

---

## 2. Panel del inspector de propiedades (`InspectorPane.vue`)

Al seleccionar una imagen o un vídeo, el panel lateral derecho (atajo `I`) presenta sus metadatos de forma estructurada:

```
┌────────────────────────────────────────────────────────┐
│ INSPECTOR DE PROPIEDADES                           [✕] │
├────────────────────────────────────────────────────────┤
│ [ Vista previa multimedia ]                            │
│ 1024 × 1024 · PNG · 3.4 MB · 2026-09-21                │
│ [ ★★★★★ ]  [ ★ Favorito ]  [ 🔞 NSFW ]                 │
├────────────────────────────────────────────────────────┤
│ Prompt positivo                           [📋 Copiar]  │
│ ┌────────────────────────────────────────────────────┐ │
│ │ [masterpiece] [1girl] [solo] [cyberpunk city]      │ │
│ │ [neon reflections] [rain] [volumetric lighting]    │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Prompt negativo                           [📋 Copiar]  │
│ ┌────────────────────────────────────────────────────┐ │
│ │ worst quality, low quality, bad anatomy, bad hands │ │
│ └────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────┤
│ Parámetros de generación                               │
│ Modelo:   animagine_xl_3.1.safetensors                 │
│ Hash:     31e35c80  [🔍 Buscar en Civitai]             │
│ Sampler:  DPM++ 2M Karras                              │
│ Pasos:    28             Escala CFG: 7.0               │
│ Semilla:  2849104812     [📋 Copiar]                   │
├────────────────────────────────────────────────────────┤
│ LoRAs detectados (2)                                   │
│ • CyberpunkStyle (Peso: 0.85)             [+ En prompt]│
│ • DetailedEyes (Peso: 0.6)                [+ En prompt]│
├────────────────────────────────────────────────────────┤
│ ▼ Metadatos en bruto (Flujo JSON de ComfyUI)           │
└────────────────────────────────────────────────────────┘
```

---

## 3. Pastillas interactivas de tokens de prompt

Omera segmenta las cadenas de prompts en pastillas de tokens en lugar de mostrar un bloque plano de texto sin estructura:

- **Búsqueda en 1 clic**: Al pulsar cualquier pastilla de token (p. ej., `[cyberpunk city]`), se inicia de inmediato una búsqueda en toda la biblioteca para ese concepto específico.
- **Copia en 1 clic**: Haga clic en el icono de copia en la esquina superior derecha del cuadro de prompt para copiar el texto limpio y sin caracteres de escape al portapapeles.
- **Descubrimiento de etiquetas**: Las pastillas facilitan la identificación de estilos de artistas, esquemas de iluminación o etiquetas de calidad para su posterior reutilización.

---

## 4. Modelos Checkpoint e identificación de hashes

Las herramientas generativas suelen incrustar hashes cortos (p. ej., `31e35c80`) o hashes SHA256 completos en lugar de nombres de archivo reconocibles.

- Omera consulta automáticamente su **Caché de modelos** local (tabla SQLite `model_cache`) para traducir hashes crípticos a nombres amigables como `"Animagine XL 3.1"`.
- Si se detecta un hash desconocido, puede importar un archivo `cache.json` de AUTOMATIC1111 o consultarlo directamente en Civitai (consulte [Modelos y biblioteca LoRA](../04-intelligent-curation/models-and-loras.md)).

---

## 5. Metadatos en bruto y JSON de flujos de ComfyUI

Para usuarios avanzados y directores técnicos que necesitan inspeccionar conexiones entre nodos:
- Despliegue el acordeón de **Metadatos en bruto** en la parte inferior del inspector para examinar la estructura JSON original sin modificaciones.
- Puede copiar el bloque completo del flujo de trabajo en JSON para pegarlo en editores de texto o compartirlo con su equipo.
