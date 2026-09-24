# Omera Identity Residual Audit & Classification Report

**Document Version:** 1.0.0  
**Completion Date:** 2026-09-24  
**Governing Issue:** [#134](https://github.com/BerryUIKI/Omera/issues/134)  
**Governing Documents:** [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md), [OMERA_MIGRATION.md](OMERA_MIGRATION.md), [API_CONTRACTS.md](API_CONTRACTS.md)  
**Auditor:** Lead Engineer / CTO

---

## 1. Executive Summary

As part of the final verification package (**R6**) for the product transition from *Berry AI Studio* / *Berry AIGC Toolbox* to **Omera**, this document provides a comprehensive residual audit across the entire codebase.

Every single occurrence of the name `Berry`, `berry`, and related legacy identifiers in the repository has been audited, cataloged, and classified into one of four intentional categories:
1. **Category A: Active Omera Identity** (The canonical runtime identity for current and future releases)
2. **Category B: Legacy Compatibility Bridges & Readers** (Strictly retained to guarantee 100% lossless pre-1.0 data migration)
3. **Category C: Historical Artifacts & Documentation** (Changelogs, migration specifications, and execution logs)
4. **Category D: Organization Attribution & Copyright** (Permanent brand and authorship belonging to the BerryUIKI organization)

No orphaned, unintentional, or active runtime references to the legacy product name remain.

---

## 2. Classification Matrix

### Category A: Active Omera Identity (Target)

These represent the active runtime specifications verified through R1–R5:

| Surface | Active Omera Identifier | Specification / Implementation |
| :--- | :--- | :--- |
| **Product Display Name** | `Omera` | Window title, UI headers, About modal, menus, documentation |
| **OS Application Bundle ID** | `com.berryuiki.omera` | `src-tauri/tauri.conf.json`, MAS workflow, OS app directories |
| **Executable Name** | `omera` / `omera.exe` | `src-tauri/Cargo.toml` (`name = "omera"`), release workflows |
| **Active SQLite Database** | `omera.db` | Primary database opened by `src-tauri/src/lib.rs` and `AppState` |
| **Active WAL & SHM Files** | `omera.db-wal`, `omera.db-shm` | Transactional WAL persistence in app data directory |
| **Frontend Settings Store** | `omera_*` | `src/utils/config.ts` (`omera_locale`, `omera_theme`, etc.) |
| **OS Credential Service** | `Omera` | Keyring service name in `src-tauri/src/config_store.rs` |
| **Worker Thread Pools** | `omera-thumb-*`, `omera-scan-*` | Rayon and tokio threads in `omera-scan` and `src-tauri` |
| **Backup Snapshot Prefix** | `omera_snapshot_*` | `src-tauri/src/cloud_backup.rs` (`omera_snapshot_YYYY-MM-DD_HHMMSS.zip`) |
| **IPC Events (Canonical)** | `omera://*` | `omera://export-progress`, `omera://scan-progress` |
| **Updater Repository Target**| `BerryUIKI/Omera` | Canonical GitHub repository for updates and release assets |
| **Installer Assets** | `Omera_<OS>_<Arch>.<ext>` | `Omera_Windows_x64.exe`, `Omera_macOS_aarch64.dmg`, `Omera_Linux_x64.AppImage` |
| **Signature Env Key** | `OMERA_UPDATE_PUBLIC_KEY` | Evaluated at compile time in `src-tauri/build.rs` and `update_verification.rs` |
| **Cargo Member Crates** | `omera-*` | `omera-domain`, `omera-metadata`, `omera-scan`, `omera-storage` |

---

### Category B: Legacy Compatibility Bridges & Readers (Retained)

In accordance with [OMERA_MIGRATION.md](OMERA_MIGRATION.md), all pre-1.0 releases must preserve and seamlessly discover legacy user data without requiring manual user intervention. The following identifiers are intentionally retained:

| Identifier / Pattern | File Location | Retention Rationale |
| :--- | :--- | :--- |
| `berry.db`, `berry.db-wal`, `berry.db-shm` | `crates/omera-storage/src/legacy_migration.rs`, `src-tauri/src/legacy_migration.rs` | Used by discovery routines to detect and migrate unmigrated SQLite databases. |
| `com.berryuiki.berryaistudio` | `src-tauri/src/legacy_migration.rs` | Used to inspect previous app data containers on Windows, macOS, and Linux. |
| `com.berryuiki.berryaigctoolbox` | `src-tauri/src/legacy_migration.rs` | Used to discover legacy v0.1.x installations. |
| `Berry-AI-Studio` | `src-tauri/src/config_store.rs` | Keyring service name fallback for reading existing S3/WebDAV credentials. |
| `Berry-AIGC-Toolbox` | `src-tauri/src/config_store.rs` | Keyring service name fallback for v0.1.x credentials. |
| `berry_*` local storage keys | `src/utils/config.ts`, `src/utils/thumbnail.ts`, `src/i18n/index.ts`, `src/App.vue` | Fallback readers for user preferences saved in previous browser/WebView storage. |
| `berry_snapshot_*` | `crates/omera-domain/src/cloud_backup.rs`, `src-tauri/src/cloud_backup.rs` | Enables importing and restoring historical backup zip archives created by Berry. |
| `berry_version` field | `crates/omera-domain/src/cloud_backup.rs`, `src-tauri/src/cloud_backup.rs`, `src/types.ts` | Backward-compatible serde field in snapshot manifests for version tracking. |
| `berry://export-progress` | `src-tauri/src/commands.rs`, `src/components/ExportModal.vue` | Backward-compatible IPC event alias emitted in parallel with `omera://export-progress`. |
| `BERRY_UPDATE_PUBLIC_KEY` | `src-tauri/src/update_verification.rs`, `src-tauri/build.rs` | Fallback environment variable for verifying releases signed with the legacy private key. |
| `BerryUIKI/Berry-AI-Studio` URL | `src-tauri/src/update_verification.rs` | Allowed GitHub download source origin during transition period. |
| `berry.*\.exe`, `berry.*\.dmg`, `berry.*\.appimage` | `src/utils/updater.ts` | Fallback regex asset matching in updater to parse legacy release assets if needed. |
| `__BERRY_THUMBNAIL_DIAGNOSTICS__` | `src/utils/thumbnail.ts` | Development global alias on `window` alongside `__OMERA_THUMBNAIL_DIAGNOSTICS__`. |

---

### Category C: Historical Artifacts & Documentation (Retained)

These files document the historical provenance, evolution, and exact execution of the migration:

| File Location | Scope & Content |
| :--- | :--- |
| `CHANGELOG.md` | Historical release notes for `v0.1.0` through `v0.3.0` documenting Berry AI Studio milestones. |
| `docs/OMERA_MIGRATION.md` | Data migration contract, safety invariants, and WAL preservation guarantees. |
| `docs/OMERA_RENAME_EXECUTION.md` | Master plan and execution gates for packages R0 through R6. |
| `docs/OMERA_RENAME_INVENTORY.md` | Baseline R0 inventory of release assets, paths, and storage roots. |
| `docs/OMERA_RENAME_STATUS.md` | Handoff log and status tracking across the R0–R6 progression. |
| `crates/omera-storage/src/legacy_migration.rs` | Migration receipt format `berry-to-omera-v1` recorded upon successful migration. |

---

### Category D: Organization Attribution & Copyright (Permanent)

The project continues to be maintained and published under the **BerryUIKI** open-source organization:

| Identity | Usage / Context | Target Value |
| :--- | :--- | :--- |
| **GitHub Organization** | Repository origin and organization URL | `https://github.com/BerryUIKI` |
| **GitHub Pages** | Official website host | `https://berryuiki.github.io/Omera/` |
| **Bundle ID Domain** | Reverse-DNS domain authority | `com.berryuiki.*` (`com.berryuiki.omera`) |
| **Copyright Notice** | Software copyright in `tauri.conf.json` and web pages | `Copyright © 2026 BerryUIKI` |
| **Author Attribution** | Author in Cargo manifests | `Berry Wahlberg <berry@berryuiki.com>` |

---

## 3. Verification Checklist

- [x] **Zero Active Misnamings**: No active code writes to `berry.db`, `berry_*`, or `Berry-AI-Studio` without fallback semantics.
- [x] **Precedence Invariant**: In all settings readers, Omera values win unconditionally over legacy values even when falsy (`false`, `0`, `""`).
- [x] **Safe Migration Isolation**: Legacy database discovery operates read-only with SQLite Online Backup (`rusqlite::backup::Backup`); WAL journals are never directly file-copied.
- [x] **Internationalization Audited**: All 7 UI locales (`en`, `zh-CN`, `zh-TW`, `ja`, `de`, `fr`, `es`) refer to `omera.db` and display `Omera`.
- [x] **Documentation & Wiki Aligned**: All 181 wiki pages across 7 languages, architecture diagrams, benchmark reports, and web landing pages use the Omera identity while preserving BerryUIKI organization credit.
- [x] **Installer & Release Consistency**: Release workflows build `Omera_<OS>_<Architecture>.<ext>` bundles and update verification enforces signed payloads.

---

## 4. Sign-Off & Conclusion

With the completion of the R6 residual audit and documentation alignment, the full rename roadmap (R0–R6) established in [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md) is fully implemented, verified, and ready for lead sign-off and closure of Issue [#134](https://github.com/BerryUIKI/Omera/issues/134).
