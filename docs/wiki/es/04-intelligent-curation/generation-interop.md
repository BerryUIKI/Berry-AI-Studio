# Interoperabilidad de generación

Berry AI Studio actúa como el compañero de trabajo activo para sus herramientas de generación creativa, ofreciendo comunicación bidireccional mediante API con **ComfyUI** y **AUTOMATIC1111 / SD.Next**.

---

## 1. Configuración de los servicios de generación

Configure los puntos de enlace de conexión en **Preferencias > Interoperabilidad de generación**:

- **URL base de ComfyUI**: Predeterminado `http://127.0.0.1:8188`
- **URL base de SD WebUI**: Predeterminado `http://127.0.0.1:7860`

Cada campo de configuración incluye un botón para **«Probar conexión»**. Berry envía una solicitud ligera de diagnóstico (`/system_stats` para ComfyUI o `/sdapi/v1/options` para WebUI) y muestra un distintivo de estado **En línea (🟢)** o **Desconectado (🔴)**.

---

## 2. Envío de flujos de trabajo a ComfyUI (`send_to_comfyui`)

Al examinar una obra o vídeo generado en ComfyUI:
1. En el Inspector de propiedades derecho, localice la tarjeta **Interoperabilidad de generación**.
2. Haga clic en **«Enviar a ComfyUI»**.
3. El backend de Berry se conecta al punto de enlace HTTP `/prompt` de ComfyUI y envía exactamente el mismo grafo de nodos y parámetros latentes extraídos de la imagen.
4. Su instancia de ComfyUI carga de inmediato el flujo de trabajo y lo sitúa en la cola de renderizado, sin tener que arrastrar y soltar archivos entre ventanas.

---

## 3. Envío de parámetros a SD WebUI (`send_to_webui`)

Para imágenes generadas en AUTOMATIC1111, Forge o SD.Next:
1. Haga clic en **«Enviar a SD WebUI»** en el Inspector.
2. Berry genera una carga útil para txt2img con el prompt positivo, prompt negativo, pasos, sampler, escala CFG, semilla y resolución.
3. La solicitud se envía a la API `/sdapi/v1/txt2img` de WebUI, rellenando automáticamente los campos de la interfaz o lanzando una nueva renderización.

---

## 4. Ciclo de vigilancia activa del flujo de generación

Al combinar la **Interoperabilidad de generación** con una **Carpeta en Modo C (Flujo de ingesta AIGC)**:
1. Envía un flujo de trabajo a ComfyUI o WebUI directamente desde Berry.
2. El motor generativo renderiza la imagen y la guarda en su carpeta de salida.
3. El monitor en segundo plano de Berry detecta el nuevo archivo, estabiliza el bloqueo de escritura (retardo de 500 ms), extrae los metadatos, genera la miniatura WebP y sitúa la nueva creación en la parte superior de la galería en tiempo real.
4. Se completa así un bucle creativo continuo y sin interrupciones entre la fase de generación y la de curación visual.
