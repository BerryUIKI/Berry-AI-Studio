# Sintaxis de búsqueda y filtros visuales

Berry AI Studio cuenta con un sistema de búsqueda dual: **Búsqueda por sintaxis estructurada** para un filtrado técnico preciso y **Búsqueda semántica con IA** para consultas conceptuales en lenguaje natural.

---

## 1. Búsqueda por sintaxis y lenguaje de consultas

En la barra de búsqueda superior (`/` o `Ctrl + F`), puede introducir palabras clave libres o formular consultas estructuradas de clave-valor:

### Búsqueda básica de texto libre
- Al escribir palabras sin prefijos, se buscan coincidencias en nombres de archivo, rutas relativas, prompts positivos, prompts negativos y nombres de modelos:
  ```
  cyberpunk neon rain
  ```
- Utilice comillas dobles para buscar frases exactas:
  ```
  "cyberpunk street" "rainy reflections"
  ```

---

## 2. Referencia de sintaxis clave-valor

Berry analiza los tokens de búsqueda en estructuras `SearchCriteria` en código Rust nativo, consultando índices SQLite con tiempos de respuesta inferiores al milisegundo:

| Clave de token | Ejemplo de sintaxis | Descripción |
| :--- | :--- | :--- |
| `prompt:` | `prompt:"masterpiece, 1girl"` | Coincidencia de palabras en el prompt positivo de generación. |
| `neg:` | `neg:"bad hands, blurry"` | Coincidencia de palabras en el prompt negativo. |
| `model:` | `model:"animagine_xl"` | Filtrar por nombre de checkpoint del modelo. |
| `hash:` | `hash:31e35c80` | Coincidencia por hash corto o hash SHA256 completo del modelo. |
| `sampler:` | `sampler:"Euler a"` | Filtrar por algoritmo de muestreo (sampler). |
| `steps:` | `steps:30`, `steps:20..40`, `steps:>=25` | Coincidencia de pasos exactos o rangos numéricos. |
| `cfg:` | `cfg:7`, `cfg:>=7.5`, `cfg:5..10` | Coincidencia de escala de guía CFG o rangos. |
| `seed:` | `seed:12345678` | Buscar por una semilla de generación específica. |
| `rating:` | `rating:5`, `rating:>=4`, `rating:1..3` | Filtrar por puntuación de estrellas del usuario. |
| `aesthetic:`| `aesthetic:>=7.0` | Filtrar por puntuación de predicción estética. |
| `fav:` | `fav:true`, `fav:false` | Filtrar por estado de favorito. |
| `is:` | `is:nsfw`, `is:sfw` | Filtrar por marcador de contenido sensible. |
| `type:` | `type:image`, `type:video` | Filtrar por tipo de contenedor multimedia. |
| `duration:` | `duration:>=5`, `duration:5..30` | Filtrar por duración de vídeo en segundos. |
| `fps:` | `fps:>=30`, `fps:24..60` | Filtrar por tasa de fotogramas de vídeo. |

### Sintaxis de rangos numéricos
- **Rango entre valores (`min..max`)**: `steps:20..35` (entre 20 y 35 pasos inclusive).
- **Mayor o igual que (`>=`)**: `cfg:>=7.0` (escala de guía 7.0 o superior).
- **Menor o igual que (`<=`)**: `rating:<=2` (2 estrellas o inferior).
- **Coincidencia exacta**: `rating:5` (exactamente 5 estrellas).

---

## 3. El panel lateral de filtros visuales (`FilterDrawer.vue`)

Si prefiere una interfaz gráfica en lugar de escribir sintaxis de búsqueda, haga clic en el botón del **Panel de filtros (`☰ Filtros`)** situado a la derecha de la barra de búsqueda para desplegarlo:

```
┌────────────────────────────────────────────────────────┐
│ Búsqueda visual & Filtros               [Restablecer] [✕]│
├────────────────────────────────────────────────────────┤
│ Modelo Checkpoint                                      │
│ [ Todos los modelos ▾                                ] │
│                                                        │
│ Sampler                                                │
│ [ Todos los samplers ▾                               ] │
│                                                        │
│ Puntuación de estrellas                                │
│ [ ★★★★★ (Solo 5 estrellas) ▾                         ] │
│                                                        │
│ Rango de pasos                                         │
│ Mín: [ 20 ] ─────────────●──────────── Máx: [ 50 ]     │
│                                                        │
│ Rango de escala CFG                                    │
│ Mín: [ 5.0 ] ────────────●──────────── Máx: [ 12.0 ]   │
│                                                        │
│ Tipo de medio                                          │
│ (●) Todos        ( ) Imágenes         ( ) Vídeos       │
│                                                        │
│ Marcadores de contenido                                │
│ [✓] Solo favoritos     [ ] Solo contenido sensible     │
└────────────────────────────────────────────────────────┘
```

El panel de filtros cuenta con un contador que muestra de un vistazo cuántos criterios están limitando activamente los resultados.

---

## 4. Opciones de ordenación de la galería (`SortBar.vue`)

A la derecha de la cabecera del lienzo de la galería, puede ordenar los resultados activos:

### Campos de ordenación:
- **Fecha de modificación (`modified_at`)**: Orden cronológico según la marca de tiempo del archivo.
- **Nombre de archivo / Ruta (`path`)**: Orden alfabético por ruta en el sistema de archivos.
- **Tamaño de archivo (`size_bytes`)**: Orden según el espacio que ocupan en disco.
- **Puntuación (`rating`)**: Orden según la calificación de estrellas otorgada.
- **Puntuación estética (`aesthetic_score`)**: Orden según la puntuación de predicción estética neural.

### Dirección de ordenación:
- Haga clic en el botón de dirección para alternar entre **Descendente (`↓`)** (mayor o más reciente primero) y **Ascendente (`↑`)** (menor o más antiguo primero).
