# 团队协作与多数据库工作站

对于设计工作室、游戏美术团队以及拥有多名艺术家协同创作的视觉机构，资产通常集中存储在共享网络存储（NAS、SMB、NFS）上。Omera 突破了单机 SQLite 的限制，可平滑扩展为**多数据库团队协同工作站**架构。

---

## 1. 多数据库协同架构

Omera 在底层通过异步 `StorageEngine` 特征对象实现了对三种数据库存储引擎的无缝抽象支持：

| 存储引擎后端 | 推荐团队规模 | 并发控制模型 | 性能指标基准 |
| :--- | :--- | :--- | :--- |
| **SQLite (单机默认)** | 1 位创作者 / 独立库 | 单写多读 WAL 模式 | 本地 NVMe 固态硬盘上查询延迟 <0.5 ms。 |
| **MySQL 8.0+ / MariaDB** | 2 至 50+ 位并发协作人员 | 行级锁机制与 `ngram` 全文索引分词 | 在 50 万+ 规模图库上保持 5 ms 以内高速响应。 |
| **PostgreSQL 14+** | 2 至 100+ 位并发协作人员 | MVCC 多版本并发、`tsvector` GIN 倒排索引与 `LISTEN/NOTIFY` 事件通道 | 超低延迟多机实时广播协同。 |

```mermaid
graph TD
    NAS[(工作室共享 NAS 阵列: SMB / NFS / WebDAV)]
    DB[(中央团队数据库: PostgreSQL 14+ / MySQL 8+)]

    subgraph 创作工作站 A (Windows)
        A_UI[Omera 前端工作台]
        A_Thumb[本机 NVMe 缩略图高速缓存]
        A_UI --- A_Thumb
    end

    subgraph 创作工作站 B (macOS)
        B_UI[Omera 前端工作台]
        B_Thumb[本机 NVMe 缩略图高速缓存]
        B_UI --- B_Thumb
    end

    A_UI -->|Z:\ai_vault| NAS
    B_UI -->|/Volumes/ai_vault| NAS

    A_UI <-->|OCC 乐观版本校验与同步| DB
    B_UI <-->|OCC 乐观版本校验与同步| DB
```

---

## 2. 跨平台共享存储根目录映射机制

在由 Windows、macOS 与 Linux 混合组成的多元工作室中，不同操作系统访问同一个 NAS 共享文件夹的文件路径表示方式截然不同：
- Windows 系统路径：`Z:\ai_vault\2026\character_01.png`
- macOS 挂载路径：`/Volumes/ai_vault/2026/character_01.png`
- Linux 挂载路径：`/mnt/nas/ai_vault/2026/character_01.png`

### Omera 的优雅破局方案：
1. **跨平台中立根 UUID**：Omera 为中央共享存储根目录分配全局唯一的存储根 UUID（持久化记录在 `storage_roots` 表）。
2. **规范化中立 URI**：在共享数据库中，所有物理文件的路径均以与平台无关的标准 URI 格式存储：
   ```
   omera://550e8400-e29b-41d4-a716-446655440000/2026/character_01.png
   ```
3. **客户端本地挂载映射**：在**首选项设置 > 团队协同与数据库**中，每位成员仅需将该根 UUID 映射到本机操作系统的具体挂载盘符或路径。Omera 在运行时动态秒级换算，实现 macOS 美术打上的标签，Windows 端同事瞬间即时可见。

---

## 3. 乐观并发控制 (OCC) 数据防护

当团队多名成员同时对同一批图片进行评分、打标或归档相册时，Omera 采用严谨的**乐观并发控制 (Optimistic Concurrency Control)** 杜绝脏写与数据覆盖：

- 数据库记录中每一行均维护一个递增的 `version` 版本戳字段。
- 成员提交评分变更时，触发 `set_file_rating_occ(file_id, new_rating, expected_version)`。
- **差异冲突解决策略**：
  - **评分与主封面图 Hero**：采用最后写入胜出（Last-Write-Wins / LWW）策略，UI 立即响应更新。
  - **标签与相册分类**：采用集合并集策略（Set-Union Merge），若成员 A 为图片打上 `"角色"`，成员 B 打上 `"概念草图"`，合并后两个标签均会完好保留。
  - **图片删除**：采用软删除（Soft-Deletion）标记，杜绝删除与修改冲突时的幽灵数据复活。

---

## 4. 多级实时协同同步架构

Omera 提供了三层由浅入深的团队同步机制：

1. **第 1 层：变更日志轮询 (Change Log Polling - 默认零配置)**：
   - Omera 每 3 秒向服务端的 `change_log` 事务流查询最新变动。开箱即用，无需对服务器进行复杂调优。
2. **第 2 层：PostgreSQL `LISTEN / NOTIFY` (<50 ms 极速广播)**：
   - 接入 PostgreSQL 时，Omera 自动建立异步长连接通知通道。任意一台电脑提交的任何元数据变动，均能在 50 毫秒内瞬间推送到其他所有成员屏幕上。
3. **第 3 层：分布式 WebSocket 集线器**：
   - 针对大型企业级私有化部署，可选挂载轻量级广播分发服务。

---

## 5. 客户端本地 NVMe 缩略图按需缓存

在千兆或万兆局域网中，频繁从 NAS 远程加载数万张缩略图会严重挤占宝贵的网络吞吐：
- Omera 将生成的 WebP 缩略图统一保存在各台创作电脑本机的**高速 NVMe 固态硬盘**上（`thumbnails/` 目录）。
- 美术师在浏览画廊时，仅在首次查看时按需生成并本地固化。
- 这使得在画廊中极速翻页刷图时的网络流量几乎为零，将所有的 NAS 局域网带宽留给高分辨率原图渲染与大模型训练。

---

## 6. 中央数据库迁移向导 (`MigrationWizardModal.vue`)

如果你最初使用的是单机 SQLite 个人图库，随着业务发展需要升级为多人团队数据库：
1. 打开**首选项设置 > 团队协同与数据库**。
2. 点击**“打开迁移向导...”**。
3. 迁移向导将自动预检本地 SQLite 图库数据，允许你选择 MySQL 8.0+ 或 PostgreSQL 14+，并生成针对特定 SQL 方言优化的完整 DDL 建表结构与批量事务 INSERT 导出脚本。
4. 按照向导给出的命令行终端指令，直接在数据库服务器中执行导入，即可无缝完成团队化升级。
