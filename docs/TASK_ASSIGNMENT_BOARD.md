# Engineering assignment board

Updated: 2026-09-23. Named people can be assigned by the product owner; role labels below are proposed work packages, not claims that someone is already working.

## Ready to dispatch from dev

Baseline at preparation: `4f57246`. Start from current approved `dev`, record the exact SHA, and read draft PR #138 before touching overlapping files. Each assignment gets a separate PR targeting `dev`.

| Suggested role | First assignment | Prompt | Scope boundary |
| --- | --- | --- | --- |
| Frontend interaction engineer | #119 PromptStatsModal RPC/close defect, then #120 action-event mismatch | E1 / E8 | Do not rewrite the shared dialog system or broad App.vue state; coordinate those pending changes |
| Metadata engineer | #121 Flux/Easy-Use fixtures and parser handling; #122 LoRA footer parsing | E10 | No schema changes, unbounded traversal or network fetches |
| UI/theme engineer | #123 light-theme contrast and select readability | E9 | Avoid config/identity changes; coordinate any shared SettingsModal diff with #138 |
| QA/contract engineer | Reproduction fixtures and coverage matrix for #103/#104/#108/#113 | E11 | Tests must target the chosen baseline; report differences between dev and the draft snapshot |
| Developer tooling engineer | #133 version-source/bump proposal | E12 | Proposal first; release-input implementation requires lead review |

## Coordinate before implementation

| Tasks | Dependency | Work that can proceed now |
| --- | --- | --- |
| E1 shared dialogs; E2 navigation | Existing draft #138 implementation and shared component ownership | Reproductions, keyboard scenarios, design review |
| E3 gallery layout; E4 thumbnails/append | Decide which #138 helpers are adopted; sequence overlapping VirtualGrid edits | Visual acceptance cases and production-helper review |
| E5 refresh/detail cache | Lead confirms mutation/IPC invalidation contract and draft disposition | Before/after query-call instrumentation and delayed-response cases |
| E6 inference/semantic UI | Lead confirms failure-count and hydrated-result contracts | DTO fixtures and mock-driven states |
| E7 remote settings/polling | Existing draft changes to config and polling | Listener lifecycle tests and misleading-state reproductions |
| E8 album imports/folder tree | Safe import result contract and folder-query scope | Interaction prototypes; no direct filesystem mutation |
| E9 #132 branding/version badge, #127 NSFW presentation | Brand/version source and scope/count behavior | Presentation prototypes and localized text |

## Blocked on lead-owned contracts

- #134 migration progress/source-choice/cleanup UI: proposed APIs are not registered. Backend preview tokens, receipts and cleanup revalidation must exist first.
- #128 automatic NSFW classification: persistent manual-override provenance must be specified before automatic writes.
- File collision/retry UI: stable per-file partial-result DTO from L6/#100 is required; do not parse rejection strings.
- Identity/crate/setting/database rename: only after the lead's migration activation gate. Do not perform a global replace as a standalone branding task.
- Release publishing: blocked on signing, installer/upgrade matrix and native qualification.

## Lead review queue

1. Decide draft #138 adoption per issue; split incomplete work and approve exact baselines for dependent engineers.
2. Stabilize migration/config/file-operation APIs and lifecycle/lock ownership before UI integration.
3. Complete migration receipts, source discovery and data-preserving activation; keep cleanup a separately confirmed flow.
4. Evaluate measured schema/index changes, not engine replacement by assumption.
5. Qualify signing and real platform installers before release.

The lead should provide contracts, review and small high-impact changes. Routine implementation is dispatched through [HANDOFF_PROMPTS.md](HANDOFF_PROMPTS.md), with acceptance in [ENGINEERING_HANDOFF.md](ENGINEERING_HANDOFF.md). #118 and remote collaboration remain deferred product work.
