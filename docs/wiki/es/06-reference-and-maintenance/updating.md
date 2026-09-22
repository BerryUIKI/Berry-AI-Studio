# Actualizaciones y ciclo de vida

Berry AI Studio cuenta con un actualizador automático silencioso y directo sobre la instalación actual, diseñado para incorporar mejoras y correcciones sin interrumpir su trabajo ni poner en riesgo sus datos.

---

## 1. Búsqueda de actualizaciones

### Comprobación automática al inicio:
De manera predeterminada, Berry consulta la API de GitHub Releases cuando se abre la aplicación:
- Si hay una versión más reciente disponible, se muestra un indicador en el menú **Ayuda**.
- Puede activar o desactivar este comportamiento en **Preferencias > General > Buscar actualizaciones al inicio**.

### Comprobación manual:
Puede buscar actualizaciones de forma manual en cualquier momento:
- Seleccione **Ayuda > Buscar actualizaciones...** en la barra de menús superior.

---

## 2. Ventana de actualización directa (`UpdateModal.vue`)

Cuando se detecta una nueva versión, se abre la ventana de **Actualización**:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Nueva versión disponible: v0.3.1                                   [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Hay una nueva versión de Berry AI Studio disponible (Actual: v0.3.0).   │
│                                                                        │
│ Notas de la versión:                                                   │
│ • Analizador optimizado de metadatos de vídeo ComfyUI para HunyuanVideo.│
│ • Menor latencia en paginación profunda Keyset para más de 100k fotos. │
│ • Selector de puntuación de 10 estrellas en la barra flotante de lotes.│
├────────────────────────────────────────────────────────────────────────┤
│ Progreso de la descarga:                                               │
│ [██████████████████████████░░░░░░░░░░] 68% (12,4 MB / 18,2 MB · 4 MB/s)│
├────────────────────────────────────────────────────────────────────────┤
│ [ Cancelar ]                             [ ⚡ Instalar y reiniciar ahora ]│
└────────────────────────────────────────────────────────────────────────┘
```

### Proceso de instalación:
1. Haga clic en **«🚀 Descargar & actualizar automáticamente»** o **«⚡ Instalar y reiniciar ahora»**.
2. Berry descarga el paquete oficial correspondiente a su sistema operativo directamente desde GitHub Releases a una carpeta temporal.
3. Una vez finalizada la descarga, Berry solicita reiniciar la aplicación.
4. El actualizador nativo se aplica de forma silenciosa sobre la instalación existente y reinicia automáticamente Berry AI Studio.

---

## 3. Garantías de preservación de datos

Actualizar Berry AI Studio **nunca modifica ni borra sus datos de usuario**:

- **Seguridad de la base de datos**: Su base de datos `berry.db`, álbumes personalizados, etiquetas de colores, puntuaciones y relaciones de pilas de ráfagas se encuentran almacenados en el directorio AppData del usuario (`%APPDATA%`, `~/Library/Application Support` o `~/.config`), de forma totalmente independiente a los binarios de la aplicación.
- **Migraciones de esquema de adición exclusiva (Append-Only)**: Cuando una nueva versión introduce novedades en la estructura de datos, el backend en Rust de Berry ejecuta **migraciones incrementales** al arrancar mediante `PRAGMA user_version`. Las migraciones añaden columnas o índices sin reescribir ni eliminar los registros existentes.
- **Ajustes permanentes**: Su archivo de preferencias `config.json`, el presupuesto de caché de miniaturas y las rutas de las carpetas se mantienen intactos tras cada actualización.
