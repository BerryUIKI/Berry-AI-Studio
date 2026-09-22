# Modos de galería y opciones de visualización

Berry AI Studio ofrece cuatro modos dedicados de presentación en la galería diseñados para adaptarse a distintos flujos de curación, desde la clasificación visual rápida hasta inspecciones técnicas minuciosas.

---

## 1. Modos de visualización de la galería

Utilice los botones de la barra de herramientas superior o los atajos del menú (`Ver`) para alternar entre los modos de visualización:

### 1. Vista Cuadrícula uniforme (`grid` — ⊞)
- **Concepto**: Tarjetas responsivas de altura fija y relación de aspecto uniforme organizadas en columnas adaptables.
- **Arquitectura responsiva**: El ancho de la tarjeta se mantiene estable al redimensionar la ventana. Berry calcula dinámicamente el número de columnas (`calculateGalleryColumns`) en lugar de estirar o comprimir las imágenes.
- **Control deslizante de zoom**: Arrastre el control de zoom o use `Ctrl + =` / `Ctrl + -` para ajustar suavemente el ancho mínimo de las tarjetas desde **130 px** (modo de vista previa compacta) hasta **360 px** (modo de gran detalle).
- **Virtualización**: Solo se renderizan en el DOM los elementos visibles en el área de visualización (más un pequeño búfer de sobreexploración previo). Explorar una biblioteca de 100.000 imágenes consume prácticamente la misma memoria que una de 100 imágenes.

### 2. Vista Cascada (`masonry` — ▤)
- **Concepto**: Disposición fluida en múltiples columnas que preserva la proporción de aspecto nativa de cada imagen.
- **Obras sin recortar**: Ideal para colecciones con fondos panorámicos horizontales, conceptos de personajes verticales (9:16) y entornos mixtos. Las imágenes se escalan limpiamente mediante `object-fit: contain` sin recortar los bordes.
- **Distribución por columna más corta**: Los elementos nuevos se ubican en la columna con menor altura vertical acumulada, manteniendo un muro visual equilibrado y estético.

### 3. Vista Lista de alta densidad (`table` — ☰)
- **Concepto**: Disposición tabular virtualizada estilo hoja de cálculo con altura de fila fija de 46 px.
- **Densidad de información**: Muestra micro-miniaturas (nivel de 36 px) junto a columnas detalladas:
  - Casilla de verificación de selección
  - Nombre de archivo y ruta relativa del directorio
  - Formato del contenedor multimedia (`PNG`, `WEBP`, `MP4`)
  - Dimensiones (`Ancho × Alto`)
  - Tamaño del archivo (formateado en KB/MB)
  - Marca de tiempo de fecha de modificación
  - Puntuación de estrellas (0–5 / 1–10)
  - Puntuación de predicción estética

### 4. Vista de coincidencias por similitud visual
- **Activación**: Haga clic en **«Buscar imágenes similares»** en el inspector derecho o en el menú contextual de cualquier imagen.
- **Banner superior de coincidencias**: Muestra la imagen de referencia original, el control deslizante de umbral de similitud (**0% a 95%** en pasos del 5%) y el selector de límite de resultados (**20, 50, 100, 200** elementos).
- **Distintivo de porcentaje de coincidencia**: Cada tarjeta coincidente muestra una pastilla de color con la puntuación de afinidad (p. ej., `94% Coincidencia`), calculada mediante la distancia del coseno de vectores CLIP.
- **Salir**: Pulse `Esc` o haga clic en el botón de cerrar del banner (`✕`) para regresar a la vista de galería habitual.

---

## 2. Distintivos en tarjetas y superposiciones visuales

En **Preferencias > Galería**, puede activar o desactivar **Mostrar distintivos en tarjetas**. Al habilitarse, las tarjetas muestran pastillas informativas claras:

- **Distintivo de formato / contenedor**: Indica si el archivo es `.png`, `.webp`, `.jpg`, `.mp4` o `.webm`.
- **Distintivo de dimensiones**: Resolución nativa en píxeles (p. ej., `1024×1024` o `832×1216`).
- **Duración y FPS de vídeo**: En animaciones y vídeos, muestra el tiempo de reproducción (p. ej., `00:04`) y la tasa de fotogramas (`24 fps`).
- **Distintivo de generador**: Identifica la firma del motor de generación (p. ej., `WebUI`, `ComfyUI`, `NovelAI`, `Fooocus`).
- **Superposición de puntuación**: Muestra la puntuación activa de estrellas (★ 1–5).
- **Marcador de favorito**: Icono de estrella dorada en la esquina superior derecha para imágenes marcadas como favoritas.

---

## 3. Difuminado de privacidad para contenido sensible (NSFW)

Para garantizar la privacidad durante presentaciones o al trabajar en espacios compartidos, Berry AI Studio incluye protección integrada de contenido:

- **Difuminado automático (`blur_nsfw` en ajustes)**: Cualquier activo clasificado como `is_nsfw` o identificado con calificación para adultos se oculta tras una máscara agresiva de difuminado CSS.
- **Haga clic para revelar**: Al pulsar el icono de ojo (`👁`) o la tarjeta, se desactiva temporalmente el difuminado para ese elemento concreto.
- **Sección global sensible**: La barra lateral izquierda incluye el filtro rápido **Sensible (18+) (🔞)** para auditar o reclasificar todo el material sensible desde un único lugar.
