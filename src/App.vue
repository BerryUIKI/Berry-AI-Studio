<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openFolderDialog } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Album,
  AppInfo,
  FileSortField,
  Folder,
  ImageFile,
  LibraryCounts,
  NavTarget,
  ScanProgress,
  SearchCriteria,
  SimilarFileItem,
  SortDirection,
  Tag,
  StackSummary,
} from "./types";
import { getFileName } from "./utils/image";
import TitleBar from "./components/TitleBar.vue";
import MenuBar from "./components/MenuBar.vue";
import Sidebar from "./components/Sidebar.vue";
import FileList from "./components/FileList.vue";
import VirtualGrid from "./components/VirtualGrid.vue";
import SortBar from "./components/SortBar.vue";
import SearchBar from "./components/SearchBar.vue";
import InspectorPane from "./components/InspectorPane.vue";
import LightboxModal from "./components/LightboxModal.vue";
import StatusBar from "./components/StatusBar.vue";
import FilterDrawer from "./components/FilterDrawer.vue";
import BatchActionBar from "./components/BatchActionBar.vue";
import AlbumModal from "./components/AlbumModal.vue";
import TagModal from "./components/TagModal.vue";
import PromptStatsModal from "./components/PromptStatsModal.vue";
import ModelManagerModal from "./components/ModelManagerModal.vue";
import FileOperationModal from "./components/FileOperationModal.vue";
import DatabaseManagerModal from "./components/DatabaseManagerModal.vue";
import ShortcutsHelpModal from "./components/ShortcutsHelpModal.vue";
import SettingsModal from "./components/SettingsModal.vue";
import UpdateModal from "./components/UpdateModal.vue";
import AutoTagModal from "./components/AutoTagModal.vue";
import ClipManagerModal from "./components/ClipManagerModal.vue";
import LoraManagerModal from "./components/LoraManagerModal.vue";
import AddFolderModal from "./components/AddFolderModal.vue";
import OnboardingModal from "./components/OnboardingModal.vue";
import CompareModal from "./components/CompareModal.vue";
import { t } from "./i18n";
import { countActiveFilters, criteriaToQuery } from "./utils/search";
import { requestBatchThumbnails } from "./utils/thumbnail";
import { loadAppConfig, saveAppConfig } from "./utils/config";
import { checkForUpdates } from "./utils/updater";

const info = ref<AppInfo | null>(null);
const folders = ref<Folder[]>([]);
const libraryCounts = ref<LibraryCounts | null>(null);
const albums = ref<Album[]>([]);
const albumCounts = ref<Record<number, number>>({});
const tags = ref<Tag[]>([]);
const activeTarget = ref<NavTarget>({ type: "all" });
const files = shallowRef<ImageFile[]>([]);
const filesLoading = ref(false);
const searchQuery = ref("");
const isSemanticSearch = ref(false);
const clipModalOpen = ref(false);
const loraModalOpen = ref(false);
const gridItemWidth = ref(200);
const similaritySourceFile = ref<ImageFile | null>(null);
const rawSimilarityFiles = shallowRef<ImageFile[]>([]);
const similarityThreshold = ref<number>(0);
const similarityLimit = ref<number>(
  Number(localStorage.getItem("berry_similarity_limit")) || 50
);

// UI Pane Toggles (Eagle Studio layout)
const sidebarOpen = ref(true);
const inspectorOpen = ref(true);
const lightboxFile = ref<ImageFile | null>(null);

// Modals
const updateModalOpen = ref(false);
const settingsModalOpen = ref(false);
const filterDrawerOpen = ref(false);
const promptStatsModalOpen = ref(false);
const modelManagerModalOpen = ref(false);
const dbManagerModalOpen = ref(false);
const shortcutsHelpModalOpen = ref(false);
const fileOpModalOpen = ref(false);
const fileOpMode = ref<"move" | "copy" | "trash">("move");
const fileOpTargetFiles = ref<ImageFile[]>([]);
const albumModalOpen = ref(false);
const albumTargetFileIds = ref<number[]>([]);
const tagModalOpen = ref(false);
const tagTargetFileIds = ref<number[]>([]);
const autoTagModalOpen = ref(false);
const autoTagTargetFile = ref<ImageFile | null>(null);
const inspectorRef = ref<InstanceType<typeof InspectorPane> | null>(null);
const addFolderModalOpen = ref(false);
const onboardingModalOpen = ref(false);
/** Session-level guard: once the onboarding modal is dismissed it can never reopen. */
let onboardingDismissedThisSession = false;
const compareModalOpen = ref(false);
const compareImages = ref<ImageFile[]>([]);

// Image Stacking State
const stackMap = ref<Record<string, { count: number; heroId: number | null }>>({});
const expandedStacks = ref<Set<string>>(new Set());

// Filter Metadata
const distinctModels = ref<string[]>([]);
const distinctSamplers = ref<string[]>([]);
const activeCriteria = ref<SearchCriteria>({});
const activeFilterCount = computed(() => countActiveFilters(activeCriteria.value));

// Selection
const selectedFile = ref<ImageFile | null>(null);
const selectedFilePaths = ref<Set<string>>(new Set());

// Fast lookup map computed once per files change (O(1) lookups on selection)
const filePathMap = computed(() => {
  const map = new Map<string, ImageFile>();
  for (const f of files.value) {
    map.set(f.path, f);
  }
  return map;
});

const selectedFilesList = computed(() => {
  if (selectedFilePaths.value.size === 0) return [];
  const map = filePathMap.value;
  const list: ImageFile[] = [];
  for (const path of selectedFilePaths.value) {
    const f = map.get(path);
    if (f) list.push(f);
  }
  return list;
});

const viewMode = ref<"grid" | "table">(
  (localStorage.getItem("berry_default_view") as "grid" | "table") || "grid",
);
const blurNsfw = ref(localStorage.getItem("berry_blur_nsfw") !== "false");
const showCardBadges = ref(localStorage.getItem("berry_card_badges") !== "false");

function setViewMode(mode: "grid" | "table") {
  viewMode.value = mode;
  localStorage.setItem("berry_default_view", mode);
}

function onSettingsSaved(settings: {
  locale?: string;
  autoScan: boolean;
  blurNsfw: boolean;
  showCardBadges: boolean;
  defaultView: "grid" | "table";
  thumbnailMaxEdge?: number;
  autoCheckUpdate?: boolean;
}) {
  blurNsfw.value = settings.blurNsfw;
  showCardBadges.value = settings.showCardBadges;
  setViewMode(settings.defaultView);
}

const sortField = ref<FileSortField>("modified_at");
const sortDirection = ref<SortDirection>("desc");
const progress = ref<ScanProgress | null>(null);
const error = ref("");

let unlisten: UnlistenFn | null = null;

