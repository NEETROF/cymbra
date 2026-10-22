# add-lingua-italian-read-aloud — Italian voices, named and spoken as the page writes them

## Why

Change 15 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Read-aloud
(`add-lingua-read-aloud`) speaks a word, its dictionary form and its sentence with a voice on the
device, in the language the page is read in, and Réglages lets the reader choose that voice. The
code is keyed by a language tag, so a speaker reading `it` already works. What it does with Italian
was measured on the seven voice lists captured on real browsers
(`apps/lingua-extension/test/fixtures/voices/`) and on Apple's Italian voice itself (`Alice`,
rendered with macOS `say`, the audio compared byte for byte), as French's change 47 measured its own:

- **The voices.** Chrome on macOS lists `Alice` (it-IT) first, then Apple's eight Eloquence voices
  under suffixed names (`Eddy (Italian (Italy))` … `Shelley (Italian (Italy))`), already
  deprioritised, and the network voice `Google italiano`; Safari on macOS, iOS and the Simulator and
  Firefox on macOS list `Alice` alone; Chrome on Windows lists only `Google italiano`, not on the
  device, so no Italian voice speaks there until the reader installs one or allows the online voices; Firefox for Android writes
  `ita-ITA-default` and `ita-ITA-f00`. No Swiss Italian voice is captured anywhere. So the automatic
  choice is already a voice of Italy wherever one exists; the region table is what is missing:
- **Firefox for Android's Italian region is unread.** The three-letter table knows the language
  `ita` but not the region `ITA`, so Réglages names it « ITA », the code, in every interface language.
- **An elided piece spoken alone is a clipped sound.** Change 6 gives each elision piece its own
  span, so `l'` of « l'uomo » gets a card. Spoken alone by Alice, `l'` is a clipped « l » (8,048 B
  against « lo »'s 14,714 B); spoken with its word it is right: « l'uomo » is byte for byte « luomo »,
  « dell'acqua » « dellacqua ».
- **A truncation's text is another word.** Change 6 reads `po'` as `poco`, `da'` as `dai`: spoken as
  the token's text, the card says a word the page did not write. Spoken as written it is right:
  « po' » is byte for byte « po », « da' » « dà », « di' » « di ».
- **The apostrophe for an accent, spoken as written, loses the stress.** Change 10 reads `perche'`
  as `perché`; spoken as written, « perche' » is byte for byte « perche » and « citta' » « citta »,
  both unlike « perché » and « città »; « c'e' » is « ce », unlike « c'è ». Spoken as the token's
  text it is right.
- **A lone `è` is a letter's name.** The copula alone lasts 1.03 s against « e »'s 0.18 s (the
  voice names the accented letter, as it does `à`, `ò`, `ì`, `ù`); « è. » is the vowel, 0.18 s —
  and « È vero » is byte for byte « E' vero », so in a sentence the voice reads it right.

And Réglages cannot yet say "no Italian voice is installed", nor offer an Italian sample, nor hold the words
Italian's level titles will take: those words live in the catalogue's `languages` modules, which name
English, Spanish and French only.

Nothing of this is visible before `enable-lingua-italian` (change 20) lists an Italian pair.

## What Changes

- **Italy first for Italian** (design D1): `PREFERRED_REGIONS.it = ["it"]` — within a quality tier,
  a voice of Italy before a Swiss one, should a device offer it; the others keep the browser's order.
  A voice the reader downloaded for its quality still comes first; another accent is the reader's
  choice in Réglages. No new deprioritised name: Italian's Eloquence set is the suffixed eight the
  list already holds.
- **Italy's region in three letters** (D2): `THREE_LETTER_REGIONS` gains `ita` → `it`:
  `ita-ITA-default` is a voice of Italy, named « Italie » in French, "Italy" in English, « Italia »
  in Spanish.
- **The Italian words in the catalogue** (D3): `src/i18n/{fr,en,es}/languages.ts` gain `italian` —
  its name and forms (« Italien », « d'italien », « l'italien », « italienne »; "Italian";
  « Italiano », « de italiano », « el italiano », « italiana »), the Windows language a voice is
  installed with (« Italien (Italie) », "Italian (Italy)", « Italiano (Italia) ») and the sentence a
  preview reads, in Italian whatever the interface — and the catalogue's named languages gain `it`
  (`src/analyzer/language-labels.ts`), so the voice block reads them through the speaker's language,
  without widening the extension's `StudiedLanguage` type, which change 20 widens; the level titles
  name Italian from then.
- **An Italian piece is heard as the page writes it** (D4): when the speaker reads Italian, a card
  opened on an elided piece reads it with the rest of the word the page writes it against (`l'` →
  « l'uomo », `dell'` → « dell'acqua »), and with the next token's accented text where change 10 read
  it (« c'e' » → « c'è », the page hit carrying that text to the card); a piece of a split
  articulated preposition reads the written word (« al »); a truncation reads as written (« po' »,
  « da' »); any other card reads as a French card does, so « perche' » reads `perché`; then a word of
  one vowel, accented or a capital, is spoken with a full stop (« è. », « E. ») — except « é », which a
  stop silences. The button is labelled with the form seen, without the stop.
- **English, Spanish and French do not move** (D5): their voice rankings over every captured list
  and every setting are pinned and pass unchanged; the card's rule reads Italian only.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *An Italian voice from Italy by default*, *Italian voices are
  named in the reader's interface language*, *Réglages speaks of Italian voices in the reader's
  interface language* and *An Italian piece is heard as the page writes it*. Every delta is ADDED:
  `add-lingua-read-aloud`, `localise-lingua-settings`, `add-lingua-native-language-labels` and
  `add-lingua-french-read-aloud` (open) hold the requirements this builds on; this change archives
  after them and after changes 6 and 10, whose pieces and spellings it reads.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `apps/lingua-extension/src/reading/speech.ts` (`PREFERRED_REGIONS.it`, `THREE_LETTER_REGIONS.ita`),
  `src/reading/wordpopup.ts` (`heardWord`), `src/reading/session.ts` and `src/reading/selection-card.ts`
  (the next token's text on the page hit), `src/i18n/{fr,en,es}/languages.ts` (`italian`),
  `src/analyzer/language-labels.ts` (`NamedLanguage`); their tests, the labels lint, and the
  voice-ranking snapshot, French pinned first.
- Untouched: lingua-core, the packs, every golden, the server, the site.
- **Effort, against 1–2 ideal days**: 1–1.5 (design, *Effort*).
