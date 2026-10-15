// The statistics the captures show (add-site-lingua-showcase D4): a backup holds no daily counts,
// so the tool writes the thirty days ending on the run's — the statistics page's default range — under
// `cymbra-lingua-daily-v4`, through the store's own message, and the page shows a believable month
// whatever day the owner runs it. Pure: the same day gives the same records. A unit test reads them
// back with src/state/dailystats.ts's own reader.

/** The key the counts live under (src/state/dailystats.ts, `DAILY_KEY`). */
export const DAILY_KEY = "cymbra-lingua-daily-v4";

/** How many days are written, the run's own the last: the statistics page opens on thirty (src/stats/view.ts). */
export const DAYS = 30;

const DAY_MS = 86_400_000;

// A reader of the first studied language most days, of the second now and then. Thirty entries
// each, oldest first.
const FIRST = {
  exposures: [
    520, 0, 760, 610, 0, 880, 940, 430, 0, 700, 1050, 0, 820, 590, 640, 0, 910, 520, 780, 1240, 0, 860, 1020, 450, 0,
    1180, 930, 1310, 690, 1120,
  ],
  wordsLearned: [5, 0, 7, 5, 0, 8, 9, 3, 0, 6, 10, 0, 7, 5, 6, 0, 9, 4, 7, 11, 0, 8, 10, 3, 0, 9, 8, 12, 6, 10],
  reviews: [
    10, 0, 10, 10, 0, 14, 10, 8, 0, 10, 20, 0, 10, 10, 10, 0, 10, 8, 10, 20, 0, 10, 14, 10, 0, 20, 10, 20, 10, 14,
  ],
};
const SECOND = {
  exposures: [
    0, 280, 0, 0, 330, 0, 0, 0, 360, 0, 0, 240, 0, 0, 0, 310, 0, 0, 260, 0, 420, 0, 0, 380, 290, 0, 0, 340, 0, 0,
  ],
  wordsLearned: [0, 2, 0, 0, 3, 0, 0, 0, 3, 0, 0, 2, 0, 0, 0, 3, 0, 0, 2, 0, 4, 0, 0, 3, 2, 0, 0, 3, 0, 0],
  reviews: [0, 5, 0, 0, 5, 0, 0, 0, 6, 0, 0, 5, 0, 0, 0, 5, 0, 0, 5, 0, 8, 0, 0, 5, 5, 0, 0, 6, 0, 0],
};

/**
 * The thirty UTC days ending on `nowMs`'s, oldest first, each with the counts of the reader's two
 * studied languages, labelled with their native language: the value to store under `DAILY_KEY`.
 * `unknownSeen` is a fifth of the words read, as a B1 reader meets them.
 * @param {{ nowMs: number, languages: readonly [string, string], native: string }} options
 */
export function recentDays({ nowMs, languages, native }) {
  const today = Math.floor(nowMs / DAY_MS);
  const records = {};
  for (let i = 0; i < DAYS; i++) {
    const day = today - (DAYS - 1) + i;
    const byLanguage = {};
    for (const [language, counts] of [
      [languages[0], FIRST],
      [languages[1], SECOND],
    ]) {
      const exposures = counts.exposures[i];
      if (exposures === 0 && counts.reviews[i] === 0) continue;
      byLanguage[language] = {
        exposures,
        unknownSeen: Math.round(exposures / 5),
        wordsLearned: counts.wordsLearned[i],
        reviews: counts.reviews[i],
        native,
      };
    }
    if (Object.keys(byLanguage).length > 0) records[day] = byLanguage;
  }
  return records;
}
