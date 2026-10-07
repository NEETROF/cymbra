# add-lingua-native-language-sync-client — the device sends the language of a gloss and of a day, to a server that stores them

## Why

Change 12 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the ninth of stage 1: the wire of decisions M4 and M14. The server stores the gloss language of a
card and the native language of a day's statistic (change 10); the card carries its label on the
device (change 11). This change makes the device send both, read the label back, and hold what
the server cannot store yet.

The rule « server first » has a client side: a device SHALL NOT send a non-French gloss, or a
statistic labelled with a native language, to a server that answers it does not store them — the
field would be dropped on the wire and the row labelled `fr` for ever. The data state read at the
start of every sync says what the server stores (change 10's flag); the device pushes its
non-French cards only when it is true, as it pushes its non-English cards only to a server that
keys cards by language.

A day's statistics carry the native language of the device that counted them, so the back office
can break usage down by pair. On the device, the daily statistics move to a fourth shape, keyed by
day and studied language with the native language of the day beside the counts; the third shape
is read once, as French, since every device so far is French-native.

The native language is new collected data (M14): the privacy policy's Lingua annex, in French and
English, and the App Store privacy answers say so in this change, before the release that sends
it. The Spanish policy does not exist yet; it is written with the Spanish site (changes 29, 31).

This is a silent release for a reader of French: every card's label is `fr`, every day's native
language is `fr`, and the server reads an absent value as `fr` — a device built before this
change and one built with it write the same rows.

## What Changes

- **The data state read** at the start of a sync records whether the server stores the labels
  (`native_language`, change 10), beside `card_language`.
- **Cards.** The push sends `gloss_language` with every card, and pushes a card whose gloss is not
  French only when the server stores labels; the pull reads `gloss_language` into the applied
  operation (change 11 applies it).
- **Daily statistics, v4.** The device's daily statistics are keyed by day and studied language
  and carry the native language of the day; v3 is read once as French. The push sends
  `native_language`, and sends a statistic whose native language is not French only when the
  server stores labels. The recorders take the native language from the engine's.
- **Disclosures.** The privacy annex (fr, en) says a card's gloss is synced with its language and a
  day's statistics with the reader's native language; the App Store answers recorded with the
  Apple app say the same. *The backup records the reader's language profile* is reworded: the
  profile is never sent as such; the native language travels as the label of a day's statistics,
  and the language of a gloss as the card's.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-sync`: ADDED *A device sends the language of a gloss only to a server that stores it*.
- `lingua-stats`: ADDED *A device's daily statistics carry its native language*.
- `lingua-decks-review`: MODIFIED *The backup records the reader's language profile* — one sentence
  and one scenario reworded for the native language that now travels on a statistic; every
  other sentence and scenario kept. Held by no open change (`refine-lingua-review-session` holds
  two other requirements of this capability).
- `lingua-privacy`: ADDED *The disclosures name the languages a card and a day carry*. *Lingua's
  privacy disclosures match what it collects* is held by the parked `add-lingua-remote-translation`
  and is not modified; the new requirement stands beside it.

## Impact

- **Products.** Cymbra Lingua (`apps/lingua-extension`: `src/sync/sync.ts`, `src/state/dailystats.ts`,
  `src/analyzer/port.ts`, the recorders' callers, their tests; `apps/lingua-apple/README.md`, the
  App Store answers) and the site (`apps/site/src/pages/confidentialite.md`, `en/privacy.md`, the
  Lingua annex). The agent plugin is local and sends nothing. ID, Music, Live and the back office
  are untouched; the backend is change 10, already deployed when this ships.
- **Order.** Built for a store only after change 10 is deployed and checked from outside (its
  task 5.2); a device that meets an older server holds its non-French cards and statistics, and
  sends them at a later sync. The disclosures ship in the same release.
- **No byte moves for a reader of French**: the rows written are the same; the English baseline
  pins no sync message.
- **Not here.** The Spanish privacy policy (changes 29, 31); the back office's breakdown by pair
  (optional `add-admin-lingua-pair-usage`); the native language's choice (change 20).
