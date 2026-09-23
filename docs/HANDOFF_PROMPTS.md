# Engineer handoff prompts

Use one prompt per assignment. Replace `BASELINE_COMMIT` with the maintainer-provided implementation commit before dispatch. Do not substitute the documentation-only PR commit. Read [ENGINEERING_HANDOFF.md](ENGINEERING_HANDOFF.md) for task cards and [DELIVERY_ROADMAP.md](DELIVERY_ROADMAP.md) for dependencies.

Use [TASK_ASSIGNMENT_BOARD.md](TASK_ASSIGNMENT_BOARD.md) to choose a ready assignment and its baseline. `4f57246` is the current `dev` starting point for independent tasks; `5fabdd8` is an unmerged review snapshot, not automatically approved for integration. If `dev` has advanced, record its new exact SHA and check overlap before dispatch. Validation evidence is in [VALIDATION_STATUS.md](VALIDATION_STATUS.md).

These prompts assign implementation to ordinary engineers while the lead owns architecture, persistence, migration, update trust and release qualification. They do not authorize publishing releases, merging to main, rewriting migrations, deleting legacy data, or adding undocumented backend APIs.

## Standard assignment wrapper

Include this wrapper with each task prompt if the engineer does not already have the repository instructions:

```text
Work in BerryUIKI/Omera. Your approved implementation baseline is BASELINE_COMMIT.
Read AGENTS.md, docs/ENGINEERING_HANDOFF.md, docs/API_CONTRACTS.md,
docs/IPC_REFERENCE.md, and your phase in docs/DELIVERY_ROADMAP.md first.
Use a codex/<issue>-<description> branch based on the approved dev integration;
target dev with your PR. If the baseline is not available in that integration,
report the dependency rather than inventing a base or discarding local changes.

Implement only the assigned task. Preserve legacy compatibility and unrelated
work. The runtime rename, schema, credential store, cleanup authorization,
installer identity and signing trust belong to the lead. Proposed migration APIs
are not implemented: use explicit mocks for UI exploration and report the gate;
do not wire production code to an invented command. Keep UI strings in all
supported locale files and engineering documentation in English.

Reproduce the bug before changing code. Reuse existing production helpers rather
than reimplementing their logic in tests. Keep scope small, update affected
interface documentation, run applicable checks, and record manual QA where
required. Do not claim an unrun check passed. Do not merge, publish, or close an
issue just because a draft implementation exists.

Deliver: PR URL targeting dev; issue and exact baseline/commit; concise behavior
change; touched interfaces; automated/manual evidence; risks and remaining
dependencies. If blocked, provide the failing command or missing contract and
the smallest decision needed from the lead. Avoid broad refactors outside scope.
```

## E1 — Dialog and prompt-statistics interaction

```text
Implement task E1 in docs/ENGINEERING_HANDOFF.md, starting with issue #119, then
address #103 in a separate PR. Read AGENTS.md and docs/API_CONTRACTS.md.
Use the approved BASELINE_COMMIT and target dev.

Verify get_prompt_stats uses { isNegative, limit }; inspect every caller and the
registered command. Fix PromptStatsModal close behavior. Audit the existing
v-dialog directive for nested dialogs, focus trapping/return, Escape, inert
backgrounds and teardown. Gallery shortcuts must not run behind a dialog or
inside editable controls. Do not redesign persistence or the command schema.

Record keyboard-only and repeated-open/close evidence. Include a regression test
that would fail with the actual old behavior. Report remaining dialogs separately.
```

## E2 — Gallery navigation and recovery states

```text
Implement E2 / #104 from docs/ENGINEERING_HANDOFF.md. Read AGENTS.md,
docs/API_CONTRACTS.md and the current gallery-navigation helper. Use
BASELINE_COMMIT and target dev.

Preserve anchor file plus offset across contexts, page loading and view modes.
Handle missing anchors without infinite paging. Fix library/folder-empty,
no-match and error actions with localized text. Preserve selection consistency.
Do not replace cursor APIs or alter the database schema.

Verify Grid, Waterfall and Table at narrow/wide widths, deep navigation, deleted
anchors and rapid context changes. State which manual cases actually ran.
```