function handleWindowKeyDown(e: KeyboardEvent) {
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") {
    if (e.key === "Escape") {
      (document.activeElement as HTMLElement)?.blur();
    }
    return;
  }

  // Settings Modal: Cmd+, / Ctrl+,
  if ((e.ctrlKey || e.metaKey) && e.key === ",") {
    e.preventDefault();
    settingsModalOpen.value = !settingsModalOpen.value;
    return;
  }

  // Help Modal: ?
  if (e.key === "?" || (e.shiftKey && e.key === "/")) {
    e.preventDefault();
    shortcutsHelpModalOpen.value = !shortcutsHelpModalOpen.value;
    return;
  }

  // Focus Search Bar: / or Cmd+F / Ctrl+F
  if (e.key === "/" || ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F"))) {
    e.preventDefault();
    const searchInput = document.querySelector<HTMLInputElement>(".search-bar-eagle input");
    searchInput?.focus();
    searchInput?.select();
    return;
  }

  // Toggle Inspector: I / i (without ctrl/meta)
  if ((e.key === "i" || e.key === "I") && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    inspectorOpen.value = !inspectorOpen.value;
    return;
  }

  // Toggle Sidebar: B / b (without ctrl/meta)
  if ((e.key === "b" || e.key === "B") && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    sidebarOpen.value = !sidebarOpen.value;
    return;
  }

  // Open Add Folder Modal: Cmd+O / Ctrl+O
  if ((e.ctrlKey || e.metaKey) && (e.key === "o" || e.key === "O")) {
    e.preventDefault();
    addFolderModalOpen.value = true;
    return;
  }

  // Stacking: Group (Ctrl+G) or Ungroup (Ctrl+Shift+G)
  if ((e.ctrlKey || e.metaKey) && (e.key === "g" || e.key === "G")) {
    e.preventDefault();
    if (e.shiftKey) {
      void onUnstackSelected();
    } else {
      void onStackSelected();
    }
    return;
  }

  // Set Hero Cover for Stack: Alt+S
  if (e.altKey && (e.key === "s" || e.key === "S")) {
    e.preventDefault();
    void onSetHeroSelected();
    return;
  }

  // Compare Mode: C (without modifiers)
  if ((e.key === "c" || e.key === "C") && !e.ctrlKey && !e.metaKey && !e.altKey && !lightboxFile.value) {
    e.preventDefault();
    void onTriggerCompare();
    return;
  }

  // Select All: Cmd+A / Ctrl+A
  if ((e.ctrlKey || e.metaKey) && (e.key === "a" || e.key === "A")) {
    e.preventDefault();
    onSelectAll();
    return;
  }

  // Escape: Close modals, lightbox, or clear selection
  if (e.key === "Escape") {
    if (lightboxFile.value) {
      lightboxFile.value = null;
      return;
    }
    if (updateModalOpen.value) {
      updateModalOpen.value = false;
      return;
    }
    if (settingsModalOpen.value) {
      settingsModalOpen.value = false;
      return;
    }
    if (shortcutsHelpModalOpen.value) {
      shortcutsHelpModalOpen.value = false;
      return;
    }
    if (dbManagerModalOpen.value) {
      dbManagerModalOpen.value = false;
      return;
    }
    if (modelManagerModalOpen.value) {
      modelManagerModalOpen.value = false;
      return;
    }
    if (promptStatsModalOpen.value) {
      promptStatsModalOpen.value = false;
      return;
    }
    if (filterDrawerOpen.value) {
      filterDrawerOpen.value = false;
      return;
    }
    if (albumModalOpen.value) {
      albumModalOpen.value = false;
      return;
    }
    if (tagModalOpen.value) {
      tagModalOpen.value = false;
      return;
    }
    if (fileOpModalOpen.value) {
      fileOpModalOpen.value = false;
      return;
    }
    if (clipModalOpen.value) {
      clipModalOpen.value = false;
      return;
    }
    if (onboardingModalOpen.value) {
      void onOnboardingComplete();
      return;
    }
    if (addFolderModalOpen.value) {
      addFolderModalOpen.value = false;
      return;
    }
    if (compareModalOpen.value) {
      compareModalOpen.value = false;
      return;
    }
    if (loraModalOpen.value) {
      loraModalOpen.value = false;
      return;
    }
    if (selectedFilePaths.value.size > 0) {
      onClearSelection();
      return;
    }
    if (similaritySourceFile.value) {
      exitSimilaritySearch();
      return;
    }
  }

  // Open Lightbox: Space or Enter
  if (e.key === " " || e.key === "Enter") {
    if (!lightboxFile.value && (selectedFile.value || selectedFilesList.value.length > 0)) {
      e.preventDefault();
      lightboxFile.value = selectedFile.value || selectedFilesList.value[0];
      return;
    }
  }

  // Star Ratings: 1 - 5 (or 0 to clear)
  if (["0", "1", "2", "3", "4", "5"].includes(e.key) && !lightboxFile.value) {
    const targetFile =
      selectedFile.value || (selectedFilesList.value.length > 0 ? selectedFilesList.value[0] : null);
    if (targetFile && targetFile.id != null) {
      e.preventDefault();
      const rating = e.key === "0" ? null : parseInt(e.key, 10);
      if (selectedFilesList.value.length > 1) {
        void onBatchRate(rating);
      } else {
        void invoke("set_file_rating", { fileId: targetFile.id, rating });
        onFileRated(targetFile.id, rating);
      }
      return;
    }
  }

  // Favorite toggle: F
  if ((e.key === "f" || e.key === "F") && !lightboxFile.value) {
    const targetFile =
      selectedFile.value || (selectedFilesList.value.length > 0 ? selectedFilesList.value[0] : null);
    if (targetFile) {
      e.preventDefault();
      if (selectedFilesList.value.length > 1) {
        const anyUnfav = selectedFilesList.value.some((f) => !f.is_favorite);
        void onBatchToggleFavorite(anyUnfav);
      } else {
        void onBatchToggleFavorite(!targetFile.is_favorite);
      }
      return;
    }
  }

  // Delete / Trash: Delete or Backspace
  if ((e.key === "Delete" || e.key === "Backspace") && !lightboxFile.value) {
    if (selectedFilesList.value.length > 0 || selectedFile.value) {
      e.preventDefault();
      if (selectedFilesList.value.length === 0 && selectedFile.value) {
        selectedFilePaths.value.add(selectedFile.value.path);
      }
      onBatchTrash();
      return;
    }
  }
}

