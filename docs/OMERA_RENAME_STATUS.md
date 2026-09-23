# Omera rename: current context and next actions

Updated: 2026-09-23. This is a handoff snapshot, not a release-readiness statement. For the full work inventory and dependency gates, read [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md); for data safety, read [OMERA_MIGRATION.md](OMERA_MIGRATION.md).

## Verified repository state

- Integration branch: `dev`, at `05715da44bcf9c50144fd5f58a2624866176f934` when this snapshot was prepared. Fetch and record the newer SHA before starting work.
- [PR #162](https://github.com/BerryUIKI/Omera/pull/162) (folder navigation), [PR #163](https://github.com/BerryUIKI/Omera/pull/163) (complete rename handoff) and [PR #164](https://github.com/BerryUIKI/Omera/pull/164) (R1 visible branding) are **merged** into `dev`. PR #164 passed the macOS, Ubuntu and Windows checks. The PR branch content matches current `dev`; there are no remaining R1 file differences to push.
- The two worktrees used for this handoff, `F:/dev/Omera-dev` and `F:/dev/Omera-rename-handoff`, were clean before this documentation change. The old remote R1 branch was deleted after merge. Do not recreate/push it just because a local branch still exists.
- [Issue #134](https://github.com/BerryUIKI/Omera/issues/134) remains open. R1 is complete **only as a presentation rename**. The agent reported passing `pnpm run build`, `pnpm run test:stack` (102), `cargo fmt --check`, workspace Clippy with warnings denied, `cargo test --workspace` (158), and IPC reference generation (140 signatures). This handoff did not rerun those local commands; the three PR CI jobs were checked on GitHub.

## What users see now, and what still uses Berry identity

R1 changed the document/window title, title bar, About/update/onboarding/export wording, seven UI locales, default new export ZIP prefix, generated showcase branding and four README files. It preserved the runtime and installed identity by design.

The live configuration still has Tauri `productName: "Berry AI Studio"` and `identifier: "com.berryuiki.berryaistudio"`; the active library is `berry.db`; browser settings still write `berry_*`; the keyring service, Cargo packages/crates, backup archive format, updater source/trust configuration and release workflows still include legacy identifiers. Those are R2–R5 work. A renamed title is **not** a migrated library, qualified installer or safe update.

The app icon has not changed. `src/assets/logo.png` and `src-tauri/icons/` need an owner decision if Omera should have a new mark. The current abstract mark may be retained intentionally; do not generate a replacement without an approved design. Images beside text controls can use empty decorative `alt` text to avoid repeating the Omera name to screen readers; treat that as a small accessibility follow-up, not a migration gate.

Current public wiki content still has many old product-name references (181 matching Markdown files at this audit). README has been updated, but a separate current-docs PR is needed before calling the public-facing rename complete. Preserve historical releases, compatibility examples, author/copyright credit and third-party names.

## Ordered work queue

| Order | Owner | Work and completion evidence |
| --- | --- | --- |
| 1. R0 inventory | Lead, with Agent research help | Capture released Berry version/installer matrix; Windows/macOS/Linux old data and WebView roots; `berry_*` mapping; old credential identifiers; `berry.db` and `berry_snapshot_*` fixtures with WAL data; existing Omera-library precedence. Record an exact current `dev` baseline. |
| 2. Current-docs follow-up | Documentation Agent; can run beside R0 | Update the live wiki and user guides to Omera in a separate PR, review links and all languages, classify every retained Berry reference. Optional decorative-logo `alt` cleanup in a small UI PR. |
| 3. R2 migration/recovery service | Lead | Implement backend discovery, source choice, migration lock/receipt, SQLite backup/validation, config/key/credential import, staged activation and separately confirmed cleanup. Add restart/failure fixtures before changing runtime identity. Finalize proposed IPC in [API_CONTRACTS.md](API_CONTRACTS.md) and update [IPC_REFERENCE.md](IPC_REFERENCE.md) only when registered. |
| 4. R3 internal rename | Code Agent after R2 contract approval; lead review | Rename internal crates/packages/imports and regenerate manifests/lockfiles coherently. Keep persisted names/installer identity unchanged until R4 integration. |
| 5. R4 runtime activation | Lead | Switch Tauri identifier/productName, executable, active `omera.db`, `omera_*` writes and Omera credential service as one verified activation. Keep old readers and backups. Test failure never creates a misleading empty library. |
| 6. R5 release/update qualification | Lead with Agent CI help | Canonical Omera updater/release assets, signed installer checks and real Berry → Omera → next Omera upgrades on supported platforms, including skipped versions and documented manual recovery where needed. |
| 7. R6 final audit | Documentation Agent; lead release review | Update remaining current docs/website/CI examples to match tested behavior; classify intentional legacy strings. Close #134 only after migration and upgrade evidence exists. |

R0/R2 are the immediate lead-owned critical path. The documentation Agent can work independently now. R3 may be researched but not integrated before the lead approves its baseline and migration contract. The later image-compression work ([#158](https://github.com/BerryUIKI/Omera/issues/158), [#159](https://github.com/BerryUIKI/Omera/issues/159), [#160](https://github.com/BerryUIKI/Omera/issues/160)) remains after rename stabilization.

Other open work at this snapshot: [#102](https://github.com/BerryUIKI/Omera/issues/102) long-running command/lock ownership, [#107](https://github.com/BerryUIKI/Omera/issues/107) WebView asset/CSP boundaries, and [#124](https://github.com/BerryUIKI/Omera/issues/124) single-instance lifecycle are safety-related and should be coordinated with R2/R4. [#135](https://github.com/BerryUIKI/Omera/issues/135) is measured SQLite evolution, not a reason to replace the engine during the rename. [#128](https://github.com/BerryUIKI/Omera/issues/128) automatic NSFW classification and [#137](https://github.com/BerryUIKI/Omera/issues/137) in-app help are separate product/UI assignments. Recheck issue states before dispatch; this list is a dated snapshot, not an automatic assignment.

## Non-negotiable acceptance for the identity switch

- Preserve and read old databases, config, `berry_*` preferences, credentials and backup archives before writing new Omera locations. New Omera values win even when `false`, `0` or empty. A changed WebView origin cannot be assumed to expose old localStorage; provide a bridge/import path.
- SQLite migration includes committed WAL state, integrity/foreign-key/schema checks and retained source/recovery copies. Do not rewrite applied migrations, reset app/schema versions, rename a live database or erase WAL/SHM.
- Cleanup is a later, optional, preview-bound user decision. Default Keep; protect external media, user-selected vaults and backups; system Trash failure cannot fall back to permanent deletion.
- Do not rely on repository redirects alone for updater correctness. Validate signed assets, OS/architecture selection, old installer replacement and a subsequent Omera update. No release or `main` merge based only on a green development build.

## Handoff instructions

For an Agent task, give the package ID from [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md), a fresh `dev` SHA, and one bounded PR target. Reference #134 without closing it. Include changed interfaces, exact checks actually run, manual platform evidence, retained legacy readers and remaining dependencies. For the lead, begin with R0 fixtures and the R2 backend contract; the user-facing R1 code does not need to be pushed again.
