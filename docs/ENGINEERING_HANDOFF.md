# Omera engineering handoff

Updated: 2026-09-23. This is an execution plan, not a completion report.

Start here: [API_CONTRACTS.md](API_CONTRACTS.md), [IPC_REFERENCE.md](IPC_REFERENCE.md), and copy-ready [HANDOFF_PROMPTS.md](HANDOFF_PROMPTS.md). Existing and proposed interfaces are explicitly separated. Engineers must not wire production code to a proposed migration command.

## Ownership and working rules

The lead maintainer/Codex owns high-impact changes: application identity and data locations, legacy discovery and migration, cleanup authorization, schema evolution, database restore, credential persistence, update trust, installer transition, and the architecture/release documentation. These tasks are **reserved**, not assignments for general implementation engineers.

General implementation engineers own the bounded GUI, localization, parser, test, and performance tasks below. This document delegates work to humans; it does not start automated agents. Work on one issue per branch/PR where practical, based on and targeting `dev`. `main` is release-only. Follow `AGENTS.md`; do not rewrite applied migrations or perform a global Berry-to-Omera replacement. Preserve BerryUIKI attribution and legacy fixtures.

The integration branch is `dev`; `codex/repository-review-fixes` is an unmerged review branch. The existing code work is preserved at commit `5fabdd8` in [draft PR #138](https://github.com/BerryUIKI/Omera/pull/138), not a release candidate. Documentation PR #136 has merged. The current documentation follow-up starts from `dev` commit `4f57246`. See [VALIDATION_STATUS.md](VALIDATION_STATUS.md) for exact evidence and [TASK_ASSIGNMENT_BOARD.md](TASK_ASSIGNMENT_BOARD.md) for ready versus gated work. Do not silently use the draft snapshot as an approved integration baseline or duplicate changes already present in it.

The GitHub repository has already moved to [BerryUIKI/Omera](https://github.com/BerryUIKI/Omera). Runtime product/identifier, database, crates, and settings have **not** yet completed migration. Existing `berry-*` source paths in this handoff are intentional until the maintainer's coordinated rename lands.

## Reserved high-impact work

| ID | Scope | Gate before dependent work |
| --- | --- | --- |
| L1 / [#134](https://github.com/BerryUIKI/Omera/issues/134) | Full Omera identity; pre-1.0 discovery; resumable database/config/credential migration; verified, separately confirmed cleanup | Follow [OMERA_MIGRATION.md](OMERA_MIGRATION.md); stable status/preview/execute contracts before cleanup UI |
| L2 / [#98](https://github.com/BerryUIKI/Omera/issues/98) | Safe staged restore and startup connection lifecycle | WAL-inclusive snapshots, no active file replacement, retained recovery copies, interrupted restore tests |
| L3 / [#106](https://github.com/BerryUIKI/Omera/issues/106), [#112](https://github.com/BerryUIKI/Omera/issues/112) | Atomic authoritative config, credentials, revision conflicts, legacy key precedence | Compatible defaults on both sides; old origin export/import fallback; no secret loss or repeated migration |
| L4 / [#101](https://github.com/BerryUIKI/Omera/issues/101) | Signed updates, explicit platform/architecture selection, new repository trust, real installer migration | Release signing provisioned; tampered/missing signatures rejected; direct old-version and subsequent-update tests |
| L5 / [#135](https://github.com/BerryUIKI/Omera/issues/135) | Measured schema/query evolution and storage decision | [STORAGE_EVOLUTION.md](STORAGE_EVOLUTION.md); baseline before a schema/index change |
| L6 / [#99](https://github.com/BerryUIKI/Omera/issues/99), [#100](https://github.com/BerryUIKI/Omera/issues/100) | File-operation data safety, partial-result contract and recovery journal | No overwrite or permanent-delete fallback; safe sidecars; backend error/partial-success fixtures |
| L7 / [#107](https://github.com/BerryUIKI/Omera/issues/107), [#124](https://github.com/BerryUIKI/Omera/issues/124) | Asset/CSP boundaries and single-instance lifecycle | Registered-root scope and revoke behavior; no concurrent migration or restore writers |
| L8 / [#102](https://github.com/BerryUIKI/Omera/issues/102) | Shared-state ownership and blocking command execution | Named per-command lock/worker contracts and cancellation; no global lock held across network, decode, or long inference |

Engineers may submit reproductions, fixtures, UI proposals, or measurements for L1–L8, but implementation requires lead ownership. UI cleanup confirmation must never implement deletion by itself.

## General-engineer task cards

### E1 — Modal and keyboard behavior (#103, #119)

Files: `src/utils/dialog.ts`, modal components, `src/App.vue`, `src/components/VirtualGrid.vue`.

- Continue the existing shared `v-dialog` implementation. Audit nested dialogs, visible focus, focus return, Tab/Shift+Tab, Escape, and background inertness. Remove duplicate global listeners with correct teardown.
- Include PromptStatsModal: resolve its documented RPC argument mismatch and close behavior; do not change the Rust command's schema without checking every caller.
- Keep editing shortcuts inside inputs; gallery shortcuts must not fire behind a modal. Give every dialog a localized accessible name.
- Acceptance: keyboard-only open/use/close, nested-dialog dismissal, opener removed while open, repeated mount/unmount, and reduced-motion cases work. Record browser/manual evidence; source-text assertions alone are insufficient.

### E2 — Gallery navigation and empty states (#104)

Files: `src/utils/gallery-navigation.ts`, `VirtualGrid.vue`, `FileList.vue`, `App.vue`.

- Verify the in-progress anchor-by-file/offset restoration across folder, search, sort and display-mode transitions. Restore beyond the first page without unbounded concurrent loads.
- Handle deleted anchors, changed filters, empty libraries, empty folders, no search matches, and query errors with the correct localized action.
- Acceptance: no jump to a different image after page reload; no infinite paging when an anchor disappears; new contexts start predictably; recover actions work without stale selection.

### E3 — Layout, selection dock and stack presentation (#125, #126, #129)

Files: `VirtualGrid.vue`, gallery CSS, `BatchActionBar.vue`, `App.vue`.

- Preserve the AGENTS.md fixed-card-width rule. Issue #125's request for less right-side whitespace must not introduce stretching cards; propose alignment/gap treatment within that rule.
- Keep selection actions within the gallery viewport and usable at narrow widths and with sidebars. Support the requested single-selection dock without obscuring content.
- Reduce repeated badges on expanded stacks and add discoverable hover/focus behavior with equivalent keyboard access.
- Acceptance: Grid/Waterfall/Table at narrow and wide sizes; 1/many selected items; open/closed sidebars; rapid dragging and stack expansion/collapse; no layout overflow.

### E4 — Thumbnail presentation and incremental gallery work (#111, #115)

Files: `src/utils/thumbnail.ts`, `lru-cache.ts`, `gallery-state.ts`, `VirtualGrid.vue`, `FileList.vue`. Backend pool/lifecycle changes require lead review.

- Preserve failure placeholders and offer a working retry in Grid and Table. Never fall back to decoding full originals for failed thumbnails.
- Verify revision/generation invalidation, in-flight deduplication, bounded concurrency, and lazy look-ahead. Do not repopulate a cleared cache from old requests.
- Continue the real `GalleryPages`/`WaterfallGeometry` helpers: append work should depend on the new page, while scroll work depends on the visible window. Account for reactive length/spacer updates after in-place appends, hero replacement, and changed geometry.
- Acceptance: production-helper tests and real UI tests cover append, resize, source replacement, retry, cancellation and cache clearing. No whole-library image mounting or eager decoding.

### E5 — Refresh scheduling and detail cache (#114, #116)

Files: `App.vue`, existing library-change payload types. Lead review required for new SQL.

- In-progress code bulk-loads album counts and renders initial files earlier. Finish mutation-aware refresh scheduling instead of reloading every sidebar/facet before every gallery refresh.
- Complete stale in-flight detail generation fencing and invalidation after metadata mutations. Merge hydrated metadata into current UI state without reverting newer ratings/favorites/NSFW values.
- Acceptance: count IPC calls for startup, rating change, scan completion and one-folder file changes; stale detail responses cannot resurrect invalidated data; selected metadata updates even when file ID is unchanged.

### E6 — Inference feedback and semantic UI (#105, #109, #117)

Files: `ClipManagerModal.vue`, semantic-search consumers and related tests. Storage/inference algorithm changes remain lead-reviewed.

- Use backend failed/remaining counts. Show completion only when no work remains; distinguish failures, cancellation and success. A failed first batch must not starve later images.
- Verify `SimilarFileItem` IPC payloads are consumed consistently, stale queries are ignored, and result grouping respects the current scope.
- Acceptance: all-failed, partially failed, canceled, empty, stale-query and successful-index cases. Backend top-k/bulk hydration changes need deterministic ranking and dimension-mismatch tests before closure.

### E7 — Unsupported remote settings and polling lifecycle (#110, #113)

Files: `SettingsModal.vue`, `MigrationWizardModal.vue`, `src/utils/collaborationSync.ts`.

- Keep unavailable remote storage visibly unavailable. A successful TCP/database ping must not imply the application actually uses MySQL/PostgreSQL.
- Preserve the existing stop/restart generation fence and stable visibility listener. Add real production-engine tests for listener removal and stale requests, including stop while a poll awaits completion.
- Acceptance: no hidden polling in SQLite-only mode, no duplicate timers/listeners, no unsupported backend saved as active. Do not implement a remote backend in this task.

### E8 — Local interaction defects (#120, #130, #131)

Files: `BatchActionBar.vue`, album/drop consumers, sidebar/folder tree, `App.vue`.

- Match emitted action names to listeners; regression-test actual selection-to-command behavior.
- Preserve all selected IDs during album drag/drop. External file drops into managed vaults must call the lead-owned safe import service, not perform ad-hoc filesystem writes.
- Add hierarchical folder navigation, explicit direct/recursive scope, and stable expansion state without recursively mounting every library row.
- Acceptance: multi-select drag, missing IDs, non-managed targets, keyboard actions, deep folders and empty nodes; no silent command failure or accidental file movement.

### E9 — Theme, NSFW display and localized branding UI (#123, #127, #128, #132)

Files: `TitleBar.vue`, theme CSS, seven locale files, relevant cards/settings.

- Fix light-theme dropdown contrast and hardcoded dark surfaces. Implement discrete NSFW badges and Blur/Hide/Show presentation with accessible controls.
- Prompt-based NSFW classification must preserve explicit user overrides; storage needed to distinguish manual/automatic decisions is a lead-owned prerequisite, not an improvised schema edit.
- Make the version badge open the existing update modal and add the Omera GitHub link. Use shared version/brand values after L1/#133 establishes them.
- Acceptance: all seven locales compile, keyboard focus is visible, light/dark/system themes work, manual classification wins, and hidden results/counts are consistent.

### E10 — Metadata parser fixtures (#121, #122)

Files: `crates/berry-metadata/`, parser tests and Inspector display.

- Add sanitized real fixtures for ComfyUI Flux/CLIPTextEncodeFlux, supported Easy-Use chains and linked prompt nodes; handle cycles/missing nodes with bounded traversal.
- Parse WebUI/Liblib footer `Lora N` entries and retain existing LoRA extraction behavior.
- Acceptance: positive and malformed fixtures, no parser panic, stable existing metadata tests, no fetching arbitrary URLs while parsing.

### E11 — Production regression coverage and CI (#108)

Files: `tests/`, `.github/workflows/ci.yml`, `package.json`.

- Existing changes replace copied LRU/waterfall test implementations with production imports and add frontend suites to CI. Extend coverage for E1–E10 using actual helpers/components.
- Keep benchmarks observational with hardware/commit/dataset metadata. Do not call Node helper timings browser frame times or assert arbitrary timing thresholds on shared CI runners.
- Acceptance: CI runs build, stack, memory and scroll suites plus the repository Rust checks. Record manual gallery QA separately; passing helper tests does not replace it.

### E12 — Version tooling (#133)

Create a one-command version bump using an agreed authoritative source. Update generated manifests and lockfiles consistently; validate SemVer and refuse a dirty/conflicting partial bump. This touches release inputs: propose the contract first and get lead review before integrating. Do not reset versions for Omera or trigger a release from the bump command.

### Deferred product request (#118)

The ComfyUI batch conversion/compression node is a new feature, not a prerequisite for rename safety. Specify node inputs, outputs, supported image formats, destination/collision behavior, cancellation and packaging before implementation. Reuse safe export services; schedule after the migration release stabilizes.

## In-progress code: known limits

- The working tree already contains attempts for #98–#117. They are not all complete, and no issue should be closed merely because related code exists.
- `recovery::migrate_copy` is a tested database-copy primitive, not the complete L1 migration coordinator, receipt, discovery, or cleanup UX.
- Omera identity activation is pending. Old WebView-origin settings and macOS sandbox access still require explicit migration coverage.
- Signed update verification exists in progress; signing-key provisioning, release signing and real installer tests remain incomplete.
- Long-running synchronous inference/network handlers remain under #102. Split this work into named commands and document lock ownership; do not mechanically mark handlers async while leaving blocking work on the runtime thread.
- Thumbnail worker connections now have an explicit release hook. Lifecycle callers must stop submissions before release; the hook alone is not concurrent restore safety.
- CSP, watcher initialization/revocation, config revision conflicts, credential entry reuse, file-operation partial results and detail invalidation need their acceptance cases completed.
- Native GUI and installer acceptance tests have not been completed. Do not label the working tree release-ready.

## Definition of done

Each PR includes its issue, exact behavior change, validation evidence and remaining limitations. Use English engineering documentation and localized user-facing strings. Ask for product clarification only when the issue and this document cannot resolve an ambiguity. Report a blocker with the failing command/error and the smallest required dependency.

Required cross-layer checks:

```sh
pnpm run build
pnpm run test:stack
pnpm run test:memory
pnpm run bench:scroll
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Gallery work additionally requires Grid, Waterfall and Table at narrow/wide sizes, rapid scrollbar dragging, stack expansion/collapse, keyboard navigation and reduced-motion mode. Record OS, screen size, dataset and observed results. Publishing and issue closure follow verified integration, not task assignment.