const thumbProgress = ref<{ current: number; total: number; active: boolean } | null>(null);
let unlistenThumb: UnlistenFn | null = null;
let thumbProgressHideTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(async () => {
  window.addEventListener("keydown", handleWindowKeyDown);
  try {
    info.value = await invoke<AppInfo>("get_app_info");

    // Load persistent configuration from config.json (auto-migrating localStorage)
    const cfg = await loadAppConfig();
    viewMode.value = cfg.default_view || "grid";
    blurNsfw.value = cfg.blur_nsfw;
    showCardBadges.value = cfg.show_card_badges;

    await reloadFolders();
    await refreshCounts();
    await reloadFiltersMeta();
    await loadAlbumsAndTags();
    await loadFiles();

    if (!cfg.has_completed_onboarding && !onboardingDismissedThisSession) {
      onboardingModalOpen.value = true;
    }

    // Safely process any due pipeline trash cleanups
    void invoke("process_pipeline_cleanups").catch(() => {});

    if (cfg.auto_scan && folders.value.length > 0) {
      void runBackgroundStartupScan();
    }

    if (cfg.auto_check_update && info.value?.app_version) {
      void checkForUpdates(info.value.app_version).then((res) => {
        if (res.status === "update_available") {
          updateModalOpen.value = true;
        }
      });
    }
  } catch (e) {
    error.value = String(e);
  }

  unlisten = await listen<ScanProgress>("scan-progress", (event) => {
    progress.value = event.payload;
  });

  unlistenThumb = await listen<{ current: number; total: number; done: boolean }>(
    "thumbnail-progress",
    (event) => {
      const p = event.payload;
      if (thumbProgressHideTimer) clearTimeout(thumbProgressHideTimer);
      thumbProgress.value = {
        current: p.current,
        total: p.total,
        active: !p.done && p.total > 0 && p.current < p.total,
      };
      if (p.done || p.current >= p.total) {
        thumbProgressHideTimer = setTimeout(() => {
          thumbProgress.value = null;
        }, 1500);
      }
    },
  );
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleWindowKeyDown);
  unlisten?.();
  unlistenThumb?.();
});

async function reloadFolders() {
  folders.value = await invoke<Folder[]>("list_folders");
}

async function runBackgroundStartupScan() {
  for (const f of folders.value) {
    try {
      await invoke("scan_folder", { folderId: f.id });
    } catch (e) {
      console.warn(`Startup auto-scan skipped for folder ${f.path}:`, e);
    }
  }
  await refreshCounts();
  await loadFiles();
}

async function refreshCounts() {
  try {
    libraryCounts.value = await invoke<LibraryCounts>("get_library_counts");
  } catch (e) {
    console.error("Failed to fetch library counts:", e);
  }
}

async function reloadFiltersMeta() {
  try {
    distinctModels.value = await invoke<string[]>("list_distinct_models");
    distinctSamplers.value = await invoke<string[]>("list_distinct_samplers");
  } catch (e) {
    console.error("Failed to load distinct models/samplers:", e);
  }
}

async function loadAlbumsAndTags() {
  try {
    albums.value = await invoke<Album[]>("list_albums");
    const counts: Record<number, number> = {};
    for (const album of albums.value) {
      counts[album.id] = await invoke<number>("count_album_files", {
        albumId: album.id,
      });
    }
    albumCounts.value = counts;
    tags.value = await invoke<Tag[]>("list_tags");
  } catch (e) {
    console.error("Failed to load albums/tags:", e);
  }
}

function onFolderAdded(folder: Folder) {
  void reloadFolders();
  void refreshCounts();
  void reloadFiltersMeta();
  activeTarget.value = { type: "folder", folder };
  selectedFile.value = null;
  lightboxFile.value = null;
  void loadFiles();
}

function onFolderRemoved(folderId: number) {
  folders.value = folders.value.filter((f) => f.id !== folderId);
  void refreshCounts();
  void reloadFiltersMeta();
  if (activeTarget.value.type === "folder" && activeTarget.value.folder.id === folderId) {
    activeTarget.value = { type: "all" };
    selectedFile.value = null;
    lightboxFile.value = null;
  }
  void loadFiles();
}

function onSelectNav(target: NavTarget) {
  similaritySourceFile.value = null;
  rawSimilarityFiles.value = [];
  activeTarget.value = target;
  selectedFile.value = null;
  lightboxFile.value = null;
  void loadFiles();
}

const targetTitle = computed(() => {
  if (similaritySourceFile.value) {
    return `🔍 ${t.value.preview.similaritySearchTitle} ${getFileName(similaritySourceFile.value.path)}`;
  }
  switch (activeTarget.value.type) {
    case "all":
      return t.value.nav.allImages;
    case "favorites":
      return t.value.nav.favorites;
    case "nsfw":
      return t.value.nav.sensitive;
    case "folder":
      return activeTarget.value.folder.path.split(/[\\/]/).pop() || activeTarget.value.folder.path;
    case "album":
      return `📚 ${activeTarget.value.album.name}`;
    case "tag":
      return `🏷️ #${activeTarget.value.tag.name}`;
  }
});

async function onFolderScanned(_folderId: number) {
  await refreshCounts();
  await reloadFiltersMeta();
  await loadAlbumsAndTags();
  await loadFiles();
}

function onFileSelected(file: ImageFile, event?: MouseEvent) {
  selectedFile.value = file;
  if (event?.metaKey || event?.ctrlKey) {
    toggleSelectFile(file);
  } else if (event?.shiftKey && selectedFilesList.value.length > 0) {
    const lastFile = selectedFilesList.value[selectedFilesList.value.length - 1];
    const idx1 = files.value.findIndex((f) => f.path === lastFile.path);
    const idx2 = files.value.findIndex((f) => f.path === file.path);
    if (idx1 !== -1 && idx2 !== -1) {
      const [start, end] = idx1 < idx2 ? [idx1, idx2] : [idx2, idx1];
      for (let i = start; i <= end; i++) {
        selectedFilePaths.value.add(files.value[i].path);
      }
    }
  }
}

function toggleSelectFile(file: ImageFile) {
  if (selectedFilePaths.value.has(file.path)) {
    selectedFilePaths.value.delete(file.path);
  } else {
    selectedFilePaths.value.add(file.path);
  }
}

function onSelectAll() {
  selectedFilePaths.value = new Set(files.value.map((f) => f.path));
}

function onClearSelection() {
  selectedFilePaths.value.clear();
}

function onToggleAll() {
  if (selectedFilePaths.value.size === files.value.length) {
    selectedFilePaths.value.clear();
  } else {
    onSelectAll();
  }
}

async function onBatchRate(rating: number | null) {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length === 0) return;

  try {
    await invoke("set_files_rating", { fileIds: ids, rating });
    const idSet = new Set(ids);
    files.value = files.value.map((f) => {
      if (f.id != null && idSet.has(f.id)) {
        return { ...f, rating: rating ?? undefined };
      }
      return f;
    });
    if (selectedFile.value?.id != null && idSet.has(selectedFile.value.id)) {
      selectedFile.value.rating = rating ?? undefined;
    }
  } catch (e) {
    error.value = String(e);
  }
}

function onActivateFile(file: ImageFile) {
  selectedFile.value = file;
  lightboxFile.value = file;
}

function onLightboxNavigate(file: ImageFile) {
  selectedFile.value = file;
  lightboxFile.value = file;
}

function onFileRated(fileId: number, rating: number | null) {
  const idx = files.value.findIndex((f) => f.id === fileId);
  if (idx !== -1) {
    const updated = [...files.value];
    updated[idx] = { ...updated[idx], rating: rating ?? undefined };
    files.value = updated;
  }
  if (selectedFile.value?.id === fileId) {
    selectedFile.value.rating = rating ?? undefined;
  }
  if (lightboxFile.value?.id === fileId) {
    lightboxFile.value.rating = rating ?? undefined;
  }
}

function onOpenAlbumModal(fileIds?: number[]) {
  albumTargetFileIds.value = fileIds ?? [];
  albumModalOpen.value = true;
}

