import { ref, computed } from "vue";
import { en } from "./locales/en";
import { zhCN } from "./locales/zh-CN";
import { zhTW } from "./locales/zh-TW";
import { ja } from "./locales/ja";
import { de } from "./locales/de";
import { fr } from "./locales/fr";
import { es } from "./locales/es";

export type LocaleKey = "en" | "zh-CN" | "zh-TW" | "ja" | "de" | "fr" | "es";
export type LocaleSetting = "auto" | LocaleKey;

export interface LocaleOption {
  key: LocaleSetting;
  label: string;
}

export const SUPPORTED_LOCALES: LocaleOption[] = [
  { key: "auto", label: "Auto (跟随系统 / System)" },
  { key: "zh-CN", label: "简体中文" },
  { key: "zh-TW", label: "繁體中文" },
  { key: "en", label: "English" },
  { key: "ja", label: "日本語" },
  { key: "de", label: "Deutsch" },
  { key: "fr", label: "Français" },
  { key: "es", label: "Español" },
];

const messages: Record<LocaleKey, typeof en> = {
  en,
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  ja,
  de,
  fr,
  es,
};

export function getSystemLocale(): LocaleKey {
  try {
    const navLang = navigator.language || "";
    if (navLang.startsWith("zh-TW") || navLang.startsWith("zh-HK")) return "zh-TW";
    if (navLang.startsWith("zh")) return "zh-CN";
    if (navLang.startsWith("ja")) return "ja";
    if (navLang.startsWith("de")) return "de";
    if (navLang.startsWith("fr")) return "fr";
    if (navLang.startsWith("es")) return "es";
  } catch {
    // fallback
  }
  return "en";
}

function getInitialSetting(): LocaleSetting {
  try {
    let saved = localStorage.getItem("omera_locale") as LocaleSetting | null;
    if (!saved) {
      saved = localStorage.getItem("berry_locale") as LocaleSetting | null;
      if (saved) {
        localStorage.setItem("omera_locale", saved);
      }
    }
    if (saved && (saved === "auto" || saved in messages)) {
      return saved;
    }
  } catch {
    // fallback
  }
  return "auto";
}

export const currentLocaleSetting = ref<LocaleSetting>(getInitialSetting());

export const currentLocale = computed<LocaleKey>(() => {
  if (currentLocaleSetting.value === "auto") {
    return getSystemLocale();
  }
  return currentLocaleSetting.value;
});

export function setLocale(setting: LocaleSetting) {
  currentLocaleSetting.value = setting;
  try {
    localStorage.setItem("omera_locale", setting);
  } catch {
    // ignore
  }
}

export const t = computed(() => messages[currentLocale.value] || messages.en);
