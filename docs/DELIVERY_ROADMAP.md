# Omera delivery roadmap

This roadmap separates lead-owned safety work from general-engineer implementation. Detailed task cards and issue links are in [ENGINEERING_HANDOFF.md](ENGINEERING_HANDOFF.md). No dates are promised before dependencies and validation are complete.

Dispatch work using [HANDOFF_PROMPTS.md](HANDOFF_PROMPTS.md). Interface changes must follow [API_CONTRACTS.md](API_CONTRACTS.md) and update [IPC_REFERENCE.md](IPC_REFERENCE.md). The lead provides contracts and reviews high-impact changes; ordinary engineering tasks should be assigned rather than folded into an unbounded lead implementation branch.

## Phase 0 — Establish a reproducible baseline

Lead: commit the accepted identity/storage documents, preserve the current working diff, run the required checks, and publish an explicitly identified implementation baseline for engineers. Document failed/incomplete checks. Do not distribute the documentation-only commit as if it contains code fixes.

Engineers: reproduce their assigned issues against that baseline; attach sanitized fixtures and concise reproduction steps. Do not start overlapping changes to `App.vue` without agreeing on ownership/order.

Exit: engineering handoff, issue assignments and baseline reference exist; open defects are distinguished from completed work.

## Phase 1 — Safety and migration contracts

Lead: L1–L8. Keep SQLite, establish legacy discovery/receipt semantics, staged copy/restore, new config/key mapping and credential policy, cleanup previews and authoritative update/installer identity. Append schema changes only after measurements and compatibility review.

Engineers: E10 parser fixtures, E11 test infrastructure and isolated E1/E7 reproductions can proceed independently. E12 may propose version tooling but cannot modify release identity yet.

Exit: migration/restore fault tests pass, frontend contracts are stable, ownership of paths/keys is documented, and cleanup cannot run without a validated receipt plus user confirmation.

## Phase 2 — UI and bounded performance work

Engineers: E1–E9 in small PRs. Recommended sequence in shared files: E1 keyboard boundaries -> E2 navigation -> E3/E4 layout and rendering -> E5 refresh/cache -> E6 semantic state -> E8 interactions. E9 theme/parser display can proceed on non-overlapping components. Coordinate `App.vue`, gallery and settings edits rather than independently rewriting them.

Lead: review changes crossing persistence, filesystem, credential, update, or security boundaries. Run L5 benchmarks before approving schema/index changes.

Exit: each issue's functional cases pass, production helper tests run in CI, and full manual gallery/accessibility evidence is recorded.

## Phase 3 — Coordinated identity activation

Lead: switch to `com.berryuiki.omera`, `omera.db`, `omera_*`, Omera credentials, executable/packages/crates and the Omera repository only after migration services are ready. Audit source, workflow, installer, backup format and exported branding. Preserve compatibility aliases and historical migrations. Review E12 version tooling and E9 branding integration.

Engineers: update localized presentation against the approved brand constants; implement migration progress/source-choice/cleanup UI against the lead-owned commands. Show exact previewed cleanup paths; keep the default action non-destructive.

Exit: fresh install and supported Berry-to-Omera migrations pass, including skipped versions, inaccessible legacy settings, multiple libraries, failures and declined cleanup. Legacy source data remains intact unless the user explicitly approved verified cleanup.

## Phase 4 — Release qualification

Lead: verify signing configuration and real Windows/macOS/Linux target installers, plus Omera-to-next-Omera updates. Verify data, credentials, shortcuts, uninstall records and restore after update. Publish migration/recovery instructions and known limitations.

Engineers: finish E11 QA evidence and run regression checks against the exact candidate commit. Do not infer success on untested architectures.

Exit: required automated checks and platform/manual matrix pass; release blockers are resolved; only then integrate through `dev` to the release branch and publish. A renamed development build is not a qualified release.

## Phase 5 — Stabilization and 1.0 review

All pre-1.0 releases retain legacy detection and supported import. Keep cleanup optional. Resolve verified migration problems before removing compatibility paths. Decide the 1.0 import policy explicitly and document it; never use version 1.0 as permission to erase legacy data. Remote collaboration remains a separate product decision after stabilization.

## Phase 6 — Library image transformation (planned; issue #118 design)

Follow [IMAGE_TRANSFORM_PLAN.md](IMAGE_TRANSFORM_PLAN.md). This product work begins after the identity/migration and safe file-operation gates, rather than joining the release-critical rename path. Sequence: repair the existing export codec, metadata and write-result behavior; add export preview/progress; add optional transform during managed import; then offer verified post-import batch transformation with keep/archive/Trash choices. Target-size controls follow only after the core flows are qualified.

The lead owns catalog identity, staged filesystem publication, metadata policy, source disposition, archive/recovery and any schema/API decision. General engineers can implement bounded export/UI packages against approved contracts. Exit requires codec/extension, source-preservation, collision, interruption/restart and archive-restore evidence; documentation or an export-only PR does not claim the full feature shipped.
