# Resolución de problemas y preguntas frecuentes (FAQ)

Esta guía recopila soluciones a problemas comunes, casos límite y preguntas frecuentes sobre Berry AI Studio.

---

## 1. Resolución de problemas habituales

### Problema: Las nuevas imágenes de mi generador de IA no aparecen en la galería
- **Causa**: Es posible que la carpeta no esté configurada como un Flujo de ingesta AIGC activo, o que el monitor de archivos esté esperando a que el generador finalice la escritura en disco (período de retardo).
- **Solución**:
  1. Compruebe que la carpeta esté registrada en la barra lateral izquierda.
  2. Si utiliza AUTOMATIC1111 o ComfyUI, asegúrese de que la carpeta esté configurada en **Modo C: Flujo de ingesta** para activar la estabilización anti-bloqueo de escritura.
  3. Haga clic derecho sobre la carpeta en la barra lateral y seleccione **«Recolectar imágenes»** o **«Escanear»** para forzar una comprobación inmediata.

### Problema: Las miniaturas tardan en cargar o muestran cuadros grises provisionales
- **Causa**: En importaciones iniciales muy voluminosas, los hilos de trabajo de Rayon reducen y codifican las imágenes en segundo plano por lotes.
- **Solución**:
  1. Haga clic en **⚡ Actividad** en la barra de estado inferior para verificar si la cola de procesamiento de miniaturas sigue trabajando.
  2. En **Preferencias > Galería**, asegúrese de que el **Presupuesto de caché de miniaturas** esté fijado al menos en `2048 MB`.
  3. Evite ejecutar renderizados pesados de vídeo o videojuegos de alta exigencia simultáneamente durante la primera importación masiva de carpetas.

### Problema: La búsqueda devuelve 0 resultados cuando estoy seguro de que el prompt contiene la palabra
- **Causa**: Puede encontrarse en **Modo Semántico (`🧠`)** en lugar del **Modo Sintaxis (`🔍`)**, o filtrando por un campo inexistente.
- **Solución**:
  1. Revise el icono en la barra de búsqueda: pulse el icono del cerebro para regresar a la **Búsqueda por sintaxis (`🔍`)**.
  2. Si busca frases exactas con espacios, encierre los términos entre comillas: `prompt:"cyberpunk city"`.
  3. Abra el **Panel de filtros (`☰ Filtros`)** y pulse **«Restablecer»** para asegurarse de que no haya filtros activos (como un filtro de 5 estrellas) ocultando los resultados.

### Problema: Las pilas aparecen separadas o se muestran tarjetas duplicadas
- **Causa**: Una pila puede fragmentarse si las imágenes miembro se renombraron o movieron externamente desde el explorador de archivos del sistema operativo.
- **Solución**: Seleccione las tarjetas afectadas en la galería y pulse `Ctrl + G` para reagruparlas de forma limpia en una única pila unificada.

### Problema: «Enviar a ComfyUI» muestra un error de conexión rechazada
- **Causa**: ComfyUI no se encuentra en ejecución local o está escuchando en un puerto diferente.
- **Solución**:
  1. Abra **Preferencias > Interoperabilidad de generación**.
  2. Verifique que la **URL base de ComfyUI** coincida con la que muestra su terminal (predeterminada: `http://127.0.0.1:8188`).
  3. Haga clic en **«Probar conexión»** para comprobar que el puerto responde.

---

## 2. Preguntas frecuentes (FAQ)

### ¿Berry AI Studio es completamente gratuito?
Sí. Berry AI Studio es software libre y de código abierto bajo licencia **AGPL-3.0**. No incluye suscripciones, muros de pago ni funcionalidades bloqueadas.

### ¿Puede Berry AI Studio gestionar bibliotecas con más de 100.000 o 500.000 archivos?
Sí. Berry ha sido concebido desde sus cimientos para manejar colecciones inmensas mediante:
- **Paginación profunda por cursor Keyset** (`search_files_cursor_page`) que mantiene tiempos de respuesta inferiores al milisegundo sin importar el tamaño del catálogo.
- **SQLite en modo Write-Ahead Logging (WAL)** para lecturas no bloqueantes de alto rendimiento.
- **Virtualización dinámica del DOM** que renderiza exclusivamente los elementos visibles en pantalla.

### ¿Berry AI Studio sube mis prompts o imágenes a la nube?
No. Todo el escaneo, extracción de metadatos, almacenamiento en base de datos e inferencia de IA (CLIP y WD14) se ejecutan 100% en local en su ordenador. No se recopila telemetría ni analítica alguna.

### ¿Puedo arrastrar imágenes de Berry directamente a ComfyUI o Discord?
Sí. Al arrastrar una tarjeta desde la galería hacia su navegador web o cualquier otra aplicación externa se emiten datos estándar de transferencia de archivos del sistema operativo, conservando intactos todos los metadatos incrustados.

### ¿Qué sucede si elimino una carpeta de la barra lateral izquierda?
Al quitar una carpeta de Berry AI Studio se eliminan la carpeta y sus registros indexados de la base de datos de Berry. **En ningún caso se borran ni mueven sus archivos multimedia del disco.**

### ¿Varios miembros de un equipo pueden colaborar sobre la misma biblioteca?
Sí. Cambiando el motor de SQLite a un servidor compartido de **MySQL 8.0+** o **PostgreSQL 14+** en **Preferencias > Equipo y base de datos**, múltiples creadores pueden conectarse a una biblioteca de red compartida con sincronización en tiempo real y asignación de rutas multiplataforma.
