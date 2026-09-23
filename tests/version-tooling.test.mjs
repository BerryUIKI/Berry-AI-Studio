import test from 'node:test';
import assert from 'node:assert/strict';
import {
  parseSemVer,
  compareSemVer,
  computeNextVersion,
  readCurrentVersions,
  updateManifests,
} from '../scripts/bump-version.mjs';

test('parseSemVer parses valid SemVer components', () => {
  const v1 = parseSemVer('0.3.0');
  assert.deepEqual(v1, {
    major: 0,
    minor: 3,
    patch: 0,
    prerelease: null,
    build: null,
    raw: '0.3.0',
  });

  const v2 = parseSemVer('1.2.3-beta.1+build.42');
  assert.equal(v2.major, 1);
  assert.equal(v2.minor, 2);
  assert.equal(v2.patch, 3);
  assert.equal(v2.prerelease, 'beta.1');
  assert.equal(v2.build, 'build.42');

  assert.equal(parseSemVer('invalid'), null);
  assert.equal(parseSemVer('0.3'), null);
  assert.equal(parseSemVer('v0.3.0'), null);
});

test('compareSemVer correctly orders versions', () => {
  assert.ok(compareSemVer('0.3.1', '0.3.0') > 0);
  assert.ok(compareSemVer('0.4.0', '0.3.9') > 0);
  assert.ok(compareSemVer('1.0.0', '0.9.9') > 0);
  assert.equal(compareSemVer('0.3.0', '0.3.0'), 0);
  assert.ok(compareSemVer('0.2.9', '0.3.0') < 0);
});

test('computeNextVersion calculates next versions correctly', () => {
  assert.equal(computeNextVersion('0.3.0', 'patch'), '0.3.1');
  assert.equal(computeNextVersion('0.3.0', 'minor'), '0.4.0');
  assert.equal(computeNextVersion('0.3.0', 'major'), '1.0.0');
  assert.equal(computeNextVersion('0.3.0', '0.3.5'), '0.3.5');

  // Rejects downgrade
  assert.throws(() => computeNextVersion('0.3.0', '0.2.0'), /strictly greater/);
  assert.throws(() => computeNextVersion('0.3.0', '0.3.0'), /strictly greater/);

  // Rejects invalid string
  assert.throws(() => computeNextVersion('0.3.0', 'not-a-version'), /not a valid SemVer/);
});

test('current repository manifests are synchronized', () => {
  const versions = readCurrentVersions();
  assert.ok(versions.packageJson, 'package.json should have a version');
  assert.equal(versions.packageJson, versions.cargoToml, 'Cargo.toml should match package.json');
  assert.equal(versions.packageJson, versions.tauriConf, 'tauri.conf.json should match package.json');
});

test('updateManifests simulation produces correct string replacements', () => {
  const result = updateManifests('0.3.1', true);
  assert.match(result.newPkg, /"version": "0.3.1"/);
  assert.match(result.newTauri, /"version": "0.3.1"/);
  assert.match(result.newCargo, /\[workspace\.package\][\s\S]*?version = "0.3.1"/);
});
