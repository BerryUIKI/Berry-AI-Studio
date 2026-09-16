# 🚀 Releasing & Packaging Guide

This guide describes the release process, versioning conventions, and build artifact management for **Berry AI Studio**.

---

## 🏷️ Versioning Strategy

Berry AI Studio adheres strictly to [Semantic Versioning 2.0.0](https://semver.org/):

$$\text{v}\langle\text{MAJOR}\rangle.\langle\text{MINOR}\rangle.\langle\text{PATCH}\rangle$$

- **MAJOR**: Breaking changes to the core architecture or SQLite database schema that require manual intervention.
- **MINOR**: New user features (e.g., new metadata format parsers, new studio layout modules, AI tagging).
- **PATCH**: Bug fixes, performance enhancements, and localization updates.

When preparing a release, update the version number consistently in:
1. `Cargo.toml` (`[workspace.package] version = "X.Y.Z"`)
2. `src-tauri/tauri.conf.json` (`"version": "X.Y.Z"`)
3. `package.json` (`"version": "X.Y.Z"`)
4. `CHANGELOG.md` (Add release date and feature summary under `## [X.Y.Z]`)

---

## 📦 Release Asset Naming Convention

All pre-compiled release assets uploaded to [GitHub Releases](https://github.com/BerryUIKI/Berry-AI-Studio/releases) follow the unified naming convention:

$$\text{<AppName>}\_\text{<OS>}\_\text{<Architecture>}.\text{<extension>}$$

### Supported Platforms & Target Binaries

| Platform (OS) | Architecture | Target Triple | Asset File Name |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64 (64-bit) | `x86_64-pc-windows-msvc` | `Berry-AI-Studio_Windows_x64.exe` *(NSIS Setup)* |
| **Windows** | x86_64 (64-bit) | `x86_64-pc-windows-msvc` | `Berry-AI-Studio_Windows_x64.zip` *(Portable)* |
| **macOS** | Apple Silicon (ARM64) | `aarch64-apple-darwin` | `Berry-AI-Studio_macOS_aarch64.dmg` |
| **macOS** | Intel (x86_64) | `x86_64-apple-darwin` | `Berry-AI-Studio_macOS_x64.dmg` |
| **Linux** | x86_64 (64-bit) | `x86_64-unknown-linux-gnu` | `Berry-AI-Studio_Linux_x64.AppImage` |
| **Linux** | x86_64 (64-bit) | `x86_64-unknown-linux-gnu` | `Berry-AI-Studio_Linux_x64.deb` |

---

## 🤖 Automated CI/CD Release Workflow

Releases are automatically built and published across all platforms via GitHub Actions (`.github/workflows/release.yml`) whenever a version tag is pushed:

```bash
# 1. Ensure you are on the main branch with a clean working tree
git checkout main
git pull origin main

# 2. Tag the release commit
git tag v0.1.1

# 3. Push the tag to GitHub
git push origin v0.1.1
```

The GitHub Actions runner will build the Tauri application on Windows, macOS, and Linux runners, bundle the installers, and publish the draft release on GitHub Releases.
