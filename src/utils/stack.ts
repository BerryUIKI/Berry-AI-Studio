import type { ImageFile } from "../types";

export type StackDisplayMap = Record<string, { count: number; heroId: number | null }>;

function heroRank(file: ImageFile, heroId: number | null | undefined): number {
  if (heroId != null && file.id === heroId) return 0;
  if (file.stack_order === 0) return 1;
  return 2;
}

/** Resolve exactly one display hero per stack, even when legacy data has duplicate orders. */
export function resolveStackHeroPaths(
  files: ImageFile[],
  stackMap: StackDisplayMap,
): Map<string, string> {
  const heroes = new Map<string, ImageFile>();
  for (const file of files) {
    const stackId = file.stack_id;
    if (!stackId || (stackMap[stackId]?.count ?? 1) <= 1) continue;
    const current = heroes.get(stackId);
    const heroId = stackMap[stackId]?.heroId;
    if (!current || heroRank(file, heroId) < heroRank(current, heroId)) {
      heroes.set(stackId, file);
    }
  }
  return new Map([...heroes].map(([stackId, file]) => [stackId, file.path]));
}

export function collapseStackMembers(
  files: ImageFile[],
  stackMap: StackDisplayMap,
  stackId?: string,
): ImageFile[] {
  const heroPaths = resolveStackHeroPaths(files, stackMap);
  return files.filter((file) => {
    if (!file.stack_id || (stackId && file.stack_id !== stackId)) return true;
    const info = stackMap[file.stack_id];
    if (!info || info.count <= 1) return true;
    return file.path === heroPaths.get(file.stack_id);
  });
}
