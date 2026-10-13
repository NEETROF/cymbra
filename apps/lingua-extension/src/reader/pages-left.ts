// What is left of the section on screen and of the book, in screens of the reader's
// (add-lingua-reader-progress-toggle D2, D3): pure arithmetic over what foliate-js knows — the
// laid-out section's page shown and page count, and every section's size in bytes. The adapter
// (foliate.ts) reads those; this is what is tested.

/** Where the view is in the section on screen, as foliate's paginator says it. */
export interface SectionPages {
  /** The scrolled flow: screens, with no blank page around them. */
  scrolled: boolean;
  /** The page shown, by its index in the paginator's count. */
  page: number;
  /** The paginator's count: in the paginated flow, a blank page at each end around the text's. */
  pages: number;
}

/**
 * The pages after the one shown in the section on screen (D2): its text pages less the page shown
 * in the paginated flow, the screens below the one shown in the scrolled flow. Null when the
 * section holds no page to count.
 */
export function pagesLeftInSection({ scrolled, page, pages }: SectionPages): number | null {
  if (!Number.isFinite(page) || !Number.isFinite(pages)) return null;
  const text = scrolled ? pages : pages - 2;
  if (text <= 0) return null;
  // The page shown, from 0: the paginated flow's first text page follows its blank one.
  const shown = scrolled ? page : page - 1;
  return Math.min(text - 1, Math.max(0, text - 1 - shown));
}

/**
 * An estimate of the pages left in the book (D3): those left in the section on screen, then the
 * sections after it at the bytes per page of the one on screen. `sizes` holds each section's
 * bytes, 0 for one outside the reading order. Null when the section on screen cannot be measured.
 */
export function pagesLeftInBook(
  sizes: readonly number[],
  index: number,
  section: SectionPages,
  leftInSection: number | null = pagesLeftInSection(section),
): number | null {
  const size = sizes[index] ?? 0;
  const text = section.scrolled ? section.pages : section.pages - 2;
  if (leftInSection === null || size <= 0 || text <= 0) return null;
  const rest = sizes.slice(index + 1).reduce((sum, bytes) => sum + Math.max(0, bytes), 0);
  return leftInSection + Math.round(rest / (size / text));
}