async function onStackSelected() {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length < 2) return;
  try {
    await invoke("stack_images", { fileIds: ids });
    await loadFiles();
  } catch (err) {
    error.value = String(err);
  }
}

async function onUnstackSelected() {
  const file = selectedFile.value || (selectedFilesList.value.length > 0 ? selectedFilesList.value[0] : null);
  if (!file?.stack_id) return;
  try {
    await invoke("unstack_images", { stackId: file.stack_id });
    expandedStacks.value.delete(file.stack_id);
    await loadFiles();
  } catch (err) {
    error.value = String(err);
  }
}

async function onSetHeroSelected() {
  const file = selectedFile.value || (selectedFilesList.value.length > 0 ? selectedFilesList.value[0] : null);
  if (!file?.stack_id || file.id == null) return;
  try {
    await invoke("set_stack_hero", { stackId: file.stack_id, heroFileId: file.id });
    await loadFiles();
  } catch (err) {
    error.value = String(err);
  }
}

function onToggleStackExpand(stackId: string) {
  if (expandedStacks.value.has(stackId)) {
    expandedStacks.value.delete(stackId);
  } else {
    expandedStacks.value.add(stackId);
  }
  void loadFiles();
}

async function onTriggerCompare(customStackId?: string) {
  if (customStackId) {
    try {
      const members = await invoke<ImageFile[]>("get_stack_members", { stackId: customStackId });
      if (members.length > 0) {
        compareImages.value = members;
        compareModalOpen.value = true;
        return;
      }
    } catch (err) {
      console.warn("Failed to load stack members for compare:", err);
    }
  }

  // Otherwise compare selected files
  if (selectedFilesList.value.length >= 2) {
    compareImages.value = selectedFilesList.value;
    compareModalOpen.value = true;
  } else if (selectedFile.value?.stack_id) {
    try {
      const members = await invoke<ImageFile[]>("get_stack_members", { stackId: selectedFile.value.stack_id });
      if (members.length > 0) {
        compareImages.value = members;
        compareModalOpen.value = true;
      }
    } catch (err) {
      console.warn("Failed to load stack members for compare:", err);
    }
  }
}

async function onCompareSetHero(img: ImageFile) {
  if (!img.stack_id || img.id == null) return;
  try {
    await invoke("set_stack_hero", { stackId: img.stack_id, heroFileId: img.id });
    await loadFiles();
  } catch (err) {
    console.error("Failed to set hero in compare:", err);
  }
}

async function onOnboardingComplete() {
  // Immediately prevent any re-opening — this is the critical guard
  onboardingDismissedThisSession = true;
  onboardingModalOpen.value = false;
  try {
    const cfg = await loadAppConfig();
    await saveAppConfig({
      ...cfg,
      has_completed_onboarding: true,
    });
  } catch (err) {
    console.warn("Failed to mark onboarding complete:", err);
  }
  // Reload data in the background (errors here should NOT affect the modal state)
  try {
    await reloadFolders();
    await refreshCounts();
    await loadFiles();
  } catch (err) {
    console.warn("Post-onboarding data reload failed:", err);
  }
}

function onBatchAddToAlbum() {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length > 0) {
    onOpenAlbumModal(ids);
  }
}

function onAddedToAlbum(_album: Album) {
  selectedFilePaths.value.clear();
}

function onOpenTagModal(fileIds?: number[]) {
  tagTargetFileIds.value = fileIds ?? [];
  tagModalOpen.value = true;
}

function onBatchTag() {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length > 0) {
    onOpenTagModal(ids);
  }
}

function handleOpenAutoTag(file?: ImageFile) {
  autoTagTargetFile.value = file ?? selectedFile.value;
  autoTagModalOpen.value = true;
}

async function onAutoTagsApplied() {
  await loadAlbumsAndTags();
  await refreshCounts();
  if (inspectorRef.value) {
    await inspectorRef.value.loadTags();
  }
}

async function onBatchToggleFavorite(isFavorite: boolean) {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length === 0) return;
  try {
    await invoke("set_files_favorite", { fileIds: ids, isFavorite });
    const updated = files.value.map((f) => {
      if (selectedFilePaths.value.has(f.path)) {
        return { ...f, is_favorite: isFavorite };
      }
      return f;
    });
    files.value = updated;
    if (selectedFile.value && selectedFilePaths.value.has(selectedFile.value.path)) {
      selectedFile.value.is_favorite = isFavorite;
    }
  } catch (err) {
    error.value = String(err);
  }
}

async function onBatchToggleNsfw(isNsfw: boolean) {
  const ids = selectedFilesList.value
    .map((f) => f.id)
    .filter((id): id is number => id != null);
  if (ids.length === 0) return;
  try {
    await invoke("set_files_nsfw", { fileIds: ids, isNsfw });
    const updated = files.value.map((f) => {
      if (selectedFilePaths.value.has(f.path)) {
        return { ...f, is_nsfw: isNsfw };
      }
      return f;
    });
    files.value = updated;
    if (selectedFile.value && selectedFilePaths.value.has(selectedFile.value.path)) {
      selectedFile.value.is_nsfw = isNsfw;
    }
  } catch (err) {
    error.value = String(err);
  }
}

function onBatchMove() {
  fileOpTargetFiles.value = [...selectedFilesList.value];
  fileOpMode.value = "move";
  fileOpModalOpen.value = true;
}

function onBatchCopy() {
  fileOpTargetFiles.value = [...selectedFilesList.value];
  fileOpMode.value = "copy";
  fileOpModalOpen.value = true;
}

function onBatchTrash() {
  fileOpTargetFiles.value = [...selectedFilesList.value];
  fileOpMode.value = "trash";
  fileOpModalOpen.value = true;
}

async function onFileOpCompleted() {
  selectedFilePaths.value.clear();
  await refreshCounts();
  await loadFiles();
}

function onUpdateFile(file: ImageFile) {
  const idx = files.value.findIndex((f) => f.id === file.id);
  if (idx !== -1) {
    const updated = [...files.value];
    updated[idx] = { ...file };
    files.value = updated;
  }
  if (selectedFile.value?.id === file.id) {
    selectedFile.value = { ...file };
  }
  if (lightboxFile.value?.id === file.id) {
    lightboxFile.value = { ...file };
  }
}

