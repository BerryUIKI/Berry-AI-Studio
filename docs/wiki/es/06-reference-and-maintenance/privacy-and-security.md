# Arquitectura de privacidad y seguridad

Omera está desarrollado bajo una filosofía de diseño **100% centrado en lo local (local-first) y con cero telemetría**. En una época donde los flujos de trabajo de IA generativa involucran estilos artísticos propietarios, conceptos privados de personajes y activos confidenciales de clientes, Omera garantiza que su trabajo creativo permanezca estrictamente en su equipo.

---

## 1. Telemetría cero y funcionamiento sin conexión

### Cero comunicaciones no autorizadas
- Omera contiene **cero píxeles de seguimiento, cero SDKs analíticos y cero servicios de reporte de fallos** (sin Google Analytics, Sentry, Mixpanel ni PostHog).
- Puede ejecutar Omera completamente desconectado de Internet o tras cortafuegos corporativos aislados (air-gapped) sin merma alguna de su funcionalidad.

### Solicitudes de red salientes
Omera efectúa conexiones de red **únicamente** bajo tres circunstancias explícitas:
1. **Actualizaciones de la aplicación**: Cuando **Buscar actualizaciones al inicio** está activo (o se solicita manualmente mediante `Ayuda > Buscar actualizaciones...`), Omera consulta la API pública de GitHub Releases (`https://api.github.com/repos/BerryUIKI/Omera/releases/latest`).
2. **Resolución de hashes en Civitai**: Cuando pulsa explícitamente el botón para consultar en Civitai un hash de checkpoint no reconocido, Omera realiza una consulta HTTP al servicio público de Civitai.
3. **Copia en la nube y sincronización de equipo**: Cuando configura un punto de enlace de AWS S3, WebDAV o un servidor central de PostgreSQL/MySQL en Preferencias.

---

## 2. Inferencia de IA exclusivamente local

Todas las funciones de aprendizaje automático en Omera se ejecutan íntegramente en la CPU o GPU de su máquina mediante sesiones locales de **ONNX Runtime** (`ort`):

- **Embeddings de CLIP / SigLIP**: El preprocesamiento de imágenes y la codificación vectorial se efectúan en local. Ningún prompt, consulta de texto o píxel se transmite a servidores remotos.
- **Autoetiquetado anime WD14**: La inferencia de la red neuronal se ejecuta sobre pesos locales (`models/`). Las predicciones de etiquetas se escriben directamente en su base de datos local SQLite.

---

## 3. Exportación y saneamiento orientados a la privacidad

Al exportar o empaquetar obras de arte para su publicación en Internet, Omera incluye un **motor de saneamiento de metadatos en 4 niveles**:

- Puede eliminar por completo grafos de nodos incrustados de ComfyUI, prompts positivos y negativos, semillas y etiquetas LoRA con un solo clic antes de compartir sus imágenes en redes sociales o Discord (consulte [Exportación y muestra web](../05-export-and-collaboration/export-and-web-showcase.md)).
- El nivel de **Limpieza total** elimina todos los bloques EXIF e ICC, produciendo un archivo con únicamente los datos brutos de los píxeles.

---

## 4. Transparencia de código abierto y licencia

Omera es software libre y de código abierto publicado bajo la licencia **GNU Affero General Public License v3.0 (AGPL-3.0)**:

- Cada línea de código del backend en Rust, de los comandos del puente de Tauri y de los componentes frontend en Vue 3 es públicamente auditable en [GitHub](https://github.com/BerryUIKI/Omera).
- Dispone de total libertad para auditar, compilar, bifurcar (fork) o desplegar Omera dentro de su espacio de trabajo personal o en entornos comerciales de estudio.
