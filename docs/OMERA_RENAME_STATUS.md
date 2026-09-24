# Omera rename: current context and next actions

Updated: 2026-09-23. This is a handoff snapshot, not a release-readiness statement. For the full work inventory and dependency gates, read [OMERA_RENAME_EXECUTION.md](OMERA_RENAME_EXECUTION.md); for data safety, read [OMERA_MIGRATION.md](OMERA_MIGRATION.md).

## Verified repository state

- Integration branch: `dev`.
- [PR #164](https://github.com/BerryUIKI/Omera/pull/164) (R1 visible branding), [PR #166](https://github.com/BerryUIKI/Omera/pull/166) (R0 baseline inventory & R2 migration engine), [PR #167](https://github.com/BerryUIKI/Omera/pull/167) (R3 internal crates and packages rename), and [PR #168](https://github.com/BerryUIKI/Omera/pull/168) (R4 runtime identity activation) are **merged** into `dev`.
- [Issue #134](https://github.com/BerryUIKI/Omera/issues/134) remains open.
- R5 (updater, installers and release assets qualification) is implemented: canonical Omera updater endpoints, strict OS/architecture asset resolution without wrong-platform fallback, support for `OMERA_UPDATE_PUBLIC_KEY`, release workflows for macOS/Windows/Linux/MAS updated, and automated updater test suite.

## What users see now, and what still uses Berry identity

R1 changed the document/window title, title bar, About/update/onboarding/export wording, seven UI locales, default new export ZIP prefix, generated showcase branding and README files.
R3 completed internal crate renames (`crates/omera-*`), package manifests, and imports.
R4 activated `com.berryuiki.omera`, `omera.db`, `omera_*` settings writes with fallback reads, and the `Omera` credential service.
R5 qualifies canonical Omera updater target (`BerryUIKI/Omera`), installer packaging (`Omera_<OS>_<Architecture>.<ext>`), and signature verification with legacy compatibility.

The app icon has not changed. `src/assets/logo.png` and `src-tauri/icons/` need an owner decision if Omera should have a new mark. The current abstract mark may be retained intentionally; do not generate a replacement without an approved design.

## Ordered work queue

| Order | Owner | Work and completion evidence |
| --- | --- | --- |
| 1. R0 inventory | Lead, with Agent research help | **Completed** in `docs/OMERA_RENAME_INVENTORY.md` (PR #166). |
| 2. R1 visible name | Code Agent | **Merged** in PR #164. |
| 3. R2 migration/recovery service | Lead | **Merged** in PR #166 (WAL-preserving migration engine, 7 IPC commands). |
| 4. R3 internal rename | Code Agent; lead review | **Merged** in PR #167 (`crates/omera-*`, Cargo packages, imports). |
| 5. R4 runtime activation | Lead | **Merged** in PR #168 (`com.berryuiki.omera`, `omera.db`, `omera_*`, `Omera` credential service, auto-migration on launch, backwards compatible snapshot format). |
| 6. R5 release/update qualification | Lead with Agent CI help | **Implemented** (Canonical `BerryUIKI/Omera` updater, strict OS/arch asset matching, `OMERA_UPDATE_PUBLIC_KEY`, updated release/MAS workflows, comprehensive updater tests). |
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
