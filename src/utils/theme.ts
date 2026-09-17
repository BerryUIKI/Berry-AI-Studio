export type AppTheme = "system" | "midnight" | "graphite" | "light" | "violet";

let systemThemeQuery: MediaQueryList | null = null;
let systemThemeListener: (() => void) | null = null;

export function normalizeTheme(value: string | null | undefined): AppTheme {
  return value === "midnight" || value === "graphite" || value === "light" || value === "violet"
    ? value
    : "system";
}

export function applyTheme(theme: AppTheme): void {
  if (systemThemeQuery && systemThemeListener) {
    systemThemeQuery.removeEventListener("change", systemThemeListener);
    systemThemeQuery = null;
    systemThemeListener = null;
  }
  const root = document.documentElement;
  systemThemeQuery = theme === "system" ? window.matchMedia("(prefers-color-scheme: light)") : null;
  const resolved = systemThemeQuery?.matches ? "light" : theme === "system" ? "midnight" : theme;
  root.dataset.themeSetting = theme;
  root.dataset.theme = resolved;
  root.style.colorScheme = resolved === "light" ? "light" : "dark";

  if (systemThemeQuery) {
    systemThemeListener = () => applyTheme("system");
    systemThemeQuery.addEventListener("change", systemThemeListener);
  }
}
