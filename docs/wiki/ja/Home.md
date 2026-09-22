# Berry AI Studio — 公式Wiki＆ユーザーガイド

**Berry AI Studio** (`v0.3.0`) の公式ユーザーマニュアルおよびナレッジベースへようこそ。

Berry AI Studio は、AI生成クリエイター、プロンプトエンジニア、ビジュアルデザインスタジオ向けに設計された、オープンソースかつローカルファーストのメディアアセットマネージャー＆プロンプトワークベンチです。**Tauri v2**、**Rust**、**Vue 3** を基盤に構築され、数百枚の作品から50万枚以上の大規模ライブラリまで、ミリ秒未満の検索クエリ応答、完全クラウド非依存（ローカル動作）、包括的なAI生成メタデータ抽出を実現します。

---

## 🧭 ナビゲーション＆目次

### [第1章：導入と基本操作](01-getting-started/installation.md)
- **[インストールとシステム要件](01-getting-started/installation.md)**: ハードウェア要件、Windows インストーラー／ポータブル版、macOS Universal／Apple Silicon ビルド、Linux AppImage／deb、初期設定ウィザード。
- **[ワークスペースと画面構成](01-getting-started/workspace-layout.md)**: フレームレスウィンドウ、メニューバー、3ペイン構成（サイドバー、ギャラリー、インスペクター）、ステータスバー、フローティング一括アクションバーの解説。
- **[キーボードショートカット一覧](01-getting-started/keyboard-shortcuts.md)**: グローバルショートカット、アンカー選択、ブラインド星評価、クイックインスペクション、ナビゲーションキー。

### [第2章：ライブラリ管理と閲覧](02-library-management/folder-modes-and-import.md)
- **[メディアのインポートとフォルダモード](02-library-management/folder-modes-and-import.md)**: モードA（外部リンク）、モードB（管理プロジェクト保管庫）、モードC（デバウンス監視＆遅延クリーンアップ対応 AIGC パイプライン）。対応画像（PNG, WebP, JPEG）および動画（MP4, WebM）フォーマット。
- **[ギャラリー表示モードと表示オプション](02-library-management/gallery-views.md)**: グリッド表示（130px〜360pxのズーム対応）、ウォーターフォール（元の縦横比を維持）、リスト表示、類似画像検索マッチビュー。カードバッジと NSFW センシティブぼかし。
- **[整理・評価・タグ管理](02-library-management/organization-and-tags.md)**: 10段階評価（星評価）、お気に入り、カスタムアルバム、8色のカラータグ分類、一括ドラッグ＆ドロップ、一括アクションバー。
- **[動画・モーションメディア対応](02-library-management/video-support.md)**: AnimateDiff、Wan2.1、HunyuanVideo、SVD 再生；コマ送り／コマ戻し、ループ・速度制御 HUD、埋め込み動画ワークフロー解析。

### [第3章：検索・絞り込み・メタデータ分析](03-discovery-and-analytics/search-and-filtering.md)
- **[検索構文とビジュアルフィルター](03-discovery-and-analytics/search-and-filtering.md)**: 高度なキー・バリュー型クエリ構文（`prompt:`, `neg:`, `model:`, `cfg:>=7`, `steps:20..40`）、数値範囲指定、スライドアウト式フィルタードロワー。
- **[AIGC メタデータとプロンプトインスペクション](03-discovery-and-analytics/metadata-and-prompts.md)**: AUTOMATIC1111、ComfyUI、NovelAI、Fooocus、InvokeAI の可逆メタデータ抽出；トークン化されたインタラクティブなプロンプトチップと生実行グラフ。
- **[プロンプト分析とインサイト](03-discovery-and-analytics/prompt-insights.md)**: ライブラリ全体のトークン出現頻度分布、ポジティブ／ネガティブワードランキング、ユーザー評価との相関分析。

