# Referencia completa de configuración

La ventana de preferencias de Berry AI Studio (`SettingsModal.vue`) se abre mediante `Archivo > Preferencias / Configuración...` o con el atajo `Ctrl + ,` / `Cmd + ,`. Todos los ajustes se conservan de forma permanente en `config.json` dentro del directorio de datos de la aplicación.

---

## Pestaña 1: Preferencias generales

| Campo de ajuste | Clave en `config.json` | Valor predeterminado | Descripción |
| :--- | :--- | :--- | :--- |
| **Idioma de la aplicación** | `locale` | `"auto"` | Opciones disponibles: `auto` (sigue el idioma del SO), `en` (Inglés), `zh-CN` (Chino simplificado), `zh-TW` (Chino tradicional), `ja` (Japonés), `de` (Alemán), `fr` (Francés), `es` (Español). |
| **Vista predeterminada de galería** | `default_view` | `"grid"` | Modo de visualización al arrancar: `"grid"` (Cuadrícula uniforme), `"masonry"` (Cascada) o `"table"` (Lista detallada). |
| **Escanear automáticamente al inicio** | `auto_scan` | `true` | Comprueba automáticamente las carpetas indexadas en busca de archivos nuevos o modificados al abrir Berry. |
| **Intervalo del escaneo inicial** | `startup_scan_interval_minutes`| `360` | Tiempo mínimo de enfriamiento en minutos entre reconciliaciones completas de disco (`30`, `60`, `360`, `1440`). Evita lecturas innecesarias si se reinicia la aplicación frecuentemente. |
| **Buscar actualizaciones al inicio** | `auto_check_update` | `true` | Consulta silenciosamente los lanzamientos en GitHub Releases al iniciar y muestra un indicador si hay una versión más reciente. |

---

## Pestaña 2: Galería y protección de seguridad

| Campo de ajuste | Clave en `config.json` | Valor predeterminado | Descripción |
| :--- | :--- | :--- | :--- |
| **Tema de color** | `theme` | `"system"` | Paleta visual de la interfaz: `"system"`, `"midnight"` (oscuro OLED), `"graphite"` (gris neutro de estudio), `"violet"` (púrpura creativo) o `"light"` (claro). |
| **Difuminar contenido sensible (NSFW)** | `blur_nsfw` | `true` | Aplica una máscara de difuminado CSS sobre las creaciones marcadas como adultas o sensibles hasta que se hace clic sobre ellas. |
| **Mostrar distintivos en tarjetas** | `show_card_badges` | `true` | Muestra el formato (`PNG`, `MP4`), resolución (`1024×1024`), generador y puntuación directamente sobre las tarjetas de la galería. |
| **Lado máximo de miniaturas** | `thumbnail_max_edge` | `384` | Tamaño de borde de las miniaturas generadas (múltiplos de 64): `256` (Compacto), `384` (Estándar recomendado), `448` (HD), `512` (Ultra). |
| **Presupuesto de caché de miniaturas** | `thumbnail_cache_budget_mb` | `2048` | Espacio máximo en disco (en megabytes) asignado a la caché de miniaturas WebP. Al superarse, se eliminan automáticamente los niveles menos usados recientemente (LRU). |
| **Limpiar caché** | N/A | N/A | Botón de acción para purgar inmediatamente todas las miniaturas en caché del disco. |
| **Diagnóstico de miniaturas** | N/A | N/A | Abre las métricas en tiempo real del grupo de hilos Rayon y de la memoria caché LRU. |

---

## Pestaña 3: Pilas y ráfagas

| Campo de ajuste | Clave en `config.json` | Valor predeterminado | Descripción |
| :--- | :--- | :--- | :--- |
| **Habilitar apilamiento automático** | `auto_stack` | `true` | Agrupa de forma automática variaciones de generación consecutivas en tarjetas estilo baraja de cartas. |
| **Umbral de similitud de prompt** | `stack_similarity_threshold` | `0.85` | Puntuación mínima de similitud de Jaccard tokenizada (de 0.0 a 1.0) requerida para unir imágenes en una misma pila. |
| **Ventana de tiempo máxima (minutos)** | `stack_time_window_minutes` | `180` | Intervalo temporal máximo permitido entre generaciones consecutivas para considerarse parte de una misma ráfaga. |
| **Permitir varias pilas abiertas** | `allow_multiple_open_stacks` | `false` | Si es `false`, desplegar una pila contrae automáticamente cualquier otra abierta. Si es `true`, pueden permanecer varias pilas expandidas a la vez. |
| **Restablecer advertencias ocultas** | N/A | N/A | Reactiva todos los diálogos de advertencia (como el aviso de combinación de pilas) si marcó previamente «No volver a mostrar». |

