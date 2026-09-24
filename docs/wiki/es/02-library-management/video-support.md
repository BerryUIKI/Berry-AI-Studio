# Compatibilidad con vídeo y medios en movimiento

A medida que la generación de IA se expande hacia el vídeo (AnimateDiff, SVD, Wan2.1, HunyuanVideo, CogVideoX, LTX-Video), Omera proporciona soporte nativo de primer nivel para obras animadas y archivos de vídeo en formatos **MP4** y **WebM**.

---

## 1. Contenedores y formatos de vídeo compatibles

Omera analiza los contenedores de vídeo directamente en código Rust nativo (`omera-metadata`):

- **MP4 (`.mp4`)**: Analiza la jerarquía de cajas ISOBMFF (`ftyp`, `moov`, `trak`, `mdia`, `minf`, `stbl`).
  - Extrae automáticamente las dimensiones del vídeo (`Ancho × Alto`), la tasa de fotogramas (FPS), la duración de reproducción y el códec de vídeo (`H.264`, `H.265 / HEVC`, `AV1`).
  - Inspecciona las cajas `moov/udta` en busca de grafos de ejecución y parámetros incrustados de ComfyUI.
- **WebM (`.webm`)**: Analiza el contenedor EBML para flujos de vídeo VP8, VP9 y AV1, leyendo la duración y las dimensiones del fotograma directamente de la cabecera del flujo.

---

## 2. Tarjetas de vídeo en la galería y vistas previas dinámicas

En el lienzo de la galería, los archivos de vídeo se distinguen claramente de las imágenes estáticas:

- **Distintivo de duración**: Muestra el tiempo exacto de reproducción en la esquina (p. ej., `00:05` o `01:24`).
- **Distintivos de FPS y formato**: Muestra indicadores como `MP4 · 24fps` o `WEBP · 30fps`.
- **Generación de miniaturas**:
  - Dado que los archivos de vídeo carecen de decodificadores tradicionales en las bibliotecas de imágenes estándar, el frontend WebView de Omera captura automáticamente el primer fotograma clave mediante un canvas `<video>` oculto de HTML5, lo codifica en base64 y el backend en Rust lo almacena como una miniatura WebP reducida mediante `save_video_thumbnail`.
- **Reproducción al pasar el cursor**: Al situar el puntero sobre una tarjeta de vídeo, se inicia una vista previa ligera directamente en la cuadrícula sin necesidad de abrir el reproductor completo.

---

## 3. Reproductor de vídeo en Lightbox (`LightboxModal.vue`)

Al pulsar `Espacio` o `Intro` sobre cualquier tarjeta de vídeo, se abre el **Reproductor de vista rápida en pantalla completa (Lightbox)**:

```
┌────────────────────────────────────────────────────────────────────────┐
│ [✕]                                                          [★ Fav]   │
│                                                                        │
│                    [ LIENZO DE REPRODUCCIÓN DE VÍDEO ]                 │
│                                                                        │
│                                                                        │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ [▶ / ⏸] [⏪ 1f] [1f ⏩] [00:03 / 00:08] ──●───────── [1.0x ▾] [🔁] [🔊] │ │
│ └────────────────────────────────────────────────────────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ Tira: [Miniatura] [Miniatura] [● Vídeo actual] [Miniatura]         │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### Controles de vídeo en el Lightbox:
1. **Reproducir / Pausar**: Haga clic en el lienzo o pulse `Espacio`.
2. **Avance fotograma a fotograma**: Pulse `←` o `→` (o haga clic en los botones del panel HUD) para avanzar o retroceder fotograma a fotograma y analizar detalles de movimiento.
3. **Velocidad de reproducción**: Ajuste el selector entre **0.25x**, **0.5x**, **1.0x**, **1.5x** y **2.0x**.
4. **Bucle y sonido**: Active la reproducción continua en bucle (`🔁`) y controle el volumen o silenciamiento (`🔊`).
5. **Navegación por tira de miniaturas**: Desplácese entre las imágenes y vídeos adyacentes de la carpeta activa mediante la tira inferior.

---

## 4. Inspección de flujos de trabajo de vídeo en ComfyUI

Muchos generadores de vídeo utilizan complejos flujos de ComfyUI en varias fases (p. ej., prompt de texto → imagen latente inicial → módulo de movimiento AnimateDiff → guía de ControlNet openpose → escalador espacial).

Al inspeccionar un archivo de vídeo generado con ComfyUI:
- El **Inspector de propiedades** extrae y muestra los prompts de texto positivos y negativos utilizados por el modelo de movimiento.
- El **Acordeón de metadatos en bruto** renderiza el grafo completo de ejecución, incluyendo el checkpoint del modelo, los LoRAs de movimiento, la longitud de la ventana de contexto, el solapamiento de fotogramas y la configuración de decodificación VAE.
- Puede pulsar **«Enviar a ComfyUI»** para cargar exactamente el mismo flujo en su instancia activa de ComfyUI y realizar variaciones o refinamientos.