### [第4章：インテリジェントキュレーションとAIエンジン](04-intelligent-curation/stacks-and-bursts.md)
- **[画像スタック・バーストグループ化・画像比較](04-intelligent-curation/stacks-and-bursts.md)**: 自動バーストクラスタリング（Jaccard プロンプト類似度＋時間枠）、トランプ風スタックカード、表紙（Hero）画像、安全なスタック解除、低評価下書き整理ツール、2画面並列画像比較（`C`）モード。
- **[AI セマンティック検索と自動タグ付け](04-intelligent-curation/ai-semantic-and-tagger.md)**: ローカル ONNX CLIP/SigLIP 自然言語テキスト・画像検索、画像間ビジュアル類似度検索、WD14 Danbooru アニメ自動タグ付け。
- **[チェックポイントモデルと LoRA ライブラリ](04-intelligent-curation/models-and-loras.md)**: モデルの自動カタログ化、A1111 `cache.json` ハッシュ解決、Civitai 逆引き、LoRA トリガーワード管理、1クリックプロンプト挿入。
- **[生成環境との連携](04-intelligent-curation/generation-interop.md)**: ComfyUI（`/prompt`）および AUTOMATIC1111（`/sdapi/v1/txt2img`）との直接 API 連携、リアルタイム接続ステータス監視。

### [第5章：エクスポート・クラウドバックアップ・チーム共有](05-export-and-collaboration/export-and-web-showcase.md)
- **[一括エクスポート・トランスコード・Webショーケース](05-export-and-collaboration/export-and-web-showcase.md)**: Rayon によるマルチスレッド高速変換、4段階のプライバシーメタデータ消去、動的ファイル名テンプレート、ZIP アーカイブ出力、単一 HTML ファイルによる独立型ウェブギャラリー出力。
- **[クラウドスナップショットバックアップとメディアミラーリング](05-export-and-collaboration/cloud-backup-and-sync.md)**: AWS S3、Cloudflare R2、MinIO、WebDAV、ローカル NAS への SQLite `VACUUM INTO` オンラインスナップショット；ETag / SHA-256 差分検出と帯域幅制限付き増分同期。
- **[マルチデータベース・チームスタジオ](05-export-and-collaboration/team-collaboration.md)**: 共有 MySQL 8.0+ / PostgreSQL 14+ サーバーへのスケールアップ；クロスプラットフォームストレージルートマッピング（Windows ドライブレターと macOS/Linux パスの正規化）、楽観的並行性制御（OCC）、クライアント側の NVMe サムネイルキャッシュ。

### [第6章：システムリファレンスと保守](06-reference-and-maintenance/settings-reference.md)
- **[環境設定パラメータリファレンス](06-reference-and-maintenance/settings-reference.md)**: 8つの設定タブの全パラメータ詳細ガイド。
- **[データベースとキャッシュのメンテナンス](06-reference-and-maintenance/database-maintenance.md)**: SQLite WAL 最適化・圧縮（`VACUUM`）、データベースのバックアップと復元、サムネイルキャッシュバジェット（LRU 破棄）、処理キューステータス診断。
- **[アップデートとライフサイクル](06-reference-and-maintenance/updating.md)**: アプリ内自動更新、バージョンアップ時の完全データ保護保証、手動リリース更新手順。
- **[プライバシーとセキュリティアーキテクチャ](06-reference-and-maintenance/privacy-and-security.md)**: 100% オフラインファースト、テレメトリ完全ゼロ、ローカル AI 推論分離、AGPL-3.0 ライセンス。
- **[トラブルシューティングとよくある質問 (FAQ)](06-reference-and-maintenance/troubleshooting-and-faq.md)**: よくある問題の解決策、パフォーマンス最適化のヒント、FAQ。
- **[プロダクト用語集](06-reference-and-maintenance/glossary.md)**: 専門用語の定義（表紙画像/Hero、取り込みパイプライン、Jaccard 類似度、キーセットカーソル、OCC、トランプ風スタックカードなど）。

---

## ⚡ クイックスタート・ショートカットキー

| 操作 | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **クイックルック (ライトボックス)** | `Space` / `Enter` | `Space` / `Return` |
| **2画面並列画像比較** | `C` | `C` |
| **スタックにまとめる** | `Ctrl + G` | `Cmd + G` |
| **スタックを解除** | `Ctrl + Shift + G` | `Cmd + Shift + G` |
| **スタックの表紙に設定** | `Alt + S` | `Option + S` |
| **選択画像を星1〜5評価** | `1` ～ `5` (`0` でクリア) | `1` ～ `5` (`0` でクリア) |
| **お気に入り切り替え** | `F` | `F` |
| **検索バーにフォーカス** | `/` または `Ctrl + F` | `/` または `Cmd + F` |
| **インスペクターを表示/非表示** | `I` | `I` |
| **サイドバーを表示/非表示** | `B` | `B` |
| **環境設定を開く** | `Ctrl + ,` | `Cmd + ,` |
