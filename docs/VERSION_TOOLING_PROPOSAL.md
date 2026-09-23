# Version Tooling & Release Input Contract Proposal (#133)

## Status
- **Author**: Engineering pair
- **Target Issue**: #133 (Part of E12)
- **Review Requirement**: Lead review required before release integration
- **Scope**: Version bump automation, manifest synchronization, and SemVer consistency verification.

---

## 1. Problem Statement & Motivation
Currently, version identifiers are maintained manually across multiple files in the workspace:
- `package.json` (`version`)
- `Cargo.toml` (`[workspace.package] version`, inherited by all workspace member crates via `version.workspace = true`)
- `src-tauri/tauri.conf.json` (`version`)
- `Cargo.lock`
- `pnpm-lock.yaml`

Manual editing creates risks of partial bumps, drift across manifests, invalid SemVer strings, dirty working-copy commits, and broken builds on CI.

---

## 2. Proposed Architecture & Authority Contract

### 2.1 Authoritative Source
`package.json` is designated as the primary version authority for the repository, with `Cargo.toml` and `src-tauri/tauri.conf.json` kept in strict 1:1 synchronization.

All Rust member crates (`berry-domain`, `berry-storage`, `berry-metadata`, `berry-scan`, `berry-tagger`, `berry-clip`, and `src-tauri`) inherit from `[workspace.package].version` in the root `Cargo.toml`. Therefore, updating the root `Cargo.toml` automatically updates the entire Rust workspace.

### 2.2 Version Files in Scope
| File | Role | Sync Mechanism |
| --- | --- | --- |
| `package.json` | Root npm package version | JSON file edit |
| `Cargo.toml` | Root Cargo workspace package version | TOML `[workspace.package]` replacement |
| `src-tauri/tauri.conf.json` | Tauri 2 application version | JSON file edit |
| `Cargo.lock` | Rust dependency lockfile | Regenerated via `cargo metadata` |
| `pnpm-lock.yaml` | Node dependency lockfile | Regenerated via `pnpm install --lockfile-only` |

---

## 3. Preflight Safety Gates

The version bump script (`scripts/bump-version.mjs`) enforces the following gates before making any modifications:

1. **Clean Working Tree**:
   - `git status --porcelain` must be empty (unless running with `--allow-dirty` for local testing).
   - Prevents partial bumps from polluting unrelated staged or unstaged work.

2. **Current Consistency Check**:
   - Before bumping, the tool verifies that `package.json`, `Cargo.toml`, and `tauri.conf.json` already have identical versions.
   - If a mismatch is detected, the bump refuses to proceed and reports the drift.

3. **SemVer Validation**:
   - The target version must satisfy strict SemVer 2.0.0 (`^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$`).
   - Downgrades are rejected (target must be greater than current version according to SemVer ordering).
   - In accordance with `AGENTS.md` and `DELIVERY_ROADMAP.md`, versions must not be reset to `0.0.0` or prematurely bumped to `1.0.0` before migration stabilization.

---

## 4. Command Line Interface

```bash
# Verify consistency across all manifests without modifying files
node scripts/bump-version.mjs --check

# Dry run: preview planned changes
node scripts/bump-version.mjs patch --dry-run
node scripts/bump-version.mjs 0.3.1 --dry-run

# Perform version bump: patch, minor, major, or explicit version
node scripts/bump-version.mjs patch
node scripts/bump-version.mjs minor
node scripts/bump-version.mjs 0.4.0
```

---

## 5. Non-Goals & Release Safety Boundaries

In strict compliance with `docs/ENGINEERING_HANDOFF.md`:
- **No Git Tags or Remote Pushing**: The script only updates local manifests and lockfiles; it does not commit, create git tags, or push to remotes.
- **No Automated Release Trigger**: Bumping versions does not trigger CI deployment or release publishing.
- **No Identity Mutation**: Bundle identifiers (`com.berryuiki.*`), binary names, and storage paths remain unchanged by this tool; identity migration is gated by lead-owned Phase 3.
