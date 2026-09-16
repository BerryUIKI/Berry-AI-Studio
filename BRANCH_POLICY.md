# Branch Management Policy

## Overview

The repository workflow is structured around two primary long-lived branches: `main` (production release branch) and `dev` (active development integration branch). The branch workflow is:

> **All pull requests target `dev` first. `dev` is merged into `main` only
> as a unified release PR. `main` carries official tagged releases only.**

## Branch Structure

### `main` — Official Release Branch (default, protected)

- 🔒 **Protected branch** — direct pushes are **FORBIDDEN**
- **Accepts release PRs from `dev` only** (no feature branches target `main`)
- Linear history enforced (rebase / squash, no merge commits)
- Force pushes and deletions are blocked
- Conversation resolution required
- Every merge into `main` is a formal release (tagged `vX.Y.Z`)

### `dev` — Development Integration Branch

- All feature / fix / chore / docs branches target `dev`
- Acts as the staging area; code lands here first and is integration-tested
- When `dev` reaches a release-ready state, a single release PR merges `dev` → `main`

### Feature / Fix / Chore / Docs Branches

- **Naming**: `feature/<name>`, `fix/<name>`, `chore/<name>`, `docs/<name>`
  - Example: `feature/metadata-parser`
  - Example: `fix/stacking-ux-performance`
  - Example: `docs/branch-policy-update`
- **Workflow**:
  1. Create from `dev`: `git checkout dev && git pull && git checkout -b feature/your-feature`
  2. Make commits with clear messages
  3. Push and open a PR **targeting `dev`** (NOT `main`)
  4. After approval and merge to `dev`, the feature branch can be deleted

### `old/main` — Legacy Archive (read-only)

- Archive of the legacy C#/.NET codebase (WPF v1.x + Avalonia v2.0), full history preserved
- Reference only; no development happens here

## Release Process

1. Feature development happens in `feature/*` / `fix/*` branches
2. PRs merge into `dev` for integration testing
3. When `dev` is release-ready, increment the version number across the project
4. Open the release PR: `dev` → `main`
5. Merge the release PR into `main`
6. Tag the release commit on `main` (`vX.Y.Z`) and push the tag to trigger GitHub Actions release publishing
7. `main` always reflects the latest official release

## Branch Protection Rules

### `main`

✅ **Enabled Rules**:
- Require pull request reviews before merging
- Dismiss stale reviews when new commits are pushed
- Require linear history
- Require conversation resolution before merging
- Block force pushes
- Block deletions

❌ **Direct pushes to `main` are BLOCKED**
❌ **Feature branches must NOT target `main`** — only `dev` → `main` release PRs

## Workflow Example

```bash
# Start a feature branch from dev
git checkout dev
git pull origin dev
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: add new feature"

# Push and create PR targeting dev
git push -u origin feature/my-feature
gh pr create --base dev --title "feat: my feature"

# After merge to dev, sync local dev
git checkout dev
git pull origin dev

# When dev is release-ready, open the release PR to main
gh pr create --base main --head dev --title "Release v0.2.0"
```

## Questions?

If you have questions about this policy, please open an issue or reach out to the maintainers.

---

**Last Updated**: 2026-09-16
**Policy Version**: 3.1
