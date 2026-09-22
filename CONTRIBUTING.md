# Contributing to Omera

Thank you for your interest in contributing to **Omera**! We welcome bug reports, feature suggestions, new metadata format extractors, translation contributions, and code improvements.

---

## Branching Model

- **`main`**: Production release branch. Contains tagged releases (`v0.1.1`, etc.).
- **`dev`**: Active development branch. All feature branches and bug fix PRs should target `dev`.
- **`feature/*`**: Feature branches branched from `dev`.
- **`fix/*`**: Bug fix branches branched from `dev`.

---

## Local Development Setup

### Prerequisites

- **Node.js**: v18.0 or higher
- **pnpm**: `npm install -g pnpm`
- **Rust**: 1.75+ (`rustup default stable`)
- **Tauri Prerequisites**: Follow the [Tauri 2 Prerequisites Guide](https://v2.tauri.app/start/prerequisites/) for your operating system.

### Running Locally

```bash
# 1. Clone your fork
git clone https://github.com/<your-username>/Omera.git
cd Omera

# 2. Checkout dev branch
git checkout dev

# 3. Install frontend dependencies
pnpm install

# 4. Start the Tauri development app (hot-reloading enabled)
pnpm run tauri dev
```

---

## Development Guidelines

- Read [`AGENTS.md`](AGENTS.md) before making automated or agent-assisted changes.
- Keep Tauri commands thin and place reusable behavior in the appropriate Rust crate or TypeScript utility.
- Preserve gallery virtualization: scrolling work must scale with the visible window, not the complete library.
- Add Serde-compatible defaults when extending persisted configuration.
- Add SQLite changes as new migrations; never modify an applied migration.
- Keep engineering documentation in English and add user-facing strings to every locale.

The current performance model and optimization backlog are documented in [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md).

## Testing and Code Quality

Before opening a pull request, ensure all checks pass:

```bash
# Frontend TypeScript check and production build
pnpm run build

# Rust workspace compilation check
cargo check --workspace

# Rust unit tests
cargo test --workspace

# Frontend stack behavior tests
pnpm run test:stack

# Rust formatting & linter
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

---

## Contributing Translations

We welcome new languages and translation improvements.

- All locales live in `src/i18n/locales/*.ts`.
- To add a new locale:
  1. Create `src/i18n/locales/<locale-code>.ts` based on `src/i18n/locales/en.ts`.
  2. Register the new locale in `src/i18n/index.ts` under `SUPPORTED_LOCALES`.
  3. Ensure all keys match the `LocaleDictionary` interface.

---

## Pull Request Checklist

- The pull request targets `dev`.
- Relevant tests pass locally.
- User-visible behavior is documented and localized.
- Performance-sensitive gallery or scanner changes include before/after measurements.
- No unrelated generated files or local configuration are included.

## License

By contributing to Omera, you agree that your contributions will be licensed under the **AGPL-3.0 License**.
