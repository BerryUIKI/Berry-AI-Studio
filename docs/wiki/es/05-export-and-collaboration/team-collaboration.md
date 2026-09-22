# Estudio de equipo con base de datos compartida

Para estudios de diseño, empresas de videojuegos y agencias visuales donde múltiples artistas trabajan sobre un almacenamiento de red compartido (NAS, SMB, NFS), Berry AI Studio puede escalar más allá del SQLite local y transformarse en un **Estudio de colaboración en equipo multibase de datos**.

---

## 1. La arquitectura multibase de datos

Berry ofrece una capa de abstracción asíncrona `StorageEngine` compatible con tres motores:

| Motor de base de datos | Tamaño de equipo recomendado | Modelo de concurrencia | Perfil de rendimiento |
| :--- | :--- | :--- | :--- |
| **SQLite (Predeterminado)** | 1 usuario por biblioteca | Monousuario escritura / Multi-lector WAL | Latencia <0,5 ms en discos SSD NVMe locales. |
| **MySQL 8.0+ / MariaDB** | De 2 a más de 50 usuarios concurrentes | Bloqueo a nivel de fila e índices de texto `ngram` | Consultas sub-5ms en bibliotecas compartidas de más de 500.000 activos. |
| **PostgreSQL 14+** | De 2 a más de 100 usuarios concurrentes | MVCC, índices GIN con `tsvector` y `LISTEN/NOTIFY` | Colaboración en tiempo real de bajísima latencia con difusión de eventos. |

```mermaid
graph TD
    NAS[(NAS compartido de estudio: SMB / NFS / WebDAV)]
    DB[(BD central de estudio: PostgreSQL 14+ / MySQL 8+)]

    subgraph Estación de trabajo A (Windows)
        A_UI[Interfaz Berry]
        A_Thumb[Caché de miniaturas NVMe local]
        A_UI --- A_Thumb
    end

    subgraph Estación de trabajo B (macOS)
        B_UI[Interfaz Berry]
        B_Thumb[Caché de miniaturas NVMe local]
        B_UI --- B_Thumb
    end

    A_UI -->|Z:\ai_vault| NAS
    B_UI -->|/Volumes/ai_vault| NAS

    A_UI <-->|Comprobación de versiones OCC y Sync| DB
    B_UI <-->|Comprobación de versiones OCC y Sync| DB
```

---

## 2. Asignación multiplataforma de raíces de almacenamiento

Un desafío recurrente en estudios con sistemas operativos mixtos radica en que Windows, macOS y Linux utilizan formatos de ruta distintos para acceder a la misma carpeta de red compartida:
- Windows: `Z:\ai_vault\2026\character_01.png`
- macOS: `/Volumes/ai_vault/2026/character_01.png`
- Linux: `/mnt/nas/ai_vault/2026/character_01.png`

### Solución de Berry:
1. **UUIDs de raíz universales**: Berry genera un UUID único para la raíz del almacenamiento compartido (almacenado en la tabla `storage_roots`).
2. **URIs normalizadas**: En la base de datos, las rutas se guardan con formato neutro e independiente de la plataforma:
   ```
   berry://550e8400-e29b-41d4-a716-446655440000/2026/character_01.png
   ```
3. **Mapeo de montaje en el cliente**: En **Preferencias > Equipo y base de datos**, cada artista asigna el UUID de raíz a la ruta de montaje de su propio sistema operativo. Berry traduce las rutas dinámicamente sobre la marcha: una obra etiquetada por un artista en macOS es accesible al instante para otro en Windows.

---

## 3. Control de concurrencia optimista (OCC)

Cuando varios miembros del equipo califican, etiquetan o clasifican la misma colección de manera simultánea, Berry evita la corrupción de datos mediante el **Control de concurrencia optimista (OCC)**:

- Cada registro de activo incluye una columna de versión a nivel de fila (`version`).
- Cuando un artista actualiza una puntuación, Berry envía la llamada `set_file_rating_occ(file_id, new_rating, expected_version)`.
- **Políticas de resolución de conflictos**:
  - **Puntuaciones y portadas principales**: Prevalece la última escritura (Last-Write-Wins - LWW) con actualización inmediata en la interfaz.
  - **Etiquetas y álbumes**: Fusión por unión de conjuntos (si el Artista A añade la etiqueta `"Personaje"` y el Artista B añade `"Concepto"`, ambas etiquetas se conservan).
  - **Eliminaciones**: Marcadores de borrado lógico que evitan reapariciones fantasma accidentales.

---

## 4. Niveles de sincronización colaborativa en tiempo real

Berry sincroniza los cambios de estado del equipo mediante una arquitectura de tres niveles:

1. **Nivel 1: Sondeo del registro de cambios (Predeterminado / Sin configuración)**:
   - Berry consulta la tabla de diario `change_log` cada 3 segundos en busca de nuevos identificadores de transacción. No requiere configuración adicional en el servidor.
2. **Nivel 2: PostgreSQL `LISTEN / NOTIFY` (Latencia <50 ms)**:
   - Al usar PostgreSQL, Berry establece un canal de notificaciones asíncrono. Cualquier cambio efectuado en una estación de trabajo se transmite a los demás clientes en menos de 50 ms.
3. **Nivel 3: Concentrador distribuido por WebSockets**:
   - Servicio de difusión opcional de alta velocidad pensado para grandes infraestructuras corporativas.

---

## 5. Caché de miniaturas bajo demanda en el cliente

Transferir miles de miniaturas a través de la red local (incluso en 1 Gbps o 10 Gbps) puede congestionar el ancho de banda del NAS:
- Berry guarda las miniaturas WebP reducidas en la unidad **SSD NVMe local** de cada estación (`thumbnails/`).
- Al explorar la biblioteca compartida, las miniaturas se generan y almacenan en la caché local bajo demanda.
- Esto mantiene el tráfico de red prácticamente en cero durante desplazamientos rápidos por la galería, dejando el canal del NAS totalmente libre para renderizados de alta resolución o entrenamiento de modelos.

---

## 6. Asistente de migración: SQLite a MySQL / PostgreSQL (`MigrationWizardModal.vue`)

Si comenzó con una biblioteca SQLite monousuario y desea migrar a una base de datos centralizada de equipo:
1. Abra **Preferencias > Equipo y base de datos**.
2. Haga clic en **«Abrir asistente de migración...»**.
3. Berry inspeccionará su base de datos SQLite local, le permitirá elegir entre MySQL 8.0+ o PostgreSQL 14+ y generará los esquemas DDL optimizados para el dialecto junto a los archivos transaccionales SQL de migración por lotes.
4. Ejecute el script resultante en su servidor de base de datos para migrar la biblioteca completa del estudio.
