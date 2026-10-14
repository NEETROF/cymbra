## Context

`add-lingua-youtube-captions` (A below) gives YouTube's English captions a reading surface of
their own: it reads the whole track by replaying the player's own request, turns cues into
sentences once (its D4), draws Lingua's caption line inside the player (its D3), and opens the
page's cards on it (its D6). Its cards translate as a page's do: on demand, through
`createTranslatorPort` (`apps/lingua-extension/src/translate/create-port.ts`), which returns a
`TranslatorSource` — `(language) => TranslatorPort | null` — answering `null` unless
« Traduction étendue » is on and the pair's models are on the device.

The 2026-09-23 spike also found that the replayed caption request accepts `&tlang=<language>`:
YouTube then returns its own machine translation of the track, with the same events and timings,
the sentences redistributed across cues (the first cue of the French version read the tail of the
previous sentence). That is the second source this change uses. The owner re-measures it with A's
probes, once, in each browser (A's task 1.2 step 2, the `tlang` field of P2): no new probe here. If
no browser gets a translated track there, D2's source is gone and this change is reconsidered before
it starts.

The engine reaches readers through `add-lingua-translation-delivery` on Chromium and Firefox
desktop, and since through `add-lingua-translation-android` and `add-lingua-translation-safari`. A
reader who has not turned it on — most readers — has no sentence translation at all today, outside
« Traduction en ligne » on press.

## Goals / Non-Goals

**Goals:**
- The meaning of the caption on screen, at a glance, for a reader who wants it, without stopping
  the video.
- The engine's translation when the reader has it; YouTube's, labelled as YouTube's, when not.
- Nothing kept, nothing sent beyond YouTube's own request, the line off until asked for.

**Non-Goals:**
- Word alignment of YouTube's translation, or any use of it outside the line.
- « Traduction en ligne » on the line: it stays on the cards, on press (A's D6).
- Translating into anything but the reader's gloss language.
- Anything A leaves out (mobile, iPad, other languages, other sites).

## Decisions

### D1 — One line, one source at a time, chosen by the engine seam

The line asks `source("en")` (A's D11: captions are English) when it is shown and whenever the
setting or the model state changes — `translatorSource` already watches both keys. A port means
the engine; `null` means YouTube's translation. The two are never mixed on one line, and a reader
cannot pick YouTube's over a ready engine: the engine's answer is the one the cards show too, and
showing two different translations of the same sentence would be confusing.

*Alternative rejected:* a choice of source in Réglages. One more setting for a case nobody asked
for; the engine is the better answer whenever it is there.

### D2 — YouTube's translation: the replayed request with `tlang`, once, when first shown

With no engine, the first time the line is shown on a video the mode replays the caption request it
already rebuilt (A's task 2.3) with `tlang=<gloss language>` (`fr` or `es`, the reader's native
language), only when the track list says YouTube translates that track. The answer is parsed by A's
json3 parser and shown **per cue**, since its sentences are redistributed across cues; it is never
cut into words, never used to mark one, and never reaches a card. It adds no third party — YouTube
already serves the video — and it sends no reader text, so it respects "a reader who chose nothing:
no sentence leaves the device". Fetching only when the line is first shown keeps the default case
(line hidden) to the one request A makes.

*Alternatives rejected.* Fetching at load: a second request for a line most readers will never show.
Translating YouTube's English cues with the online service: sends caption text to Cymbra for every
line, which « Traduction en ligne »'s press-only rule exists to avoid.

### D3 — Engine pre-translation is paced, and yields to the reader

With an engine, the line keeps the next three sentences ahead of the playhead translated, one
request at a time, and asks for the next only when the previous answered. A card's request (A's
cards) goes first: pre-translation asks nothing while one is in flight. `TranslatorPort.translate`
(`src/translate/port.ts`) takes no cancellation, so a card opened while a sentence is being
translated waits for that one sentence — at most one, never the window ahead. That keeps **A
translation blocks nothing else** true from the reader's side. Answers are kept for the session, so a seek back costs
nothing; a seek forward restarts the window at the new position. In degraded mode (A's D7) there are
no sentences ahead, so the engine translates the line on screen when it appears.

*Alternative rejected:* translating the whole track at load. Minutes of engine time on a slow device,
competing with the reader's own cards, for lines that may never be watched.

### D4 — Hidden by default, the choice kept per device

The point of the mode is to read the English; the translation is a help, not the default. The line
is hidden until the reader shows it from the control beside the caption line or in Réglages, and the
choice is a per-device preference (`cymbra-lingua-caption-translation` in `chrome.storage.local`),
like « Démarrer automatiquement sur YouTube ». It is not synchronised.

### D5 — The label follows the interface, the translation follows the glosses

YouTube's translation goes into the language of the reader's glosses (French or Spanish), and its
label — « Traduction automatique YouTube » in French — follows the interface language, in French,
English and Spanish. Nothing is built for the language matrix beyond that.

### D6 — Sequencing

This change MODIFIES **No network requests** and **The caption request carries nothing of the
reader's** on top of A's text, so it archives after A (and after the delivery, which A follows). Its
block of **No network requests** changes only A's caption bullet, so a later block can be written on
top of it the same way. `add-lingua-usage-counts` (open proposal), which writes its own block of that
requirement on top of A's, must list this change in its `archiveAfter` too: whichever archives last
wins.

## Risks / Trade-offs

- **[`tlang` quality on automatic tracks]** → labelled as YouTube's, hidden by default, never used
  for a card.
- **[YouTube becomes a place where Lingua shows a sentence translation not its own]** → accepted
  knowingly (2026-09-23); the label says whose it is.
- **[The engine is busy when the reader opens a card]** → pre-translation yields (D3).
- **[YouTube stops honouring `tlang`]** → the line says no translation is available; nothing else
  depends on it.
- **[A track YouTube does not translate]** → the line says so (French interface: « Pas de
  traduction pour ces sous-titres »).

## Migration Plan

Additive. One per-device preference, absent meaning hidden. Rollback is a release without the line;
the preference is then ignored.

## Open questions for the owner

1. **With no engine, should the line also point to « Traduction étendue »?** *Recommended:* no — the
   line stays one quiet line; the setting lives in Réglages, where its download size is stated.
2. **Three sentences ahead with the engine — enough?** *Recommended:* start with three and adjust
   from the manual pass (task 3.1): fewer on a slow device costs a blank line, more costs engine time
   the reader's cards need.
