# 🍇 Berry AI Studio

<div align="center">

**[English](README.md)** | **[简体中文](README.zh-CN.md)** | **[繁體中文](README.zh-TW.md)** | **[日本語](README.ja.md)**

[![Website](https://img.shields.io/badge/公式HP-GitHub%20Pages-12b5cb.svg)](https://berryuiki.github.io/Berry-AI-Studio/)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.7+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-42b883)](https://vuejs.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/BerryUIKI/Berry-AI-Studio/releases)

*AI画像クリエイターとプロンプトエンジニアのための、高速ローカルAIGCメタデータインデクサー＆アセット管理スタジオ。*

<br/>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/gui_preview_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/screenshots/gui_preview_light.svg">
  <img alt="Berry AI Studio Preview" src="docs/screenshots/gui_preview_dark.svg" width="100%">
</picture>

</div>

---

## 🌟 概要

**Berry AI Studio** は、AI画像クリエイターやプロンプトエンジニア向けに設計されたデスクトップ特化型画像アセット管理スタジオです。主要なAI画像生成ツールで作成された画像メタデータ（プロンプト、モデル名、サンプラー、ステップ数、CFG、シード値、ワークフローJSON）をローカルのSQLiteに高速インデックス化。**3ペインスタジオワークスペース**、スムーズな仮想スクロールグリッド、トークン化プロンプトインスペクター、全画面ライトボックスビューア、スマート整理機能を提供します。

> 🚀 **新アーキテクチャへの刷新**: Berry v0.1.0+ は **Tauri 2 + Rust + Vue 3** を採用してゼロから再構築されました。従来の旧 C#/.NET 版は `archive/old-main` タグおよび `old/main` ブランチにアーカイブされています。

---

## ✨ 主な機能

### 🎨 3ペインスタジオ UI
- **ネイティブ品質のフレームレスウィンドウ**: 統合メニューバー（`ファイル`、`編集`、`表示`、`ツール`、`ヘルプ`）、ドラッグ領域、コントロールボタンを備えたタイトルバー。
- **左側ナビゲーションサイドバー**: メディアライブラリ（すべての画像、お気に入り、センシティブ 18+）、フォルダ階層ツリー（リアルタイムスキャン表示）、カラータグ、スマートアルバム。
- **中央ギャラリー、仮想グリッド＆ウォーターフォール**: 数万枚の画像を軽快に表示する仮想スクロール、Masonry ウォーターフォールレイアウト、サムネイル拡大縮小スライダー（130px〜360px）、グリッド（⊞）/ ウォーターフォール（▦）/ リスト（☰）切り替え。
- **右側プロパティインスペクター**: 大画面プレビューカード、`0〜5` スター評価、お気に入り切り替え、トークン化プロンプトチップ＆ワンクリックコピー、検出された LoRA タグ、生成パラメータ表示、折りたたみ式ワークフローJSONビューア。
- **全画面クイックルック (ライトボックス)**: スペースキーまたは Enter で即座に全画面表示。マウスホイール拡大縮小、パン移動、矢印キー画像送り対応。

### 🔍 ロスレスな AIGC メタデータ解析エンジン
プロンプト、ネガティブプロンプト、モデル、ハッシュ、サンプラー、ステップ数、CFG、シード値、サイズを自動抽出：
- **WebUI (AUTOMATIC1111 / SD.Next)**: PNG `parameters` チャンク、WebP EXIF。
- **ComfyUI**: 完全な Prompt および Workflow JSON グラフ構文解析、LoRA ローダーの自動検出。
- **NovelAI**: Comment および Description 署名解析。
- **Fooocus / Fooocus-MRE**: パラメータおよびベースモデル解析。
- **InvokeAI & EasyDiffusion**: 埋め込みメタデータおよび JSON サイドカーファイル。
- **対応フォーマット**: PNG、JPG/JPEG、WebP、MP4 動画、`.txt` サイドカーテキスト。

### 🗃️ マルチモード取り込み＆スマートスタッキング
- **マルチモードフォルダ**: 外部リンクモード（インプレース参照・ゼロコピー）、管理ボルトモード（専用ストレージ管理）、AIGC パイプラインモード（WebUI/ComfyUI 出力フォルダの自動監視とデバウンス収集）。
- **インテリジェント連写スタック**: 類似プロンプトと生成時刻から連写画像を自動グルーピング。
- **インタラクティブなスタックカード**: トランプデッキ風カード表示、枚数バッジ、インライン展開/折りたたみ、カバー指定（`Alt+S`）、左右並列比較モード（`C`）、安全確認付きスタック統合。

### 🧠 モデル、プロンプト＆ AI インテリジェンス
- **ローカル CLIP セマンティック検索**: ローカル ONNX CLIP/SigLIP モデルによる自然言語プロンプト画像検索。
- **AI 自動タグ付け＆類似画像検索**: WD14 / Danbooru によるアニメ自動タグ付け、スタイル・構図の類似画像逆引き検索。
- **LoRA トリガーワードライブラリ**: LoRA ディレクトリをスキャンし、Civitai メタデータとマッチング、トリガーワードをプロンプトへワンクリック注入。
- **プロンプト頻度分析**: 正負プロンプトのキーワード出現頻度と平均評価を統計分析。
- **モデルマネージャー**: Civitai SHA256 キャッシュ同期、モデルハッシュ逆引き検索、ワンクリックモデル絞り込み。
- **データベースメンテナンス**: SQLite VACUUM 最適化、バックアップのエクスポートと復元。

### 🌐 多言語＆自動アップデート確認
- **7言語ネイティブ対応**: 日本語、英語、簡体字中国語、繁体字中国語、ドイツ語、フランス語、スペイン語。
- **OS言語の自動検出 (Auto)**: システム環境設定に自動追従。
- **GitHub Releases アップデート確認**: **ヘルプ > アップデートを確認...** からワンクリックで最新リリースノート確認とダウンロード。

---

## ⌨️ 主なショートカットキー

| ショートカット | 機能 | ショートカット | 機能 |
| :--- | :--- | :--- | :--- |
| `Space` / `Enter` | 全画面ライトボックスの開閉 | `0` 〜 `5` | スター評価を設定 (0 で解除) |
| `F` | お気に入りの切り替え | `B` | 左側サイドバーの表示/非表示 |
| `I` | 右側インスペクターの表示/非表示 | `/` または `Ctrl+F` | 検索バーにフォーカス |
| `Ctrl+A` | 現在のビューの画像をすべて選択 | `Esc` | 選択解除 / ダイアログ・ライトボックスを閉じる |
| `Ctrl+O` | 画像フォルダを追加 | `Ctrl+,` | 設定画面を開く |
| `Delete` | 選択した画像をゴミ箱へ移動 | `?` | ショートカット一覧を表示 |

---

## 📦 リリースパッケージ命名規則

[GitHub Releases](https://github.com/BerryUIKI/Berry-AI-Studio/releases) で配布されるビルド済みバイナリは標準命名規則に従います：

$$\text{<アプリ名>}\_\text{<OS>}\_\text{<アーキテクチャ>}.\text{<拡張子>}$$

| OS プラットフォーム | アーキテクチャ | 形式 | 配布ファイル名 |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64 (64-bit) | NSIS インストーラー | `Berry-AI-Studio_Windows_x64.exe` |
| **Windows** | x86_64 (64-bit) | ポータブル Zip | `Berry-AI-Studio_Windows_x64.zip` |
| **macOS** | Apple Silicon (ARM64) | DMG ディスクイメージ | `Berry-AI-Studio_macOS_aarch64.dmg` |
| **macOS** | Intel (x86_64) | DMG ディスクイメージ | `Berry-AI-Studio_macOS_x64.dmg` |
| **Linux** | x86_64 (64-bit) | AppImage | `Berry-AI-Studio_Linux_x64.AppImage` |
| **Linux** | x86_64 (64-bit) | Debian パッケージ | `Berry-AI-Studio_Linux_x64.deb` |

---

## 🛠️ ソースコードからのビルド

### 必要環境
1. **Node.js** (v18+) および **pnpm** (`npm install -g pnpm`)
2. **Rust** (1.75+): [rustup.rs](https://rustup.rs/) からインストール
3. **C++ ビルドツール**: Windows (MSVC Build Tools), macOS (Xcode CLI), Linux (`libwebkit2gtk-4.1`).

### ビルド手順
```bash
# 1. リポジトリをクローン
git clone https://github.com/BerryUIKI/Berry-AI-Studio.git
cd Berry-AI-Studio

# 2. フロントエンド依存関係のインストール
pnpm install

# 3. 開発モードの起動 (ホットリロード)
pnpm run tauri dev

# 4. リリース用バイナリのビルド
pnpm run tauri build
```

出力バイナリは `src-tauri/target/release/bundle/` に生成されます。

---

## 📄 ライセンス

本プロジェクトは **AGPL-3.0 ライセンス** のもとで公開されています。詳細は [LICENSE](LICENSE) ファイルをご参照ください。
