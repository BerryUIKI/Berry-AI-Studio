import test from 'node:test';
import assert from 'node:assert/strict';
import { compareSemver, findMatchingAsset } from '../src/utils/updater.ts';

test('compareSemver: compares exact versions and prefixes', () => {
  assert.equal(compareSemver('0.3.0', '0.3.0'), 0);
  assert.equal(compareSemver('v0.3.0', '0.3.0'), 0);
  assert.equal(compareSemver('0.3.0', 'v0.3.0'), 0);
  assert.equal(compareSemver('v0.3.0', 'v0.3.0'), 0);
});

test('compareSemver: compares major, minor, patch', () => {
  assert.equal(compareSemver('0.3.1', '0.3.0'), 1);
  assert.equal(compareSemver('0.3.0', '0.3.1'), -1);
  assert.equal(compareSemver('0.4.0', '0.3.9'), 1);
  assert.equal(compareSemver('1.0.0', '0.9.9'), 1);
  assert.equal(compareSemver('0.2.9', '0.3.0'), -1);
});

test('compareSemver: compares prereleases properly', () => {
  // Release without prerelease is greater than with prerelease
  assert.equal(compareSemver('0.3.0', '0.3.0-dev'), 1);
  assert.equal(compareSemver('0.3.0-dev', '0.3.0'), -1);
  assert.equal(compareSemver('0.3.0-beta.2', '0.3.0-beta.1'), 1);
  assert.equal(compareSemver('0.3.0-beta.1', '0.3.0-beta.2'), -1);
  assert.equal(compareSemver('0.3.0-rc.1', '0.3.0-rc.1'), 0);
});

test('findMatchingAsset: Windows matches Omera and legacy installers and never returns other OS assets', () => {
  const winUA = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)';
  const winPlatform = 'Win32';

  const mixedAssets = [
    { name: 'Omera_macOS_aarch64.dmg', browser_download_url: 'https://example.com/mac', size: 100 },
    { name: 'Omera_Linux_x64.AppImage', browser_download_url: 'https://example.com/linux', size: 100 },
    { name: 'Omera_Windows_x64.exe', browser_download_url: 'https://example.com/win-exe', size: 100 },
    { name: 'Omera_Windows_x64.zip', browser_download_url: 'https://example.com/win-zip', size: 100 },
  ];

  const matched = findMatchingAsset(mixedAssets, winUA, winPlatform);
  assert.ok(matched);
  assert.equal(matched.name, 'Omera_Windows_x64.exe');

  // Legacy fallback when only Berry releases exist
  const legacyAssets = [
    { name: 'Berry-AI-Studio_macOS_aarch64.dmg', browser_download_url: 'https://example.com/mac', size: 100 },
    { name: 'Berry-AI-Studio_Windows_x64.exe', browser_download_url: 'https://example.com/legacy-win', size: 100 },
  ];
  const legacyMatched = findMatchingAsset(legacyAssets, winUA, winPlatform);
  assert.ok(legacyMatched);
  assert.equal(legacyMatched.name, 'Berry-AI-Studio_Windows_x64.exe');

  // Strict matching: returns null when no Windows asset is present (does not pick first asset)
  const nonWinAssets = [
    { name: 'Omera_macOS_aarch64.dmg', browser_download_url: 'https://example.com/mac', size: 100 },
    { name: 'Omera_Linux_x64.AppImage', browser_download_url: 'https://example.com/linux', size: 100 },
  ];
  assert.equal(findMatchingAsset(nonWinAssets, winUA, winPlatform), null);
});

test('findMatchingAsset: macOS matches architecture and Omera naming with fallback', () => {
  const macArmUA = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; arm64)';
  const macIntelUA = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; x86_64)';
  const macPlatform = 'MacIntel';

  const macAssets = [
    { name: 'Omera_Windows_x64.exe', browser_download_url: 'https://example.com/win', size: 100 },
    { name: 'Omera_macOS_x64.dmg', browser_download_url: 'https://example.com/mac-x64', size: 100 },
    { name: 'Omera_macOS_aarch64.dmg', browser_download_url: 'https://example.com/mac-arm', size: 100 },
  ];

  const armMatched = findMatchingAsset(macAssets, macArmUA, macPlatform);
  assert.ok(armMatched);
  assert.equal(armMatched.name, 'Omera_macOS_aarch64.dmg');

  const x64Matched = findMatchingAsset(macAssets, macIntelUA, macPlatform);
  assert.ok(x64Matched);
  assert.equal(x64Matched.name, 'Omera_macOS_x64.dmg');

  // Legacy fallback
  const legacyMacAssets = [
    { name: 'Berry-AI-Studio_macOS_aarch64.dmg', browser_download_url: 'https://example.com/legacy-mac', size: 100 },
  ];
  const legacyMatched = findMatchingAsset(legacyMacAssets, macArmUA, macPlatform);
  assert.ok(legacyMatched);
  assert.equal(legacyMatched.name, 'Berry-AI-Studio_macOS_aarch64.dmg');

  // Strict matching: returns null when only Windows or Linux assets exist
  const winOnly = [
    { name: 'Omera_Windows_x64.exe', browser_download_url: 'https://example.com/win', size: 100 },
  ];
  assert.equal(findMatchingAsset(winOnly, macArmUA, macPlatform), null);
});

test('findMatchingAsset: Linux matches AppImage and deb with strict rejection of foreign assets', () => {
  const linuxUA = 'Mozilla/5.0 (X11; Linux x86_64)';
  const linuxPlatform = 'Linux x86_64';

  const linuxAssets = [
    { name: 'Omera_Windows_x64.exe', browser_download_url: 'https://example.com/win', size: 100 },
    { name: 'Omera_macOS_aarch64.dmg', browser_download_url: 'https://example.com/mac', size: 100 },
    { name: 'Omera_Linux_x64.deb', browser_download_url: 'https://example.com/deb', size: 100 },
    { name: 'Omera_Linux_x64.AppImage', browser_download_url: 'https://example.com/appimage', size: 100 },
  ];

  const matched = findMatchingAsset(linuxAssets, linuxUA, linuxPlatform);
  assert.ok(matched);
  assert.equal(matched.name, 'Omera_Linux_x64.AppImage');

  // When only Windows exists
  assert.equal(findMatchingAsset([linuxAssets[0]], linuxUA, linuxPlatform), null);
});
