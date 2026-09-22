# Modelos Checkpoint y biblioteca LoRA

Gestionar cientos de modelos checkpoint de Stable Diffusion y adaptaciones LoRA refinadas suele ser un reto en los flujos de IA generativa. Berry AI Studio ofrece catalogación integrada, resolución de hashes y administración de palabras disparadoras (trigger words).

---

## 1. Catálogo de modelos Checkpoint (`ModelManagerModal.vue`)

Berry registra de forma automática cada modelo checkpoint detectado en su biblioteca.

### Descubrimiento automático de modelos:
- Al indexar archivos, Berry extrae los nombres de los checkpoints y los hashes del modelo a partir de los metadatos PNGInfo y EXIF incrustados.
- Abra **Herramientas > Gestor de modelos...** para examinar el catálogo con todos los modelos, sus hashes cortos, sumas de comprobación SHA256 completas y recuentos totales de imágenes.

### Resolución de hashes con `cache.json` de AUTOMATIC1111:
- Los modelos checkpoint a menudo aparecen como hashes de 8 caracteres (p. ej., `31e35c80`).
- Si dispone de una instalación previa de AUTOMATIC1111:
  1. Haga clic en **«Importar A1111 cache.json»** en el Gestor de modelos.
  2. Seleccione el archivo `cache.json` de su WebUI (habitualmente en `<raiz_webui>/cache.json`).
  3. Berry importa la correspondencia a su tabla local `model_cache`, traduciendo al instante los hashes crípticos de toda su biblioteca a nombres de modelos reconocibles y legibles.

### Resolución de hashes SHA256 en Civitai:
- Para modelos sin nombre local registrado, puede hacer clic en el botón de consulta a Civitai en el Inspector para buscar en la base de datos pública de Civitai mediante el hash del modelo.

---

## 2. Biblioteca de palabras disparadoras LoRA (`LoraManagerModal.vue`)

Las adaptaciones de bajo rango (LoRA) precisan palabras de activación o disparadoras (trigger words) concretas en el prompt para reproducir fielmente personajes, estilos artísticos o indumentarias.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Biblioteca de disparadores LoRA                                    [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [🔍 Buscar por nombre, hash o disparador... ] [+ Añadir LoRA] [📁 Escanear]│
├────────────────────────────────────────────────────────────────────────┤
│ Nombre LoRA             │ Peso recom.│ Palabras disparadoras  │Acciones │
├─────────────────────────┼────────────┼────────────────────────┼─────────┤
│ CyberpunkCityStyle      │ 0.80       │ cyberpunk, neon signs, │ [Copiar]│
│                         │            │ futuristic alleys      │[Inyectar]
│ GenshinRaidenShogun     │ 0.85       │ raiden shogun, purple  │ [Copiar]│
│                         │            │ braid, glowing katana  │[Inyectar]
│ StudioGhibliVintage     │ 0.70       │ ghibli style, vintage  │ [Copiar]│
│                         │            │ watercolor, cel shaded │[Inyectar]
└────────────────────────────────────────────────────────────────────────┘
```

### Características clave de LoRA:
1. **LoRAs detectados en el Inspector**:
   - Al examinar una obra, el Inspector de propiedades detecta automáticamente la sintaxis de prompts `<lora:nombre:peso>` y los nodos `LoraLoader` de ComfyUI.
   - Lista los LoRAs encontrados, sus pesos aplicados y las palabras disparadoras reconocidas.
2. **Inyección de prompts en un clic**:
   - Haga clic en **«Copiar con <lora>»** para copiar la cadena formateada `<lora:nombre:0.8>` directamente al portapapeles.
   - Haga clic en cualquier pastilla de palabra disparadora para copiarla y usarla de inmediato en su editor de prompts.
3. **Importación de archivos auxiliares de Civitai (`.civitai.info`)**:
   - Si descarga LoRAs acompañados de archivos informativos de Civitai (`.civitai.info` o `.json`), Berry extrae automáticamente los hashes del modelo, la arquitectura base (SD 1.5, SDXL, Pony, Flux), las palabras disparadoras entrenadas y las imágenes de vista previa.
4. **Escaneo de directorios locales**:
   - Indique a Berry su carpeta local de LoRAs (`models/Lora/`).
   - Berry escaneará todos los archivos `.safetensors` y sus imágenes de muestra asociadas, construyendo una biblioteca de referencia sin conexión completamente indexada y con capacidad de búsqueda.
