## Why

`add-lingua-youtube-captions` makes YouTube's English captions readable as Lingua text: Lingua's
own caption line, highlights, cards, percentage, exposure. Its cards translate exactly as a page's
do — on demand, a word or a phrase at a time. What it does not give is the meaning of the whole
line while the video plays, which is what a learner watching a video at speed reaches for most.

This part was in the YouTube change's first draft (its D6). The owner split it out on 2026-10-14:
it is where the product questions are — Lingua showing YouTube's own translation to a reader with
no engine, how hard the engine may be pushed while a video plays, whether the line is shown by
default — and the reading part should not wait for them. This change is that part, on its own,
built on top of the caption mode once it exists.

## What Changes

- **A translation line under the caption**, hidden until the reader shows it, the choice kept per
  device (« Afficher la traduction des sous-titres »), from a control beside the caption line and
  in Réglages.
- **With the engine** (« Traduction étendue » on and its model ready), each sentence is translated
  ahead of time, a few sentences ahead of the playhead, never ahead of what the reader asks on a
  card.
- **Without it**, and **on YouTube only**, the line shows **YouTube's own automatic translation**
  of the track into the reader's gloss language — a whole-line translation with no word alignment,
  labelled « Traduction automatique YouTube », fetched once per video when the line is first
  shown. It is never used to mark a word, never kept on a card, never sent anywhere, never mixed
  with an engine's answer, and never shown in degraded mode.

## Capabilities

### New Capabilities

_None._ The translation line belongs to `lingua-youtube-captions`, which
`add-lingua-youtube-captions` creates.

### Modified Capabilities
- `lingua-youtube-captions`: three requirements **added** — the translation line and its
  preference, where its translation comes from, and that YouTube's translation is never kept.
- `lingua-browser-extension`: **No network requests**, as `add-lingua-youtube-captions` leaves it,
  gains YouTube's translation of the track in the caption mode's bullet — only while the line is
  shown with no engine ready.
- `lingua-privacy`: **The caption request carries nothing of the reader's**, added by
  `add-lingua-youtube-captions`, also allows the target language on the replayed request, and the
  Lingua annex says whose translation the line shows without an engine.

## Impact

**Products.** Cymbra Lingua's extension only (Chromium, Firefox, Safari on macOS — the caption
mode's variants). No backend change, no RPC, no `.proto` change. The site's privacy page gains one
sentence in the Lingua annex, in French, English and Spanish, and in Italian on change 41's page where
it exists.

**Consumed, not redeclared.** The caption mode, its track replay, its parser and its sentences
(`add-lingua-youtube-captions`); the engine seam `createTranslatorPort` and **A translation blocks
nothing else** (`lingua-translation`); **A machine translation is never stored** for the engine's
translations.

**Code.** `apps/lingua-extension`: the translation line beside the caption line (`src/youtube/`),
its pacing of the engine, the `tlang` replay, one per-device preference in Réglages, new copy in
French, English and Spanish, and in Italian once change 33's `src/i18n/it/` exists (`src/i18n/`).

**Ordering.** Archives **after** `add-lingua-youtube-captions` and
`add-lingua-translation-delivery` (`archiveAfter`): its blocks of **No network requests** and of
**The caption request carries nothing of the reader's** are written on top of the caption mode's.
The open proposal `add-lingua-usage-counts`, which also rewrites **No network requests**, must
archive after this change as well.

**A product fact, accepted knowingly** (decided 2026-09-23): for a reader without the engine,
YouTube becomes a place where Lingua shows a sentence translation that is not its own. The label
says whose it is.

**Size.** 3–4 ideal days, including half a day of the owner's manual pass.
