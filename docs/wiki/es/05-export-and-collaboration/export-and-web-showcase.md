# Exportación por lotes, transcodificación y muestra web

Omera cuenta con un motor de empaquetado y exportación de alto rendimiento (`ExportModal.vue`) potenciado por procesamiento multihilo con **Rayon**. Admite transcodificación de formatos, eliminación de metadatos por privacidad, plantillas dinámicas de nombres de archivo y generación de muestras HTML interactivas sin dependencias externas.

---

## 1. Apertura de la exportación por lotes

Para abrir el cuadro de exportación:
- Seleccione una o varias imágenes o pilas en la galería.
- Haga clic en **«Exportar...»** en la barra flotante de acciones por lotes, o pulse `Ctrl + E` / `Cmd + E` (o seleccione `Edición > Exportar...`).

---

## 2. Transcodificación de formatos y compresión

Omera convierte y recodifica imágenes en paralelo aprovechando todos los núcleos de su CPU:

| Formato de destino | Opciones y ajustes | Caso de uso idóneo |
| :--- | :--- | :--- |
| **Formato original** | Conserva exactamente los bytes y el contenedor de origen. | Copia de seguridad o archivo sin pérdidas. |
| **WebP** | Control de calidad regulable (1–100%, predeterminado 85%). Gran compresión. | Publicación web, Discord y páginas de portafolio. |
| **JPEG** | Codificación JPEG progresiva estándar (calidad 1–100%). | Compatibilidad universal con visores antiguos. |
| **PNG** | Compresión completamente sin pérdidas. | Entrega profesional y artes gráficas para impresión. |

### Restricciones de resolución máxima:
Puede limitar las dimensiones máximas para evitar distribuir por error imágenes pesadas en 4K u 8K:
- **Original** (sin reducción de escala)
- **4K UHD** (lado máximo: 3840 px)
- **2K QHD** (lado máximo: 2048 px)
- **Full HD** (lado máximo: 1080 px)
- **Límite personalizado** (definido por el usuario en píxeles)

---

## 3. Saneamiento de metadatos de privacidad en 4 niveles

Muchos creadores desean compartir sus obras en línea protegiendo sus prompts propietarios, incrustaciones negativas o semillas. Omera proporciona **cuatro niveles discretos de privacidad**:

1. **Mantener todo**:
   - Conserva todos los bloques de metadatos incrustados (grafos de ComfyUI, parámetros de A1111, comentarios de NovelAI y datos EXIF).
2. **Borrar solo prompts**:
   - Elimina las cadenas de texto del prompt positivo y negativo, pero conserva los parámetros técnicos de generación (sampler, pasos, escala CFG, semilla y nombre del modelo).
3. **Borrar todos los metadatos y flujos de IA**:
   - Elimina por completo los grafos de nodos de ComfyUI, los bloques de parámetros de A1111, las etiquetas de LoRAs y las firmas de generadores.
4. **Limpieza total (Solo píxeles)**:
   - Elimina absolutamente todo, incluidas cabeceras EXIF, perfiles de color ICC y firmas de software. El archivo exportado contiene únicamente los datos de imagen de mapa de bits sin procesar.

---

## 4. Plantillas de nombres de archivo y archivos auxiliares (Sidecar)

Personalice la nomenclatura de los archivos de salida usando variables dinámicas con vista previa en tiempo real:

### Variables compatibles:
- `{name}`: Nombre del archivo original sin extensión.
- `{id}`: Identificador único de base de datos o UUID.
- `{index}`: Índice secuencial del archivo exportado (001, 002, 003...).
- `{date}`: Fecha de creación (`AAAA-MM-DD`).
- `{rating}`: Puntuación de estrellas (p. ej., `5star`).
- `{model}`: Nombre del modelo checkpoint.
- `{seed}`: Valor de la semilla de generación.

*Ejemplo de plantilla*: `{date}_{model}_{seed}_{name}` → `2026-09-22_animagine_xl_2849104812_cyberpunk_01.webp`

### Archivos auxiliares de metadatos (Sidecar):
Puede generar automáticamente archivos complementarios junto a cada imagen exportada:
- **Ninguno**: Sin archivos auxiliares.
- **Texto del prompt (.txt)**: Exporta el prompt positivo de generación como archivo de texto plano.
- **Metadatos JSON completos (.json)**: Exporta la estructura completa de metadatos y el flujo de nodos de ComfyUI.

---

## 5. Generador de álbum web interactivo independiente (HTML Showcase)

Omera puede empaquetar sus activos exportados en una **galería web de archivo único completamente autocontenida** (`index.html`):

- **Cero dependencias**: No requiere servidor web, Node.js ni librerías JavaScript externas. Basta con hacer doble clic para abrirla en cualquier navegador.
- **Características incluidas**:
   - Estética oscura de estudio a juego con Omera.
   - Cuadrícula responsiva de galería con carga diferida de miniaturas.
   - Visor Lightbox a pantalla completa con zoom y desplazamiento por arrastre y rueda de ratón.
   - Inspector desplegable de parámetros de generación y prompts.
   - Barra de búsqueda instantánea por palabras clave en el cliente.
- **Opciones de empaquetado**: Exporte directamente a una carpeta o comprima todo en un único archivo `.zip` para envíos a clientes o publicación web.
