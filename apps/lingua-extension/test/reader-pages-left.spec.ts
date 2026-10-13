import { describe, expect, it } from "vitest";
import { pagesLeftInBook, pagesLeftInSection } from "@/reader/pages-left.ts";

// The pages left (add-lingua-reader-progress-toggle D2, D3): counted on the laid-out section in the
// reader's screens; the book's estimated from the section on screen.

/** A paginated section of `text` text pages, on its `shown`th (from 1): foliate's count adds two blank pages. */
const paginated = (shown: number, text: number) => ({ scrolled: false, page: shown, pages: text + 2 });

describe("the pages left in the section on screen", () => {
  it("counts the text pages after the one shown, in the paginated flow", () => {
    expect(pagesLeftInSection(paginated(6, 10))).toBe(4);
    expect(pagesLeftInSection(paginated(1, 10))).toBe(9);
  });

  it("says nothing is left on the last page", () => {
    expect(pagesLeftInSection(paginated(10, 10))).toBe(0);
    // foliate's trailing blank page counts as the last.
    expect(pagesLeftInSection(paginated(11, 10))).toBe(0);
    expect(pagesLeftInSection(paginated(0, 10))).toBe(9);
  });

  it("counts the screens below the one shown, in the scrolled flow", () => {
    expect(pagesLeftInSection({ scrolled: true, page: 0, pages: 5 })).toBe(4);
    expect(pagesLeftInSection({ scrolled: true, page: 4, pages: 5 })).toBe(0);
  });

  it("counts nothing in a section with no page", () => {
    expect(pagesLeftInSection({ scrolled: false, page: 0, pages: 2 })).toBeNull();
    expect(pagesLeftInSection({ scrolled: true, page: 0, pages: 0 })).toBeNull();
    expect(pagesLeftInSection({ scrolled: false, page: NaN, pages: 12 })).toBeNull();
  });
});

describe("the pages left in the book, estimated", () => {
  it("adds the sections after this one at this one's bytes per page", () => {
    // 10 pages of 1000 bytes on screen, 4 left; then 5000 + 3000 bytes: 8 more pages.
    expect(pagesLeftInBook([2000, 10_000, 5000, 3000], 1, paginated(6, 10))).toBe(12);
  });

  it("leaves out the sections outside the reading order", () => {
    expect(pagesLeftInBook([10_000, 0, 5000, 0], 0, paginated(10, 10))).toBe(5);
  });

  it("is what is left of the last section, in the last one", () => {
    expect(pagesLeftInBook([10_000, 4000], 1, { scrolled: true, page: 1, pages: 4 })).toBe(2);
  });

  it("gives no estimate for a section it cannot measure", () => {
    expect(pagesLeftInBook([0, 5000], 0, paginated(1, 3))).toBeNull();
    expect(pagesLeftInBook([5000, 5000], 0, { scrolled: false, page: 0, pages: 2 })).toBeNull();
    expect(pagesLeftInBook([5000], 3, paginated(1, 3))).toBeNull();
  });
});
