# update-lingua-privacy-annex-languages — the privacy annex names the languages Lingua stores

## Why

Change 31 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, with M12 and M14. Annex B of the privacy policy (`/confidentialite/`, `/en/privacy/`,
and `/es/privacidad/` from change 29) was written when Lingua taught English alone, and still reads that way: it says
it highlights "English words", downloads "a translation model (25.8 MB)", reads aloud with "an
English voice", and syncs a "declared English level". Spanish shipped since; the matrix adds
readers of other native languages.

What Lingua stores and syncs per language today, and does not say: each status, level, card and
day of statistics belongs to a studied language (the server stores it: `known_words.proto`,
`deck.proto`, `stats.proto`). Change 12 added the gloss's language and the day's native language to
the deck and statistics rows. When the reader creates an account, asks for a code or a new password,
or adds a password, the extension sends Cymbra ID a language — the browser's today, Lingua's interface
language or the browser's once change 17 lands (M12) — that the account keeps as its language; it is
not synced. The extended translation downloads the models of the reader's pairs — one or
two per pair. The annex's date was not moved when change 12 edited it.

## What Changes

- **Annex B in each published language** (French, English, Spanish) says, without naming one
  language as if it were the only one: the studied languages; the status, level, card and day each
  stored under its studied language; the gloss's language and the day's native language (change 12's
  rows, kept); the native-language setting staying on the device (not synced as a setting) while
  the native language reaches Cymbra as each card's gloss language and with each day's statistics,
  and, in its own paragraph of the account part (not the sync table), the language the extension
  sends to Cymbra ID — Lingua's interface language or the browser's — kept as the account's language;
  the models of the reader's pairs, downloaded only when the reader turns the setting on, a pair added
  later only when the reader asks; the voice of the language read aloud. The `updated` date moves.
- **The erasure path** is named by its function, not by French interface labels, in the English and
  Spanish annexes.
- **The App Store privacy answers' notes** (`apps/lingua-apple/README.md`) name the studied
  language on the rows they describe; the categories do not change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-privacy`: ADDED *The annex names the languages Lingua stores*. *Lingua's privacy
  disclosures match what it collects* is held by the open `add-lingua-remote-translation` and read
  as written; *The disclosures name the languages a card and a day carry* (change 12) stands.

## Impact

- **Products.** The site's three privacy pages (Annex B) and `apps/lingua-apple/README.md`. No code.
- **Order.** After change 29 (the Spanish page). Archived after changes 29 and 12. The site deploy that
  publishes it comes after `add-lingua-native-language-server` is deployed and checked from outside (its
  5.2) and before the store release that carries `add-lingua-native-language-sync-client` (its 6.1): the
  annex says the native language reaches Cymbra with each card and day.
- **The owner** reviews the three annexes and publishes them with a site deploy (M18); the App Store
  privacy answers' categories are unchanged, so App Store Connect needs no edit.
