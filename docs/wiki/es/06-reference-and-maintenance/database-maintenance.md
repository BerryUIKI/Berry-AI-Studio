# Mantenimiento de bases de datos y caché

Berry AI Studio está diseñado para funcionar continuamente sin requerir apenas mantenimiento. Sin embargo, a medida que califica, elimina y reorganiza decenas de miles de obras, realizar compactaciones periódicas de la base de datos y gestionar la caché en disco ayuda a mantener el rendimiento en su nivel óptimo.

---

## 1. Ventana de mantenimiento de base de datos (`DatabaseManagerModal.vue`)

Abra la ventana de mantenimiento desde **Archivo > Gestión de base de datos...** o desde el botón de la base de datos en el pie de herramientas rápidas de la barra lateral izquierda.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Mantenimiento de base de datos y compactación de almacenamiento    [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Métricas de almacenamiento                                             │
│ • Archivo de BD:      berry.db (Modo WAL)                              │
│ • Archivos indexados: 48.210 archivos en 6 carpetas                    │
│ • Espacio en disco:   128,4 MB (Base de datos) / 1,42 GB (Miniaturas)  │
│ • Álbumes y etiquetas:12 álbumes, 45 etiquetas                         │
│ • Versión del esquema:v14 (14 migraciones aplicadas)                   │
│ • Páginas libres:     1.420 páginas (~5,6 MB recuperables)             │
├────────────────────────────────────────────────────────────────────────┤
│ Operaciones de mantenimiento                                           │
│ [ 🧹 Ejecutar VACUUM ]             [ 💾 Exportar copia… ]              │
│ [ ↺ Restaurar desde copia… ]       [ 🗑 Limpiar caché de miniaturas ]  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Compactación de SQLite (`VACUUM`)

### ¿Qué es la fragmentación de la base de datos?
Al eliminar elementos, quitar etiquetas o disolver pilas, SQLite marca las páginas de disco subyacentes como «libres» (freelist) en lugar de reducir de inmediato el tamaño del archivo `.db` en disco.

### Ejecución de `VACUUM`:
- Al hacer clic en **«Ejecutar VACUUM»**, se activa la rutina nativa de compactación de SQLite.
- Berry reconstruye el archivo de base de datos en una estructura contigua y limpia, liberando las páginas vacías y reduciendo el tamaño físico del archivo en disco.
- **Seguridad**: La operación es completamente transaccional. Si el equipo se apaga durante la compactación, SQLite realiza una reversión automática sin riesgo de corrupción.

---

## 3. Copia de seguridad y restauración de la base de datos

### Creación de una copia local (`backup_database`)
- Haga clic en **«Exportar copia…»** para crear una copia de seguridad consistente y verificada sin bloquear la base de datos.
- Berry utiliza la API de copia en caliente de SQLite, permitiéndole generar el respaldo mientras continúa utilizando el estudio normalmente.

### Restauración desde una copia (`restore_database`)
- Si necesita restablecer su biblioteca en un equipo nuevo o deshacer modificaciones erróneas, haga clic en **«Restaurar…»**.
- Berry creará una copia de seguridad de seguridad previa (`berry.db.rollback`), aplicará el archivo restaurado y recargará inmediatamente el estado completo de la biblioteca en el estudio.

---

## 4. Gestión de la caché de miniaturas y expulsión LRU

Las miniaturas se guardan en `<app_data_dir>/thumbnails/` en formato WebP de alta compresión.

### Presupuesto configurable de caché en disco:
- En **Preferencias > Galería**, puede configurar el **Presupuesto de caché de miniaturas** (predeterminado: `2048 MB` / 2 GB).
- Berry registra la fecha de último acceso de cada miniatura en la tabla de base de datos `thumbnail_cache_entries`.
- Cuando el tamaño acumulado de las miniaturas supera este presupuesto, Berry elimina automáticamente los niveles menos utilizados recientemente mediante una política **LRU (Least Recently Used)**.

### Purga de la caché:
- Si desea recuperar espacio en disco de inmediato, pulse **«Limpiar caché de miniaturas»**.
- Berry eliminará todos los archivos WebP en caché y reiniciará la tabla de registros. Las miniaturas se volverán a generar bajo demanda la próxima vez que explore sus carpetas.

---

## 5. Diagnóstico de miniaturas (`ThumbnailDiagnosticsModal.vue`)

Para monitorizar el rendimiento y depurar colas de trabajo:
- Abra **Preferencias > Galería > Diagnóstico** para consultar:
  - Número de hilos trabajadores activos de Rayon (p. ej., `4 hilos en ejecución`).
  - Colas de generación de miniaturas pendientes.
  - Trabajos de generación finalizados frente a cancelados.
  - Tasa de aciertos de la caché en memoria LRU.
