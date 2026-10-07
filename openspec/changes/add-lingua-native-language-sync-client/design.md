# Design — add-lingua-native-language-sync-client

## Context

See proposal.md (Why). The sync client today (`apps/lingua-extension/src/sync/sync.ts`):

| Step | What |
|---|---|
| `checkErasure` | reads `GetDataState`, records `cardLanguage` (the server keys cards by language) |
| `pushCards` | maps `exportCardOps` to `PushCards`; a non-English card goes only when `cardLanguage` |
| `fetchCards` | `PullCards` with the accepted languages; applies through `applyCardOps` |
| `pushStats` | one `DailyStat` per (day, studied language) from `cymbra-lingua-daily-v3` |
| `state/dailystats.ts` | v3: day → studied language → `{exposures, unknownSeen, wordsLearned, reviews}`; v2 read once as English (`add-lingua-language-stats-review` D3) |
| recorders | `dailyRecorder(area)` from the reading session, the review page and the review session, with the studied language |

The engine's native language is one per engine (`nativeLanguage()`, change 4); the background
reads it without an engine through `storedNativeLanguage` (`state/storage.ts`).

## Goals / Non-Goals

**Goals:**
- Every card and every day's statistic reaches the server with its language label.
- Nothing labelled other than `fr` reaches a server that cannot store the label.
- The disclosures say so before the release ships.

**Non-Goals:**
- The Spanish policy page (changes 29, 31).
- Reconciling a card's gloss across two natives beyond last-write-wins (change 10 D2).
- A per-device history of native languages: a day has one, the last one counted.

## Decisions

### D1 — The data state records `languageLabels`, read every sync

`checkErasure` keeps `this.languageLabels = res.languageLabels === true` beside `cardLanguage`
(`GetDataStateResponse.language_labels`, change 10 D4; not `nativeLanguage`, which already names
the engine's method and the statistic's field). A field proto3 does not know is `false`; so is
a rolled-back server's answer.

### D2 — Cards: send the label always, send a non-French label only to a server that stores it; pull as a client that reads labels

`pushCards` sets `glossLanguage: c.gloss_language ?? ""` — the engine leaves the key out for a
French gloss (change 11 D2), and the server reads an empty value as `fr` — and pushes a card
whose label `(c.gloss_language ?? "fr") !== "fr"` only when `languageLabels`; otherwise the
card stays local and is pushed at a later sync, as non-English cards are held when
`cardLanguage` is false (the push is the whole deck each time).

`fetchCards` sends `anyGlossLanguage: true` (change 10 D7: this build reads every label, change
11 shows a gloss the reader can read) and passes `gloss_language` into the operation
`applyCardOps` reads. Before this build the server withheld non-French cards from this device
and kept no memory of them, and the cursor passed them: the first pull that says it reads labels
pulls again from the start, once, under a stored marker (`cymbra-lingua-sync-labels`), exactly
as *Widening a device's languages pulls again from the start* does for a language added.

Alternative: send a non-French gloss unlabelled to an older server. The row would be labelled
`fr` for ever: the one case the programme's rule exists to prevent.

### D3 — Daily statistics v4: the native language of the day, beside the counts

`cymbra-lingua-daily-v4`: day → studied language → `{exposures, unknownSeen, wordsLearned,
reviews, native}`. `loadDailyStats` reads v4; when none is written yet it reads v3 once, filing
each record with `native: "fr"` (every device so far is French-native, M22), and with neither
v4 nor v3 it reads v2 as English and French, as v3 does today. v3 stays in `STORE_KEYS` and is
not retired, as v2 is kept today: a downgraded build reads v3 (the risk
`add-lingua-language-stats-review` accepted), and `dropRetiredKeys` must not delete it before the
first v4 write. `bump` takes the native language, and the recorders take it from the engine
(`nativeLanguage()`) — the reading session, the review page and the review session hold a port.
A day counted under two native languages keeps the last one, with every count.

A downgrade has two consequences, named and accepted as v3's were: a downgraded build pushes
its stale v3 copy, so the day of the upgrade drops to v3's lower counts on the server until the
next upgrade pushes v4 again; and an erasure made on the downgraded build clears v3 and v2
only, so an upgrade afterwards pushes the erased v4 days again — the stores do not allow a
downgrade, and `clearDailyStats` on this build clears v2, v3 and v4.

`pushStats` sends `nativeLanguage: stat.native`, and sends a statistic whose native language is
not `fr` only when `languageLabels` is true; the others go as today. `clearDailyStats` erases v4
with the rest.

Alternative: the native language on the push request rather than on each statistic. The server
stores it per row (change 10 D2); a restored backup from another native would mislabel the day.

### D4 — The profile is never sent as such

*The backup records the reader's language profile* keeps every sentence but one: "The profile
SHALL never be sent to the server" becomes "The profile SHALL never be sent to the server as such:
the native language travels only as the label of a day's statistics (change 12), and the
language of a gloss as the card's (change 11)". The scenario *Never synced* keeps its name and
says what a request carries.

### D5 — The disclosures, in the same release

`confidentialite.md` and `en/privacy.md`, Lingua annex, table « Ce qui est synchronisé si vous
êtes connecté »: the deck row names « sa traduction, et la langue de cette traduction »; the
statistics row names « votre langue maternelle (celle des traductions) »; and the "stays on the
device" list is unchanged. `apps/lingua-apple/README.md`, the App Store answers: the « What it is in Lingua » column of
the User Content row adds the gloss language, that of the Usage Data row the native language;
the categories and the linkage do not change, so the App Store Connect form's answers do not
change — the form has no per-type description; the README's column is where the detail lives.
The Spanish annex comes with the Spanish site (changes 29, 31), which carries these two rows.

## Risks / Trade-offs

- **A non-French label dropped by an older server** → D2 and D3 hold them; the flag is read at
  every sync.
- **Cards withheld before this build** → the one-time pull from the start under its marker (D2).
- **A French card held back by mistake** → the label's default is spelled out: an absent label
  is `fr`, and only a label other than `fr` waits for the flag (D2).
- **A v3 record read as French on a device that was not** → no device could be: the native
  language's choice is change 20, after this; a restored backup carries its cards' labels, and
  the statistics it carries are the device's own.
- **A reader's day counted twice under two natives** → one row per (day, studied language) on
  the device and on the server; the last native wins with its counts.

## Migration Plan

One release, after change 10 is deployed and checked from outside. The disclosures ship in it.
A device updating reads v3 once; a device downgraded reads v3 only (the risk
`add-lingua-language-stats-review` accepted for v2).
