// Committed pair lists for the Lingua page's unit tests (change:
// add-site-lingua-matrix-pages), so that a pair added to `src/data/lingua-coverage.json`
// (changes 34, 35) or a figure the pack update refreshes moves no test: the tests describe
// the page for these lists, and only `test/lingua-pairs.spec.ts`'s *real files* block reads
// the live data, for its structure. The byte pin's own list is
// `test/fixtures/lingua/taken-with.json`, refreshed with its fixtures.
import type { Coverage, Routes } from "../../src/lib/lingua-pairs";

/** Today's shipped pairs, en-fr and es-fr, with their figures when change 30 was written. */
export const TODAY: Coverage = {
  tops: [5000, 10000, 20000],
  glossed: {
    "en-fr": [95.1, 90.1, 78.9],
    "es-fr": [87.6, 77.2, 63.7],
  },
};

/**
 * The model catalogue's routes for every pair of the matrix, as committed when change 30
 * was written: en-fr, es-en and en-es direct, es-fr through English.
 */
export const ROUTES: Routes = {
  "en-fr": ["en-fr/base-memory/2.0"],
  "es-fr": ["es-en/base-memory/2.0", "en-fr/base-memory/2.0"],
  "es-en": ["es-en/base-memory/2.0"],
  "en-es": ["en-es/base-memory/2.1"],
};

/**
 * Today's pairs and the two changes 34 and 35 ship: es-en (Spanish for English speakers) and
 * en-es (English for Spanish speakers), with figures of their own.
 */
export const MATRIX: Coverage = {
  tops: TODAY.tops,
  glossed: { ...TODAY.glossed, "es-en": [90.0, 80.0, 70.0], "en-es": [96.0, 91.0, 80.0] },
};