async function loadFiles() {
  similaritySourceFile.value = null;
  rawSimilarityFiles.value = [];
  filesLoading.value = true;
  selectedFilePaths.value.clear();
  try {
    const q = searchQuery.value.trim();
    if (q) {
      if (isSemanticSearch.value) {
        // Text-to-image semantic search via active CLIP model
        try {
          const matches = await invoke<SimilarFileItem[]>("search_by_text_prompt", {
            prompt: q,
            limit: similarityLimit.value || 50,
          });
          files.value = matches.map((m) => ({
            ...m.file,
            similarity_score: m.score,
          }));
        } catch (clipErr) {
          // If no model loaded, open CLIP modal so user can load one
          console.warn("Semantic search failed or model not loaded:", clipErr);
          clipModalOpen.value = true;
          files.value = [];
        }
      } else {
        const folderId = activeTarget.value.type === "folder" ? activeTarget.value.folder.id : null;
        files.value = await invoke<ImageFile[]>("search_files_by_query", {
          query: q,
          folderId,
          sort: sortField.value,
          direction: sortDirection.value,
        });
      }
    } else {
      const criteria: SearchCriteria = {
        sort: sortField.value,
        direction: sortDirection.value,
      };
      if (activeTarget.value.type === "folder") {
        criteria.folder_id = activeTarget.value.folder.id;
      } else if (activeTarget.value.type === "favorites") {
        criteria.is_favorite = true;
      } else if (activeTarget.value.type === "nsfw") {
        criteria.is_nsfw = true;
      } else if (activeTarget.value.type === "album") {
        criteria.album_id = activeTarget.value.album.id;
      } else if (activeTarget.value.type === "tag") {
        criteria.tag_id = activeTarget.value.tag.id;
      }
      files.value = await invoke<ImageFile[]>("search_files", { criteria });
    }

    // Refresh stack summaries for active folder or all
    try {
      const folderId = activeTarget.value.type === "folder" ? activeTarget.value.folder.id : null;
      const stacks = await invoke<StackSummary[]>("list_stacks", { folderId });
      const map: Record<string, { count: number; heroId: number | null }> = {};
      for (const s of stacks) {
        map[s.stack_id] = { count: s.count, heroId: s.hero_image_id };
      }
      stackMap.value = map;

      // Filter out non-hero stack members unless that stack is expanded
      if (files.value.length > 0) {
        files.value = files.value.filter((f) => {
          if (!f.stack_id) return true;
          const info = stackMap.value[f.stack_id];
          if (!info || info.count <= 1) return true;
          if (expandedStacks.value.has(f.stack_id)) return true;
          // Only show the Hero Cover (stack_order === 0 or matches hero_image_id)
          return f.stack_order === 0 || (f.id != null && f.id === info.heroId);
        });
      }
    } catch (stackErr) {
      console.warn("Failed to load stack metadata:", stackErr);
    }

    // Background async batch generation for initial slice of files
    if (files.value.length > 0) {
      void requestBatchThumbnails(files.value.slice(0, 200));
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    filesLoading.value = false;
  }
}

function onInjectPrompt(text: string) {
  if (searchQuery.value.trim()) {
    searchQuery.value = `${searchQuery.value.trim()}, ${text}`;
  } else {
    searchQuery.value = text;
  }
}

function applySimilarityFilter() {
  const minScore = similarityThreshold.value / 100;
  const filtered = rawSimilarityFiles.value
    .filter((f) => (f.similarity_score ?? 0) >= minScore)
    .sort((a, b) => (b.similarity_score ?? 0) - (a.similarity_score ?? 0));
  files.value = filtered;
  if (filtered.length > 0) {
    if (!selectedFile.value || !filtered.some((f) => f.id === selectedFile.value?.id)) {
      selectedFile.value = filtered[0];
    }
  } else {
    selectedFile.value = null;
  }
}

function onSimilarityThresholdChange() {
  applySimilarityFilter();
}

function onSimilarityLimitChange() {
  localStorage.setItem("berry_similarity_limit", String(similarityLimit.value));
  if (similaritySourceFile.value) {
    void handleFindSimilar(similaritySourceFile.value);
  }
}

async function handleFindSimilar(file: ImageFile) {
  if (!file.id) return;
  filesLoading.value = true;
  try {
    const items = await invoke<SimilarFileItem[]>("find_similar_to_file", {
      fileId: file.id,
      limit: similarityLimit.value,
    });
    if (items.length === 0) {
      const models = await invoke<string[]>("get_file_embedding_models", {
        fileId: file.id,
      });
      if (models.length === 0) {
        alert(t.value.preview.noEmbeddingFound);
        return;
      }
    }
    similaritySourceFile.value = file;
    selectedFilePaths.value.clear();
    rawSimilarityFiles.value = items.map((item) => ({
      ...item.file,
      similarity_score: item.score,
    }));
    applySimilarityFilter();
    if (files.value.length > 0) {
      void requestBatchThumbnails(files.value.slice(0, 200));
    }
    if (lightboxFile.value) {
      lightboxFile.value = null;
    }
  } catch (err) {
    console.error("Find similar error:", err);
    error.value = String(err);
  } finally {
    filesLoading.value = false;
  }
}

function exitSimilaritySearch() {
  similaritySourceFile.value = null;
  rawSimilarityFiles.value = [];
  similarityThreshold.value = 0;
  void loadFiles();
}

function onSearch(query: string) {
  searchQuery.value = query;
  void loadFiles();
}

function onClearSearch() {
  searchQuery.value = "";
  activeCriteria.value = {};
  void loadFiles();
}

function onApplyFilters(criteria: SearchCriteria) {
  activeCriteria.value = criteria;
  const q = criteriaToQuery(criteria);
  searchQuery.value = q;
  void loadFiles();
}

function onResetFilters() {
  activeCriteria.value = {};
  searchQuery.value = "";
  void loadFiles();
}

function onApplyStatsSearch(query: string) {
  searchQuery.value = query;
  activeCriteria.value = {};
  void loadFiles();
}

function onFilterByModel(modelName: string) {
  activeCriteria.value = { ...activeCriteria.value, model_name: modelName };
  searchQuery.value = criteriaToQuery(activeCriteria.value);
  void loadFiles();
}

function onFilterByHash(modelHash: string) {
  activeCriteria.value = { ...activeCriteria.value, model_hash: modelHash };
  searchQuery.value = criteriaToQuery(activeCriteria.value);
  void loadFiles();
}

async function onDropMoveFiles(payload: { filePaths: string[]; folderId: number }) {
  try {
    await invoke("move_files", {
      filePaths: payload.filePaths,
      targetFolderId: payload.folderId,
    });
    await onFileOpCompleted();
  } catch (err) {
    error.value = String(err);
  }
}

async function onDropAddFilesToAlbum(payload: { fileIds: number[]; albumId: number }) {
  try {
    await invoke("add_files_to_album", {
      albumId: payload.albumId,
      fileIds: payload.fileIds,
    });
    await loadAlbumsAndTags();
  } catch (err) {
    error.value = String(err);
  }
}

async function onDropTagFiles(payload: { fileIds: number[]; tagId: number }) {
  try {
    await invoke("tag_files", {
      tagId: payload.tagId,
      fileIds: payload.fileIds,
    });
    await loadAlbumsAndTags();
    await loadFiles();
  } catch (err) {
    error.value = String(err);
  }
}

async function onDatabaseChanged() {
  await reloadFolders();
  await refreshCounts();
  await reloadFiltersMeta();
  await loadAlbumsAndTags();
  await loadFiles();
}

async function onAddFolderFromMenu() {
  try {
    const selected = await openFolderDialog({ directory: true, multiple: false });
    if (typeof selected !== "string") return;
    const folder = await invoke<Folder>("add_folder", { path: selected });
    onFolderAdded(folder);
  } catch (e) {
    error.value = String(e);
  }
}

async function onScanActiveFromMenu() {
  if (activeTarget.value.type === "folder") {
    try {
      await invoke("scan_folder", { folderId: activeTarget.value.folder.id });
      await onFolderScanned(activeTarget.value.folder.id);
    } catch (e) {
      error.value = String(e);
    }
  } else if (folders.value.length > 0) {
    try {
      await invoke("scan_folder", { folderId: folders.value[0].id });
      await onFolderScanned(folders.value[0].id);
    } catch (e) {
      error.value = String(e);
    }
  }
}

async function onRescanAllFromMenu() {
  for (const folder of folders.value) {
    try {
      await invoke("scan_folder", { folderId: folder.id });
    } catch (e) {
      console.error(e);
    }
  }
  await refreshCounts();
  await reloadFiltersMeta();
  await loadAlbumsAndTags();
  await loadFiles();
}

function onZoomIn() {
  gridItemWidth.value = Math.min(360, gridItemWidth.value + 20);
}

function onZoomOut() {
  gridItemWidth.value = Math.max(130, gridItemWidth.value - 20);
}

function onResetZoom() {
  gridItemWidth.value = 200;
}
</script>

<template>
  <div class="app-window-eagle">
    <!-- Custom Frameless Titlebar (Eagle Studio Style with Top MenuBar) -->
    <TitleBar
      :title="t.app.title"
      :subtitle="info ? `v${info.app_version}` : undefined"
    >
      <template #leading>
        <!-- Toggle Sidebar Button (Eagle style at far left before software name) -->
        <button
          type="button"
          class="titlebar-quick-btn"
          :class="{ active: sidebarOpen }"
          :title="sidebarOpen ? '隐藏导航栏 (B)' : '显示导航栏 (B)'"
          style="margin-left: 8px;"
          @click="sidebarOpen = !sidebarOpen"
        >
          <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h11A1.5 1.5 0 0 1 15 3.5v9a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 1 12.5v-9zM2.5 3a.5.5 0 0 0-.5.5v9a.5.5 0 0 0 .5.5H5V3H2.5zm3.5 10h7.5a.5.5 0 0 0 .5-.5v-9a.5.5 0 0 0-.5-.5H6v10z"/>
          </svg>
        </button>
      </template>

      <template #menu>
        <MenuBar
          @add-folder="onAddFolderFromMenu"
          @scan-active="onScanActiveFromMenu"
          @rescan-all="onRescanAllFromMenu"
          @open-db-manager="dbManagerModalOpen = true"
          @open-settings="settingsModalOpen = true"
          @select-all="onSelectAll"
          @clear-selection="onClearSelection"
          @batch-album="onBatchAddToAlbum"
          @toggle-sidebar="sidebarOpen = !sidebarOpen"
          @toggle-inspector="inspectorOpen = !inspectorOpen"
          @open-lightbox="selectedFile ? onActivateFile(selectedFile) : null"
          @zoom-in="onZoomIn"
          @zoom-out="onZoomOut"
          @reset-zoom="onResetZoom"
          @open-prompt-stats="promptStatsModalOpen = true"
          @open-model-manager="modelManagerModalOpen = true"
          @open-clip-manager="clipModalOpen = true"
          @open-lora-manager="loraModalOpen = true"
          @open-shortcuts-help="shortcutsHelpModalOpen = true"
          @open-updater="updateModalOpen = true"
          @open-about="settingsModalOpen = true"
        />
      </template>

      <template #actions>
        <!-- Toggle Inspector Button -->
        <button
          type="button"
          class="titlebar-quick-btn"
          :class="{ active: inspectorOpen }"
          :title="inspectorOpen ? '隐藏检查器 (I)' : '显示检查器 (I)'"
          @click="inspectorOpen = !inspectorOpen"
        >
          <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h11A1.5 1.5 0 0 1 15 3.5v9a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 1 12.5v-9zM2.5 3a.5.5 0 0 0-.5.5v9a.5.5 0 0 0 .5.5H10V3H2.5zm8.5 10h2.5a.5.5 0 0 0 .5-.5v-9a.5.5 0 0 0-.5-.5H11v10z"/>
          </svg>
        </button>
      </template>
    </TitleBar>

    <!-- Main Three-Pane Studio Layout -->
    <div class="studio-layout">
      <!-- Left Sidebar (Collapsible) -->
      <Sidebar
        v-if="sidebarOpen"
        :folders="folders"
        :counts="libraryCounts"
        :albums="albums"
        :album-counts="albumCounts"
        :tags="tags"
        :active-target="activeTarget"
        :progress="progress"
        @folder-added="onFolderAdded"
        @removed="onFolderRemoved"
        @scanned="onFolderScanned"
        @select-nav="onSelectNav"
        @open-album-modal="() => onOpenAlbumModal()"
        @open-tag-modal="() => onOpenTagModal()"
        @open-prompt-stats="promptStatsModalOpen = true"
        @open-model-manager="modelManagerModalOpen = true"
        @open-db-manager="dbManagerModalOpen = true"
        @open-shortcuts-help="shortcutsHelpModalOpen = true"
        @open-add-folder-modal="addFolderModalOpen = true"
        @move-files-to-folder="onDropMoveFiles"
        @add-files-to-album="onDropAddFilesToAlbum"
        @tag-files="onDropTagFiles"
      />

      <!-- Center Gallery Canvas (Eagle Grid/Waterfall) -->
      <main class="gallery-canvas">
        <!-- Top Toolbar -->
        <div class="gallery-topbar">
          <!-- Breadcrumbs / Title -->
          <div class="topbar-left">
            <h2 class="target-title">
              {{ targetTitle }}
              <span class="items-count-badge">({{ files.length }})</span>
            </h2>
          </div>

          <!-- Search Bar & Filter Chips -->
          <div class="topbar-center">
            <SearchBar
              v-model="searchQuery"
              v-model:is-semantic="isSemanticSearch"
              :loading="filesLoading"
              :result-count="searchQuery.trim() ? files.length : null"
              @search="onSearch"
              @clear="onClearSearch"
              @open-clip-manager="clipModalOpen = true"
            />
            <button
              type="button"
              class="filter-btn"
              :class="{ active: filterDrawerOpen || activeFilterCount > 0 }"
              :title="t.search.filters"
              @click="filterDrawerOpen = true"
            >
              <span class="filter-icon">🔍</span>
              <span class="filter-label">{{ t.search.filters }}</span>
              <span v-if="activeFilterCount > 0" class="filter-count-badge">
                {{ activeFilterCount }}
              </span>
            </button>
          </div>

          <!-- Sort, Zoom, and View Mode Actions -->
          <div class="topbar-right">
            <!-- Sort Bar -->
            <SortBar
              v-model:sort-field="sortField"
              v-model:sort-direction="sortDirection"
              @change="loadFiles"
            />

            <!-- Zoom Slider (Eagle style slider for grid thumbnail size) -->
            <div v-if="viewMode === 'grid'" class="zoom-slider-wrapper" :title="t.preview.zoomGrid">
              <span class="zoom-icon small">▪</span>
              <input
                v-model.number="gridItemWidth"
                type="range"
                min="130"
                max="360"
                step="10"
                class="zoom-slider"
              />
              <span class="zoom-icon large">◼</span>
            </div>

            <!-- View Mode Switch -->
            <div class="view-mode-toggle">
              <button
                type="button"
                class="toggle-btn"
                :class="{ active: viewMode === 'grid' }"
                :title="t.view.grid"
                @click="setViewMode('grid')"
              >
                ⊞
              </button>
              <button
                type="button"
                class="toggle-btn"
                :class="{ active: viewMode === 'table' }"
                :title="t.view.table"
                @click="setViewMode('table')"
              >
                ☰
              </button>
            </div>
          </div>
        </div>

        <!-- Similarity Search Banner -->
        <div v-if="similaritySourceFile" class="similarity-banner">
          <div class="similarity-banner-left">
            <span class="similarity-badge">🔍 {{ t.preview.similaritySearchTitle }}</span>
            <span class="similarity-file-name" :title="getFileName(similaritySourceFile.path)">
              {{ getFileName(similaritySourceFile.path) }}
            </span>
            <span class="similarity-count">
              ({{ files.length }} / {{ rawSimilarityFiles.length }} {{ t.search.images }})
            </span>
          </div>
          <div class="similarity-banner-controls">
            <div class="similarity-control-group">
              <label for="similarity-threshold-slider" class="similarity-control-label">
                {{ t.preview.similarityThreshold }}:
                <span class="similarity-threshold-val">≥ {{ similarityThreshold }}%</span>
              </label>
              <input
                id="similarity-threshold-slider"
                type="range"
                min="0"
                max="95"
                step="5"
                v-model.number="similarityThreshold"
                class="similarity-slider"
                @input="onSimilarityThresholdChange"
              />
            </div>
            <div class="similarity-control-group">
              <label for="similarity-limit-select" class="similarity-control-label">
                {{ t.preview.similarityLimit }}:
              </label>
              <select
                id="similarity-limit-select"
                v-model.number="similarityLimit"
                class="similarity-limit-select"
                @change="onSimilarityLimitChange"
              >
                <option v-for="l in [20, 50, 100, 200]" :key="l" :value="l">
                  {{ l }}
                </option>
              </select>
            </div>
          </div>
          <button
            type="button"
            class="similarity-exit-btn"
            :title="t.preview.similaritySearchExit"
            @click="exitSimilaritySearch"
          >
            ✕ {{ t.preview.similaritySearchExit }}
          </button>
        </div>

        <!-- Main Viewport: Grid or Table -->
        <div class="gallery-viewport">
          <VirtualGrid
            v-if="viewMode === 'grid'"
            :files="files"
            :selected-file="selectedFile"
            :selected-file-paths="selectedFilePaths"
            :loading="filesLoading"
            :item-min-width="gridItemWidth"
            :blur-nsfw="blurNsfw"
            :show-card-badges="showCardBadges"
            :stack-map="stackMap"
            :expanded-stacks="expandedStacks"
            @select="onFileSelected"
            @activate="onActivateFile"
            @toggle-select="toggleSelectFile"
            @find-similar="handleFindSimilar"
            @toggle-stack-expand="onToggleStackExpand"
            @compare-stack="onTriggerCompare"
          />

          <FileList
            v-else
            :files="files"
            :selected-file="selectedFile"
            :selected-file-paths="selectedFilePaths"
            :loading="filesLoading"
            @select="onFileSelected"
            @activate="onActivateFile"
            @toggle-select="toggleSelectFile"
            @toggle-all="onToggleAll"
          />

          <!-- Floating Batch Action Bar -->
          <BatchActionBar
            :selected-count="selectedFilesList.length"
            :total-count="files.length"
            :selected-files="selectedFilesList"
            @clear-selection="onClearSelection"
            @select-all="onSelectAll"
            @set-rating="onBatchRate"
            @add-to-album="onBatchAddToAlbum"
            @add-tag="onBatchTag"
            @auto-tag-selected="handleOpenAutoTag()"
            @toggle-favorite="onBatchToggleFavorite"
            @toggle-nsfw="onBatchToggleNsfw"
            @move="onBatchMove"
            @copy="onBatchCopy"
            @trash="onBatchTrash"
          />
        </div>
      </main>

      <!-- Right Inspector Panel (Collapsible) -->
      <InspectorPane
        v-if="inspectorOpen"
        ref="inspectorRef"
        :file="selectedFile"
        :selected-count="selectedFilesList.length"
        @close="inspectorOpen = false"
        @open-lightbox="onActivateFile"
        @open-tag-modal="onOpenTagModal([$event])"
        @open-album-modal="onOpenAlbumModal([$event])"
        @open-auto-tag-modal="handleOpenAutoTag"
        @update-file="onUpdateFile"
        @filter-by-model="onFilterByModel"
        @filter-by-hash="onFilterByHash"
        @find-similar="handleFindSimilar"
        @open-lora-manager="loraModalOpen = true"
        @register-lora="() => { loraModalOpen = true; }"
      />
    </div>

    <!-- Bottom Status Bar -->
    <StatusBar
      :total-count="libraryCounts?.total ?? files.length"
      :filtered-count="files.length"
      :selected-count="selectedFilesList.length"
      :info="info"
      :progress="progress"
      :thumb-progress="thumbProgress"
      :has-filter="!!searchQuery.trim() || activeFilterCount > 0"
    />

    <!-- Fullscreen Lightbox Modal (Eagle Quick Look) -->
    <LightboxModal
      v-if="lightboxFile"
      :file="lightboxFile"
      :files="files"
      @close="lightboxFile = null"
      @navigate="onLightboxNavigate"
      @update-file="onUpdateFile"
      @find-similar="handleFindSimilar"
    />

    <!-- Modals & Drawers -->
    <FilterDrawer
      :open="filterDrawerOpen"
      :models="distinctModels"
      :samplers="distinctSamplers"
      :initial-criteria="activeCriteria"
      @close="filterDrawerOpen = false"
      @apply="onApplyFilters"
      @reset="onResetFilters"
    />

    <PromptStatsModal
      :open="promptStatsModalOpen"
      @close="promptStatsModalOpen = false"
      @apply-search="onApplyStatsSearch"
    />

    <ModelManagerModal
      :show="modelManagerModalOpen"
      @close="modelManagerModalOpen = false"
      @filter-model="onFilterByModel"
      @filter-hash="onFilterByHash"
    />

    <DatabaseManagerModal
      :show="dbManagerModalOpen"
      @close="dbManagerModalOpen = false"
      @database-changed="onDatabaseChanged"
    />

    <ShortcutsHelpModal
      :show="shortcutsHelpModalOpen"
      @close="shortcutsHelpModalOpen = false"
    />

    <FileOperationModal
      :open="fileOpModalOpen"
      :mode="fileOpMode"
      :files="fileOpTargetFiles"
      :folders="folders"
      @close="fileOpModalOpen = false"
      @completed="onFileOpCompleted"
    />

    <AlbumModal
      :open="albumModalOpen"
      :file-ids="albumTargetFileIds"
      @close="albumModalOpen = false"
      @created="loadAlbumsAndTags"
      @updated="loadAlbumsAndTags"
      @deleted="loadAlbumsAndTags"
      @added-to-album="onAddedToAlbum"
    />

    <TagModal
      :open="tagModalOpen"
      :file-ids="tagTargetFileIds"
      @close="tagModalOpen = false"
      @created="loadAlbumsAndTags"
      @updated="loadAlbumsAndTags"
      @deleted="loadAlbumsAndTags"
      @tagged="loadAlbumsAndTags"
    />

    <!-- WD14 AI Auto-Tagger Modal -->
    <AutoTagModal
      :show="autoTagModalOpen"
      :selected-file="autoTagTargetFile"
      :selected-file-count="selectedFilesList.length"
      :selected-file-ids="selectedFilesList.map((f) => f.id).filter((id): id is number => typeof id === 'number')"
      @close="autoTagModalOpen = false"
      @tags-applied="onAutoTagsApplied"
    />

    <!-- CLIP / SigLIP AI Semantic Search Manager Modal -->
    <ClipManagerModal
      :show="clipModalOpen"
      @close="clipModalOpen = false"
      @indexed="loadFiles"
    />

    <!-- LoRA Trigger Words Manager Modal -->
    <LoraManagerModal
      :show="loraModalOpen"
      @close="loraModalOpen = false"
      @inject-prompt="onInjectPrompt"
    />

    <!-- Settings Modal -->
    <SettingsModal
      :show="settingsModalOpen"
      :info="info"
      @close="settingsModalOpen = false"
      @save="onSettingsSaved"
    />

    <!-- Update Modal -->
    <UpdateModal
      :show="updateModalOpen"
      :current-version="info?.app_version || '0.1.1'"
      @close="updateModalOpen = false"
    />

    <!-- Multi-Mode Add Folder Modal -->
    <AddFolderModal
      v-if="addFolderModalOpen"
      :open="addFolderModalOpen"
      @update:open="addFolderModalOpen = $event"
      @folder-added="onFolderAdded"
    />

    <!-- Onboarding Setup Wizard Modal -->
    <OnboardingModal
      v-if="onboardingModalOpen"
      :open="onboardingModalOpen"
      @update:open="!$event && onOnboardingComplete()"
      @complete="onOnboardingComplete"
    />

    <!-- Side-by-Side Compare Modal -->
    <CompareModal
      v-if="compareModalOpen"
      :open="compareModalOpen"
      :images="compareImages"
      @update:open="compareModalOpen = $event"
      @set-hero="onCompareSetHero"
    />
  </div>
</template>

<style scoped>
.app-window-eagle {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #18181c;
  color: #f1f5f9;
  overflow: hidden;
}

.titlebar-quick-btn {
  background: transparent;
  border: none;
  color: #71717a;
  width: 32px;
  height: 28px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s;
}

.titlebar-quick-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.titlebar-quick-btn.active {
  color: #a855f7;
  background: rgba(168, 85, 247, 0.12);
}

.studio-layout {
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
  overflow: hidden;
}

.gallery-canvas {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: #18181c;
  position: relative;
}

.gallery-topbar {
  height: 42px;
  min-height: 42px;
  padding: 0 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  background: #18181c;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 10;
  overflow: hidden;
}

.topbar-left {
  display: flex;
  align-items: center;
  max-width: 160px;
  min-width: 0;
  flex-shrink: 0;
  overflow: hidden;
}

.target-title {
  margin: 0;
  font-size: 0.86rem;
  font-weight: 700;
  color: #f8fafc;
  display: flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.items-count-badge {
  font-size: 0.72rem;
  font-weight: 500;
  color: #71717a;
  flex-shrink: 0;
}

.topbar-center {
  flex: 1;
  min-width: 100px;
  max-width: 460px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.filter-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: #a1a1aa;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 0.74rem;
  display: flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: all 0.12s;
  height: 30px;
}

.filter-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.filter-btn.active {
  background: rgba(168, 85, 247, 0.16);
  border-color: rgba(168, 85, 247, 0.35);
  color: #d8b4fe;
}

.filter-count-badge {
  font-size: 0.64rem;
  padding: 1px 5px;
  border-radius: 999px;
  background: #a855f7;
  color: #ffffff;
  font-weight: 600;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.zoom-slider-wrapper {
  display: flex;
  align-items: center;
  gap: 5px;
  background: rgba(255, 255, 255, 0.03);
  padding: 3px 6px;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  height: 28px;
  flex-shrink: 0;
}

.zoom-icon {
  color: #71717a;
  font-size: 0.65rem;
}

.zoom-icon.large {
  font-size: 0.85rem;
}

.zoom-slider {
  width: 55px;
  height: 3px;
  accent-color: #a855f7;
  cursor: pointer;
}

.view-mode-toggle {
  display: flex;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 5px;
  overflow: hidden;
  height: 28px;
  flex-shrink: 0;
}

.toggle-btn {
  background: transparent;
  border: none;
  color: #71717a;
  padding: 0 7px;
  font-size: 0.78rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s;
}

.toggle-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #ffffff;
}

.toggle-btn.active {
  background: rgba(168, 85, 247, 0.2);
  color: #f3e8ff;
  font-weight: 600;
}

.gallery-viewport {
  flex: 1;
  position: relative;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Responsive Adaptive Breakpoints */
@media (max-width: 1100px) {
  .zoom-slider-wrapper {
    display: none;
  }
}

@media (max-width: 900px) {
  .filter-label {
    display: none;
  }
  .topbar-left {
    max-width: 100px;
  }
}

/* Similarity Search Banner */
.similarity-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 14px;
  background: linear-gradient(90deg, rgba(99, 102, 241, 0.15), rgba(139, 92, 246, 0.15));
  border-bottom: 1px solid rgba(99, 102, 241, 0.3);
  gap: 12px;
  animation: fadeIn 0.2s ease;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.similarity-banner-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
}

.similarity-banner-controls {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
}

.similarity-control-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.similarity-control-label {
  font-size: 0.74rem;
  color: #c7d2fe;
  white-space: nowrap;
  user-select: none;
  display: flex;
  align-items: center;
  gap: 4px;
}

.similarity-threshold-val {
  font-weight: 700;
  color: #38bdf8;
  min-width: 40px;
}

.similarity-slider {
  accent-color: #6366f1;
  width: 90px;
  height: 4px;
  cursor: pointer;
}

.similarity-limit-select {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(99, 102, 241, 0.4);
  color: #e2e8f0;
  border-radius: 4px;
  font-size: 0.72rem;
  padding: 2px 6px;
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s ease;
}

.similarity-limit-select:focus {
  border-color: #6366f1;
}

.similarity-badge {
  font-size: 0.78rem;
  font-weight: 700;
  color: #a5b4fc;
  white-space: nowrap;
}

.similarity-file-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}

.similarity-count {
  font-size: 0.74rem;
  color: #c7d2fe;
  white-space: nowrap;
}

.similarity-exit-btn {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #f1f5f9;
  border-radius: 5px;
  padding: 3px 8px;
  font-size: 0.72rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.similarity-exit-btn:hover {
  background: rgba(239, 68, 68, 0.85);
  border-color: rgba(239, 68, 68, 0.9);
  color: #fff;
}
</style>
