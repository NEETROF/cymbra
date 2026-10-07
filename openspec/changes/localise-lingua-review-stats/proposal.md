# localise-lingua-review-stats — review and statistics read their copy from the catalogue

## Why

Change 16 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, the third of the four that move the interface's copy into the
catalogue of change 13. Review (`review/review-page.ts`, `review/view.ts`) and statistics
(`stats/stats.html`, `stats/view.ts`, `stats/ladder.ts`) hold ≈ 60 unique texts: « Réviser »,
« Afficher la réponse », the four grades, « Je connais ✓ », « Rien à réviser pour l'instant. »,
« ${n} carte(s) à revoir », « Mots lus », « Mots appris », « Révisions », « Vocabulaire connu »,
the ladder's notes with their ` ` escapes, « Aucune carte ajoutée — … », « ${added}
carte${added > 1 ? "s" : ""} ajoutée… » — the one hand-rolled plural.

`refine-lingua-review-session` (#696) rewrites the review's copy (« Pas su », « Su », « Encore
10 », « Ne plus me le montrer », the session's end) and names its answers by meaning in its
requirements: the programme has it merge before this change, so its strings are extracted here.

## What Changes

- **Review and statistics read their copy from `src/i18n/<language>/{review,stats}.ts`**, picked
  by the interface language their hosts read; `stats.html` is filled at mount and carries `lang`.
- **Counts as plural forms**: « carte(s) » and the ladder's « ${n} mots » keep their French
  strings with the raw count; English and Spanish get `one`/`other` forms through
  `Intl.PluralRules` and the locale's numbers (« 1 234 » in French would move a byte, so French
  keeps the raw count — change 13 D2).
- **The ladder's escapes** (` `, « … ») are the French entries' bytes.
- **The French byte for byte**: the `review-page`, `view`, `stats`, `stats-view` and `ladder` spec
  files pass unchanged; each surface gains one test in English.
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
- **Order.** After change 13 and after #696 merges; independent of 14, 15, 17.
- **Not here.** The languages' names in the review's filter and the statistics (19's seam); the
  level scale's acronym (19).
