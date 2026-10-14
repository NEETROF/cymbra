import { cardsOf, countAt, type ReviewOutlook } from "../state/review-outlook.ts";

// The popup's deck and « Réviser (N) » on every tab (add-lingua-comeback-hints D8), decided here,
// pure. On a tab whose page answered, its figures: the page's language's deck and what a review
// would hold there today. Away from one, the language the review would open in away from a page —
// the last one chosen in the review on this device, else the first studied — counted from the
// outlook: the popup needs no engine. « Réviser » opens the side panel on Chromium; on Firefox and
// Safari the in-page drawer when the tab has the reader, and otherwise the review page in a tab.

/** What the popup's deck line and button show. */
export interface DeckFigure {
  deckCount: number;
  dueCount: number;
}

/** When and in what the outlook is counted. */
export interface AwayContext {
  /** The last language chosen in the review on this device, or null. */
  reviewLanguage: string | null;
  now: number;
  dayStart: number;
  perDay: number;
}

/** The figure for the tab: the page's when it answered, else the outlook's, else nothing. */
export function deckFigure(page: DeckFigure | null, outlook: ReviewOutlook | null, away: AwayContext): DeckFigure {
  if (page) return { deckCount: page.deckCount, dueCount: page.dueCount };
  if (!outlook || outlook.languages.length === 0) return { deckCount: 0, dueCount: 0 };
  const chosen = outlook.languages.find((entry) => entry.language === away.reviewLanguage);
  const language = (chosen ?? outlook.languages[0]!).language;
  return {
    deckCount: cardsOf(outlook, language),
    dueCount: countAt(outlook, [language], away.now, away.dayStart, away.perDay),
  };
}

/** Where « Réviser » (and « Statistiques ») open from the popup. */
export type ReviewDestination = "panel" | "drawer" | "tab";

/**
 * The side panel where the browser has one (Chromium); where the review lives in the page (Firefox,
 * Safari), the drawer on a tab with the reader, and the review page in a tab of its own elsewhere —
 * a browser page, a page the reader cannot run on.
 */
export function reviewDestination(reviewInPage: boolean, hasReader: boolean): ReviewDestination {
  if (!reviewInPage) return "panel";
  return hasReader ? "drawer" : "tab";
}