---

## Pestaña 4: Interoperabilidad de generación

| Campo de ajuste | Clave en `config.json` | Valor predeterminado | Descripción |
| :--- | :--- | :--- | :--- |
| **URL base de ComfyUI** | `comfyui_url` | `"http://127.0.0.1:8188"` | Dirección HTTP de su servidor ComfyUI local. Incluye botón de prueba de conexión. |
| **URL base de SD WebUI** | `webui_url` | `"http://127.0.0.1:7860"` | Dirección HTTP de su instancia AUTOMATIC1111 / Forge / SD.Next. Incluye botón de prueba de conexión. |

---

## Pestaña 5: Equipo y base de datos

| Campo de ajuste | Clave en `config.json` | Valor predeterminado | Descripción |
| :--- | :--- | :--- | :--- |
| **Motor de base de datos** | `storage_backend` | `"sqlite"` | Motor de datos activo: `"sqlite"`, `"mysql"` o `"postgres"`. |
| **URL de conexión remota** | `remote_connection_url` | `""` | Cadena de conexión a la base de datos (p. ej., `postgres://user:pass@host:5432/berry_studio`). |
| **Identificador de estación (Client ID)**| `client_identifier` | `""` | Nombre único del equipo utilizado para el registro de cambios y la resolución de conflictos OCC. |
| **Asignaciones de raíz de almacenamiento**| `root_mappings` | `{}` | Mapeo de directorios multiplataforma que enlaza los UUIDs centrales de raíz con las rutas de montaje locales del SO. |
| **Probar conexión** | N/A | N/A | Envía un ping al servidor remoto de base de datos y muestra la latencia de red en milisegundos. |
| **Asistente de migración** | N/A | N/A | Abre el asistente para exportar esquemas y datos desde SQLite hacia MySQL/PostgreSQL. |

---

## Pestaña 6: Copia en la nube y sincronización

| Campo de ajuste | Clave en `config.json` | Valor predeterminado | Descripción |
| :--- | :--- | :--- | :--- |
| **Proveedor de almacenamiento** | `cloud_backup.provider` | `"local_path"` | Protocolo de almacenamiento: `"local_path"`, `"webdav"` o `"s3"`. |
| **URL / Credenciales WebDAV** | `cloud_backup.webdav_*` | `""` | URL del punto de enlace, usuario y contraseña para WebDAV en Nextcloud o Synology. |
| **Punto de enlace / Claves S3** | `cloud_backup.s3_*` | `""` | URL del servicio, nombre del bucket, región, Access Key y Secret Key para S3 / Cloudflare R2 / MinIO. |
| **Estrategia delta** | `cloud_sync.strategy` | `"fingerprint"` | `"fingerprint"` (tamaño + ETag) o `"checksum"` (verificación estricta mediante SHA-256). |
| **Hilos de transferencia concurrentes**| `cloud_sync.threads` | `4` | Número de subprocesos paralelos de subida para archivos multimedia. |
| **Límite de ancho de banda** | `cloud_sync.bandwidth_limit_kbs` | `0` | Velocidad máxima de subida en KB/s (0 = sin límite). |

---

## Pestaña 7: Resumen de analizadores de metadatos

Muestra el estado de diagnóstico en tiempo real de los analizadores sin pérdidas integrados:
- Analizador del bloque PNG `parameters` de AUTOMATIC1111 / SD.Next (Activo 🟢)
- Analizador de grafos de nodos de ComfyUI y JSON `workflow` (Activo 🟢)
- Analizador de bloques `Comment` y `Description` de NovelAI (Activo 🟢)
- Analizador de bloques de parámetros de Fooocus / Fooocus-MRE (Activo 🟢)
- Analizador JSON `sd-metadata` e `invokeai_metadata` de InvokeAI (Activo 🟢)
- Analizador de flujos de vídeo ISOBMFF para MP4 y EBML para WebM (Activo 🟢)

---

## Pestaña 8: Almacenamiento e info

- **Versión**: Muestra la versión actual de la aplicación (p. ej., `v0.3.0`).
- **Esquema de base de datos**: Informa del nivel de migración activo en SQLite (p. ej., `Esquema v14`).
- **Ubicación de la base de datos**: Ruta absoluta a `berry.db`.
- **Botones de acceso rápido a carpetas**:
  - `Abrir configuración`: Abre la carpeta que contiene `config.json`.
  - `Abrir base de datos`: Abre el directorio con `berry.db` y los diarios WAL.
  - `Abrir miniaturas`: Abre el directorio de caché de miniaturas WebP.
  - `Abrir modelos`: Abre el directorio con los pesos de los modelos ONNX.
