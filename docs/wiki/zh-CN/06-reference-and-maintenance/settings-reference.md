# 首选项与设置完整参考

通过菜单栏 `文件 > 首选项 / 设置...` 或快捷键 `Ctrl + ,` / `Cmd + ,` 即可随时唤起应用设置主窗口（`SettingsModal.vue`）。所有配置变更均实时安全持久化保存于本地应用数据目录下的 `config.json` 文件中。

---

## 标签页 1：常规偏好 (General Preferences)

| 配置字段 | 在 `config.json` 中的键名 | 默认值 | 功能说明与选项 |
| :--- | :--- | :--- | :--- |
| **界面显示语言** | `locale` | `"auto"` | 可选：`auto`（跟随操作系统）、`en`（英语）、`zh-CN`（简体中文）、`zh-TW`（繁体中文）、`ja`（日语）、`de`（德语）、`fr`（法语）、`es`（西班牙语）。 |
| **默认画廊视图** | `default_view` | `"grid"` | 启动软件时默认使用的图片展示方式：`"grid"`（网格瀑布流）、`"masonry"`（保留原始宽高比瀑布流）或 `"table"`（详细列表）。 |
| **启动时自动扫描** | `auto_scan` | `true` | 软件冷启动时，自动增量检查已添加文件夹中的新增或变动图片。 |
| **启动扫描冷却时间** | `startup_scan_interval_minutes`| `360` | 完整文件夹磁盘对齐的最小冷却时间间隔（分钟）：`30`、`60`、`360`（推荐 6 小时）、`1440`（24 小时）。避免频繁重启软件时产生磁盘无谓颠簸。 |
| **启动时自动检查更新** | `auto_check_update` | `true` | 软件启动时自动后台联网检测 GitHub 最新发布版本，若有更新则在界面优雅提示。 |

---

## 标签页 2：显示与安全保护 (Display & Safety)

| 配置字段 | 在 `config.json` 中的键名 | 默认值 | 功能说明与选项 |
| :--- | :--- | :--- | :--- |
| **界面主题配色** | `theme` | `"system"` | 视觉主题：`"system"`（跟随系统）、`"midnight"`（深夜纯黑）、`"graphite"`（石墨中性暗灰）、`"violet"`（紫罗兰创意紫）或 `"light"`（浅色）。 |
| **默认遮罩敏感内容 (NSFW)** | `blur_nsfw` | `true` | 对标记为敏感或成人分级的图片自动覆以高斯模糊遮罩，点击后方可临时揭开展开。 |
| **显示卡片角标** | `show_card_badges` | `true` | 在网格卡片上常驻显示格式（`PNG`、`MP4`）、尺寸（`1024×1024`）、生成工具徽章及星级评分。 |
| **缩略图分辨率规格 (64倍数优化)** | `thumbnail_max_edge` | `384` | 本地生成的 WebP 缩略图最大边长：`256`（紧凑 / 极省内存）、`384`（标准推荐平衡）、`448`（高清）、`512`（超清大屏）。 |
| **缩略图缓存磁盘预算** | `thumbnail_cache_budget_mb` | `2048` | 缩略图占用的磁盘空间上限（MB，默认 2GB）。超出上限后，系统基于 LRU 算法自动淘汰清理最早未访问的缓存层级。 |
| **清理缩略图缓存** | 无 | 无 | 一键清空本地磁盘上所有已生成的 WebP 缩略图缓存。 |
| **缩略图线程诊断** | 无 | 无 | 查看后台 Rayon 并发解码线程池及 LRU 内存命中率指标。 |

---

## 标签页 3：堆栈与批次 (Stacks & Bursts)

| 配置字段 | 在 `config.json` 中的键名 | 默认值 | 功能说明与选项 |
| :--- | :--- | :--- | :--- |
| **启用自动堆栈** | `auto_stack` | `true` | 自动将提示词相同或相近的连续生成图片合并为扑克牌折叠卡片。 |
| **提示词相似度聚合阈值** | `stack_similarity_threshold` | `0.85` | 聚合为同一堆栈所需的最低正向 Prompt 分词 Jaccard 相似系数（0.0 至 1.0）。 |
| **生成批次最大时间窗口** | `stack_time_window_minutes` | `180` | 判定为同一次跑批生成的最大时间间隔上限（分钟）。超出该时间跨度即便提示词完全一致也不会自动合并。 |
| **允许同时展开多个堆栈** | `allow_multiple_open_stacks` | `false` | 为 `false` 时，展开一个新堆栈会自动收起此前展开的堆栈；为 `true` 时支持画廊中多组堆栈同时处于展开平铺状态。 |
| **重置已隐藏的警告对话框** | 无 | 无 | 重新启用所有之前勾选了“不再显示此警告”的确认弹窗（例如堆栈合并安全提示）。 |

---

## 标签页 4：生成环境互通 (Generation Interop)

