# Estructura y anatomía del espacio de trabajo

Omera cuenta con un espacio de trabajo de escritorio de 3 paneles inspirado en **Eagle / Lightroom**, diseñado para una curación visual de alta densidad, máxima velocidad con el teclado y una visualización libre de distracciones.

---

## 1. Anatomía de la ventana y entorno de escritorio

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Logo] Omera   [ Archivo  Edición  Ver  Herramientas  Ayuda ]   [ _ ] [ □ ] [ ✕ ] │  <- Barra de título y menús
├──────────────┬──────────────────────────────────────────────────────────┬──────────────┤
│              │ [🔍 Buscar: prompt, modelo, puntuación... ] [🧠] [☰ Filtros]│              │
│ BARRA LATERAL├──────────────────────────────────────────────────────────┤  INSPECTOR   │
│ DE NAVEGACIÓN│                                                          │ DE PROPIEDAD │
│              │                 LIENZO DE GALERÍA CENTRAL                │              │
│ - Todo       │                                                          │ - Previa     │
│ - Favoritos  │  [ Tarjeta ]  [ Tarjeta ]  [ Pila póquer (4) ]  [ Tarjeta]│ - Prompts    │
│ - Carpetas   │                                                          │ - Modelo/CFG │
│ - Álbumes    │  [ Tarjeta ]  [ Tarjeta ]  [ Tarjeta ]          [ Tarjeta]│ - Tags LoRA  │
│ - Etiquetas  │                                                          │ - Interop    │
│              │                                                          │              │
│ [Herramientas]                                                          │              │
├──────────────┴──────────────────────────────────────────────────────────┴──────────────┤
│ 1.248 / 8.920 elementos | 3 seleccionados | Esquema v14 | Listo | [⚡ Actividad]        │  <- Barra de estado
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Barra de título y menús nativos de la aplicación

La ventana utiliza un diseño sin marcos con una barra de título personalizada (`TitleBar.vue`) y un sistema de menús de escritorio (`MenuBar.vue`):

### Referencia del menú superior
- **Archivo**:
  - `Añadir carpeta...` (`Ctrl + O` / `Cmd + O`): Conectar un nuevo directorio local o de red.
  - `Escanear carpeta actual`: Reindexar la carpeta activa en busca de archivos nuevos o modificados.
  - `Reescanear todas las carpetas`: Forzar un escaneo incremental en todas las carpetas registradas.
  - `Gestión de base de datos...`: Abrir herramientas de compactación, estadísticas de tamaño y copias de seguridad.
  - `Preferencias / Configuración...` (`Ctrl + ,` / `Cmd + ,`): Abrir la ventana central de ajustes de 8 pestañas.
  - `Salir` (`Alt + F4`): Cerrar Omera de forma ordenada.
- **Edición**:
  - `Seleccionar todo` (`Ctrl + A` / `Cmd + A`): Seleccionar todos los activos de la vista de galería activa.
  - `Deseleccionar todo` (`Esc`): Desmarcar todos los elementos seleccionados.
  - `Añadir etiquetas por lotes...`: Asignar o quitar etiquetas a los archivos seleccionados.
  - `Añadir al álbum por lotes...`: Asignar los archivos seleccionados a un álbum.
  - `Mover a carpeta...` / `Copiar a carpeta...`: Transferir físicamente archivos a otra carpeta indexada.
  - `Exportar...` (`Ctrl + E` / `Cmd + E`): Abrir la ventana de exportación, transcodificación y empaquetado.
  - `Mover a la papelera` (`Supr` / `Retroceso`): Enviar la selección de forma segura a la papelera de reciclaje del sistema operativo.
- **Ver**:
  - `Vista Cuadrícula`: Cambiar a tarjetas responsivas uniformes de altura fija.
  - `Cascada (Proporción original)`: Cambiar a columnas que conservan la relación de aspecto original sin recortes.
  - `Vista Lista`: Cambiar a una vista tabular compacta estilo hoja de cálculo.
  - `Alternar barra lateral` (`B`): Mostrar u ocultar el panel de navegación izquierdo.
  - `Alternar inspector` (`I`): Mostrar u ocultar el panel de metadatos derecho.
  - `Vista previa a pantalla completa (Lightbox)` (`Espacio` / `Intro`): Abrir la vista rápida ampliada.
  - `Acercar` (`Ctrl + =`), `Alejar` (`Ctrl + -`), `Restablecer zoom` (`Ctrl + 0`).
