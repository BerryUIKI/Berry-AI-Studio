# Omera rename execution and agent handoff

Status: implementation plan for [#134](https://github.com/BerryUIKI/Omera/issues/134), not evidence that the runtime identity or migration has shipped. This document is the single entry point for the **entire** Berry AI Studio → Omera rename. The safety protocol is specified in [OMERA_MIGRATION.md](OMERA_MIGRATION.md); API status is in [API_CONTRACTS.md](API_CONTRACTS.md); release gates are in [RELEASING.md](RELEASING.md). Read those contracts before changing code.

Last audited against `dev` commit `232d1e20` on 2026-09-23. Always fetch current `dev`, record its exact SHA, inspect later changes, and use an isolated branch/worktree. Earlier handoff documents still contain historical baseline references (`4f57246`, draft PR #138); #138 has since merged. Do not base new work on those historical references.

## Target identity and compatibility boundary

| Surface | Target for new writes/builds | Legacy input that must remain readable |
| --- | --- | --- |
| Visible product name | `Omera` | Old name in historical records, screenshots and migration explanations |
| Tauri application ID | `com.berryuiki.omera` | `com.berryuiki.berryaistudio` and other verified released layouts |
| Repository | `https://github.com/BerryUIKI/Omera` | Old URLs only where needed to recognize/update older clients |
| Executable and package | `omera` | Prior installer/executable identity for upgrade or manual handoff |
| Internal Cargo crates | `omera-*` and matching Rust identifiers | Historical schema/serialized fields only where persisted compatibility requires them |
| Library database | `omera.db` under the new app data root | `berry.db`, including committed WAL contents and old backup archives |
| Browser-local settings | `omera_*` for new writes | All known `berry_*` keys from supported pre-1.0 clients and old WebView origins |
| Credential service | `Omera` | Previous service and account names while importing credentials |
| New backup/export branding | `omera_*` / `Omera_*` as appropriate | `berry_snapshot_*`, archive entry `berry.db`, `berry_version` and older manifests |
| Release assets | `Omera_<OS>_<Architecture>.<extension>` | Published Berry asset names and installer metadata for upgrade analysis |

Preserve `BerryUIKI` attribution, copyright and license history. Preserve generator names, third-party URLs, users' file names, database values and historical migrations. Classify every search result as **new active identity**, **legacy reader**, **historical evidence**, **third-party name**, or **attribution** before editing; never run a blind global replacement.

## Work packages, ownership and order

Use separate, reviewable PRs targeting `dev`. One package may need several PRs when it touches different platforms. The owner may give code Agents a package explicitly; a package label does not authorize them to activate a later package early.

| Package | Owner | Starts when | Exit |
| --- | --- | --- | --- |
| R0 — baseline and compatibility inventory | Lead with Agent research help | Now | Released-version/installer matrix, old data locations, backup formats, key/credential mapping, fixtures and exact baseline recorded |
| R1 — visible name and current documentation | Code Agent | Now | App-facing strings say Omera in all seven locales; new showcase/export labels and current docs updated; runtime storage/installer identity untouched |
| R2 — migration and recovery services | Lead | After R0 | Discovery, source choice, WAL-safe staging, receipt, config/key/credential import, activation gating and optional cleanup pass fault tests |
| R3 — internal code/package rename | Code Agent under lead review | After R2 contracts and integration baseline are approved | Crate/package paths, imports, generated lockfiles and build scripts renamed coherently; no persisted identifier silently changes |
| R4 — runtime identity activation | Lead | R2 green; coordinate R3 | Tauri ID/productName, executable, active DB/config/key/credential targets switch as a tested unit; no empty-library fallback |
| R5 — updater, installers and release assets | Lead with Agent CI help | R4 integrated | Signed cross-platform Berry → Omera and Omera → next Omera paths qualified; skipped-version path or documented manual recovery exists |
| R6 — final documentation and residual audit | Code Agent; lead reviews release guidance | R4/R5 behavior known | Current docs/wiki/website, release workflow examples and translations match actual behavior; all residual Berry identifiers classified |

R1 can merge early because it changes presentation, but it is **not** the complete rename. R3 code review can begin earlier in an isolated branch, yet it cannot be integrated if it changes the installed executable/package or a persisted wire format before R2 is ready. Do not combine R1–R5 into one enormous PR. #134 stays open until R4/R5 validation succeeds.

## R0 — inventory before activation

- Capture actual published Berry releases and installers, not just current source: Windows NSIS uninstall/install scope, executable and shortcuts; macOS bundle/provisioning/App Store constraints; Linux package IDs and desktop entries. Record which historical version can perform a bridge export and which requires manual import.
- Enumerate platform-specific app data, WebView profile, credential and cache roots for `com.berryuiki.berryaistudio` and known earlier layouts. Distinguish app-owned data from user-selected managed vaults, linked external folders and shared model directories. Do not infer deletion ownership from a directory name.
- Inventory `berry_*` keys from `src/utils/config.ts`, `src/i18n/index.ts`, settings, thumbnail helpers, tests and old released clients. Specify old → new key, type, default, source precedence and whether it is security-sensitive. A present `false`, `0` or empty value is not absent.
- Current frontend key examples include `berry_locale`, `berry_theme`, `berry_autoscan`, `berry_blur_nsfw`, `berry_card_badges`, `berry_default_view`, `berry_thumbnail_max_edge`, `berry_thumbnail_cache_budget_mb`, `berry_similarity_limit`, `berry_auto_check_update`, `berry_silent_install`, `berry_comfyui_url`, and `berry_webui_url`. Map each to the same suffix under `omera_*` only after verifying the source type and authoritative backend config. `berry_last_startup_scan_*` is keyed by source identity: decide whether it can be mapped safely or should be re-derived; never reuse it blindly for a different folder ID. This is a source snapshot, not a promise that no other legacy key exists.
- Inventory backend `config.json` fields, credential service/account names, `berry.db` paths, `berry_snapshot_*` formats, archive entry names, `berry_version`, event names such as `berry://export-progress`, and any external integration contracts. Keep fixtures containing these legacy values.
- Establish source library counts, associations, schema/user_version, integrity check, foreign-key check and WAL-resident changes. Keep immutable before/after fixtures for empty, normal, multiple and damaged libraries.

Useful audit commands (run from the latest checkout; inspect results, do not auto-replace):

```sh
rg -n -i 'Berry AI Studio|Berry-AI-Studio|berry-ai-studio|com\.berryuiki\.berryaistudio|berry\.db|berry_snapshot|berry_|berry-' src src-tauri crates .github docs README.md package.json Cargo.toml
rg -n 'BerryUIKI/Berry|github.com/BerryUIKI' src src-tauri .github docs README.md
rg -n 'app_data_dir|config\.json|keyring|credential|export-progress|release' src-tauri/src src/utils
```

Do not include generated `target`, `node_modules`, vendored data or lockfile noise in the textual residual report; still regenerate and inspect lockfiles when manifests change.

## R1 — brand presentation that Agents can implement first

Audit `src/components/TitleBar.vue`, `SettingsModal.vue`, `UpdateModal.vue`, onboarding and export UI, `src/i18n/locales/{en,zh-CN,zh-TW,ja,de,fr,es}.ts`, `index.html`, generated HTML showcase code in `crates/berry-scan/src/`, current README and user docs. Change display-name strings, accessible names/alt text and future exported showcase branding. `src-tauri/tauri.conf.json` `app.windows[].title` may change in R1; `productName` and `identifier` wait for R4 because they affect installed identity. Change `berry_export_` only as a **new output filename default**, after checking no reader relies on it.

Do not invent an Omera visual mark. Inspect `src/assets/logo.png` and `src-tauri/icons/`; if they depict the old brand, list exact assets needing owner-approved replacements. Do not silently leave an old-wordmark image with misleading new alt text. Keep existing BerryUIKI credit and historical migration text. Check the already completed #132 title-bar link/version behavior before editing overlapping code.

R1 PR acceptance: visible app surfaces and all seven locales are consistent; keyboard/accessible names are accurate; package/installer/data locations have not changed; residual old strings are classified; the PR description plainly says **presentation rename only**. Current docs may be updated in a separate R1 documentation PR when the wiki scope is large.

## R2 — migration, settings and data safety (lead-owned)

Follow [OMERA_MIGRATION.md](OMERA_MIGRATION.md) and its failure matrix. This package owns the risky code; other Agents can add fixtures, review tests and mock the approved UI contracts.

1. Discover legacy roots and a pre-existing Omera root before opening a new empty database. If Omera already has data, it wins; if several sources exist, require an explicit source choice. Detect concurrent old-app writers and unauthorized or escaping paths.
2. Create a durable receipt containing source identity, destination, schema/version, per-artifact state and validation. Acquire a migration lock; stage on the destination volume; resume or report partial work after a crash. No secret values in logs/receipts.
3. Snapshot SQLite through the backup API to include committed WAL state. Check integrity, foreign keys, schema compatibility, entity counts and relations. Apply only append-only migrations to the staged copy; do not rewrite historical migrations, reset versions, move a live DB, or remove WAL/SHM.
4. Import backend configuration, browser settings, model references and credentials with explicit precedence. Backend config remains authoritative. An Omera value present as `false`, `0` or `""` wins. Reading `berry_*` inside a new WebView origin is insufficient: provide a legacy-app bridge or versioned settings export/import when direct origin access is unavailable. Verify new credential reads before considering old entries eligible for cleanup.
5. Publish `omera.db` and config per file only after validation. Start DB connections, watchers, thumbnail and inference workers after activation. If migration fails, preserve the source and staging evidence and offer retry/choice rather than opening an empty replacement library.
6. Show cleanup **after** Omera reopens and reads migrated data. Default to Keep. Preview exact eligible app-owned paths and sizes; bind confirmation to that preview; recheck destination health and source revision. Move eligible files to system Trash only; report partial failures and keep a retryable receipt. Never touch user media/vaults/backups or silently remove old credentials.

Proposed IPC in [API_CONTRACTS.md](API_CONTRACTS.md) is **not registered yet**. The lead must finalize DTOs, serialization, structured errors, status/progress, preview expiration and cancel/retry behavior before production UI calls. Update [IPC_REFERENCE.md](IPC_REFERENCE.md) and contract tests when commands are registered. `legacy_migration_complete` is an older config flag, not proof of this migration or authorization to clean up.

The currently proposed IPC sequence is `get_legacy_migration_status` → `preview_legacy_migration({ sourceId })` → `start_legacy_migration({ planId })` → `get_legacy_migration_job({ jobId })`, followed only after verified activation by `preview_legacy_cleanup({ receiptId })` → `confirm_legacy_cleanup({ previewId, confirmed: true })` or `defer_legacy_cleanup({ receiptId })`. `sourceId`, `planId`, `jobId`, `receiptId` and `previewId` are backend-issued tokens; the UI cannot pass an arbitrary deletion path. See [API_CONTRACTS.md](API_CONTRACTS.md) for the draft fields and error shape. Each proposed command must remain a mock until the backend registers and tests it.

## R3/R4 — coherent code and runtime identity switch

- Update `src-tauri/tauri.conf.json` productName, identifier, window title, descriptions and platform bundle metadata in a coordinated activation. Set the executable/package to `omera`, and rename `src-tauri`/root package identity and `crates/berry-*` to `omera-*` only with updated Cargo workspace membership, dependency keys, Rust import paths, `main.rs` library reference, build scripts, tests, benches and regenerated `Cargo.lock`/JavaScript lockfile. Preserve the current version number.
- Centralize active DB name and app paths. The current `berry.db` literals span `src-tauri/src/lib.rs`, `commands.rs`, backup/restore and thumbnail paths; all active users must resolve the same `omera.db`. Avoid a mixed run where one command writes `berry.db` and another reads `omera.db`. Keep explicit old-path readers only in migration and backup compatibility code.
- Write new browser settings under `omera_*`, with mapped reads from supported `berry_*` sources for every pre-1.0 release. Do not repeatedly reimport after a receipt. Preserve existing user choices and secrets. Do not transform arbitrary user-defined values that merely contain the word Berry.
- New backups may use Omera names/metadata, but restore and cloud listing must continue to recognize historical `berry_snapshot_*` ZIPs, embedded `berry.db` and `berry_version`. Version the backup format if entry/manifest layout changes; exercise old → new and new → new restores. Event names such as `berry://export-progress` are IPC: change producer, consumer, documentation and tests together, or retain a compatibility alias during the transition.
- Reconcile thumbnails, models, managed-vault paths, sidecars, cloud root mappings and exported references. Copy or rebuild disposable caches only after primary data is safe. Do not rebase user-owned external media paths into the new app root.

The activation PR must include a rollback/recovery narrative. Reverting a binary is not a valid rollback after a schema upgrade; retained source snapshots and a validated restore path are the recovery mechanism.

## R5 — update, installer and publishing compatibility

Audit `src/utils/updater.ts`, backend update verification/commands, `.github/workflows/release.yml`, `.github/workflows/mas-release.yml`, platform packaging and [RELEASING.md](RELEASING.md). The repository URL is already `BerryUIKI/Omera`; verify every check, allowlist, download and user link uses the intended destination. Maintain signature verification and exact OS/architecture/installer matching; do not select the first asset or bypass missing trust configuration.

Do not assume GitHub repository redirects make old clients upgrade correctly. Test direct Berry → Omera from each supported published version, including skipped bridge releases, then Omera → next Omera. Where automatic upgrade cannot work, provide a manual installer plus data-import/recovery instructions. Check Windows install/uninstall/shortcut identity and elevation; macOS bundle ID, provisioning/sandbox/App Store identity; Linux package/desktop identity and upgrade behavior. Do not remove an old installation before a user has a recoverable data path. Signing keys are retained unless a separately designed trust transition is necessary.

CI may build renamed artifacts before release, but no tag, release, publish, or merge to `main` follows merely from a green build. Publishing waits for the real platform matrix and [RELEASING.md](RELEASING.md) gate.

## Verification and reporting

For each PR: state package, exact `dev` baseline SHA, interfaces changed, legacy readers retained, tests run with outcomes, manual checks actually performed, and remaining gates. Target `dev`; link #134 without `Closes #134` until the full rename and upgrade matrix are qualified. Do not mix unrelated issue fixes into rename PRs.

Minimum automated checks for affected code: `pnpm build`, applicable frontend tests, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, generated IPC reference check when commands change, and CI on Windows/macOS/Linux. Run focused tests earlier and report skipped/unavailable checks honestly. R1 documentation-only PRs need link/spelling and relevant locale audit rather than a full native build.

The release candidate must exercise fresh install; legacy-only; existing Omera plus legacy; multiple source libraries; concurrent writer; corrupt/future schema; WAL-resident data; disk-full/read-only/interrupted stages; inaccessible old WebView origin; missing credential service; declined/approved/partial cleanup; old backup restore; signed direct upgrade; skipped-version upgrade; and a subsequent Omera update. Compare files, tags, albums, stacks, settings and credential accessibility before/after. Verify a failed migration never presents an empty new library as success.

## What to send back to the lead

An Agent finishing R1 should provide its PR URL, changed-file inventory, residual legacy-name classification, locale/manual QA results and any logo asset request. An Agent preparing R3 or R6 should provide a proposed diff and exact dependency on R2/R4/R5; do not independently flip runtime identity. The lead owns integration of R2/R4/R5 and the final #134 closure decision.
