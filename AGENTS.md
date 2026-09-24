# Agent Guide

## Omera Identity Migration

- Use `docs/ENGINEERING_HANDOFF.md` for ownership boundaries and task acceptance, and `docs/DELIVERY_ROADMAP.md` for dependency order. High-impact persistence, identity, cleanup, security and release changes are lead-owned; general-engineer assignments are explicitly listed there.
- Read `docs/API_CONTRACTS.md` and `docs/IPC_REFERENCE.md` before changing IPC. Use `docs/HANDOFF_PROMPTS.md` for human engineering assignments. Proposed migration APIs are not implemented commands. Regenerate the IPC inventory after command changes and validate DTO compatibility separately.

- The target identity is Omera, `com.berryuiki.omera`, repository `BerryUIKI/Omera`, database `omera.db`, and local settings prefix `omera_`.
- Follow `docs/OMERA_MIGRATION.md` and `docs/STORAGE_EVOLUTION.md`. Core crates are organized as `crates/omera-*`; legacy discovery readers and fallback paths are preserved for pre-1.0 compatibility.
- All pre-1.0 releases must retain legacy discovery and supported import. Preserve source data during migration. Cleanup requires validated destination data and a separate explicit user decision in the application.
- Never include user media, external vaults, or shared directories in automatic legacy application-data cleanup.

This file defines repository-local instructions for coding agents and automated contributors.

## Working Branch

- Use `dev` as the integration branch. Feature and fix branches start from and target `dev`.
- `main` is release-only. Do not commit or open feature pull requests directly against `main`.
- Preserve unrelated local changes. Never rewrite history or run destructive Git commands without explicit approval.

## Architecture Boundaries

- `src/`: Vue 3 and TypeScript UI. Keep expensive filesystem and image work out of the WebView thread.
- `src-tauri/`: thin Tauri command adapters. Commands validate input, release shared locks quickly, and delegate business logic.
- `crates/omera-domain/`: shared models with no I/O dependencies.
- `crates/omera-storage/`: SQLite queries and append-only schema migrations.
- `crates/omera-metadata/`: metadata parsing.
- `crates/omera-scan/`: filesystem indexing and thumbnail generation.
- `tests/`: frontend-side Node tests. Rust unit tests live beside their modules.

Do not place business rules in Tauri commands or Vue templates when they belong in a reusable Rust crate or TypeScript utility.

## Performance Rules

- Gallery work must be proportional to visible items, not total library size, during scroll.
- Keep card width stable when the viewport changes; add or remove columns instead of stretching cards.
- Debounce speculative thumbnail work until scrolling settles. Deduplicate requests and keep decoding concurrency bounded.
- Do not eagerly decode or mount every image. The disk thumbnail cache is allowed to grow lazily from the visible window and a small look-ahead range.
- Startup must render the SQLite-backed library before optional filesystem reconciliation. Never block first paint on a full directory walk.
- New queries over large libraries should be paginated or bounded unless the caller explicitly requires a complete result set.
- Respect `prefers-reduced-motion` for every nonessential animation.

See `docs/PERFORMANCE.md` for the current performance model, implemented safeguards, and prioritized follow-up work.

## Database and Configuration Changes

- SQLite migrations are append-only in `crates/omera-storage/src/migrations.rs`. Never edit an applied migration.
- New configuration fields require compatible defaults in both `src/utils/config.ts` and `src-tauri/src/commands.rs`.
- Keep legacy configuration readable with Serde defaults and localStorage migration where relevant.

## Verification

Run the checks relevant to the change. Before handing off a cross-layer change, run all of these:

```bash
pnpm run build
pnpm run test:stack
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

For gallery changes, also manually verify Grid, Waterfall, and Table modes at narrow and wide window sizes, rapid scrollbar dragging, stack expansion/collapse, keyboard navigation, and reduced-motion mode.

## Documentation and UI Text

- Repository engineering documentation is written in English.
- User-facing UI strings belong in `src/i18n/locales/`; do not introduce untranslated template text unless it is a temporary fallback.
- Update architecture or performance documentation when a change alters lifecycle, caching, virtualization, or persistence behavior.