## E3 — Layout and stack presentation

```text
Implement E3 from docs/ENGINEERING_HANDOFF.md, one issue per PR: #129, #126,
then #125. Read AGENTS.md and docs/PERFORMANCE.md. Use BASELINE_COMMIT; target dev.

Keep the action dock inside the gallery with sidebars open and support a single
selected item. Reduce expanded-stack badge repetition with keyboard-accessible
hover/focus controls. For #125 preserve fixed card width; solve whitespace with
alignment/spacing rather than stretching cards. Explain any product conflict
before implementing a contradictory layout.

Verify all gallery modes, narrow/wide layouts, stack expansion and keyboard use.
Coordinate App.vue ownership with E2/E4. Do not touch schema or safe file services.
```

## E4 — Incremental gallery and thumbnail retry

```text
Implement E4 / #111 and #115 as separate bounded PRs. Read AGENTS.md,
docs/ENGINEERING_HANDOFF.md and docs/API_CONTRACTS.md. Use BASELINE_COMMIT; target dev.

Finish retry/failure presentation in Grid and Table without decoding originals.
Test dedupe, cancel/clear generations, changed source revisions and rapid scrolling.
Use the existing GalleryPages, WaterfallGeometry and LruThumbnailCache helpers;
validate page append, hero replacement, resize and reactive spacer lengths.
Keep scrolling proportional to the visible window, and append work proportional
to the new page. Backend database connection/lifecycle changes are lead-owned.

Run production helper tests and the required real gallery matrix. Report helper
timings as helper timings, not browser FPS. Coordinate shared files with E2/E3.
```

## E5 — Refresh and hydrated-detail correctness

```text
Implement E5 / #114 and #116 from docs/ENGINEERING_HANDOFF.md in separate PRs.
Read AGENTS.md and docs/API_CONTRACTS.md. Use BASELINE_COMMIT; target dev.

Finish mutation-aware/coalesced refresh scheduling using actual event payloads.
Record startup and mutation IPC counts before/after. Fence stale in-flight detail
requests and invalidate on metadata mutations; never overwrite newer ratings,
favorites or NSFW state with an old hydrated row. Preserve currently selected
metadata refresh when the file ID stays unchanged. Do not add SQL/schema changes
without a lead-owned contract.

Test delayed responses, repeated navigation, metadata rebuild and concurrent
flags/ratings updates. Explain which redundant calls were removed and why.
```

## E6 — Inference feedback and semantic consumers

```text
Implement the frontend part of E6 / #105 and #109; verify #117 consumers without
changing ranking/storage algorithms. Read AGENTS.md and docs/API_CONTRACTS.md.
Use BASELINE_COMMIT and target dev.

Honor indexed/failed/remaining counts, preserve failure visibility, and separate
cancel from success. Do not continuously reset the failed set. Consume hydrated
SimilarFileItem results correctly and ignore stale query responses. Test empty,
all-failed, partial, canceled and loaded-model-change cases with realistic mocks.
Request a lead decision if the existing backend count contract cannot express a
case; do not patch it by guessing counts in the UI.
```

## E7 — Remote-feature honesty and polling teardown

```text
Implement E7 / #110 and #113 from docs/ENGINEERING_HANDOFF.md. Read AGENTS.md and
docs/API_CONTRACTS.md. Use BASELINE_COMMIT; target dev.

Keep MySQL/PostgreSQL unavailable as runtime choices; a connectivity test is not
working storage routing. Verify SQLite mode does not start collaboration polling.
Test the production CollaborationSyncEngine's visibility listener identity,
stop/restart generation fencing, timers and disposal while a request is pending.
Do not implement a new remote backend or change config persistence semantics.
```

## E8 — Actions, album drops and folder navigation

