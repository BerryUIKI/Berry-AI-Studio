#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const rootDir = fileURLToPath(new URL('..', import.meta.url));

const files = {
  packageJson: resolve(rootDir, 'package.json'),
  cargoToml: resolve(rootDir, 'Cargo.toml'),
  tauriConf: resolve(rootDir, 'src-tauri/tauri.conf.json'),
};

const SEMVER_REGEX = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;

export function parseSemVer(version) {
  const match = version.match(SEMVER_REGEX);
  if (!match) return null;
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    prerelease: match[4] || null,
    build: match[5] || null,
    raw: version,
  };
}

export function compareSemVer(a, b) {
  const pa = typeof a === 'string' ? parseSemVer(a) : a;
  const pb = typeof b === 'string' ? parseSemVer(b) : b;
  if (!pa || !pb) throw new Error('Invalid SemVer comparison');

  if (pa.major !== pb.major) return pa.major - pb.major;
  if (pa.minor !== pb.minor) return pa.minor - pb.minor;
  if (pa.patch !== pb.patch) return pa.patch - pb.patch;

  // No prerelease is greater than has prerelease
  if (!pa.prerelease && pb.prerelease) return 1;
  if (pa.prerelease && !pb.prerelease) return -1;
  if (pa.prerelease && pb.prerelease) {
    return pa.prerelease.localeCompare(pb.prerelease);
  }
  return 0;
}

export function readCurrentVersions() {
  const pkg = JSON.parse(readFileSync(files.packageJson, 'utf8'));
  const tauri = JSON.parse(readFileSync(files.tauriConf, 'utf8'));
  const cargoContent = readFileSync(files.cargoToml, 'utf8');

  const cargoMatch = cargoContent.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/);
  const cargoVersion = cargoMatch ? cargoMatch[1] : null;

  return {
    packageJson: pkg.version,
    cargoToml: cargoVersion,
    tauriConf: tauri.version,
  };
}

export function computeNextVersion(current, target) {
  const parsed = parseSemVer(current);
  if (!parsed) throw new Error(`Current version '${current}' is not valid SemVer`);

  if (target === 'patch') {
    return `${parsed.major}.${parsed.minor}.${parsed.patch + 1}`;
  }
  if (target === 'minor') {
    return `${parsed.major}.${parsed.minor + 1}.0`;
  }
  if (target === 'major') {
    return `${parsed.major + 1}.0.0`;
  }

  const explicit = parseSemVer(target);
  if (!explicit) {
    throw new Error(`Target version '${target}' is not a valid SemVer or bump type ('patch', 'minor', 'major')`);
  }

  if (compareSemVer(explicit, parsed) <= 0) {
    throw new Error(`Target version '${target}' must be strictly greater than current version '${current}'`);
  }

  return target;
}

export function checkCleanGit() {
  try {
    const status = execFileSync('git', ['status', '--porcelain'], { cwd: rootDir, encoding: 'utf8' });
    return status.trim().length === 0;
  } catch {
    return true; // Not a git repo or git not found
  }
}

export function updateManifests(nextVersion, dryRun = false) {
  const pkgContent = readFileSync(files.packageJson, 'utf8');
  const tauriContent = readFileSync(files.tauriConf, 'utf8');
  const cargoContent = readFileSync(files.cargoToml, 'utf8');

  const newPkg = pkgContent.replace(/"version":\s*"[^"]+"/, `"version": "${nextVersion}"`);
  const newTauri = tauriContent.replace(/"version":\s*"[^"]+"/, `"version": "${nextVersion}"`);
  const newCargo = cargoContent.replace(
    /(\[workspace\.package\][\s\S]*?version\s*=\s*)"[^"]+"/,
    `$1"${nextVersion}"`
  );

  if (!dryRun) {
    writeFileSync(files.packageJson, newPkg, 'utf8');
    writeFileSync(files.tauriConf, newTauri, 'utf8');
    writeFileSync(files.cargoToml, newCargo, 'utf8');
  }

  return { newPkg, newTauri, newCargo };
}

// CLI execution
if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  const args = process.argv.slice(2);

  if (args.includes('--help') || args.includes('-h')) {
    console.log(`Usage: node scripts/bump-version.mjs <patch|minor|major|version> [options]

Commands & Options:
  --check          Verify that all manifests currently agree on the version
  --dry-run        Preview the version bump without modifying files
  --allow-dirty    Allow running even if git working tree has uncommitted changes
  --help, -h       Show this help message
`);
    process.exit(0);
  }

  const versions = readCurrentVersions();
  const allMatch = versions.packageJson === versions.cargoToml && versions.packageJson === versions.tauriConf;

  if (args.includes('--check')) {
    console.log(`Current versions:
  package.json:            ${versions.packageJson}
  Cargo.toml (workspace):  ${versions.cargoToml}
  src-tauri/tauri.conf.json: ${versions.tauriConf}`);

    if (allMatch) {
      console.log(`✓ All manifests are synchronized at v${versions.packageJson}`);
      process.exit(0);
    } else {
      console.error(`✗ Version mismatch detected across manifests!`);
      process.exit(1);
    }
  }

  if (!allMatch) {
    console.error(`Error: Cannot bump versions while manifests are out of sync:
  package.json:            ${versions.packageJson}
  Cargo.toml (workspace):  ${versions.cargoToml}
  src-tauri/tauri.conf.json: ${versions.tauriConf}`);
    process.exit(1);
  }

  const dryRun = args.includes('--dry-run');
  const allowDirty = args.includes('--allow-dirty');

  if (!allowDirty && !dryRun && !checkCleanGit()) {
    console.error('Error: Git working tree is dirty. Stash or commit your changes before bumping version, or use --allow-dirty.');
    process.exit(1);
  }

  const target = args.find(a => !a.startsWith('--'));
  if (!target) {
    console.error('Error: Missing target version or bump type (expected patch, minor, major, or X.Y.Z).');
    process.exit(1);
  }

  const current = versions.packageJson;
  const next = computeNextVersion(current, target);

  console.log(`Bumping version: ${current} -> ${next}${dryRun ? ' (DRY RUN)' : ''}`);
  updateManifests(next, dryRun);

  if (dryRun) {
    console.log('✓ Dry run completed. No files were changed.');
  } else {
    console.log(`✓ Successfully updated package.json, Cargo.toml, and tauri.conf.json to ${next}`);
    console.log('Run `cargo check --workspace` and `pnpm install --lockfile-only` to refresh lockfiles.');
  }
}