- **Herramientas**:
  - `Estadísticas de prompts...`: Distribuciones visuales de frecuencia de palabras y correlación con puntuaciones.
  - `Gestor de modelos...`: Resolver hashes de checkpoints a nombres descriptivos conocidos.
  - `Índice de búsqueda semántica CLIP...`: Indexar embeddings de la biblioteca para búsqueda en lenguaje natural.
  - `Biblioteca de disparadores LoRA`: Gestionar palabras disparadoras y archivos sidecar `.civitai.info`.
  - `Autoetiquetador IA WD14`: Abrir el etiquetador local Danbooru para anime y estética.
- **Ayuda**:
  - `Idioma`: Cambiar al instante entre los 7 idiomas compatibles.
  - `Guía de atajos de teclado` (`?`): Mostrar la hoja de atajos dentro de la aplicación.
  - `Buscar actualizaciones...`: Consultar los lanzamientos oficiales en GitHub Releases.
  - `Acerca de Omera`: Mostrar información de versión, autoría y licencia.

---

## 3. Barra lateral de navegación izquierda (`Sidebar.vue`)

La barra lateral izquierda (`B` para alternar) proporciona acceso rápido a las colecciones de activos:

1. **Bibliotecas del sistema**:
   - **Todas las imágenes**: Vista global de la biblioteca a través de todas las carpetas conectadas con distintivo de recuento total.
   - **Favoritos (★)**: Filtro rápido para todas las imágenes marcadas con estrella o marcador.
   - **Sensible (18+) (🔞)**: Acceso directo a contenido clasificado como NSFW (con difuminado de privacidad activo por defecto).
2. **Sección de Carpetas**:
   - Muestra todas las carpetas registradas con distintivos según su modo:
     - `⚡`: Flujo de ingesta AIGC (vigilancia activa de salidas del generador).
     - `📦`: Bóveda gestionada (almacenamiento interno estructurado).
     - `📁`: Enlace externo (indexación in situ sin copia).
   - Haga clic derecho o pase el cursor para ver acciones: **Recolectar imágenes**, **Escanear**, **Reconstruir metadatos** o **Eliminar**.
   - Funciona como destino de arrastre: arrastre tarjetas de la galería directamente sobre una carpeta para moverlas o copiarlas.
3. **Sección de Álbumes**:
   - Colecciones organizadas por el usuario. Arrastre y suelte tarjetas sobre los álbumes para agregarlas.
   - Incluye el botón «+ Nuevo».
4. **Sección de Etiquetas**:
   - Pastillas de taxonomía codificadas por color (8 distintivos de color diferentes).
   - Arrastre elementos desde la galería sobre una etiqueta para aplicarla en lote.
5. **Pie de herramientas rápidas**:
   - Accesos directos para abrir Estadísticas, Modelos, Mantenimiento de base de datos y la Guía de atajos.

---

## 4. Lienzo de galería central y barra de herramientas de búsqueda

El espacio central es donde explora, selecciona y cura sus activos:

- **Barra de búsqueda (`SearchBar.vue`)**:
  - **Modo de búsqueda por sintaxis (`🔍`)**: Consulte metadatos con palabras clave o sintaxis estructurada (p. ej., `prompt:"cyberpunk" cfg:>7`).
  - **Modo de búsqueda semántica IA (`🧠`)**: Consultas en lenguaje natural de texto a imagen impulsadas por modelos locales CLIP/SigLIP.
- **Selector de modo de vista**:
  - Alterne entre los diseños de **Cuadrícula (⊞)**, **Cascada (▤)** y **Lista (☰)**.
  - **Control deslizante de zoom**: Ajusta dinámicamente el ancho mínimo de las tarjetas entre **130px** y **360px**. El número de columnas se adapta de manera responsiva sin deformar las imágenes.