```text
Implement E8 in docs/ENGINEERING_HANDOFF.md, beginning with #120; keep #130 and
#131 in separate PRs. Read AGENTS.md and docs/API_CONTRACTS.md. Use BASELINE_COMMIT;
target dev.

Trace each BatchActionBar emit to its actual listener and command. Preserve every
selected ID on album drag/drop. For external managed-vault drops, consume only an
approved safe import service; do not write filesystem logic in Vue. Add explicit
direct/recursive folder scope and stable tree expansion without mounting an
unbounded hierarchy. New backend hierarchy-query contracts need lead review.

Test actual action behavior, multi-item drops, missing IDs, deep trees and keyboard
access. If import contracts are missing, deliver the independent fixes and name
the dependency instead of silently weakening collision safety.
```

## E9 — Theme and product presentation

```text
Implement E9 from docs/ENGINEERING_HANDOFF.md. Start with #123, then #132 and #127
in separate PRs. Read AGENTS.md and docs/API_CONTRACTS.md. Use BASELINE_COMMIT;
target dev.

Fix light-theme contrast and native/select option readability. Connect the title
version badge to the existing update modal and add the Omera repository link.
Use approved brand/version values; do not change bundle ID or installer metadata.
Implement NSFW Blur/Hide/Show presentation with accessible localized controls.
#128 automatic classification must wait for the lead-owned manual-override
persistence contract; do not infer that all existing flags are automatic.

Verify seven locales, light/dark/system themes, keyboard focus and consistent
hidden-item selection/count behavior. Report the #128 dependency explicitly.
```

## E10 — Metadata parsers

```text
Implement E10 / #121 and #122 from docs/ENGINEERING_HANDOFF.md, one parser family
per PR. Read AGENTS.md and existing parser tests. Use BASELINE_COMMIT; target dev.

Add sanitized real fixtures for Flux CLIPTextEncodeFlux/Easy-Use linked prompt
graphs and WebUI/Liblib Lora N footer fields. Keep traversal bounded with cycles,
missing nodes and malformed data. Preserve existing metadata and LoRA behavior.
Do not fetch URLs or modify schema during parsing. Run the relevant Rust suites
and show the exact before/after extracted metadata for representative fixtures.
```

## E11 — Contract and UI regression coverage

```text
Implement E11 / #108 from docs/ENGINEERING_HANDOFF.md. Read AGENTS.md,
docs/API_CONTRACTS.md and docs/IPC_REFERENCE.md. Use BASELINE_COMMIT; target dev.

Extend tests against production helpers/components and actual command DTOs.
Check command inventory drift with the provided generator, then add DTO fixtures
that check field names and error shapes; a signature list alone is not enough.
Add lifecycle tests for stale requests/listeners and the changed dialog/gallery
behavior. Integrate checks into CI after the approved baseline contains their
referenced commands. Avoid duplicated production logic and flaky timing limits.

Deliver a coverage matrix mapping issues to tests and manual scenarios. Clearly
separate tested Node logic from native GUI/installer validation not performed.
```

## E12 — Version tooling proposal and implementation

```text
Prepare E12 / #133 from docs/ENGINEERING_HANDOFF.md. Read AGENTS.md,
docs/RELEASING.md and docs/DELIVERY_ROADMAP.md. Use BASELINE_COMMIT; target dev.

First propose the authoritative version source, generated files, validation,
failure/rollback behavior and developer command. After lead approval, implement
one-command version updates with a dry run and exact changed-file reporting.
Keep all Cargo/Tauri/package versions consistent, preserve lockfile correctness,
and reject invalid or partial updates. Never reset versions for Omera, modify
signing keys or publish a release. Include fixture-based tests of failure paths.
```

## Review/report template

```text
Assignment / issue:
Baseline commit:
PR and final commit:
Behavior changed:
Interfaces changed (or none):
Automated checks (exact commands and outcomes):
Manual checks (OS, viewport, dataset, outcomes):
Remaining failures / dependencies:
Persistence / security / release review needed:
Follow-up work intentionally excluded:
```

The lead reviews correctness, contract stability and scope before integration. A passed build does not authorize data cleanup or release publication. Keep deferred product work (#118 and remote collaboration) out of the rename stabilization path.
