# localise-lingua-review-stats — review and statistics read their copy from the catalogue

## Why

Change 16 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, the third of the four that move the interface's copy into the
catalogue of change 13. Review (`review/review-page.ts`, `review/view.ts`) and statistics
(`stats/stats.html`, `stats/view.ts`, `stats/ladder.ts`) hold ≈ 60 unique texts: « Réviser »,
« Afficher la réponse », today's four grades (`GRADES`, which #696 replaces), « Je connais ✓ »,
« Rien à réviser pour l'instant. », « ${n} carte(s) à revoir », « Mots lus », « Mots appris »,
« Révisions », « Vocabulaire connu », the ladder's notes with their ` ` escapes and its
`toLocaleString("fr-FR")` numbers, « Niveaux CEFR indisponibles pour cette langue… », « Aucune
carte ajoutée — … », « ${added} carte${added > 1 ? "s" : ""} ajoutée… » — the hand-rolled
plural (#696 writes a second one).

`refine-lingua-review-session` (#696) rewrites the review's copy (« Pas su », « Su », « Encore
10 », « Ne plus me le montrer », the session's end) and names its answers by meaning in its
requirements: whichever of the two merges second rebases on the other (change 15's D5) — this
change extracts #696's strings if #696 is on main by then; otherwise #696, rebased, puts them in
the catalogue in three languages, which the lint forces once `review/view.ts` is off the baseline.

## What Changes

- **Review and statistics read their copy from `src/i18n/<language>/{review,stats}.ts`**, picked
  by the interface language their hosts hand them — the side panel reads it with its preferences,
  the drawer is handed it by the reading session (change 14), the statistics page reads it before
  it mounts and fills `stats.html` with change 14's `fillPage`, with `lang`.
- **Counts as plural forms**: « ${n} carte(s) à revoir » keeps its French string and its raw
  count, as today; the ladder's numbers keep `toLocaleString("fr-FR")` in French (« 12 345 »,
  today's bytes) and take change 13's `formatNumber` in English and Spanish; English and Spanish
  counts take `one`/`other` forms through change 13's `plural`.
- **The ladder's escapes** (` `, « … ») are the French entries' bytes.
- **The French byte for byte**: the `review-page`, `stats` and `stats-view` spec files pass
  unchanged (the ladder's notes are pinned by `stats-view`; the review's view is rendered by
  `review-page`'s); each surface gains one test in English.
- **The baseline** loses these files.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *Review and statistics speak the interface language*.

No requirement is modified; *Two review surfaces*, *Review present right next to the reading*,
*Two answers on the card* and the statistics' requirements (held or not) are read under the
umbrella rule.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (the files above, the catalogue
  modules, the baseline). ID, Music, Live, the back office and the site are untouched.
- **No byte moves.** The spec files are the check.
- **Order.** After change 13 and after change 14 (`fillPage`, the drawer's language); #696 and
  this change rebase on whichever merges first; independent of 15, 17.
- **Not here.** The languages' names in the review's filter and the statistics (19's seam); the
  level scale's acronym as an entry of its own, `levelScale` (19 — this change writes « CEFR » /
  "CEFR" / « MCER » inside the one statistics note that spells it, per M19, and 19 makes that note
  read `levelScale`).
