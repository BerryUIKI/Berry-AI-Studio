# Organización, puntuaciones y etiquetas

Omera proporciona avanzados mecanismos de curación diseñados para clasificar, ordenar y priorizar decenas de miles de obras de arte con gran rapidez y sin desordenar su sistema de archivos.

---

## 1. Sistema de puntuación y estrellas

Omera utiliza un modelo de valoración de doble precisión almacenado directamente en SQLite (`files.rating` y `files.aesthetic_score`):

### Puntuaciones de estrellas (escala de 0 a 5 estrellas o de 1 a 10)
- **Atajos de teclado**: Seleccione una o más imágenes y pulse:
  - De `1` a `5`: Asigna esa puntuación de estrellas inmediatamente.
  - `0`: Elimina la puntuación de estrellas.
- **Inspector / Barra de lotes**: Admite la escala extendida de 10 estrellas (p. ej., 8/10 o 9/10) a través de menús desplegables.
- **Indexación en base de datos**: La columna `rating` está indexada con árboles B en SQLite, lo que permite consultas instantáneas como `rating:>=4` o `rating:5` en bibliotecas de más de 500.000 elementos.

### Favoritos (★ Marcadores)
- **Alternar**: Pulse `F` o haga clic en el icono de estrella en el Inspector.
- **Indicador visual**: Aparece un distintivo de estrella dorada en la esquina superior derecha de la tarjeta.
- **Acceso en la barra lateral**: Todos los elementos marcados están disponibles de inmediato en la barra lateral izquierda bajo **Favoritos (★)**.

---

## 2. Álbumes personalizados (`AlbumModal.vue`)

Los álbumes permiten agrupar creaciones relacionadas distribuidas en diferentes carpetas sin necesidad de mover los archivos físicos en el disco.

### Creación y gestión de álbumes
1. En la barra lateral izquierda, bajo la sección **Álbumes**, haga clic en **«+ Nuevo»**.
2. Introduzca un nombre para el álbum (p. ej., `"Personajes Cyberpunk"`, `"Portafolio 2026"`) y una descripción opcional.
3. Cambie el nombre o elimine álbumes en cualquier momento haciendo clic derecho sobre ellos en la barra lateral. Eliminar un álbum suprime la agrupación virtual, pero nunca borra los archivos físicos.

### Añadir elementos a los álbumes
- **Arrastrar y soltar**: Seleccione una o varias tarjetas de la galería y arrástrelas directamente sobre el álbum en la barra lateral izquierda.
- **Barra de acciones por lotes**: Haga clic en **«Álbum»** en la barra flotante de acciones por lotes.
- **Inspector derecho**: Busque y asocie álbumes directamente desde el panel del Inspector.

---

## 3. Taxonomía de etiquetas con código de colores (`TagModal.vue`)

Las etiquetas proporcionan una categorización granular y clasificación visual rápida:

### Distintivos de color predefinidos
Omera incluye 8 paletas visuales preconfiguradas:
- 🔴 Rojo
- 🟠 Naranja
- 🟡 Amarillo
- 🟢 Verde
- 🔵 Azul
- 🟣 Púrpura
- 🌸 Rosa
- ⚪ Pizarra / Gris

### Trabajo con etiquetas
- **Creación de etiquetas**: Haga clic en **«+ Nueva»** en la barra lateral, elija un color predefinido e introduzca el nombre (p. ej., `Personaje Hero`, `Borrador`, `Aprobado por cliente`).
- **Arrastrar para etiquetar**: Arrastre las tarjetas seleccionadas desde el lienzo directamente sobre la pastilla de la etiqueta en la barra lateral para aplicarla.
- **Filtrado por etiqueta**: Haga clic en cualquier etiqueta de la barra lateral para filtrar la galería al instante.
- **Etiquetado múltiple**: Puede asignar un número ilimitado de etiquetas a cada obra.

---

## 4. Barra flotante de acciones por lotes (`BatchActionBar.vue`)

Siempre que seleccione múltiples elementos (mediante `Ctrl+Clic`, `Shift+Clic` o `Ctrl+A`), la **Barra de acciones por lotes** se despliega en la parte inferior de la pantalla:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  [✓ 14 de 120 seleccionados]  [Seleccionar todo]  [Deseleccionar]                      │
│  [★ Puntuación ▾]  [🏷 Etiqueta]  [📁 Álbum]  [🧠 Autoetiquetar]  [★ Fav]  [🔞 NSFW]    │
│  [📋 Copiar rutas]  [📋 Copiar prompts]  [📂 Mover]  [📄 Copiar]  [🧹 Borradores] [📤]  │
│  [🗑 Papelera]                                                                         │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### Operaciones principales por lotes:
- **Puntuación por lotes**: Asigna una calificación uniforme de estrellas a todas las imágenes seleccionadas.
- **Etiqueta / Álbum por lotes**: Abre la ventana modal para aplicar o desvincular etiquetas y álbumes en grupo.
- **Copiar prompts**: Copia al portapapeles los prompts positivos de todas las imágenes seleccionadas, separados con delimitadores limpios `---`.
- **Copiar rutas**: Copia las rutas absolutas completas del sistema de archivos (una por línea) para utilizarlas en scripts o terminales.
- **Mover y Copiar**: Mueve o copia archivos físicos a otra carpeta indexada en Omera.
- **Eliminar borradores**: Se activa cuando la selección incluye pilas de ráfagas (consulte [Pilas y ráfagas](../04-intelligent-curation/stacks-and-bursts.md)).
- **Papelera**: Traslada de forma segura todos los archivos seleccionados a la papelera de reciclaje del sistema operativo.
