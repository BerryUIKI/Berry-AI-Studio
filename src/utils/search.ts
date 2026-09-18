import type { SearchCriteria } from "../types";

/**
 * Convert a structured SearchCriteria object into a search query string.
 */
export function criteriaToQuery(c: SearchCriteria): string {
  const parts: string[] = [];

  if (c.text?.trim()) {
    parts.push(c.text.trim());
  }
  if (c.prompt?.trim()) {
    const val = c.prompt.trim();
    parts.push(val.includes(" ") ? `prompt:"${val}"` : `prompt:${val}`);
  }
  if (c.negative_prompt?.trim()) {
    const val = c.negative_prompt.trim();
    parts.push(val.includes(" ") ? `neg:"${val}"` : `neg:${val}`);
  }
  if (c.model_name?.trim()) {
    const val = c.model_name.trim();
    parts.push(val.includes(" ") ? `model:"${val}"` : `model:${val}`);
  }
  if (c.model_hash?.trim()) {
    const val = c.model_hash.trim();
    parts.push(val.includes(" ") ? `hash:"${val}"` : `hash:${val}`);
  }
  if (c.sampler?.trim()) {
    const val = c.sampler.trim();
    parts.push(val.includes(" ") ? `sampler:"${val}"` : `sampler:${val}`);
  }

  // Steps
  if (c.min_steps != null && c.max_steps != null) {
    if (c.min_steps === c.max_steps) {
      parts.push(`steps:${c.min_steps}`);
    } else {
      parts.push(`steps:${c.min_steps}..${c.max_steps}`);
    }
  } else if (c.min_steps != null) {
    parts.push(`steps:>=${c.min_steps}`);
  } else if (c.max_steps != null) {
    parts.push(`steps:<=${c.max_steps}`);
  }

  // CFG
  if (c.min_cfg != null && c.max_cfg != null) {
    if (c.min_cfg === c.max_cfg) {
      parts.push(`cfg:${c.min_cfg}`);
    } else {
      parts.push(`cfg:${c.min_cfg}..${c.max_cfg}`);
    }
  } else if (c.min_cfg != null) {
    parts.push(`cfg:>=${c.min_cfg}`);
  } else if (c.max_cfg != null) {
    parts.push(`cfg:<=${c.max_cfg}`);
  }

  // Rating
  if (c.min_rating != null && c.max_rating != null) {
    if (c.min_rating === c.max_rating) {
      parts.push(`rating:${c.min_rating}`);
    } else {
      parts.push(`rating:${c.min_rating}..${c.max_rating}`);
    }
  } else if (c.min_rating != null) {
    parts.push(`rating:>=${c.min_rating}`);
  } else if (c.max_rating != null) {
    parts.push(`rating:<=${c.max_rating}`);
  }

  // Aesthetic
  if (c.min_aesthetic != null && c.max_aesthetic != null) {
    if (c.min_aesthetic === c.max_aesthetic) {
      parts.push(`aesthetic:${c.min_aesthetic}`);
    } else {
      parts.push(`aesthetic:${c.min_aesthetic}..${c.max_aesthetic}`);
    }
  } else if (c.min_aesthetic != null) {
    parts.push(`aesthetic:>=${c.min_aesthetic}`);
  } else if (c.max_aesthetic != null) {
    parts.push(`aesthetic:<=${c.max_aesthetic}`);
  }

  // Favorite
  if (c.is_favorite === true) {
    parts.push("fav:true");
  } else if (c.is_favorite === false) {
    parts.push("fav:false");
  }

  // NSFW
  if (c.is_nsfw === true) {
    parts.push("is:nsfw");
  } else if (c.is_nsfw === false) {
    parts.push("is:sfw");
  }

  return parts.join(" ");
}

/**
 * Count non-empty filter criteria (excluding general text, sorting, and pagination).
 */
export function countActiveFilters(c: SearchCriteria): number {
  let count = 0;
  if (c.prompt?.trim()) count++;
  if (c.negative_prompt?.trim()) count++;
  if (c.model_name?.trim()) count++;
  if (c.sampler?.trim()) count++;
  if (c.min_steps != null || c.max_steps != null) count++;
  if (c.min_cfg != null || c.max_cfg != null) count++;
  if (c.min_rating != null || c.max_rating != null) count++;
  if (c.min_aesthetic != null || c.max_aesthetic != null) count++;
  if (c.is_favorite != null) count++;
  if (c.is_nsfw != null) count++;
  return count;
}
