/** Calculate how many fixed-width cards fit inside the measured gallery width. */
export function calculateGalleryColumns(
  containerWidth: number,
  cardWidth: number,
  gap: number,
): number {
  const available = Number.isFinite(containerWidth) ? Math.max(0, containerWidth) : 0;
  const width = Number.isFinite(cardWidth) ? Math.max(1, cardWidth) : 1;
  const spacing = Number.isFinite(gap) ? Math.max(0, gap) : 0;
  return Math.max(1, Math.floor((available + spacing) / (width + spacing)));
}

/**
 * Calculate symmetrical horizontal offset to center fixed-width gallery columns,
 * eliminating asymmetric right-side empty gaps without stretching cards.
 */
export function calculateGalleryTrackOffset(
  containerWidth: number,
  cols: number,
  cardWidth: number,
  gap: number,
): number {
  const available = Number.isFinite(containerWidth) ? Math.max(0, containerWidth) : 0;
  const width = Number.isFinite(cardWidth) ? Math.max(1, cardWidth) : 1;
  const spacing = Number.isFinite(gap) ? Math.max(0, gap) : 0;
  const count = Math.max(1, Math.floor(cols));
  const trackWidth = count * width + (count - 1) * spacing;
  return Math.max(0, Math.floor((available - trackWidth) / 2));
}

