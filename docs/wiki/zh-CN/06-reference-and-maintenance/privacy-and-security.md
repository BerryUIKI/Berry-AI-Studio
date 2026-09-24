# 隐私架构与安全规范

Omera 始终恪守 **100% 本地优先 (Local-First)、零数据遥测 (Zero-Telemetry)** 的安全设计理念。在商业生成式 AI 创作流中，创作者的作品往往包含独家提示词配方、私密人物形象草稿以及高保密的商业客户资产。Omera 从底层架构上彻底保障你的所有创意资产 100% 留在你的物理电脑中。

---

## 1. 零遥测收集与离线运行保障

### 绝无隐蔽数据回传机制 (No Phone-Home)
- Omera **不包含任何网络追踪打点探针、不包含任何用户行为分析 SDK、亦不包含任何崩溃上报组件**（绝不引入 Google Analytics、Sentry、Mixpanel 或 PostHog）。
- 即使将电脑完全切断互联网连接，或置于严苛的企业级物理隔离局域网（Air-Gapped Environment）中，Omera 的所有核心功能均不受任何影响，丝滑运行。

### 唯一的对外网络通信场景清单
Omera 仅在用户明确主动触发的三种特定场景下才会产生网络请求：
1. **检查版本更新**：当开启了“启动时自动检查更新”（或在菜单中手动点击 `帮助 > 检查更新...`），Omera 会访问 GitHub 官方公开 Releases API（`https://api.github.com/repos/BerryUIKI/Omera/releases/latest`）。
2. **Civitai 模型哈希反查**：仅当你主动点击检查器中未知哈希旁的 Civitai 放大镜图标时，Omera 才会针对该 Hash 向 Civitai 公开 API 发起单次只读反查。
3. **云备份与团队协同同步**：仅当你在首选项设置中明确主动配置了 AWS S3、WebDAV 账号或局域网中心 PostgreSQL/MySQL 数据库连接串时。

---

## 2. 100% 本地化离线 AI 深度学习推理

Omera 内置的所有机器学习特征提取与自动打标能力，均通过嵌入式 **ONNX Runtime**（Rust 原生 `ort` 绑定）在你的本地 CPU 或 GPU 上纯离线计算：

- **CLIP / SigLIP 文本与视觉向量嵌入**：图像预处理、特征矩阵变换与向量生成完全在本地内存中完成。绝无任何提示词、文本查询或图像原始像素被传送到外部云端。
- **WD14 Danbooru 动漫自动打标**：神经网络前向推理完全依托本地下载的模型权重（`models/`）。预测得出的标签直接入库本地 SQLite，绝无云端处理环节。

---

## 3. 隐私优先的导出脱敏与净化

在将艺术作品导出并公开发布至社交网络或向公众展示前，Omera 提供了严谨的 **4 级元数据脱敏与隐私净化引擎**：

- 在将图片上传至社交平台或 Discord 前，只需轻轻一点，即可彻底擦除内嵌在图片中的 ComfyUI 流程图 JSON、正负向提示词字符串、随机种子以及 LoRA 标识（详见[批量导出与静态网页画册](../05-export-and-collaboration/export-and-web-showcase.md)）。
- 最高等级的**完全净化 (Full Clean)** 模式甚至会抹除所有 EXIF 相机参数头与 ICC 颜色配置文件，仅输出纯净的原始位图像素，彻底消除信息泄露隐患。

---

## 4. 开源透明性与软件授权协议

Omera 是基于 **GNU Affero 通用公共许可证第 3 版 (AGPL-3.0)** 发布的自由开源软件：

- 所有的 Rust 底层后台源码、Tauri 跨语言通信命令绑定以及 Vue 3 前端界面组件代码，均在 [GitHub 官方仓库](https://github.com/BerryUIKI/Omera) 上百分之百开源透明，接受全球开发者的公开审计。
- 你拥有完全的自由在个人工作机、企业工作室或商业生产环境中审计、编译、定制或部署 Omera。
