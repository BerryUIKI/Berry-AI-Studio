# Guía de atajos de teclado

Berry AI Studio está diseñado con un flujo de trabajo centrado en el teclado (**keyboard-first**). Puede explorar, puntuar, agrupar, inspeccionar y organizar bibliotecas masivas sin necesidad de tocar el ratón.

---

## 1. Vista rápida y navegación por la galería

| Atajo | Acción | Ámbito | Descripción |
| :--- | :--- | :--- | :--- |
| `Espacio` o `Intro` | **Abrir Lightbox** | Selección de galería | Abrir la vista rápida en alta resolución del elemento seleccionado. |
| `Esc` | **Cerrar / Deseleccionar** | Global | Cerrar el Lightbox o diálogo modal, cancelar la búsqueda o deseleccionar tarjetas. |
| `←` / `→` | **Anterior / Siguiente** | Galería y Lightbox | Mover la selección al elemento adyacente (o avanzar/retroceder 1 fotograma en reproducción de vídeo). |
| `↑` / `↓` | **Fila arriba / abajo** | Vista Cuadrícula | Mover la selección arriba o abajo en una fila visual completa. |
| `Ctrl + A` / `Cmd + A` | **Seleccionar todo** | Lienzo de galería | Seleccionar todas las tarjetas que coincidan con la búsqueda o filtro activo. |
| `Supr` o `Retroceso` | **Mover a la papelera** | Selección de galería | Enviar los elementos seleccionados a la papelera de reciclaje del sistema operativo. |

---

## 2. Curación rápida y puntuación

| Atajo | Acción | Descripción |
| :--- | :--- | :--- |
| `1` | **Puntuar 1 estrella** | Asignar 1 estrella a los elementos seleccionados. |
| `2` | **Puntuar 2 estrellas** | Asignar 2 estrellas a los elementos seleccionados. |
| `3` | **Puntuar 3 estrellas** | Asignar 3 estrellas a los elementos seleccionados. |
| `4` | **Puntuar 4 estrellas** | Asignar 4 estrellas a los elementos seleccionados. |
| `5` | **Puntuar 5 estrellas** | Asignar 5 estrellas a los elementos seleccionados. |
| `0` | **Borrar puntuación** | Eliminar la puntuación de estrellas de los elementos seleccionados. |
| `F` | **Alternar favorito** | Marcar o desmarcar como favorito los elementos seleccionados. |

> [!TIP]
> **Puntuación a ciegas (Blind Rating)**: Puede mantener una mano en las teclas de flecha (`←`, `→`) para desplazarse rápidamente por las imágenes mientras pulsa del `1` al `5` con la otra mano. Las calificaciones se guardan de forma instantánea en SQLite sin interrumpir la fluidez de visualización.

---

## 3. Apilamiento y comparación

| Atajo | Acción | Ámbito | Descripción |
| :--- | :--- | :--- | :--- |
| `Ctrl + G` / `Cmd + G` | **Agrupar en pila** | Selección de galería | Agrupar los elementos seleccionados en una pila estilo baraja de cartas. |
| `Ctrl + Shift + G` / `Cmd + Shift + G` | **Desapilar grupo** | Selección de galería | Deshacer la pila seleccionada y devolver las imágenes al estado individual. |
| `Alt + S` / `Option + S` | **Establecer como portada (Hero)** | Selección en pila | Asignar la imagen activa como la portada principal de su pila. |
| `C` | **Comparación lado a lado** | Selección de galería | Abrir la vista comparativa 1 a 1 en pantalla dividida con zoom y desplazamiento sincronizados. |

---

## 4. Paneles del espacio de trabajo y zoom

| Atajo | Acción | Descripción |
| :--- | :--- | :--- |
| `B` | **Alternar barra lateral** | Mostrar u ocultar el panel de navegación izquierdo (Biblioteca, Carpetas, Etiquetas). |
| `I` | **Alternar inspector** | Mostrar u ocultar el panel derecho de metadatos e inspección. |
| `/` o `Ctrl + F` / `Cmd + F` | **Enfocar búsqueda** | Resaltar la barra de búsqueda y seleccionar todo el texto de la consulta. |
| `Ctrl + =` / `Cmd + =` | **Acercar (Zoom in)** | Aumentar el tamaño de las miniaturas en la galería. |
| `Ctrl + -` / `Cmd + -` | **Alejar (Zoom out)** | Reducir el tamaño de las miniaturas en la galería. |
| `Ctrl + 0` / `Cmd + 0` | **Restablecer zoom** | Restablecer el tamaño de las tarjetas al ancho estándar predeterminado (256px). |
| `F11` | **Pantalla completa** | Alternar el modo de ventana a pantalla completa sin bordes. |

---

## 5. Operaciones de biblioteca y ventanas modales

| Atajo | Acción | Descripción |
| :--- | :--- | :--- |
| `Ctrl + O` / `Cmd + O` | **Añadir carpeta** | Abrir el asistente para añadir carpetas y flujos de ingesta. |
| `Ctrl + E` / `Cmd + E` | **Exportar por lotes** | Abrir el cuadro modal de transcodificación, privacidad y empaquetado. |
| `Ctrl + ,` / `Cmd + ,` | **Preferencias** | Abrir la ventana central de ajustes y preferencias de la aplicación. |
| `?` o `Shift + /` | **Guía de atajos** | Abrir la hoja interactiva de atajos de teclado. |
| `Alt + F4` | **Salir de la app** | Cerrar ordenadamente Berry AI Studio. |

---

## 6. Modificadores de selección con ratón

- **Clic simple**: Selecciona una sola tarjeta y la establece como el **ancla de selección**.
- **Shift + Clic**: Extiende la selección desde el ancla hasta la tarjeta seleccionada formando un **rango contiguo inclusivo**.
- **Ctrl + Clic** (o `Cmd + Clic` en macOS): **Alterna** individualmente la selección de una tarjeta sin deseleccionar las demás, actualizando el ancla de selección.
- **Doble clic en la portada de una pila**: Abre de inmediato el Lightbox para ver la imagen de portada ampliada (el árbitro de gestos evita expansiones accidentales).
- **Clic simple en el distintivo de recuento de la pila**: Despliega o contrae las cartas de la pila directamente en la cuadrícula.
