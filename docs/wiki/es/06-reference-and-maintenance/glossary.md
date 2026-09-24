# Glosario del producto

Este glosario define los términos esenciales del dominio, conceptos de arquitectura y funcionalidades clave utilizados a lo largo de **Omera**.

---

## A
- **AIGC (Contenido generado por IA / AI-Generated Content)**: Medios digitales (imágenes, animaciones, audio y vídeo) sintetizados mediante redes neuronales como Stable Diffusion, ComfyUI, Midjourney o Flux.
- **Flujo de ingesta AIGC (Modo C / AIGC Ingestion Pipeline)**: Modo de monitorización activa que vigila las carpetas de salida de generadores de IA, aplica una estabilización de bloqueo de 500 ms (debounce), recolecta automáticamente las creaciones y gestiona colas de limpieza retardada.
- **Álbum**: Colección curada por el usuario que agrupa obras visuales sin necesidad de mover los archivos físicos en el disco.
- **Portada principal autorizada (Authoritative Hero)**: La imagen de cubierta designada como representativa para encabezar una pila completa en la vista contraída de la galería.

## B
- **Barra flotante de acciones por lotes (Batch Action Bar)**: Barra de herramientas flotante desplegada en la parte inferior del lienzo cuando se seleccionan una o más tarjetas, ofreciendo operaciones masivas rápidas (calificar, etiquetar, exportar, enviar a la papelera).
- **Apilamiento de ráfagas (Burst Stacking)**: Agrupación automática de variaciones de generación consecutivas creadas con prompts similares y en una misma ventana de tiempo en una única tarjeta apilada.

## C
- **Escala CFG (Classifier-Free Guidance)**: Parámetro en modelos de difusión que controla con qué rigor la generación visual debe ajustarse a las instrucciones del prompt de texto.
- **Modelo Checkpoint**: Modelo base de red neuronal generativa que contiene los pesos entrenados (habitualmente en formato `.safetensors`), identificado por su nombre y hash SHA256.
- **CLIP (Contrastive Language-Image Pre-training)**: Arquitectura multimodal de red neuronal utilizada en Omera para la búsqueda semántica en lenguaje natural y el cálculo de similitud visual.
- **ComfyUI**: Motor de flujos de trabajo modulares basado en nodos para IA generativa. Omera interpreta sus grafos incrustados y permite el reenvío directo mediante la API `/prompt`.
- **Eliminar borradores (Cull Drafts)**: Acción de descarte en lote que conserva las obras Hero con mayor puntuación en una pila de ráfaga mientras traslada las variaciones secundarias a la papelera del sistema.

## E
- **Enlace externo (Modo A / External Link)**: Modo de carpeta que indexa almacenamiento local o de red in situ sin copia (zero-copy), dejando los archivos físicos intactos.

## J
- **Similitud de Jaccard (Jaccard Similarity)**: Métrica estadística utilizada para cuantificar el grado de solapamiento léxico entre dos cadenas de prompts tokenizadas durante la agrupación automática de ráfagas.

## K
- **Paginación por cursor Keyset (Keyset Cursor Pagination)**: Estrategia de consulta a base de datos que sustituye las lentas cláusulas `OFFSET N` por filtros de rango `WHERE (modified_at, id) < (?, ?)`, logrando una navegación submilimétrica en catálogos de más de 500.000 registros.

## L
- **Lightbox (Vista rápida a pantalla completa)**: Visor envolvente a pantalla completa abierto con `Espacio` o `Intro`, que admite zoom y desplazamiento con rueda y arrastre del ratón, así como avance de vídeo fotograma a fotograma.
- **LoRA (Low-Rank Adaptation)**: Adaptador de ajuste fino compacto y eficiente que se aplica sobre modelos checkpoint para introducir personajes, conceptos o estéticas particulares.

## M
- **Bóveda gestionada (Modo B / Managed Vault)**: Repositorio dedicado de la aplicación que organiza físicamente las obras importadas dentro de una estructura particionada por fecha (`AAAA/MM/UUID_nombre.ext`).
- **Cascada (Masonry Waterfall)**: Disposición de galería en columnas dinámicas que conserva la proporción de aspecto nativa de cada obra sin recortes.

## N
- **Difuminado NSFW (NSFW Blur)**: Función de protección de privacidad que superpone una capa de desenfoque sobre contenido adulto o sensible hasta que el usuario hace clic para revelarlo.

## O
- **OCC (Control de concurrencia optimista / Optimistic Concurrency Control)**: Estrategia de gestión de concurrencia que emplea números de versión a nivel de fila (`version`) para evitar pérdidas de datos cuando múltiples miembros de un equipo editan la misma biblioteca simultáneamente.

## P
- **Tarjeta en baraja de póquer (Poker-Deck Card)**: Representación visual de una pila en la galería, estilizada con capas superpuestas visibles detrás de la cubierta y un distintivo de recuento interactivo.
- **Pastillas de prompt (Prompt Chips)**: Etiquetas interactivas en el Inspector de propiedades que representan los tokens individuales de un prompt, permitiendo búsquedas y copias en un solo clic.

## S
- **Sampler / Scheduler (Muestreador / Planificador)**: Algoritmo de integración numérica (p. ej., `Euler a`, `DPM++ 2M Karras`) utilizado para desruidizar el espacio latente y obtener la imagen final.
- **Semilla (Seed)**: Valor numérico entero que inicializa el generador de números pseudoaleatorios para asegurar resultados reproducibles.
- **Archivo auxiliar (Sidecar File)**: Archivo complementario `.txt` o `.json` guardado junto a una imagen que contiene parámetros de generación o metadatos de Civitai.
- **Raíz de almacenamiento (Storage Root)**: Identificador universal abstracto (UUID) que mapea recursos compartidos de red a través de distintas rutas de montaje en diferentes sistemas operativos.

## W
- **WAL (Write-Ahead Logging)**: Modo de registro transaccional de SQLite que habilita lecturas concurrentes sin bloqueos durante la indexación en segundo plano y la decodificación de miniaturas.
- **Autoetiquetador WD14 (WD14 Tagger)**: Modelo de visión artificial (SmilingWolf) que infiere automáticamente etiquetas Danbooru y niveles de contenido sensible a partir de los píxeles de la imagen.
