import type { ImageFile } from "../types";
import type { StackDisplayMap } from "./stack";

export class GalleryPages {
  private source: ImageFile[] | null = null;
  private ids = new Set<number | string>();
  private stacks = new Map<string, number>();

  append(
    files: ImageFile[],
    incoming: ImageFile[],
    summaries: StackDisplayMap,
    expanded: Set<string>,
  ): boolean {
    if (this.source !== files) {
      this.source = files;
      this.ids.clear();
      this.stacks.clear();
      files.forEach((file, index) => {
        this.ids.add(file.id ?? file.path);
        if (file.stack_id) this.stacks.set(file.stack_id, index);
      });
    }
    let replaced = false;
    for (const file of incoming) {
      const key = file.id ?? file.path;
      if (this.ids.has(key)) continue;
      this.ids.add(key);
      const stack = file.stack_id;
      if (stack && !expanded.has(stack) && (summaries[stack]?.count ?? 1) > 1) {
        const index = this.stacks.get(stack);
        if (index !== undefined) {
          if (file.id === summaries[stack]?.heroId) {
            files[index] = file;
            replaced = true;
          }
          continue;
        }
      }
      if (stack) this.stacks.set(stack, files.length);
      files.push(file);
    }
    return replaced;
  }
}

export interface GeometryItem {
  file: ImageFile;
  index: number;
  top: number;
  left: number;
  width: number;
  height: number;
  imageHeight: number;
  column: number;
}

export function visibleWaterfallItems(
  columns: GeometryItem[][],
  top: number,
  bottom: number,
): GeometryItem[] {
  const visible: GeometryItem[] = [];
  for (const column of columns) {
    let low = 0;
    let high = column.length;
    while (low < high) {
      const mid = (low + high) >>> 1;
      if (column[mid].top + column[mid].height < top) low = mid + 1;
      else high = mid;
    }
    for (let index = low; index < column.length; index++) {
      const item = column[index];
      if (item.top > bottom) break;
      visible.push(item);
    }
  }
  return visible.sort((a, b) => a.index - b.index);
}

/** Reuse geometry for in-place page appends; replacements/layout changes reset it. */
export class WaterfallGeometry {
  private source: ImageFile[] | null = null;
  private key = "";
  items: GeometryItem[] = [];
  columns: GeometryItem[][] = [];
  private heights: number[] = [];
  height = 0;

  update(
    files: ImageFile[],
    columns: number,
    width: number,
    gap: number,
    offset = 0,
  ): this {
    const key = `${columns}:${width}:${gap}:${offset}`;
    if (files !== this.source || key !== this.key || files.length < this.items.length) {
      this.source = files;
      this.key = key;
      this.items = [];
      this.columns = Array.from({ length: columns }, () => []);
      this.heights = Array(columns).fill(0);
    }
    for (let index = this.items.length; index < files.length; index++) {
      const file = files[index];
      const column = this.heights.indexOf(Math.min(...this.heights));
      const sourceWidth = file.metadata?.width ?? 1;
      const sourceHeight = file.metadata?.height ?? 1;
      const ratio = sourceWidth > 0 && sourceHeight > 0 ? sourceHeight / sourceWidth : 1;
      const imageHeight = Math.max(96, Math.round(width * ratio));
      const item: GeometryItem = {
        file,
        index,
        top: this.heights[column],
        left: offset + column * (width + gap),
        width,
        height: imageHeight + 56,
        imageHeight,
        column,
      };
      this.items.push(item);
      this.columns[column].push(item);
      this.heights[column] += item.height + gap;
    }
    this.height = Math.max(0, ...this.heights) - (files.length ? gap : 0);
    return this;
  }
}
