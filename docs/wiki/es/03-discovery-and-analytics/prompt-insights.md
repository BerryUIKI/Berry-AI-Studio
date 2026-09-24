# Estadísticas y análisis de prompts

A medida que crece su colección de activos, entender qué palabras clave de prompt, artistas y parámetros técnicos producen sus obras mejor valoradas resulta fundamental. Omera incluye un panel modal de analítica dedicado (`PromptStatsModal.vue`) que consolida los metadatos de toda su biblioteca.

---

## 1. Apertura de las estadísticas de prompts

Para abrir el panel de análisis:
- Seleccione **Herramientas > Estadísticas de prompts...** en la barra de menús superior, o
- Haga clic en el botón **Estadísticas** en el pie de herramientas rápidas de la barra lateral izquierda.

---

## 2. Pestañas analíticas y métricas visuales

```
┌────────────────────────────────────────────────────────────────────────┐
│ Estadísticas de prompts y metadatos                                [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [ Tokens positivos ] [ Tokens negativos ] [ Modelos ] [ Samplers ]     │
├────────────────────────────────────────────────────────────────────────┤
│ Clasificado por: (●) Frecuencia   ( ) Puntuación media                 │
├────────────────────────────────────────────────────────────────────────┤
│ Puesto│ Token / Palabra clave       │ Ocurrencias │ Puntuación │ Acción│
├───────┼─────────────────────────────┼─────────────┼────────────┼───────┤
│ #1    │ masterpiece                 │ 4.120       │ ★ 4.2      │ [🔍]  │
│ #2    │ cinematic lighting          │ 2.845       │ ★ 4.7      │ [🔍]  │
│ #3    │ 1girl                       │ 2.410       │ ★ 3.9      │ [🔍]  │
│ #4    │ cyberpunk city              │ 1.890       │ ★ 4.8      │ [🔍]  │
│ #5    │ volumetric fog              │ 1.230       │ ★ 4.5      │ [🔍]  │
│ #6    │ photorealistic              │ 1.115       │ ★ 3.2      │ [🔍]  │
└────────────────────────────────────────────────────────────────────────┘
```

El panel proporciona cuatro perspectivas analíticas diferenciadas:

### 1. Tokens de prompt positivo
- Evalúa las pastillas de palabras clave individuales en todos los prompts positivos de generación.
- Muestra el recuento total de ocurrencias junto con la **Puntuación media de estrellas** de las imágenes que usan ese token.
- Resalta sus palabras clave más exitosas: términos que se correlacionan de forma consistente con calificaciones de 4★ y 5★.

### 2. Tokens de prompt negativo
- Analiza qué términos de prompt negativo se repiten con mayor frecuencia en su flujo de trabajo.
- Muy útil para detectar incrustaciones (embeddings) y palabras negativas redundantes o innecesarias que no mejoran el resultado final.

### 3. Modelos Checkpoint más usados
- Clasifica cada checkpoint de modelo de su biblioteca según el total de generaciones y la valoración del usuario.
- Ayuda a identificar qué modelos refinados ofrecen sus mejores resultados.

### 4. Samplers y Schedulers
- Clasifica los algoritmos de muestreo (p. ej., `DPM++ 2M Karras`, `Euler a`, `UniPC`) por frecuencia de uso y valoración estética.

---

## 3. Integración con la búsqueda interactiva

Cada fila de palabra clave en la tabla analítica incluye un botón de **Acción (`🔍`)**:
- Al hacer clic en el icono de búsqueda, se cierra el panel analítico, se inserta el token seleccionado en la barra de búsqueda de la galería principal (`prompt:"..."`) y se filtran en el lienzo todas las creaciones que contienen dicha palabra.
- Esto permite saltar directamente desde las estadísticas globales a la inspección de conjuntos específicos de imágenes.
