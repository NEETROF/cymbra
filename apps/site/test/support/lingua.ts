// Committed pair lists for the Lingua page's tests (change: add-site-lingua-matrix-pages),
// so that a pair added to `src/data/lingua-coverage.json` (changes 34, 35) or a figure the
// pack update refreshes moves no test: the unit tests describe the page for these lists,
// and only `test/lingua-pairs.spec.ts`'s *real files* block reads the live data, for its
// structure.
import type { Coverage, Routes } from "../../src/lib/lingua-pairs";
import takenWith from "../fixtures/lingua/taken-with.json";

/**
 * The shipped pairs `test/fixtures/lingua/main.{fr,en}.html` were taken with — en-fr and
 * es-fr, their figures of the day (see `apps/site/README.md` to refresh them).
 */
export const TODAY: Coverage = takenWith.coverage;

/** The routes those fixtures were taken with, en-fr's and es-fr's. */
export const TODAY_ROUTES: Routes = takenWith.routes;

/**
 * The model catalogue's routes for every pair of the matrix, as committed when change 30
 * was written: en-fr, es-en and en-es direct, es-fr through English.
 */
export const ROUTES: Routes = {
  ...TODAY_ROUTES,
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
