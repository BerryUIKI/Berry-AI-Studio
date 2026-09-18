import type { ImageFile } from "../types";

export type StackDisplayMap = Record<string, { count: number; heroId: number | null }>;

/** Build summaries from an already bounded result set, such as semantic matches. */
export function summarizeResultStacks(files: ImageFile[]): StackDisplayMap {
  const groups = new Map<string, { count: number; hero: ImageFile }>();
  for (const file of files) {
    if (!file.stack_id) continue;
    const group = groups.get(file.stack_id);
    if (!group) {
      groups.set(file.stack_id, { count: 1, hero: file });
      continue;
    }
    group.count += 1;
    const fileOrder = file.stack_order ?? Number.MAX_SAFE_INTEGER;
    const heroOrder = group.hero.stack_order ?? Number.MAX_SAFE_INTEGER;
    if (
      fileOrder < heroOrder ||
      (fileOrder === heroOrder &&
        (file.id ?? Number.MAX_SAFE_INTEGER) < (group.hero.id ?? Number.MAX_SAFE_INTEGER))
    ) {
      group.hero = file;
    }
  }

  const summaries: StackDisplayMap = {};
  for (const [stackId, group] of groups) {
    if (group.count <= 1) continue;
    summaries[stackId] = { count: group.count, heroId: group.hero.id };
  }
  return summaries;
}

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
