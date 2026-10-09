# add-lingua-french-read-aloud — a French voice from France, named in the reader's language

## Why

Change 47 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 3 (French studied: fr-en for English speakers, fr-es for Spanish speakers). Read-aloud
(`add-lingua-read-aloud`) speaks a word, its dictionary form and its sentence with a voice on the
device, in the language the page is read in, and Réglages lets the reader choose that voice. The
code is keyed by a language tag, so a speaker reading `fr` already works. What it does with French
was measured on the seven voice lists captured on real browsers
(`apps/lingua-extension/test/fixtures/voices/`) and on Apple's French voices themselves, and four
things read wrong:

- **A Canadian voice by default.** French has no preferred region, so the automatic choice is the
  browser's first French voice. Chrome, Firefox and Safari on macOS (and the iOS Simulator, which
  lists the host Mac's voices) list `Amélie` (fr-CA) before `Thomas` (fr-FR), alphabetically: the
  automatic French voice is Canadian on 4 of the 7 captures. It is a voice of France only on the
  iPhone, whose list happens to put `Thomas` first, on Chrome for Windows, by its one default
  voice, and on Firefox for Android once Android's voices are allowed, its two French voices
  being France's. Spanish settled the same question with a voice of Spain
  (`add-lingua-spanish-read-aloud`).
- **An Eloquence voice among the ordinary ones.** Chrome on macOS lists `Jacques`, which is Apple's
  `com.apple.eloquence.fr-FR.Jacques` — the French (France) Eloquence set has Jacques where every
  other language has Reed — under its bare name, so neither the family nor the name list catches
  it. It is offered as an ordinary voice today, and preferring France alone would make it the
  automatic French voice there (`Jacques` sorts before `Thomas`).
- **Firefox for Android's French regions.** It writes `fra-FRA-default`; the three-letter table
  knows Canada's `can` but not France's `fra`, so the region is unread and Réglages names it
  « FRA », the code, in every interface language.
- **An elided word spoken alone is a letter's name.** M21 gives each elision piece its own
  highlight span (change 40), so `l'` of « l'homme » gets a card of its own. Spoken alone by
  Apple's French voices, `l'` is byte for byte the audio of « elle », `d'` of « dé », `j'` of « ji »,
  `s'` « esse », `c'` « cé », `m'` « emme », `t'` « té »; `qu'` is a clipped sound. Spoken with the
  word it leans on it is right: « l'homme » is byte for byte « lomme », « qu'il » « kil ».

And Réglages cannot yet say "no French voice is installed" in English or Spanish, nor offer a
French sample: those words live in the catalogue's `languages` modules, which name English and
Spanish only.

Nothing of this is visible before `enable-lingua-french` (change 52) lists a French pair: no
surface reads French until then.

## What Changes

- **France first for French.** `PREFERRED_REGIONS.fr = ["fr"]`: within a quality tier, a voice of
  France comes before the others, which keep the browser's order. A voice the reader downloaded
  for its quality still comes first, as for English and Spanish. No accent setting: another accent
  is the reader's choice in Réglages, kept for French.
- **`Jacques` is an Eloquence voice.** Added to the deprioritised names: listed under « Autres
  voix » ("Other voices", « Otras voces »), never the automatic choice while an ordinary voice
  exists.
