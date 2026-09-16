import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  locale: string;
  auto_scan: boolean;
  blur_nsfw: boolean;
  show_card_badges: boolean;
  default_view: "grid" | "masonry" | "table";
  thumbnail_max_edge: number;
  similarity_limit: number;
  auto_check_update: boolean;
  silent_install: boolean;
  has_completed_onboarding: boolean;
  auto_stack: boolean;
  stack_similarity_threshold: number;
  stack_time_window_minutes: number;
  allow_multiple_open_stacks: boolean;
  suppressed_warnings: string[];
}

export const STACK_MERGE_WARNING_ID = "stack_merge";

export interface StoragePaths {
  data_dir: string;
  config_file: string;
  database_file: string;
  thumbnails_dir: string;
  models_dir: string;
  updates_dir: string;
}

const DEFAULT_CONFIG: AppConfig = {
  locale: "auto",
  auto_scan: true,
  blur_nsfw: true,
  show_card_badges: true,
  default_view: "grid",
  thumbnail_max_edge: 384,
  similarity_limit: 50,
  auto_check_update: true,
  silent_install: false,
  has_completed_onboarding: false,
  auto_stack: false,
  stack_similarity_threshold: 0.85,
  stack_time_window_minutes: 180,
  allow_multiple_open_stacks: false,
  suppressed_warnings: [],
};

/**
 * Load application configuration from persistent config.json,
 * automatically migrating legacy localStorage settings if available.
 */
export async function loadAppConfig(): Promise<AppConfig> {
  try {
    const config = await invoke<AppConfig>("get_app_config");

    // Seamless migration from localStorage for existing users
    let modified = false;
    const legacyLocale = localStorage.getItem("berry_locale");
    if (legacyLocale && config.locale === "auto") {
      config.locale = legacyLocale;
      modified = true;
    }

    const legacyAutoScan = localStorage.getItem("berry_autoscan");
    if (legacyAutoScan !== null) {
      config.auto_scan = legacyAutoScan !== "false";
    }

    const legacyBlur = localStorage.getItem("berry_blur_nsfw");
    if (legacyBlur !== null) {
      config.blur_nsfw = legacyBlur !== "false";
    }

    const legacyBadges = localStorage.getItem("berry_card_badges");
    if (legacyBadges !== null) {
      config.show_card_badges = legacyBadges !== "false";
    }

    const legacyView = localStorage.getItem("berry_default_view");
    if (legacyView === "grid" || legacyView === "masonry" || legacyView === "table") {
      config.default_view = legacyView;
    }

    const legacyThumb = localStorage.getItem("berry_thumbnail_max_edge");
    if (legacyThumb) {
      const parsed = parseInt(legacyThumb, 10);
      if (!isNaN(parsed) && parsed > 0) {
        config.thumbnail_max_edge = parsed;
      }
    }

    const legacySim = localStorage.getItem("berry_similarity_limit");
    if (legacySim) {
      const parsed = parseInt(legacySim, 10);
      if (!isNaN(parsed) && parsed > 0) {
        config.similarity_limit = parsed;
      }
    }

    if (modified) {
      void saveAppConfig(config);
    }

    // Mirror to localStorage for any synchronous instant fallbacks
    syncConfigToLocalStorage(config);

    return config;
  } catch (err) {
    console.warn("Failed to load app config from backend, using default/localStorage:", err);
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * Persist application configuration to config.json and sync to localStorage.
 */
export async function saveAppConfig(config: AppConfig): Promise<void> {
  try {
    await invoke("save_app_config", { config });
    syncConfigToLocalStorage(config);
  } catch (err) {
    console.error("Failed to save app config:", err);
    syncConfigToLocalStorage(config);
    throw err;
  }
}

export function isWarningSuppressed(config: AppConfig, warningId: string): boolean {
  return config.suppressed_warnings.includes(warningId);
}

export async function suppressWarning(warningId: string): Promise<void> {
  const config = await loadAppConfig();
  if (isWarningSuppressed(config, warningId)) return;
  await saveAppConfig({
    ...config,
    suppressed_warnings: [...config.suppressed_warnings, warningId],
  });
}

export async function resetSuppressedWarnings(): Promise<number> {
  const config = await loadAppConfig();
  const resetCount = config.suppressed_warnings.length;
  if (resetCount > 0) {
    await saveAppConfig({ ...config, suppressed_warnings: [] });
  }
  return resetCount;
}

/**
 * Mirror configuration to localStorage.
 */
function syncConfigToLocalStorage(config: AppConfig): void {
  try {
    localStorage.setItem("berry_locale", config.locale);
    localStorage.setItem("berry_autoscan", String(config.auto_scan));
    localStorage.setItem("berry_blur_nsfw", String(config.blur_nsfw));
    localStorage.setItem("berry_card_badges", String(config.show_card_badges));
    localStorage.setItem("berry_default_view", config.default_view);
    localStorage.setItem("berry_thumbnail_max_edge", String(config.thumbnail_max_edge));
    localStorage.setItem("berry_similarity_limit", String(config.similarity_limit));
    localStorage.setItem("berry_auto_check_update", String(config.auto_check_update));
    localStorage.setItem("berry_silent_install", String(config.silent_install));
  } catch {
    // Ignore localStorage failures
  }
}

/**
 * Retrieve absolute paths for data, config, database, thumbnails, and updates.
 */
export async function getStoragePaths(): Promise<StoragePaths> {
  return await invoke<StoragePaths>("get_storage_paths");
}

/**
 * Open a storage directory or highlight a file in Explorer / Finder / file manager.
 */
export async function openStorageDir(
  target: "data" | "config" | "database" | "thumbnails" | "models" | "updates"
): Promise<void> {
  await invoke("open_storage_dir", { target });
}
