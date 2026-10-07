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

### D1 — The data state records `nativeLanguage`, read every sync

`checkErasure` keeps `this.nativeLanguage = res.nativeLanguage === true` beside `cardLanguage`.
A field proto3 does not know is `false`; so is a rolled-back server's answer (change 10 D4).

### D2 — Cards: send the label always, send a non-French label only to a server that stores it

`pushCards` sets `glossLanguage: c.gloss_language` and pushes a card whose label is not `fr`
only when `nativeLanguage`; otherwise the card stays local and is pushed at a later sync, as
non-English cards are held when `cardLanguage` is false (the push is the whole deck each time).
`fetchCards` passes `gloss_language` into the operation `applyCardOps` reads (change 11 D2).

Alternative: send a non-French gloss unlabelled to an older server. The row would be labelled
`fr` for ever: the one case the programme's rule exists to prevent.

### D3 — Daily statistics v4: the native language of the day, beside the counts

`cymbra-lingua-daily-v4`: day → studied language → `{exposures, unknownSeen, wordsLearned,
reviews, native}`. `loadDailyStats` reads v4; when none is written yet it reads v3 once, filing
each record with `native: "fr"` (every device so far is French-native, M22), as v3 read v2 as
English. `bump` takes the native language, and the recorders take it from the engine
(`nativeLanguage()`) — the reading session, the review page and the review session hold a port.
A day counted under two native languages keeps the last one, with every count.

`pushStats` sends `nativeLanguage: stat.native`, and sends a statistic whose native language is
not `fr` only when `nativeLanguage` is true; the others go as today. `clearDailyStats` erases v4
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
device" list is unchanged. `apps/lingua-apple/README.md`, the App Store answers: the User
Content row adds the gloss language, the Usage Data row adds the native language; the categories
and the linkage do not change, so the App Store Connect form's answers do not change — the
recorded descriptions do.

## Risks / Trade-offs

- **A non-French label dropped by an older server** → D2 and D3 hold them; the flag is read at
  every sync.
- **A v3 record read as French on a device that was not** → no device could be: the native
  language's choice is change 20, after this; a restored backup carries its cards' labels, and
  the statistics it carries are the device's own.
- **A reader's day counted twice under two natives** → one row per (day, studied language) on
  the device and on the server; the last native wins with its counts.

## Migration Plan

One release, after change 10 is deployed and checked from outside. The disclosures ship in it.
A device updating reads v3 once; a device downgraded reads v3 only (the risk
`add-lingua-language-stats-review` accepted for v2).
