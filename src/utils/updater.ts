import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface UpdateDownloadProgress {
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  speed_bytes_per_sec: number;
  done: boolean;
  target_file: string | null;
}

export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

export interface GitHubRelease {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  html_url: string;
  assets: ReleaseAsset[];
}

export type UpdateStatus =
  | "idle"
  | "checking"
  | "up_to_date"
  | "update_available"
  | "ahead_of_release"
  | "error";

export interface UpdateCheckResult {
  status: UpdateStatus;
  currentVersion: string;
  latestVersion: string | null;
  release: GitHubRelease | null;
  errorMessage: string | null;
  matchedAsset: ReleaseAsset | null;
}

/**
 * Compare two semver version strings (e.g., '0.1.0' vs '0.2.0', 'v0.1.0-dev' vs 'v0.1.0').
 * Returns:
 *   1 if a > b (a is newer)
 *  -1 if a < b (b is newer)
 *   0 if a == b (equal)
 */
export function compareSemver(aStr: string, bStr: string): number {
  const cleanA = aStr.replace(/^v/, "").trim();
  const cleanB = bStr.replace(/^v/, "").trim();

  // Split into version and prerelease tag (e.g. "0.1.0-dev" -> ["0.1.0", "dev"])
  const [coreA, preA] = cleanA.split("-");
  const [coreB, preB] = cleanB.split("-");

  const partsA = coreA.split(".").map((n) => parseInt(n, 10) || 0);
  const partsB = coreB.split(".").map((n) => parseInt(n, 10) || 0);

  const len = Math.max(partsA.length, partsB.length);
  for (let i = 0; i < len; i++) {
    const numA = partsA[i] ?? 0;
    const numB = partsB[i] ?? 0;
    if (numA > numB) return 1;
    if (numA < numB) return -1;
  }

  // If numeric parts are equal, check prerelease:
  // A version with a prerelease is LESS than one without (e.g. 0.1.0-dev < 0.1.0)
  if (preA && !preB) return -1;
  if (!preA && preB) return 1;
  if (preA && preB) {
    const cmp = preA.localeCompare(preB, undefined, { numeric: true });
    if (cmp > 0) return 1;
    if (cmp < 0) return -1;
    return 0;
  }

  return 0;
}

/**
 * Find the most suitable asset for the current OS and architecture from release assets.
 * Strict matching: returns null if no compatible asset is found, rather than selecting an incompatible platform asset.
 */
export function findMatchingAsset(
  assets: ReleaseAsset[],
  customUserAgent?: string,
  customPlatform?: string
): ReleaseAsset | null {
  if (!assets || assets.length === 0) return null;

  const ua = customUserAgent ?? (typeof navigator !== "undefined" ? navigator.userAgent : "");
  const platform = customPlatform ?? (typeof navigator !== "undefined" ? navigator.platform : "");

  const isWindows = ua.includes("Windows") || platform.includes("Win");
  const isMac = ua.includes("Mac") || platform.includes("Mac");
  const isLinux = !isWindows && !isMac && (ua.includes("Linux") || platform.includes("Linux"));

  if (isWindows) {
    // Prefer Omera installer (.exe setup/installer or .msi), then legacy Berry, then portable zip
    return (
      assets.find((a) => /^omera.*windows.*x64.*\.exe$/i.test(a.name)) ||
      assets.find((a) => /^omera.*x64-setup\.exe$/i.test(a.name)) ||
      assets.find((a) => /^omera.*\.exe$/i.test(a.name)) ||
      assets.find((a) => /^omera.*\.msi$/i.test(a.name)) ||
      assets.find((a) => /^berry.*windows.*x64.*\.exe$/i.test(a.name)) ||
      assets.find((a) => /^berry.*x64-setup\.exe$/i.test(a.name)) ||
      assets.find((a) => /^berry.*\.exe$/i.test(a.name)) ||
      assets.find((a) => a.name.endsWith(".exe") || a.name.endsWith(".msi")) ||
      assets.find((a) => a.name.toLowerCase().includes("windows") && a.name.endsWith(".zip")) ||
      null
    );
  }

  if (isMac) {
    const isArm =
      /arm64|aarch64/i.test(ua) ||
      (typeof navigator !== "undefined" && (navigator as any).userAgentData?.architecture === "arm");
    if (isArm) {
      const armAsset =
        assets.find((a) => /^omera.*macos.*aarch64.*\.dmg$/i.test(a.name)) ||
        assets.find(
          (a) =>
            /aarch64|arm64/i.test(a.name) &&
            (a.name.endsWith(".dmg") || a.name.endsWith(".app.tar.gz"))
        ) ||
        assets.find((a) => /^berry.*macos.*aarch64.*\.dmg$/i.test(a.name));
      if (armAsset) return armAsset;
    } else {
      const x64Asset =
        assets.find((a) => /^omera.*macos.*x64.*\.dmg$/i.test(a.name)) ||
        assets.find(
          (a) =>
            /x64|x86_64/i.test(a.name) &&
            (a.name.endsWith(".dmg") || a.name.endsWith(".app.tar.gz"))
        ) ||
        assets.find((a) => /^berry.*macos.*x64.*\.dmg$/i.test(a.name));
      if (x64Asset) return x64Asset;
    }

    return (
      assets.find((a) => /^omera.*\.dmg$/i.test(a.name)) ||
      assets.find((a) => a.name.endsWith(".dmg") || a.name.endsWith(".app.tar.gz")) ||
      assets.find(
        (a) => a.name.toLowerCase().includes("macos") || a.name.toLowerCase().includes("darwin")
      ) ||
      null
    );
  }

  if (isLinux) {
    return (
      assets.find((a) => /^omera.*linux.*\.appimage$/i.test(a.name)) ||
      assets.find((a) => /^omera.*\.appimage$/i.test(a.name)) ||
      assets.find((a) => /^omera.*linux.*\.deb$/i.test(a.name)) ||
      assets.find((a) => /^omera.*\.deb$/i.test(a.name)) ||
      assets.find((a) => /^berry.*\.appimage$/i.test(a.name)) ||
      assets.find((a) => /^berry.*\.deb$/i.test(a.name)) ||
      assets.find((a) => a.name.endsWith(".AppImage")) ||
      assets.find((a) => a.name.endsWith(".deb")) ||
      assets.find((a) => a.name.toLowerCase().includes("linux")) ||
      null
    );
  }

  return null;
}

