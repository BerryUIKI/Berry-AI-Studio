# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Omera** is an open-source **local asset manager and prompt studio for
AI-generated images**: it scans folders of images/videos, extracts prompt/model/
parameter metadata (PNGInfo, EXIF, `.txt` sidecars), and indexes it into SQLite
for search, organization (albums, tags, favorites, ratings, NSFW), and model
lookup.

This is a **clean-slate rewrite** (v3.0) on the `rewrite`/`dev` branches. The
legacy C#/.NET codebase is archived read-only on `old/main`. Milestones 1 through 6
(Scaffolding, Scanning, Browsing, Search, Organization, Performance/Polish) are complete;
see [docs/ROADMAP.md](docs/ROADMAP.md) for milestone status.

## Build, Test & Lint

Frontend tooling is **pnpm**; Rust uses a single workspace at the repository root.

```bash
pnpm install                      # install frontend dependencies
pnpm tauri dev                    # run the desktop app (hot reload)
pnpm build                        # vue-tsc type-check + vite build → dist/
pnpm tauri build                  # release bundle (installer)

cargo test --workspace            # all Rust tests (all 4 crates)
cargo clippy --workspace --all-targets -- -D warnings   # must be clean
cargo fmt --all -- --check        # must be clean
```

- Rust package manager: `cargo add <crate>` inside a crate dir.
- Run a single test: `cargo test -p berry-storage db::tests::migrations_are_idempotent`.
- **`dist/` must exist before any cargo command that compiles `src-tauri`**
  (`tauri::generate_context!` embeds `frontendDist` at compile time). CI runs
  `pnpm build` before the cargo steps; locally, `pnpm tauri dev`/`build` handles
  this automatically.

## Architecture

```
Frontend (src/, Vue 3 + TS) ── invoke() ──► src-tauri/ (app shell, #[tauri::command])
                                              │
                                              ▼
                              crates/omera-domain   (pure domain types)
                              crates/omera-metadata (format detection & parsing)
                              crates/omera-storage  (SQLite + migrations)
```

A single Cargo workspace (`Cargo.toml`) — run cargo commands from the root.

| Crate | Responsibility | Notes |
|---|---|---|
| `omera` (`src-tauri/`) | Tauri shell: window setup, IPC commands, `AppState` | Thin adapters only — **no business logic** |
| `omera-domain` | `ImageFile`, `Container`, `MetadataFormat` | Depends on nothing in-repo |
| `omera-metadata` | `detect_container` (magic bytes), `extract_metadata` dispatch: PNGInfo (`parameters` chunk), EXIF (`Software` tag + dimensions), `.txt` sidecar fallback | Depends on domain + kamadak-exif |
| `omera-scan` | `Scanner`: walk → detect container → extract → batch upsert → orphan cleanup; incremental (size, mtime) skip + forced rebuild | Depends on domain + metadata + storage |
| `omera-storage` | `Database`, ordered `MIGRATIONS` | Depends on domain + rusqlite(bundled) |

**Data flow:** the frontend calls `invoke("get_app_info", …)`; the command locks
`AppState` (a `Mutex<Database>` opened in `.setup()` from the OS app-data dir),
calls into a core crate, returns a serde value or `Result<_, String>`.

**Schema versioning:** `crates/omera-storage/src/migrations.rs` holds an ordered
`MIGRATIONS: &[&str]`; `Database::migrate()` applies each pending migration in a
transaction and bumps `PRAGMA user_version`. **Never edit/reorder/delete an
applied migration — append a new one.** No ad-hoc DDL.

Full details: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Branch Policy (critical)

Read [BRANCH_POLICY.md](BRANCH_POLICY.md) before branching:

- **`main`** is the default, **protected** branch — official releases only,
  direct pushes forbidden, PR-only. It accepts release PRs **from `dev` only**.
- **`dev`** is the development integration branch — **all** feature/fix/chore/
  docs PRs target `dev`.
- Feature branches branch off `dev`, named `feature/<name>`, `fix/<name>`,
  `chore/<name>`, or `docs/<name>`.
- Conventional commits (`feat:`, `fix:`, `chore:`, `docs:`).

**Practical implication for Claude:** never create branches off `main`, never
commit directly to `main`, and point PRs at `dev` — not the default branch.

## Development Workflow

1. `git checkout dev && git pull origin dev && git checkout -b feature/your-feature`
2. Implement with tests (unit tests live next to the code, `#[cfg(test)]`)
3. Push and open a PR targeting `dev`
4. `dev` → `main` release PRs happen only when `dev` is release-ready

Prerequisites: stable Rust, Node.js LTS, pnpm, Tauri 2 platform prerequisites.