- **French-speaking regions in three letters.** `THREE_LETTER_REGIONS` gains France, Belgium and
  Switzerland (`fra`, `bel`, `che`; Canada's `can` is there): `fra-FRA-default` is a voice of France,
  named « France » in French and English, « Francia » in Spanish.
- **The French words in the catalogue.** `src/i18n/{fr,en,es}/languages.ts` gain `french`: its
  name, its forms, the Windows language a voice is installed with (« Français (France) », "French
  (France)", « Francés (Francia) ») and the sentence a preview reads, in French whatever the
  interface. The voice block reads them through the speaker's language — "No French voice is
  installed on this device.", « No hay ninguna voz francesa instalada en este dispositivo. » —
  without widening the extension's `StudiedLanguage` type, which change 52 widens (change 39, D8).
  Change 39's D8 left the labels to change 52 as well: their French words come here instead, a
  departure design D4 justifies.
- **An elided French piece is heard with the word it leans on.** When the speaker reads French, a
  card opened on an elided piece reads it with the rest of the word the page writes it against
  (`l'` → « l'homme », `qu'` → « qu'il », `jusqu'` → « jusqu'à »), and a piece of a split word reads
  the word as written (`au` for its `à`); the button is labelled with what it reads. The
  dictionary form's button is unchanged (« ▶ le »).
- **English and Spanish do not move.** Their voice rankings are pinned before the change over
  every captured list and every setting, and must pass unchanged; the card's rule reads French only.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *A French voice from France by default*, *French voices are
  named in the reader's interface language*, *Réglages speaks of French voices in the reader's
  interface language* and *An elided French word is heard with the word it leans on*. Every delta
  is ADDED: `add-lingua-read-aloud` (open) holds the read-aloud requirements this builds on,
  `localise-lingua-settings` *Réglages speaks the interface language* and
  `add-lingua-native-language-labels` *Languages are named in the interface language*; none is
  rewritten, and this change archives after them (`archiveAfter`).

## Impact

- **Products.** Cymbra Lingua's extension only (all three variants; the Safari variant ships in
  `apps/lingua-apple`, whose native code does not change):
  - *new*: `src/reading/speech.ts` — France first, `Jacques`, the three French-speaking regions;
    `src/i18n/{fr,en,es}/languages.ts` — the `french` entry; `src/analyzer/language-labels.ts` — the
    voice messages keyed by the language the speaker reads; `src/reading/wordpopup.ts` and
    `selection-card.ts` — the elided piece's text and its place in the sentence;
    `src/reading/settings-view.ts` — the speaker's language read without a cast;
  - *consumed*: the speaker, the voice block, the Android and online-voice switches and the
    per-language voice of `add-lingua-read-aloud` and `add-lingua-language-choice`; `regionName`
    of `localise-lingua-settings`; the catalogue of `add-lingua-native-language-labels`.

  The engine, the packs, the tables, the backend, ID, Music, Live, the back office and the site are
  untouched. No privacy text changes: Annex B already says a voice of the language read speaks,
  and the online voices' paragraph is the reader's opt-in.
- **Release.** Silent. No package lists a French pair before change 52, so no speaker reads `fr`;
  the stored profile's French is still dropped by `state/profile.ts` (change 39, D8).
- **Goldens.** None moves: `fr-en.golden`, `en-fr.golden`, `es-fr.golden`, `es-en.golden` and
  `en-es.golden` are produced by `crates/lingua-wasm` from the core and the packs, which this change
  does not touch — change 39's D5 counts 47 among the changes that re-bless `fr-en.golden`, but this
  one has nothing to re-bless, and not one byte of it moves; the card snapshots
  (`word-card-es-en.txt`, `word-card-en-es.txt`, `selection-rows-fr.txt`) hold no listen row.
- **Dependencies.** After change 39 (`add-lingua-french-baseline`), and after the open
  `add-lingua-read-aloud`, `localise-lingua-settings` and `add-lingua-native-language-labels`
  (`archiveAfter`). Independent of changes 40–46: the elided-piece rule acts on a card's text and
  its place in the sentence, whatever change 40 makes the piece's token, and is inert until a piece
  exists. Changes 51 and 52 consume it: 52 widens `StudiedLanguage` and finds the French words in
  place, and its dogfood runs the on-device checklist of this change (design D9).
- **Effort, against 1.5–3 ideal days.** The voice choice, `Jacques`, the regions and their tests
  over the captures: 0.5. The catalogue entry, the keyed voice messages and the block mounted in
  English and Spanish: 0.5–1. The elided piece and its card tests: 0.5–1. Spec, programme and the
  checklist: 0.25.
