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

export interface CullStackResult {
  hero: ImageFile;
  drafts: ImageFile[];
}

/**
 * Identify draft images to cull within a stack.
 *
 * Rules:
 * - Exactly one hero is preserved: highest rating, then smallest stack_order, then smallest id.
 * - Drafts are all other members whose rating is lower than the hero (or all other members if all unrated).
 */
export function identifyStackDrafts(
  files: ImageFile[],
  stackId: string,
  minKeepRating: number = 0,
): CullStackResult | null {
  const members = files.filter((f) => f.stack_id === stackId);
  if (members.length <= 1) return null;

  let hero = members[0];
  for (const member of members.slice(1)) {
    const memberRating = member.rating ?? 0;
    const heroRating = hero.rating ?? 0;
    if (memberRating > heroRating) {
      hero = member;
    } else if (memberRating === heroRating) {
      const memberOrder = member.stack_order ?? Number.MAX_SAFE_INTEGER;
      const heroOrder = hero.stack_order ?? Number.MAX_SAFE_INTEGER;
      if (memberOrder < heroOrder) {
        hero = member;
      } else if (memberOrder === heroOrder && (member.id ?? 0) < (hero.id ?? 0)) {
        hero = member;
      }
    }
  }

  const drafts = members.filter((m) => {
    if (m.id === hero.id || m.path === hero.path) return false;
    const rating = m.rating ?? 0;
    const heroRating = hero.rating ?? 0;
    if (minKeepRating > 0) {
      return rating < minKeepRating;
    }
    return heroRating > 0 ? rating < heroRating : true;
  });

  return { hero, drafts };
}

/**
 * Identify all draft images to cull from a list of selected files or stacks.
 */
export function identifyMultiStackDrafts(
  files: ImageFile[],
  selectedFilePaths?: Set<string>,
): { heroes: ImageFile[]; drafts: ImageFile[] } {
  const targetFiles =
    selectedFilePaths && selectedFilePaths.size > 0
      ? files.filter((f) => selectedFilePaths.has(f.path))
      : files;

  const stackIds = new Set<string>();
  for (const file of targetFiles) {
    if (file.stack_id) stackIds.add(file.stack_id);
  }

  const allHeroes: ImageFile[] = [];
  const allDrafts: ImageFile[] = [];

  for (const stackId of stackIds) {
    const result = identifyStackDrafts(files, stackId);
    if (result && result.drafts.length > 0) {
      allHeroes.push(result.hero);
      allDrafts.push(...result.drafts);
    }
  }

  return { heroes: allHeroes, drafts: allDrafts };
}
