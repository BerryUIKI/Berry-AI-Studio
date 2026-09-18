export const THUMBNAIL_SIZE_TIERS = [128, 192, 256, 320, 384, 448, 512, 768, 1024] as const;

/**
 * Select the smallest cache tier that covers the rendered image size.
 * The configured resolution remains a hard upper bound, while device scale
 * is capped to avoid disproportionate decode and disk costs on dense screens.
 */
export function selectThumbnailTier(
  displayEdge: number,
  configuredMaxEdge: number,
  deviceScale = 1,
): number {
  const upperBound = Math.max(128, Math.round(configuredMaxEdge));
  const renderedPixels = Math.max(
    1,
    Math.ceil(displayEdge * Math.min(2, Math.max(1, deviceScale))),
  );
  const requestedEdge = Math.min(renderedPixels, upperBound);
  const tiers = [...THUMBNAIL_SIZE_TIERS, upperBound]
    .filter((tier) => tier <= upperBound)
    .sort((left, right) => left - right);

  return tiers.find((tier) => tier >= requestedEdge) ?? upperBound;
}
