# Berry AI Studio — Wiki oficial y guía del usuario

Bienvenido a la documentación de usuario definitiva y base de conocimientos de **Berry AI Studio** (`v0.3.0`).

Berry AI Studio es un gestor de activos y banco de trabajo de prompts local-first de código abierto, diseñado específicamente para creadores de IA generativa, ingenieros de prompts y estudios de diseño visual. Construido sobre **Tauri v2**, **Rust** y **Vue 3**, gestiona bibliotecas que van desde unos pocos cientos de obras de arte hasta más de 500.000 archivos con una latencia de consulta inferior al milisegundo, sin dependencia de la nube y con una extracción exhaustiva de metadatos de generación.

---

## 🧭 Navegación y tabla de contenidos

### [Capítulo 1: Introducción y conceptos fundamentales](01-getting-started/installation.md)
- **[Instalación y requisitos del sistema](01-getting-started/installation.md)**: Requisitos de hardware, configuración en Windows y opciones portátiles, compilaciones macOS Universal y Apple Silicon, Linux AppImage/deb y el asistente de bienvenida inicial (Onboarding Wizard).
- **[Estructura y anatomía del espacio de trabajo](01-getting-started/workspace-layout.md)**: Desglose detallado de la ventana sin marco, barra de menús de la aplicación, diseño de 3 paneles (Barra lateral, Galería, Inspector), Barra de estado y Barra de acciones por lotes flotante.
- **[Guía de atajos de teclado](01-getting-started/keyboard-shortcuts.md)**: Atajos globales, anclajes de selección, puntuación rápida de estrellas a ciegas, inspección rápida y teclas de acceso directo de navegación.

### [Capítulo 2: Gestión de la biblioteca y exploración](02-library-management/folder-modes-and-import.md)
- **[Importación de medios y modos de carpeta](02-library-management/folder-modes-and-import.md)**: Comparación entre el Modo A (Enlace externo), Modo B (Bóveda gestionada) y Modo C (Flujo de ingesta AIGC con vigilancia con retardo y reciclaje programado). Formatos de imagen (PNG, WebP, JPEG) y vídeo (MP4, WebM) compatibles.
- **[Modos de galería y opciones de visualización](02-library-management/gallery-views.md)**: Dominio de la Cuadrícula uniforme (con zoom de 130px a 360px), Cascada (proporciones de aspecto originales sin recortar), Vista de lista detallada y vista de coincidencias por similitud visual. Distintivos en tarjetas y difuminado de privacidad para contenido sensible (NSFW).
- **[Organización, puntuaciones y etiquetas](02-library-management/organization-and-tags.md)**: Escala de puntuación de 10 estrellas, favoritos, álbumes personalizados, taxonomía de etiquetas de 8 colores, arrastrar y soltar por lotes y barra de herramientas de acciones por lotes.
- **[Compatibilidad con vídeo y medios en movimiento](02-library-management/video-support.md)**: Reproducción de AnimateDiff, Wan2.1, HunyuanVideo y SVD; avance fotograma a fotograma, panel HUD de bucle/velocidad e inspección de flujos de trabajo de vídeo incrustados.

### [Capítulo 3: Descubrimiento, búsqueda y analítica](03-discovery-and-analytics/search-and-filtering.md)
- **[Sintaxis de búsqueda y filtros visuales](03-discovery-and-analytics/search-and-filtering.md)**: Lenguaje avanzado de consultas clave-valor (`prompt:`, `neg:`, `model:`, `cfg:>=7`, `steps:20..40`), rangos numéricos y el panel lateral deslizable de filtros.
- **[Metadatos AIGC e inspección de prompts](03-discovery-and-analytics/metadata-and-prompts.md)**: Extracción sin pérdidas para AUTOMATIC1111, ComfyUI, NovelAI, Fooocus e InvokeAI; pastillas de tokens interactivas y grafos de ejecución sin procesar.
- **[Estadísticas y análisis de prompts](03-discovery-and-analytics/prompt-insights.md)**: Distribuciones de frecuencia de tokens en toda la biblioteca, clasificación de prompts positivos y negativos y correlación con las puntuaciones de los usuarios.

