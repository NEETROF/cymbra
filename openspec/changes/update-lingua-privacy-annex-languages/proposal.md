# update-lingua-privacy-annex-languages — the privacy annex names the languages Lingua stores

## Why

Change 31 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, with M12 and M14. Annex B of the privacy policy (`/confidentialite/`, `/en/privacy/`,
and `/es/privacidad/` from change 29) was written when Lingua taught English alone. It still says
it highlights "English words", downloads "a translation model (25.8 MB)", reads aloud with "an
English voice", and syncs a "declared English level". Spanish shipped since; the matrix adds
readers of other native languages.

What Lingua stores and syncs per language today, and does not say: each status, level, card and
day of statistics belongs to a studied language (the server stores it: `known_words.proto`,
`deck.proto`, `stats.proto`). Change 12 added the gloss's language and the day's native language to
the deck and statistics rows. Change 17 sends the interface language to Cymbra ID as the account's
e-mail language (M12). The extended translation downloads the models of the reader's pairs — one or
two per pair. The annex's date was not moved when change 12 edited it.

## What Changes

- **Annex B in each published language** (French, English, Spanish) says, without naming one
  language as if it were the only one: the studied languages; the status, level, card and day each
  stored under its studied language; the gloss's language and the day's native language (change 12's
  rows, kept); the native language staying on the device, and the interface language sent to
  Cymbra ID as the account's e-mail language; the models of the reader's pairs, downloaded only when
  the reader turns the setting on; the voice of the language read aloud. The `updated` date moves.
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
- **Order.** After change 29 (the Spanish page). Archived after changes 29 and 12.
- **The owner** reviews the three annexes and publishes them with a site deploy (M18); the App Store
  privacy answers' categories are unchanged, so App Store Connect needs no edit.
