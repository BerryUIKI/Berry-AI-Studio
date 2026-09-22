# クラウドスナップショットバックアップと同期

Berry AI Studio は、クラウドバックアップおよび差分同期エンジン（`src-tauri/src/cloud_backup.rs` および `src-tauri/src/cloud_sync.rs`）を標準搭載しており、サードパーティ製ツールに頼ることなく、データベースの自動バックアップとリモートへの増分メディアミラーリングを実現します。

---

## 1. 対応ストレージプロバイダー

リモートエンドポイントは **環境設定 > クラウドバックアップ** で設定できます：

| プロバイダー | 対応プロトコル / エンドポイント | 特徴・備考 |
| :--- | :--- | :--- |
| **AWS S3 / 互換ストレージ** | AWS S3, Cloudflare R2, MinIO, Backblaze B2, Wasabi | HMAC-SHA256 署名による Rust ネイティブの AWS Signature Version 4 (SigV4) 認証。 |
| **WebDAV** | Nextcloud, ownCloud, Synology DiskStation, QNAP NAS | HTTPS 経由の標準 HTTP Basic 認証に対応。 |
| **ローカル / ネットワークパス** | ローカルドライブ、外付け USB SSD、SMB / NFS ネットワーク共有 | ネットワークプロトコルのオーバーヘッドがない直接の高速ファイルシステム I/O。 |

---

## 2. ホット SQLite スナップショットバックアップ (`cloud_backup_create_snapshot`)

Berry は、SQLite ネイティブの `VACUUM INTO` コマンドを使用してデータベースのバックアップを作成します：

```mermaid
sequenceDiagram
    participant UI as Berry Studio UI
    participant Rust as バックエンド (cloud_backup.rs)
    participant DB as SQLite WAL (berry.db)
    participant Remote as クラウドストレージ (S3/WebDAV)

    UI->>Rust: バックアップスナップショット要求
    Rust->>DB: VACUUM INTO temp_snapshot.db (非ロック実行)
    DB-->>Rust: 時点整合性のあるデータベースコピー
    Rust->>Rust: manifest.json とともに ZIP にパッケージ化
    Rust->>Remote: ストリームアップロード (SigV4 / WebDAV PUT)
    Remote-->>Rust: アップロード完了確認 (200 OK)
    Rust-->>UI: スナップショット作成成功
```

### スナップショットの保証事項：
- **ノンブロッキング（非ロック）**: SQLite のオンライン Vacuum API を使用するため、バックアップ中も閲覧、評価、画像生成を一切中断することなく継続できます。
- **ロールバック保護**: リモートスナップショットを復元する際、Berry は適用前にローカルの安全なコピー（`berry.db.rollback`）を自動作成し、ネットワーク切断やデータ破損から保護します。

---

## 3. 増分メディアミラーリングと差分同期 (`cloud_sync.rs`)

スナップショットがデータベースを保護する一方、**差分メディアミラーリング（Delta Sync）** はローカルドライブとリモートクラウドストレージ間で画像・動画ファイルを増分同期します。

### 同期エンジンの機能：
- **差分比較戦略**:
  - *高速フィンガープリント*: ローカルのファイルサイズとリモートの HTTP ETag を比較（低速回線に最適）。
  - *厳密なチェックサム*: ストリーミング SHA-256 ハッシュを計算し、バイト単位での完全な一致を保証。
- **トークンバケット帯域幅制限**: アップロード速度の上限（KB/s）を設定可能。バックグラウンドでの同期処理がスタジオのインターネット回線を占有するのを防ぎます。
- **並行転送スレッド数**: アップロードスレッド数を 1〜8 スレッドの間で調整可能。
- **ドライランモード (Dry-Run)**: リモートストレージを変更することなく、同期対象、スキップ、削除対象のファイルを事前にシミュレーション確認。
- **リアルタイム進捗通知**: 転送バイト数、スループット速度、完了パーセント、予想残り時間（ETA）をリアルタイムで画面に通知。
