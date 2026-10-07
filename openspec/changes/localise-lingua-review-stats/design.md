# Design — localise-lingua-review-stats

## Context

See proposal.md (Why) and change 13's design. Today:

| Module | Copy |
|---|---|
| `review/view.ts` | « Réviser », « Rien à réviser pour l'instant. », « ${n} carte(s) à revoir », « Afficher la réponse », the grades (`GRADES`, which #696 replaces by its answers), « Je connais ✓ », the source label |
| `review/review-page.ts` | « Sources & confidentialité », « Sauvegarde restaurée. », `deck + " carte(s) · " + due + " à revoir"`, « Sources : ${…} », the language filter's names (change 19's seam) |
| `stats/view.ts` | « Mots lus », « Mots appris », « Révisions », « Aucun mot marqué « connu » ou « ignoré »… », « ${added} carte${added > 1 ? "s" : ""} ajoutée${…} au deck (niveau ${level}). », « Aucune carte ajoutée — … », « Niveaux CEFR indisponibles pour cette langue (pack sans données CEFR). », `${r} j` |
| `stats/ladder.ts` | « Vocabulaire connu », the notes with ` ` before « : » and « ≈ », `fmt(n)` = `toLocaleString("fr-FR")` (« 12 345 », a narrow no-break space), `sur les ${fmt(n)} mots du dictionnaire`, `(dont ${fmt} confirmés)` |
| `stats/stats.html` | two text nodes |
| `#696` | « Pas su », « Su », « Encore 10 », « Ne plus me le montrer », the session's end message, « Rythme de révision », a second hand-rolled plural; not on main when this is written |

Hosts: the side panel and the drawer mount review; the statistics page (`stats/stats.ts`), the
side panel and the drawer mount statistics. The side panel reads `chrome.storage.local` before it
mounts; the drawer is built by the reading session and is handed the interface language by it
(change 14); `stats.ts` reads no preference of its own today — `followSurfaceLook` reads the
colours for it, then it hydrates an engine and mounts — so it gains a read of the key before
`mountStats`, and fills its page with change 14's `fillPage`.

## Goals / Non-Goals

**Goals:**
- Every text of review and statistics comes from the catalogue, in the interface language.
- The French byte for byte, the ladder's escapes included.

**Non-Goals:**
- The other surfaces; the languages' names (19); the review's logic (#696).

## Decisions

### D1 — `mountReview` and `mountStats` take the interface language

`mountReview(container, port, area, opts)` gains `opts.interfaceLanguage`; `mountStats(root,
port, area, chosen?)` has no options object and gains a fifth parameter, `interfaceLanguage?`,
after the studied language, which its own call from the language picker passes on. Absent means
French, so every existing spec mounts as before. `renderReview`'s existing `opts` (`{showLanguage?}`) gains `copy` and `interfaceLanguage`
(absent = French, so `view.spec.ts` passes unchanged); the ladder's functions take the copy and
the language where they format. The controllers are untouched.

### D2 — Plurals

`plural(language, n, copy.remaining)`: French « ${n} carte(s) à revoir » with the raw count, as
today; English `{one: "1 card to review", other: "${n} cards to review"}`; Spanish likewise —
change 13's shapes. `plural(language, n, copy.cardsAdded(level))` replaces the hand-rolled
`carte${added > 1 ? "s" : ""}`
(and #696's second one, when #696 is on main), with the zero case a message of its own, as today.
The ladder's `fmt(n)` stays `toLocaleString("fr-FR")` for French — « 12 345 », today's bytes —
and is change 13's `formatNumber` for English ("12,345") and Spanish (« 12 345 », RAE).

### D3 — The ladder's bytes

The French entries carry the ` ` escapes and « ≈ » as the source does; the one-off
comparison of change 13 shows them equal.

### D4 — #696 and this change: the second one rebases

As change 15's D5: #696's copy goes into the catalogue either way, extracted here if #696 is on
main first, or by #696 rebased onto the catalogue, which the lint forces.

### D5 — The statistics note that spells the scale

« Niveaux CEFR indisponibles pour cette langue (pack sans données CEFR). » is change 13's
`stats.noLevels`, whose English already spells "CEFR" and whose Spanish « MCER » (M19); this
change keeps it, and change 19 makes it read the `levelScale` entry of its languages' module.

## Risks / Trade-offs

- **A French byte in the ladder's notes** → the `stats-view` spec pins them.
- **A grade label pinned by #696's tests** → whichever merges second rebases; the catalogue's
  French is #696's bytes, pinned by its tests.

## Migration Plan

One release, silent.
