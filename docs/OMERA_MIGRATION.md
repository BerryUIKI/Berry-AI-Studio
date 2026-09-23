# Omera identity and pre-1.0 migration contract

Status: accepted product direction; implementation and release verification pending.

Ownership: lead maintainer/Codex. General engineers consume the migration contracts and implement presentation only after the safety services are ready. See [ENGINEERING_HANDOFF.md](ENGINEERING_HANDOFF.md).

For the complete rename inventory, ordered PR packages and copy-ready Agent scope, use [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md). This document remains the data-safety contract.

## Target identity

The product is Omera. The application identifier is `com.berryuiki.omera` and the canonical repository is `https://github.com/BerryUIKI/Omera`. Use `omera` for the executable/package name, `omera-*` for workspace crates, `omera_*` for local settings, `Omera` for the credential service, and `omera.db` for the active library database. Release artifacts, websites, CI, examples, translations, exported branding, and documentation must use the new identity. Keep BerryUIKI attribution and license history intact.

Legacy identifiers are compatibility inputs, not active write destinations. Centralize them in migration code rather than scattering fallback writes throughout the application. Retain the released schema history unchanged, even where historical SQL or fixtures contain legacy names.

Changing the application identifier changes platform data locations and may change WebView storage origins, installer identity, permissions, and macOS sandbox access. A successful rename of source strings alone does not demonstrate migration compatibility.

## Compatibility window

Every release before 1.0.0 must check known legacy locations and allow migration from supported Berry libraries, settings, and backup archives. A durable migration receipt prevents repeated import of the same source. Discovery still runs when new legacy data appears. Opening an existing Omera library always takes precedence over automatic legacy import.

Do not reset the application version or SQLite schema version for the rename. Before 1.0.0, review telemetry-free test evidence and document the supported import path for 1.0.0; reaching 1.0.0 never authorizes automatic deletion of old data.

## Startup migration protocol

1. Resolve the new application data directory and platform-specific legacy directories for `com.berryuiki.berryaistudio`, including supported historical layouts. Reject symlinks/reparse points that escape approved roots. Detect concurrent legacy writers and ask the user to close the old application before activation or cleanup.
2. Discover database, configuration, credentials, model files, thumbnail cache, and WebView settings separately. Show source paths, library counts, estimated space, and conflicts. If multiple libraries exist or Omera already has a library, require a source choice or explicit import; never silently overwrite or combine unrelated file IDs.
3. Create a migration receipt with source identity, source schema version, destination, per-artifact state, and validation results. Never put secrets in the receipt. Acquire a migration lock and use a staging directory on the destination volume.
4. Snapshot SQLite through its backup API so committed WAL data is included. Never copy only a live `.db`, rename a live database, or delete its WAL/SHM files. Preserve the original. Validate integrity, foreign keys, supported schema version, entity counts and associations; apply append-only migrations to the staged copy. Reject newer unsupported schemas without modifying them.
5. Migrate configuration with compatible defaults and explicit old-to-new key mappings. A present Omera value wins over a legacy value, including `false`, `0`, and empty values. Write credentials to the new OS credential service and verify retrieval before marking them migrated; retain old entries until cleanup approval.
6. Publish validated `omera.db` and configuration atomically per file. The receipt makes the multi-file operation resumable; do not claim a filesystem-wide atomic transaction. Activate only when required artifacts are complete. Start database connections, watchers, and inference workers after activation. On failure, preserve sources and staging evidence and offer retry or source selection, not silent creation of an empty replacement library.
7. Render the migrated SQLite library before optional scans or cache work. Copy models only when verified and needed. Thumbnails are disposable: rebuild lazily or migrate valid entries and rewrite cache paths. Do not retain active references to caches scheduled for cleanup.

WebView localStorage may be inaccessible from the new identifier. Reading `berry_*` from the new origin alone is insufficient. Support a legacy-app export/bridge or explicit import of a versioned settings export where direct access is unavailable. Do not assume raw WebView profile databases can safely be edited. Backend `config.json` remains authoritative after migration.

## Cleanup requires a separate user decision

After migration succeeds and Omera reopens and reads the destination successfully, show a localized, dismissible cleanup prompt with the exact old paths, categories, and sizes. Offer **Keep old data** (default), **Review cleanup**, and a later entry in settings. Record refusal without repeatedly prompting on every launch.

Cleanup is restricted to verified legacy application-owned files listed in the receipt. Revalidate paths and source revisions immediately before cleanup. Never delete linked media, user-selected managed vaults, external folders, arbitrary backups, or shared model directories as part of an application-data cleanup. If a legacy root contains user assets, explicitly exclude them and explain why some files remain.

Require renewed confirmation if the source changed after migration. Use OS trash for eligible files; report partial failures and retain a retryable receipt. Never fall back to permanent deletion. Old credential entries may be removed only when the new entries are readable and their deletion is explicitly included in the approved cleanup scope. Application uninstall and legacy binary removal are separate installer actions, not recursive data-directory cleanup.

## Installer and update compatibility

- Use the Omera repository in update checks, verified-download allowlists, release workflows, links, and signing documentation. Keep established signing keys where available; key rotation requires its own trust transition. Missing signing credentials block automatic installation, not verification bypass.
- Match assets by OS, architecture, and installer type. Never select the first release asset as a fallback. Validate the signature again before execution.
- Inspect actual released installers to determine old executable names, NSIS uninstall entries, MSI UpgradeCode (when applicable), shortcuts, and installation scope. New identity requires explicit legacy detection and replacement handling. Test elevation boundaries and do not remove the old installation until migration/recovery access is secured.
- macOS Bundle ID changes require new provisioning/App Store identity where applicable. A sandboxed new identity may not read the old container automatically; provide user-mediated import/export. Do not present this as an ordinary in-place App Store update.
- Older clients may use the redirected GitHub URL but still enforce old download URLs or select assets incorrectly. Test direct upgrade from every supported published version, including users who skip a bridge release. Where automatic upgrade is impossible, provide a manual installer and migration instructions.
- Publish only after Berry -> Omera -> next Omera upgrade tests pass on every supported target. Keep version numbers increasing and retain recovery snapshots; binary downgrade is not a safe database rollback after schema upgrades.

## Delivery and verification

1. Update documentation and track migration/release/storage tasks in GitHub issues.
2. Implement and test migration/cleanup services before switching the runtime identity.
3. Rename application, packages, crates, UI strings, settings, database, CI, and release artifacts; allow legacy strings only in migration code, fixtures, and historical records.
4. Complete the repository review fixes and regression coverage, then run the required build, frontend, formatting, Clippy, and Rust suites.
5. Exercise fresh install, legacy-only install, existing Omera plus legacy data, concurrent legacy app, full disk, interrupted migration at each stage, corrupt/future schema, WAL-resident changes, inaccessible sandbox data, credential failures, cleanup declined/approved/partial failure, and repeated startup. Verify user data and associations before and after migration.
6. Manually verify all gallery modes and accessibility, and test real signed installers and the next-update path. Do not mark issues complete based only on a renamed development build.

See [STORAGE_EVOLUTION.md](STORAGE_EVOLUTION.md) for storage decisions and [RELEASING.md](RELEASING.md) for release gates.
