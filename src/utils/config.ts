import { invoke } from "@tauri-apps/api/core";
import type { AppTheme } from "./theme";
import type { CloudBackupConfig } from "../types";

export interface AppConfig {
  config_revision: number;
  legacy_migration_complete: boolean;
  locale: string;
  auto_scan: boolean;
  startup_scan_interval_minutes: number;
  theme: AppTheme;
  blur_nsfw: boolean;
  show_card_badges: boolean;
  default_view: "grid" | "masonry" | "table";
  thumbnail_max_edge: number;
  thumbnail_cache_budget_mb: number;
  similarity_limit: number;
  auto_check_update: boolean;
  silent_install: boolean;
  has_completed_onboarding: boolean;
  auto_stack: boolean;
  stack_similarity_threshold: number;
  stack_time_window_minutes: number;
  allow_multiple_open_stacks: boolean;
  suppressed_warnings: string[];
  comfyui_url: string;
  webui_url: string;
  storage_backend: "sqlite" | "mysql" | "postgres";
  remote_connection_url: string;
  client_identifier: string;
  root_mappings: Record<string, string>;
  cloud_backup: CloudBackupConfig;
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
  config_revision: 0,
  legacy_migration_complete: false,
  locale: "auto",
  auto_scan: false,
  startup_scan_interval_minutes: 360,
  theme: "system",
  blur_nsfw: true,
  show_card_badges: true,
  default_view: "grid",
  thumbnail_max_edge: 384,
  thumbnail_cache_budget_mb: 2048,
  similarity_limit: 50,
  auto_check_update: true,
  silent_install: false,
  has_completed_onboarding: false,
  auto_stack: false,
  stack_similarity_threshold: 0.85,
  stack_time_window_minutes: 180,
  allow_multiple_open_stacks: false,
  suppressed_warnings: [],
  comfyui_url: "http://127.0.0.1:8188",
  webui_url: "http://127.0.0.1:7860",
  storage_backend: "sqlite",
  remote_connection_url: "",
  client_identifier: "local_client",
  root_mappings: {},
  cloud_backup: {
    provider: "local_path",
    local_path: null,
    webdav_endpoint: null,
    webdav_username: null,
    webdav_password: null,
    s3_endpoint: null,
    s3_bucket: null,
    s3_region: "auto",
    s3_access_key: null,
    s3_secret_key: null,
    s3_prefix: "backups/",
    auto_backup_enabled: false,
    auto_backup_interval_days: 7,
  },
};

/**
 * Load application configuration from persistent config.json,
 * automatically migrating legacy localStorage settings if available.
 */
export async function loadAppConfig(): Promise<AppConfig> {
  try {
    const config = await invoke<AppConfig>("get_app_config");

    // Seamless migration from localStorage for existing users
    if (!config.legacy_migration_complete) {
      const legacyLocale = getStorageItem("locale");
      if (legacyLocale && config.locale === "auto") {
        config.locale = legacyLocale;
      }

      const legacyAutoScan = getStorageItem("autoscan");
      if (legacyAutoScan !== null) {
        config.auto_scan = legacyAutoScan !== "false";
      }

      const legacyTheme = getStorageItem("theme");
      if (
        legacyTheme === "system" || legacyTheme === "midnight" || legacyTheme === "graphite" ||
        legacyTheme === "light" || legacyTheme === "violet"
      ) {
        config.theme = legacyTheme;
      }

      const legacyBlur = getStorageItem("blur_nsfw");
      if (legacyBlur !== null) {
        config.blur_nsfw = legacyBlur !== "false";
      }

      const legacyBadges = getStorageItem("card_badges");
      if (legacyBadges !== null) {
        config.show_card_badges = legacyBadges !== "false";
      }

      const legacyView = getStorageItem("default_view");
      if (legacyView === "grid" || legacyView === "masonry" || legacyView === "table") {
        config.default_view = legacyView;
      }

      const legacyThumb = getStorageItem("thumbnail_max_edge");
      if (legacyThumb) {
        const parsed = parseInt(legacyThumb, 10);
        if (!isNaN(parsed) && parsed > 0) {
          config.thumbnail_max_edge = parsed;
        }
      }

      const legacyThumbnailBudget = getStorageItem("thumbnail_cache_budget_mb");
      if (legacyThumbnailBudget) {
        const parsed = parseInt(legacyThumbnailBudget, 10);
        if (!isNaN(parsed) && parsed >= 256) {
          config.thumbnail_cache_budget_mb = parsed;
        }
      }

      const legacySim = getStorageItem("similarity_limit");
      if (legacySim) {
        const parsed = parseInt(legacySim, 10);
        if (!isNaN(parsed) && parsed > 0) {
          config.similarity_limit = parsed;
        }
      }

      const legacyComfy = getStorageItem("comfyui_url");
      if (legacyComfy) {
        config.comfyui_url = legacyComfy;
      } else if (!config.comfyui_url) {
        config.comfyui_url = DEFAULT_CONFIG.comfyui_url;
      }

      const legacyWebui = getStorageItem("webui_url");
      if (legacyWebui) {
        config.webui_url = legacyWebui;
      } else if (!config.webui_url) {
        config.webui_url = DEFAULT_CONFIG.webui_url;
      }

      config.legacy_migration_complete = true;
      await saveAppConfig(config);
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
    const saved = await invoke<AppConfig>("save_app_config", { config });
    Object.assign(config, saved);
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
 * Read setting from localStorage with fallback to legacy `berry_*` key.
 * If found under legacy key, it is migrated forward to the `omera_*` key.
 */
export function getStorageItem(keySuffix: string): string | null {
  try {
    const omeraKey = `omera_${keySuffix}`;
    const value = localStorage.getItem(omeraKey);
    if (value !== null) {
      return value;
    }
    const legacyKey = `berry_${keySuffix}`;
    const legacyValue = localStorage.getItem(legacyKey);
    if (legacyValue !== null) {
      localStorage.setItem(omeraKey, legacyValue);
      return legacyValue;
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Write setting to localStorage with new `omera_*` prefix.
 */
export function setStorageItem(keySuffix: string, value: string): void {
  try {
    localStorage.setItem(`omera_${keySuffix}`, value);
  } catch {
    // Ignore localStorage failures
  }
}

/**
 * Mirror configuration to localStorage using target `omera_*` keys.
 */
function syncConfigToLocalStorage(config: AppConfig): void {
  try {
    localStorage.setItem("omera_locale", config.locale);
    localStorage.setItem("omera_autoscan", String(config.auto_scan));
    localStorage.setItem("omera_theme", config.theme);
    localStorage.setItem("omera_blur_nsfw", String(config.blur_nsfw));
    localStorage.setItem("omera_card_badges", String(config.show_card_badges));
    localStorage.setItem("omera_default_view", config.default_view);
    localStorage.setItem("omera_thumbnail_max_edge", String(config.thumbnail_max_edge));
    localStorage.setItem(
      "omera_thumbnail_cache_budget_mb",
      String(config.thumbnail_cache_budget_mb),
    );
    localStorage.setItem("omera_similarity_limit", String(config.similarity_limit));
    localStorage.setItem("omera_auto_check_update", String(config.auto_check_update));
    localStorage.setItem("omera_silent_install", String(config.silent_install));
    localStorage.setItem("omera_comfyui_url", config.comfyui_url);
    localStorage.setItem("omera_webui_url", config.webui_url);
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
