# Design — localise-lingua-review-stats

## Context

See proposal.md (Why) and change 13's design. Today:

| Module | Copy |
|---|---|
| `review/view.ts` | « Réviser », « Rien à réviser pour l'instant. », « ${n} carte(s) à revoir », « Afficher la réponse », the grades (`GRADES`), « Je connais ✓ », the source label |
| `review/review-page.ts` | « Sources & confidentialité », « Sauvegarde restaurée. », `deck + " carte(s) · " + due + " à revoir"`, « Sources : ${…} », the language filter's names (change 19's seam) |
| `stats/view.ts` | « Mots lus », « Mots appris », « Révisions », « Aucun mot marqué « connu » ou « ignoré »… », « ${added} carte${added > 1 ? "s" : ""} ajoutée${…} au deck (niveau ${level}). », « Aucune carte ajoutée — … », `${r} j` |
| `stats/ladder.ts` | « Vocabulaire connu », the notes with ` ` before « : » and « ≈ », `toLocaleString("fr-FR")`, `sur les ${fmt(n)} mots du dictionnaire`, `(dont ${fmt} confirmés)` |
| `stats/stats.html` | two text nodes |
| `#696` | « Pas su », « Su », « Encore 10 », « Ne plus me le montrer », « Remettre à apprendre », the session's end message, « Rythme de révision » |

Hosts: the side panel and the drawer mount review; the statistics page and the side panel mount
statistics; each reads `chrome.storage.local` before.

## Goals / Non-Goals

**Goals:**
- Every text of review and statistics comes from the catalogue, in the interface language.
- The French byte for byte, the ladder's escapes included.

**Non-Goals:**
- The other surfaces; the languages' names (19); the review's logic (#696).

## Decisions

### D1 — `mountReview` and `mountStats` take the copy

`mountReview(container, port, area, opts)` and `mountStats(…)` gain `opts.language`; the view
functions (`renderReview(root, view, actions, copy)`, the ladder's) take the copy. The controllers
are untouched.

### D2 — Plurals

`copy.remaining(n)`: French forms all « ${n} carte(s) à revoir » with the raw count; English
`{one: "1 card to review", other: "${n} cards to review"}`; Spanish likewise. `copy.cardsAdded(n,
level)` replaces the hand-rolled `carte${added > 1 ? "s" : ""}`, with the zero case a message of
its own, as today. The ladder's `fmt(n)` is French's `toLocaleString("fr-FR")` kept, and the
locale's for English and Spanish.

### D3 — The ladder's bytes

The French entries carry the ` ` escapes and « ≈ » as the source does; the one-off
comparison of change 13 shows them equal.

### D4 — #696 first

As change 15's D5.

## Risks / Trade-offs

- **A French byte in the ladder's notes** → the `ladder` and `stats-view` specs pin them.
- **A grade label pinned by #696's tests** → extracted after #696, pinned by its tests.

## Migration Plan

One release, silent.