### [Capítulo 4: Curación inteligente y motores de IA](04-intelligent-curation/stacks-and-bursts.md)
- **[Pilas de imágenes, ráfagas y comparación](04-intelligent-curation/stacks-and-bursts.md)**: Agrupación automática de ráfagas (similitud de prompt de Jaccard + ventanas de tiempo), tarjetas de pila estilo baraja de cartas, portadas principales (hero), aplanamiento seguro de pilas, herramienta para eliminar borradores (Cull Drafts) y modo de comparación lado a lado (`C`).
- **[Búsqueda semántica con IA y autoetiquetado](04-intelligent-curation/ai-semantic-and-tagger.md)**: Consultas en lenguaje natural de texto a imagen con modelos locales ONNX CLIP/SigLIP, vecinos visuales más cercanos de imagen a imagen y autoetiquetado estilo Danbooru/anime con WD14.
- **[Modelos Checkpoint y biblioteca LoRA](04-intelligent-curation/models-and-loras.md)**: Catalogación automática de checkpoints, resolución de hashes mediante `cache.json` de A1111, búsqueda inversa en Civitai, gestor de palabras disparadoras LoRA e inyección de prompts en un clic.
- **[Interoperabilidad con herramientas de generación](04-intelligent-curation/generation-interop.md)**: Comunicación directa mediante API con ComfyUI (`/prompt`) y AUTOMATIC1111 (`/sdapi/v1/txt2img`), con comprobación en directo del estado de la conexión.

### [Capítulo 5: Exportación, copia en la nube y colaboración](05-export-and-collaboration/export-and-web-showcase.md)
- **[Exportación por lotes, transcodificación y muestra web](05-export-and-collaboration/export-and-web-showcase.md)**: Transcodificación multihilo con Rayon, saneamiento de metadatos de privacidad en 4 niveles, plantillas dinámicas de nombres de archivo, paquetes ZIP y generación de álbumes web de muestra interactivos en un único archivo HTML sin dependencias.
- **[Instantáneas de respaldo en la nube y reflejo de medios](05-export-and-collaboration/cloud-backup-and-sync.md)**: Instantáneas en caliente con `VACUUM INTO` de SQLite hacia AWS S3, Cloudflare R2, MinIO, WebDAV o NAS local; sincronización incremental delta con detección ETag/SHA-256 y limitación de ancho de banda.
- **[Estudio de equipo con base de datos compartida](05-export-and-collaboration/team-collaboration.md)**: Escalado más allá de SQLite hacia servidores compartidos MySQL 8.0+ o PostgreSQL 14+; asignación multiplataforma de raíces de almacenamiento (normalización de letras de unidad de Windows a rutas de macOS/Linux), control de concurrencia optimista (OCC) y caché de miniaturas NVMe en el cliente.

### [Capítulo 6: Referencia del sistema y mantenimiento](06-reference-and-maintenance/settings-reference.md)
- **[Referencia completa de configuración](06-reference-and-maintenance/settings-reference.md)**: Guía exhaustiva de parámetros en las 8 pestañas de preferencias.
- **[Mantenimiento de bases de datos y caché](06-reference-and-maintenance/database-maintenance.md)**: Compactación SQLite WAL (`VACUUM`), copia de seguridad y restauración de base de datos, presupuesto de caché de miniaturas (expulsión LRU) y diagnóstico de colas.
- **[Actualizaciones y ciclo de vida](06-reference-and-maintenance/updating.md)**: Actualizador automático integrado en la aplicación, garantías de preservación de datos entre versiones y actualizaciones manuales de lanzamientos.
- **[Arquitectura de privacidad y seguridad](06-reference-and-maintenance/privacy-and-security.md)**: Modelo 100% centrado en local (offline-first), telemetría cero, aislamiento de inferencia de IA local y licencia AGPL-3.0.
- **[Resolución de problemas y preguntas frecuentes (FAQ)](06-reference-and-maintenance/troubleshooting-and-faq.md)**: Soluciones a problemas comunes, consejos de optimización del rendimiento y preguntas frecuentes.
- **[Glosario del producto](06-reference-and-maintenance/glossary.md)**: Definiciones autorizadas para términos específicos del dominio del producto (Hero, Flujo de ingesta, Similitud de Jaccard, Cursor Keyset, OCC, Tarjeta de baraja de póquer, etc.).

---

## ⚡ Atajos de inicio rápido

| Acción | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **Abrir vista rápida (Lightbox)** | `Espacio` / `Intro` | `Espacio` / `Retorno` |
| **Comparación lado a lado** | `C` | `C` |
| **Agrupar en pila** | `Ctrl + G` | `Cmd + G` |
| **Desapilar grupo** | `Ctrl + Shift + G` | `Cmd + Shift + G` |
| **Establecer como portada principal de la pila (Hero)** | `Alt + S` | `Option + S` |
| **Puntuar seleccionados (1–5 estrellas)** | `1` – `5` (`0` para borrar) | `1` – `5` (`0` para borrar) |
| **Alternar favorito** | `F` | `F` |
| **Enfocar barra de búsqueda** | `/` o `Ctrl + F` | `/` o `Cmd + F` |
| **Alternar panel del inspector** | `I` | `I` |
| **Alternar barra lateral** | `B` | `B` |
| **Abrir preferencias** | `Ctrl + ,` | `Cmd + ,` |
