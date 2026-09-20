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
