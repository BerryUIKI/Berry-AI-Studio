# Handoff baseline and validation status

Recorded: 2026-09-23. No release readiness is implied.

## Baselines

- Integrated documentation: [PR #136](https://github.com/BerryUIKI/Omera/pull/136), merged into `dev` by the repository owner/workflow. The assistant did not merge it.
- Current `dev` baseline for this documentation follow-up: `4f57246b9aeaa6ce556e960171ab590e58584f3e`. Later documentation translations are preserved.
- Unmerged implementation snapshot: `5fabdd8`, branch `codex/repository-review-fixes`, [draft PR #138](https://github.com/BerryUIKI/Omera/pull/138). Runtime changes were committed at `b1b5afb`; `5fabdd8` only removes an extra blank line. This is a checkpoint of partial remediation, not an approved feature baseline or completed rename.
- IPC_REFERENCE.md describes the 138 command signatures in `5fabdd8`. Proposed migration IPC is not part of that registry.

To inspect/check the snapshot without replacing a working checkout:

```sh
git fetch origin codex/repository-review-fixes
git show 5fabdd8:src-tauri/src/commands.rs
node scripts/generate-ipc-reference.mjs --source-ref 5fabdd8 --check
```

## Completed automated checks on the implementation snapshot

Environment: Windows, repository workspace, available local Rust/Node/pnpm toolchains. These checks were run against the checkpoint content before commit; the final source change was whitespace-only.

| Check | Result |
| --- | --- |
| `pnpm run build` | Passed |
| `pnpm run test:stack` | Passed: 23 tests |
| `pnpm run test:memory` | Passed: 2 production-cache tests |
| `pnpm run bench:scroll` | Passed: 3 production-helper tests/measurements |
| `cargo fmt --check` | Passed |
| `cargo clippy --workspace -- -D warnings` | Passed, including the final recheck |
| `cargo test --workspace` | Passed after fixing Windows connection-release teardown and an obsolete hardcoded schema-version assertion |
| IPC signature generator | All 138 registered snapshot commands parsed; no missing registered declaration |

The storage recovery suite covers source-preserving WAL-inclusive copy, upgrading only the copy, refusing existing destinations, rejection of future schemas, invalid backups, and staged restore with retained recovery data. These tests do not constitute a complete migration coordinator or crash-fault matrix.

The Node benchmark measures helper execution, not WebView painting, image decode, full application memory or native scrolling FPS. No general product speedup is inferred from it.

## Not validated / not complete

- Native Grid, Waterfall and Table matrix at narrow/wide sizes, rapid dragging, stacks, keyboard and reduced motion.
- macOS/Linux build and runtime validation for the working changes, including credential-store dependencies.
- Actual Berry-to-Omera installer migration, App Store/provisioning implications, signed release artifacts, skipped-version upgrades and the next Omera update.
- Legacy source discovery, multi-artifact receipts, source selection, old WebView-origin export/import, final Omera activation and separately approved cleanup.
- Full file-operation fault/partial-result cases, config secret reuse/conflicts, watcher initialization/revocation races and remaining #98–#117 acceptance cases described in the task cards.

Do not close issues or merge draft PR #138 based solely on this table. Treat its changes as inputs to reviewed, issue-sized implementation work. The interface/assignment documents can be distributed independently, but engineers must use the baseline and readiness gate stated for their task.