| 配置字段 | 在 `config.json` 中的键名 | 默认值 | 功能说明与选项 |
| :--- | :--- | :--- | :--- |
| **ComfyUI 基础地址** | `comfyui_url` | `"http://127.0.0.1:8188"` | 本地正在运行的 ComfyUI 服务 HTTP 终结点地址，配备“测试连接”探针。 |
| **SD WebUI 基础地址** | `webui_url` | `"http://127.0.0.1:7860"` | 本地 AUTOMATIC1111 / Forge / SD.Next 的 API 访问地址，配备“测试连接”探针。 |

---

## 标签页 5：团队协同与数据库 (Team & Collaboration)

| 配置字段 | 在 `config.json` 中的键名 | 默认值 | 功能说明与选项 |
| :--- | :--- | :--- | :--- |
| **数据库存储引擎** | `storage_backend` | `"sqlite"` | 驱动 Omera 图库的数据引擎：`"sqlite"`（单机离线）、`"mysql"` 或 `"postgres"`（团队网络共享）。 |
| **远程数据库连接串** | `remote_connection_url` | `""` | 远端数据库统一连接字符串（例如 `postgres://user:pass@192.168.1.100:5432/omera_studio`）。 |
| **工作站客户端标识 (Client ID)** | `client_identifier` | `""` | 用于在多机变更日志与 OCC 乐观锁中唯一标识当前工作站的友好名称。 |
| **共享存储根目录挂载映射** | `root_mappings` | `{}` | 跨平台文件系统挂载映射，将中央 NAS 根 UUID 绑定至当前电脑的本地挂载路径。 |
| **测试连接与延迟** | 无 | 无 | 发送网络探测包并即时反馈数据库响应网络往返延迟（ms）。 |
| **中央数据库迁移向导** | 无 | 无 | 呼出从 SQLite 批量导出转换至 MySQL/PostgreSQL 的数据迁移工具。 |

---

## 标签页 6：云端备份与快照 (Cloud Backup & Sync)

| 配置字段 | 在 `config.json` 中的键名 | 默认值 | 功能说明与选项 |
| :--- | :--- | :--- | :--- |
| **备份存储服务类型** | `cloud_backup.provider` | `"local_path"` | 远程存储协议：`"local_path"`（本地/NAS）、`"webdav"` 或 `"s3"`。 |
| **WebDAV 认证参数** | `cloud_backup.webdav_*` | `""` | WebDAV 根地址、登录用户名与应用授权访问令牌密码。 |
| **S3 协议终结点与凭据** | `cloud_backup.s3_*` | `""` | S3 Endpoint 终结点、Bucket 存储桶名、区域 Region、Access Key 与 Secret Key。 |
| **媒体差异比对策略** | `cloud_sync.strategy` | `"fingerprint"` | `"fingerprint"`（快速文件大小 + 远端 ETag 指纹）或 `"checksum"`（全量 SHA-256 校验）。 |
| **并发传输线程数** | `cloud_sync.threads` | `4` | 用于云端媒体增量同步的并发后台线程数（1 至 8）。 |
| **带宽限速 (KB/s)** | `cloud_sync.bandwidth_limit_kbs` | `0` | 最大上传带宽限制，`0` 表示不限速。 |

---

## 标签页 7：内置元数据解析引擎 (Metadata Parsers)

实时展示底层原生 Rust 无损元数据解析器的就绪与健康状态：
- AUTOMATIC1111 / SD.Next PNG `parameters` 数据块解析器 (运行就绪 🟢)
- ComfyUI 流程图节点网络与 `workflow` JSON 解析器 (运行就绪 🟢)
- NovelAI `Comment` 与 `Description` 解析器 (运行就绪 🟢)
- Fooocus / Fooocus-MRE 提示词与精炼器参数解析器 (运行就绪 🟢)
- InvokeAI `sd-metadata` 与 `invokeai_metadata` 解析器 (运行就绪 🟢)
- MP4 ISOBMFF 与 WebM EBML 视频流参数解析器 (运行就绪 🟢)

---

## 标签页 8：关于与数据存储 (Storage & About)

- **应用版本**：展示当前运行的程序正式版本号（例如 `v0.3.0`）。
- **数据库版本**：展示当前活动的 SQLite 表结构版本（例如 `数据库版本 v14`）。
- **本地 SQLite 数据库路径**：当前激活使用的 `omera.db` 绝对物理路径。
- **一键唤起系统文件管理器按钮**：
  - `打开偏好配置目录`：定位并查看 `config.json`。
  - `打开数据库所在目录`：直接访问存放 `omera.db` 与 WAL 日志的主目录。
  - `打开缩略图缓存目录`：访问本地 WebP 缩略图存放根目录。
  - `打开 AI 模型目录`：直达存放 CLIP 与 WD14 ONNX 权重的 `models/` 目录。
