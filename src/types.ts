// TypeScript mirrors of the serde types returned by the Tauri commands.
// Field names stay snake_case, matching Rust's serde defaults.

export interface AppInfo {
  app_version: string;
  schema_version: number;
  database_path: string;
}

export type FolderType = "link" | "managed" | "pipeline";
export type IngestAction = "copy" | "move";

export interface Folder {
  id: number;
  path: string;
  added_at: string;
  folder_type?: FolderType;
  source_path?: string | null;
  ingest_action?: IngestAction | null;
  grace_period_hours?: number | null;
  auto_harvest?: boolean;
}

export type Container = "png" | "jpg" | "webp" | "mp4" | "txt";

export interface ExtractedMetadata {
  format: string;
  parameters: string | null;
  raw: string | null;
  prompt: string | null;
  negative_prompt: string | null;
  width: number | null;
  height: number | null;
  seed: string | null;
  steps: number | null;
  cfg_scale: number | null;
  sampler: string | null;
  model_name: string | null;
  model_hash: string | null;
}

export type FileSortField =
  | "modified_at"
  | "path"
  | "size_bytes"
  | "rating"
  | "aesthetic_score";

export type SortDirection = "asc" | "desc";

export interface LibraryCounts {
  total: number;
  folders: Record<number, number>;
}

export interface ImageFile {
  id: number | null;
  folder_id: number;
  path: string;
  size_bytes: number;
  modified_at: number;
  container: Container;
  metadata: ExtractedMetadata | null;
  rating?: number | null;
  aesthetic_score?: number | null;
  is_favorite?: boolean;
  is_nsfw?: boolean;
  similarity_score?: number | null;
  stack_id?: string | null;
  stack_order?: number;
}

export interface FilePage {
  items: ImageFile[];
  total: number;
  offset: number;
  has_more: boolean;
}

export interface PipelineDetectedPath {
  tool_name: string;
  path: string;
  category: string;
}

export interface CleanupQueueItem {
  id: number;
  source_file_path: string;
  target_image_id: number;
  scheduled_delete_at: number;
  created_at: number;
  status: "pending" | "deleted" | "cancelled" | "failed";
}

export interface StackSummary {
  stack_id: string;
  count: number;
  hero_image_id: number | null;
}

export interface AutoStackResult {
  created_stacks: number;
  stacked_images: number;
  eligible_images: number;
  skipped_without_prompt: number;
}

export interface SimilarityMatch {
  file_id: number;
  score: number;
}

export interface SimilarFileItem {
  file: ImageFile;
  score: number;
}

export interface Album {
  id: number;
  name: string;
  description?: string | null;
  created_at: string;
}

export interface Tag {
  id: number;
  name: string;
  color?: string | null;
  created_at: string;
}

export interface PromptStat {
  text: string;
  count: number;
}

export interface ScanProgress {
  folder_id: number;
  scanned: number;
  found: number;
  discovering: boolean;
  current: string | null;
}

export interface ScanStats {
  folder_id: number;
  found: number;
  added: number;
  updated: number;
  unchanged: number;
  removed: number;
  failed: number;
  duration_ms: number;
}

export interface LibraryFilesChanged {
  folder_id: number;
  stats: ScanStats;
}

export interface SearchCriteria {
  text?: string | null;
  prompt?: string | null;
  negative_prompt?: string | null;
  model_name?: string | null;
  model_hash?: string | null;
  sampler?: string | null;
  min_steps?: number | null;
  max_steps?: number | null;
  min_cfg?: number | null;
  max_cfg?: number | null;
  min_rating?: number | null;
  max_rating?: number | null;
  min_aesthetic?: number | null;
  max_aesthetic?: number | null;
  is_favorite?: boolean | null;
  is_nsfw?: boolean | null;
  album_id?: number | null;
  tag_id?: number | null;
  folder_id?: number | null;
  sort?: FileSortField | null;
  direction?: SortDirection | null;
  limit?: number | null;
  offset?: number | null;
}

export type NavTarget =
  | { type: "all" }
  | { type: "favorites" }
  | { type: "nsfw" }
  | { type: "folder"; folder: Folder }
  | { type: "album"; album: Album }
  | { type: "tag"; tag: Tag };

export interface PromptKeywordStat {
  keyword: string;
  count: number;
  avg_rating?: number | null;
}

export interface PromptStats {
  total_analyzed: number;
  top_positive_words: PromptKeywordStat[];
  top_negative_words: PromptKeywordStat[];
  top_models: PromptKeywordStat[];
  top_samplers: PromptKeywordStat[];
}

export interface CheckpointModelStat {
  model_name: string;
  model_hash?: string | null;
  count: number;
}

export interface ModelCacheEntry {
  hash: string;
  name: string;
  title?: string | null;
  sha256?: string | null;
}

export interface DatabaseStats {
  file_count: number;
  folder_count: number;
  album_count: number;
  tag_count: number;
  model_cache_count: number;
  db_size_bytes: number;
  page_size: number;
  page_count: number;
  freelist_count: number;
}

export interface TagPrediction {
  name: string;
  category: "General" | "Character" | "Rating" | string | number;
  confidence: number;
}

export interface TaggerConfig {
  general_threshold: number;
  character_threshold: number;
  include_rating: boolean;
  max_tags: number;
}

export interface TaggerModelSummary {
  name: string;
  dir_path: string;
  model_path: string;
  tags_path: string;
  is_loaded: boolean;
}

export interface BatchTagResult {
  processed_files: number;
  tags_added: number;
}

export interface ClipModelSummary {
  name: string;
  dir_path: string;
  visual_path: string;
  textual_path: string;
  tokenizer_path: string;
  is_loaded: boolean;
}

export interface ClipIndexStatus {
  model_id: string;
  indexed_images: number;
  total_images: number;
  is_loaded: boolean;
}

export interface ClipBatchIndexResult {
  indexed_count: number;
  remaining_count: number;
  total_count: number;
}

export interface ClipModelInfo {
  model_id: string;
  name: string;
  folder_path: string;
  visual_model_path: string;
  textual_model_path: string;
  tokenizer_path: string;
  image_size: number;
  embedding_dim: number;
}

export interface LoraModel {
  id: number;
  name: string;
  hash?: string | null;
  trigger_words: string[];
  preview_url?: string | null;
  description?: string | null;
  weight_default: number;
  created_at: string;
  updated_at: string;
}

export interface DetectedLora {
  name: string;
  weight: number;
  model?: LoraModel | null;
}
