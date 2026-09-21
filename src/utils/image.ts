import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Strip the Windows `\\?\` verbatim prefix, if present.
 */
export function normalizePath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

/**
 * Convert a local file system path to a URL that the Tauri webview can load
 * via the `asset:` custom protocol.
 */
export function assetUrl(path: string): string {
  if (!path) return "";
  const clean = normalizePath(path);
  return convertFileSrc(clean);
}

/**
 * Extract filename from a path (handling both POSIX and Windows separators).
 */
export function getFileName(path: string): string {
  const clean = normalizePath(path);
  return clean.split(/[\\/]/).pop() ?? clean;
}

/**
 * Format byte count to human-readable size.
 */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Format unix timestamp (in seconds) to local date-time string.
 */
export function formatDateTime(unixSeconds: number): string {
  if (!unixSeconds) return "—";
  return new Date(unixSeconds * 1000).toLocaleString();
}

/**
 * Friendly platform name mapping for image generators:
 * maps 'a1111' / 'A1111' -> 'WebUI', 'comfyui' -> 'ComfyUI', etc.
 */
export function formatPlatformName(format: string | null | undefined): string {
  if (!format) return "";
  const lower = format.toLowerCase();
  if (lower === "a1111" || lower === "automatic1111") return "WebUI";
  if (lower === "comfyui") return "ComfyUI";
  if (lower === "novelai") return "NovelAI";
  if (lower === "fooocus" || lower === "fooocusmre") return "Fooocus";
  if (lower === "invokeai") return "InvokeAI";
  if (lower === "easydiffusion") return "EasyDiffusion";
  if (lower === "stableswarm") return "StableSwarm";
  if (lower === "sd" || lower === "stablediffusion") return "SD";
  return format;
}

/**
 * Format seconds into mm:ss or hh:mm:ss.
 */
export function formatDuration(totalSeconds?: number | null): string {
  if (totalSeconds == null || isNaN(totalSeconds) || totalSeconds < 0) return "00:00";
  const s = Math.floor(totalSeconds);
  const hrs = Math.floor(s / 3600);
  const mins = Math.floor((s % 3600) / 60);
  const secs = s % 60;
  if (hrs > 0) {
    return `${hrs.toString().padStart(2, "0")}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

/**
 * Check whether a container string represents video media.
 */
export function isVideoContainer(container: string | null | undefined): boolean {
  if (!container) return false;
  const c = container.toLowerCase();
  return c === "mp4" || c === "webm";
}