/**
 * Check for updates against GitHub Releases API.
 */
export async function checkForUpdates(currentAppVersion: string): Promise<UpdateCheckResult> {
  const currentClean = currentAppVersion.replace(/^v/, "").trim();
  const repo = "BerryUIKI/Omera";
  const url = `https://api.github.com/repos/${repo}/releases/latest`;

  try {
    const resp = await fetch(url, {
      headers: {
        Accept: "application/vnd.github.v3+json",
      },
    });

    if (!resp.ok) {
      if (resp.status === 404) {
        // No release published yet on this repository
        return {
          status: "ahead_of_release",
          currentVersion: currentClean,
          latestVersion: null,
          release: null,
          errorMessage: null,
          matchedAsset: null,
        };
      }
      throw new Error(`GitHub API returned status ${resp.status} (${resp.statusText})`);
    }

    const data: GitHubRelease = await resp.json();
    const latestClean = (data.tag_name || "").replace(/^v/, "").trim();
    const matchedAsset = findMatchingAsset(data.assets || []);

    const cmp = compareSemver(latestClean, currentClean);

    if (cmp > 0) {
      // latest > current
      return {
        status: "update_available",
        currentVersion: currentClean,
        latestVersion: latestClean,
        release: data,
        errorMessage: null,
        matchedAsset,
      };
    } else if (cmp < 0) {
      // latest < current -> dev build or ahead of official release
      return {
        status: "ahead_of_release",
        currentVersion: currentClean,
        latestVersion: latestClean,
        release: data,
        errorMessage: null,
        matchedAsset,
      };
    } else {
      // latest == current
      return {
        status: "up_to_date",
        currentVersion: currentClean,
        latestVersion: latestClean,
        release: data,
        errorMessage: null,
        matchedAsset,
      };
    }
  } catch (err: any) {
    return {
      status: "error",
      currentVersion: currentClean,
      latestVersion: null,
      release: null,
      errorMessage: err?.message || String(err),
      matchedAsset: null,
    };
  }
}

/**
 * Open external URL in browser safely.
 */
export async function openUrl(url: string): Promise<void> {
  try {
    await invoke("open_external_url", { url });
  } catch {
    window.open(url, "_blank");
  }
}

/**
 * Download update asset through backend ureq with progress streaming.
 */
export async function downloadUpdateAsset(
  url: string,
  filename: string,
  onProgress?: (progress: UpdateDownloadProgress) => void
): Promise<string> {
  let unlisten: UnlistenFn | null = null;
  if (onProgress) {
    unlisten = await listen<UpdateDownloadProgress>("update-download-progress", (event) => {
      onProgress(event.payload);
    });
  }

  try {
    const targetFile = await invoke<string>("download_update", { url, filename });
    return targetFile;
  } finally {
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * Launch the installer and exit current application for in-place upgrade.
 */
export async function installUpdate(installerPath: string, silent = false): Promise<void> {
  await invoke("install_update", { installerPath, silent });
}