- **Interruptor del panel de filtros (`FilterDrawer.vue`)**:
  - Despliegue el panel de filtrado multicriterio para filtrar por checkpoint de modelo, sampler, relación de aspecto, puntuación de estrellas y propiedades de vídeo.

---

## 5. Inspector de propiedades derecho (`InspectorPane.vue`)

El inspector lateral derecho (`I` para alternar) desvela metadatos exhaustivos y sin pérdidas para la selección activa:

- **Vista previa multimedia**: Miniatura en alta resolución con botón para mostrar u ocultar el difuminado NSFW y apertura de vista rápida (Lightbox).
- **Controles de curación**: Puntuación de estrellas (0 a 5 estrellas), alternancia de Favorito (`F`) y marcador sensible (NSFW).
- **Prompt positivo**: Vista de prompt tokenizada. Cada token se presenta como una pastilla interactiva:
  - Haga clic en cualquier token para buscarlo en toda su biblioteca.
  - Haga clic en el botón de copiar para copiar el prompt positivo completo.
- **Prompt negativo**: Texto completo del prompt negativo con botón de copia en 1 clic.
- **Parámetros técnicos de generación**:
  - Nombre del modelo y hash SHA256 del checkpoint.
  - Algoritmos de Sampler y Scheduler.
  - Número de pasos (Steps), Escala de guía CFG y Semilla (Seed, con botón de copia).
  - Dimensiones nativas de generación (`Ancho × Alto`).
- **LoRAs detectados**:
  - Muestra todas las etiquetas `<lora:nombre:peso>` o nodos de carga de LoRA en ComfyUI detectados en el archivo.
  - Muestra palabras disparadoras (triggers), valores de peso y copia en 1 clic de la etiqueta formateada para el prompt.
- **Tarjeta de interoperabilidad de generación**:
  - Muestra el estado de conexión en vivo con **ComfyUI** y **AUTOMATIC1111** locales.
  - Envíos en un solo clic con «Enviar a ComfyUI» (pone en cola el grafo de flujo de trabajo) o «Enviar a SD WebUI».
- **Acordeón de metadatos en bruto**: Muestra el bloque JSON original de parámetros sin formatear o el grafo de nodos de ComfyUI.

---

## 6. Barra de estado inferior y ventana emergente de actividad

Ubicada en la parte inferior de la ventana:
- **Contadores de filtrado y totales**: Muestra los elementos coincidentes visibles frente al tamaño total de la biblioteca (p. ej., `Filtrado: 420 / 12.500 elementos`).
- **Recuento de selección**: Muestra la cantidad de elementos seleccionados (`Seleccionado: 5 elementos`).
- **Estado de la base de datos**: Muestra el nombre del archivo de base de datos activo y la versión del esquema.
- **Ventana emergente de actividad (`⚡ Actividad`)**:
  - Abre un monitor en tiempo real con el estado de vigilancia del sistema de archivos (raíces activas, cola de cambios pendientes, salud de errores) y las colas de decodificación de miniaturas en segundo plano.

---

## 7. Barra flotante de acciones por lotes (`BatchActionBar.vue`)

Cuando se seleccionan una o más tarjetas, aparece una barra de herramientas flotante en la parte inferior central del lienzo:

- Indicador de elementos seleccionados (`X de Y seleccionados`) con botones de Seleccionar todo / Deseleccionar.
- Desplegable de puntuación por lotes (0 a 10 estrellas).
- Botones modales para «Añadir al álbum» y «Añadir etiqueta».
- Activador de «Autoetiquetar (WD14)».
- Alternancia de «Favorito».
- Botones de «Copiar rutas» y «Copiar prompts» al portapapeles.
- Diálogos de destino para «Mover» y «Copiar».
- «Eliminar borradores de baja puntuación» (se activa automáticamente cuando se seleccionan pilas).
- «Exportar...» (transcodificación por lotes, saneamiento de privacidad y galería HTML).
- «Papelera» (mueve los elementos seleccionados a la papelera de reciclaje del sistema operativo).
