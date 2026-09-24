# Pilas de imágenes, ráfagas y comparación

Los creadores de IA generativa suelen generar lotes de 10 a 50 variaciones con prompts idénticos o ligeramente modificados para encontrar la mejor composición. Sin herramientas de curación específicas, esto inunda la biblioteca con borradores casi duplicados.

Omera resuelve este problema mediante el **Apilamiento inteligente de ráfagas**, **Tarjetas en baraja de póquer**, **Selección autorizada de portadas (Hero)** y **Comparación lado a lado**.

---

## 1. Cómo funciona el apilamiento

Una **Pila de imágenes** es una colección de variaciones relacionadas agrupadas y representadas en el lienzo de la galería por una única imagen de portada denominada **Hero** (portada principal).

```mermaid
flowchart LR
    subgraph Lote generativo
        A[Variación 1 - Semilla 101]
        B[Variación 2 - Semilla 102 (Hero 5★)]
        C[Variación 3 - Semilla 103]
        D[Variación 4 - Semilla 104]
    end

    subgraph Representación en pila
        E["Tarjeta baraja póquer [Distintivo: 4] (Hero: Variación 2)"]
    end

    A & B & C & D -->|Agrupación manual o automática| E
```

### Pilas contraídas frente a expandidas
- **Contraída (por defecto)**: Se muestra como una única tarjeta en baraja de póquer con sutiles capas visibles detrás y un distintivo numérico de recuento (p. ej., `[ 4 ]`).
- **Expandida**: Al hacer clic en el distintivo de recuento, se despliegan todas las variaciones miembro directamente en la cuadrícula de la galería, permitiendo puntuarlas individualmente, inspeccionarlas o eliminarlas.

---

## 2. Apilamiento automático de ráfagas (`auto_stack_images`)

Omera puede detectar y agrupar automáticamente ráfagas de generación consecutivas en segundo plano:

### Criterios de agrupación:
1. **Similitud de tokens del prompt**: Calcula la similitud de Jaccard tokenizada sobre los prompts positivos. Puede configurar el umbral requerido en **Preferencias > Pilas y ráfagas** (predeterminado: `0.85` / 85% de coincidencia).
2. **Proximidad temporal**: Las ráfagas generativas se producen en rápida sucesión. Omera agrupa las variaciones creadas dentro de una ventana de tiempo configurable (predeterminado: `180 minutos`).
3. **Ejecución**: Puede ejecutar el autoapilamiento a petición mediante **Herramientas > Organizar biblioteca por prompt (Todas las carpetas o Carpeta actual)**, o permitir que Omera agrupe las imágenes automáticamente durante la ingesta.

---

## 3. Operaciones de apilamiento manual

Puede crear, deshacer y ajustar pilas mediante atajos de teclado:

| Acción | Atajo | Descripción |
| :--- | :--- | :--- |
| **Agrupar en pila** | `Ctrl + G` / `Cmd + G` | Combina todas las imágenes individuales o pilas seleccionadas en una única pila. |
| **Desapilar grupo** | `Ctrl + Shift + G` / `Cmd + Shift + G` | Deshace la pila seleccionada y devuelve las imágenes a tarjetas independientes. |
| **Establecer como portada (Hero)** | `Alt + S` / `Option + S` | Asigna la imagen activa como la portada principal de la pila (`stack_order = 0`). |

### Seguridad al combinar pilas y aplanamiento
En Omera, las pilas **no se pueden anidar** (no es posible colocar una pila dentro de otra). Cuando selecciona varias pilas y pulsa `Ctrl + G`:
- Omera aplana automáticamente todas las pilas de origen dentro de la pila de destino.
- Aparece un cuadro de confirmación (`StackMergeWarningModal.vue`) para prevenir agrupaciones accidentales.
- Puede marcar *«No volver a mostrar esta advertencia»* (puede restablecerse en **Preferencias > Pilas y ráfagas > Restablecer advertencias**).

---

## 4. Árbitro de gestos: Clic simple frente a doble clic

Para asegurar una interacción fluida en la galería sin conflictos entre gestos:
- **Clic simple en el distintivo de recuento**: Expande o contrae la pila directamente en la cuadrícula.
- **Clic simple en el cuerpo de la tarjeta**: Selecciona la pila (con un temporizador diferido de 240 ms).
- **Doble clic en el cuerpo de la tarjeta**: Cancela el temporizador de expansión inmediatamente y abre la imagen **Hero** en la vista rápida a pantalla completa (**Lightbox**).

---

## 5. Comparación sincronizada lado a lado (`CompareModal.vue`)

Al evaluar pequeñas diferencias (p. ej., renderizado de ojos, anatomía de manos o iluminación):
1. Seleccione dos imágenes en la galería.
2. Pulse `C` (o haga clic en **Comparar**).
3. Se abrirá la ventana modal de **Comparación lado a lado**:
   - El **Espacio A (Izquierda)** y el **Espacio B (Derecha)** muestran ambas imágenes una al lado de la otra.
   - **Zoom y desplazamiento sincronizados**: Al arrastrar o ampliar con la rueda del ratón en cualquiera de los dos visores, ambas imágenes se mueven al unísono, permitiendo comparaciones pixel a pixel al 100% sin esfuerzo.
   - **Comparación de metadatos en el HUD**: Muestra las diferencias en Semilla, Modelo, Pasos y Escala CFG.
   - **Botón «Establecer como portada»**: Pulse para fijar la imagen seleccionada como la portada principal de su pila de ráfaga.

---

## 6. Herramienta de eliminación de borradores por lotes (`CullDraftsModal.vue`)

Una vez elegida su imagen Hero ganadora y puntuados sus favoritos de 3★ o más:
1. Seleccione la pila y haga clic en **«Eliminar borradores»** en la barra flotante de acciones por lotes.
2. La ventana modal de descarte muestra todos los archivos candidatos secundarios calificados por debajo de su umbral.
3. Haga clic en **«Mover a la papelera»** para enviar las variaciones descartadas a la papelera de reciclaje de su sistema operativo en un solo clic, liberando espacio en disco mientras conserva sus mejores creaciones.
