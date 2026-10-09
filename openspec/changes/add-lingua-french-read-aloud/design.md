# Design — add-lingua-french-read-aloud

## Context

See proposal.md (Why). Read-aloud (`add-lingua-read-aloud`, shipped, not archived) is keyed by a
language tag from end to end, so French needs no new seam, only answers at the ones that exist:

| Seam | Today |
|---|---|
| `src/reading/speech.ts` `PREFERRED_REGIONS` | `{ en: ["us", "gb"], es: ["es"] }`: within a quality tier, these regions first, then the browser's order (`rankVoices`) |
| `speech.ts` `THREE_LETTER_REGIONS` | Firefox for Android's regions (`eng-GBR-default`): English- and Spanish-speaking countries, Canada's `can` among them; `THREE_LETTER_LANGUAGES` already reads `fra`/`fre` |
| `speech.ts` `DEPRIORITISED_NAMES`, `DEPRIORITISED_FAMILY` | Apple's novelty, Eloquence and legacy voices, by name (Chrome) or by identifier family (Safari, Firefox) |
| `speech.ts` `voiceLabel` | `<name> — <region>`, the region through `regionName(interfaceLanguage, code)` (`localise-lingua-settings` D4); the code where `Intl.DisplayNames` cannot name it |
| `src/analyzer/language-labels.ts` | `noVoiceInstalled`, `windowsVoiceLanguage`, `previewSentence`, `languageName` read `words(language, studied: StudiedLanguage)`, a `Record<StudiedLanguage, LanguageWords>` of `{ en, es }` |
| `src/i18n/{fr,en,es}/languages.ts` | `english` and `spanish` entries (`name`, `of`, `the`, `masculine`, `feminine`, `windowsVoice`, `preview`); en and es typed `typeof fr` |
| `src/reading/settings-view.ts` | the voice block casts `speaker.lang as StudiedLanguage` for its no-voice sentence, install help and preview |
| `src/reading/wordpopup.ts` `listensFor` | the word button reads `surface` — the piece of a split word (`de` of `del`, `add-lingua-dictionary-form-voice`'s non-goal); with a form seen apart from the dictionary form, two buttons labelled with what they read |
| `src/reading/selection-card.ts` | holds the selection's place in its sentence (`PageHit.selection`, `SelectionInput.selection`) for the translator; the card content does not carry it |
| `src/analyzer/types.ts` | the extension's `StudiedLanguage` is `en \| es`; change 39 (D8) leaves its widening to change 52, with `state/profile.ts`'s `NAMES` and the pairs |

### Measured: the French voices of the seven captures

From `apps/lingua-extension/test/fixtures/voices/`, the French voices (primary subtag `fr`, `fra`)
and what the automatic choice is, run with the code on `main` and with this change's three
`speech.ts` edits (prototype in the scratchpad, `pickVoice(voices, "fr", null, …)`):

| Capture | French voices (on the device) | Automatic today | With this change |
|---|---|---|---|
| Chrome macOS | 19 (18): `Amélie` fr-CA, `Thomas` fr-FR, `Jacques` fr-FR and 15 other Eloquence voices, `Google français` (remote) | `Amélie` (fr-CA) | `Thomas` (fr-FR) |
| Chrome Windows (set to French) | 4 (3): `Microsoft Hortense`, `Julie`, `Paul` fr-FR, `Google français` (remote) | `Hortense` (its one default) | `Hortense` |
| Firefox Android | 2 (0, Android's engine): `fra-FRA-default`, `fra-FRA-f00` | none; once allowed, `fra-FRA-default`, named « — FRA » | none; once allowed, `fra-FRA-default`, named « — France » |
| Firefox macOS | 2 (2): `Amélie` fr-CA, `Thomas` fr-FR (super-compact) | `Amélie` | `Thomas` |
| Safari iOS Simulator (the host Mac's list) | 2 (2): `Amélie`, `Thomas` | `Amélie` | `Thomas` |
| Safari iPhone (French system) | 3 (3): `Thomas` compact and super-compact, `Amélie` | `Thomas` (list order) | `Thomas` |
| Safari macOS | 2 (2): `Amélie`, `Thomas` | `Amélie` | `Thomas` |

Today the automatic French voice is Canadian on 4 of the 7 captures. The three edits make it a
voice of France on all of them, and leave English and Spanish exactly as they were: over the seven
captures, both languages, Android's voices allowed or not and the online voices allowed or not
(56 rankings), the ranked list and the automatic choice are identical before and after.

### Measured: `Jacques` is an Eloquence voice

On this Mac (macOS 26.5.2), `NSSpeechSynthesizer.availableVoices` lists Apple's Eloquence family as
nine names: `Eddy`, `Flo`, `Grandma`, `Grandpa`, `Rocko`, `Sandy` and `Shelley` in 14 languages
each, `Reed` in 13, and `Jacques` in one — `com.apple.eloquence.fr-FR.Jacques`: French (France)
has Jacques where every other language has Reed. Chrome suffixes a voice's name with its language
only when the name exists in several (`Eddy (French (France))`), so it lists this one as the bare
`Jacques`, with its name as `voiceURI`: neither the family pattern nor the name list catches it.
Safari and Firefox list no Eloquence voice.

### Measured: what Apple's French voices make of an elided word alone

`say -v Thomas` and `say -v Amélie`, rendered to 16-bit WAV and compared by their samples' hash:

| Spoken alone | Same audio as | The word in full |
|---|---|---|
| `l'` | « elle », the letter's name (Thomas and Amélie) | « le » differs |
| `d'` | « dé » | « de » differs |
| `j'` | « ji » | « je » differs |
| `s'` | « esse » | « se » differs |
| `c'` | « cé » | « ce » differs |
| `m'` | « emme » | « me » differs |
| `t'` | « té » | « te » differs |
| `n'` | the letter `n` | « ne » differs |
| `qu'` | `qu`: a clipped sound | « que » and « ku » differ |
| `lorsqu'` | « lorsque » | the same |
| `l'.` (with a period) | still « elle » | |

With the word it leans on, the same voice is right, and the audio proves it: « l'homme » is byte
for byte « lomme », « qu'il » « kil », « j'ai » « jé » — where « je ai » is not. French
typography is spoken as written: « Tu viens ? » with a narrow no-break space (U+202F), a no-break
space, guillemets or none, and « L’homme » with the typographic apostrophe, give the same audio as
their ASCII forms.

Over the French baseline corpus (`crates/lingua-wasm/tests/baseline/pages-fr.txt`, 1,012 written
words), an elided piece in the sense of M21 occurs 66 times: `l'` 18, `d'` 9, `qu'` 9, `n'` 7,
`m'` 5, `s'` 4, `c'` 3, `j'` 3, `t'` 3, `jusqu'` 3, `lorsqu'` 1, `puisqu'` 1. 61 of them (92 %)
are forms the voices spell or clip when spoken alone.

### Measured: a single-letter word spoken alone

The same holds beyond elision, in every language: Thomas and Amélie read « à » alone as « a accent
grave » (the same audio, byte for byte), Thomas reads « y » at about the length of « i grec »;
Spain's Mónica reads « y » at about the length of « i griega »; Samantha reads "I" at twice the
length of "I,". Followed by a comma, Thomas's « à » is his « a » and Mónica's « y » her « i », byte
for byte. Inflected forms that French does not pronounce are spoken alike: `homme` and `hommes`,
`parle`, `parles` and `parlent`, `aimé`, `aimer` and `aimez` give the same audio.

## Goals / Non-Goals

**Goals:**
- A voice of France by default for French, whatever order a browser lists voices in, and never an
  Eloquence voice while an ordinary one exists.
- French voices and their regions named in the reader's interface language, Firefox for Android's
  included.
- Réglages' voice block able to say, in English and Spanish, that no French voice is installed and
  how to add one, and to preview a French voice with a French sentence.
- A card on an elided French piece that says the word as French says it.
- English and Spanish read-aloud unmoved, the French interface byte for byte, nothing visible
  before change 52.

**Non-Goals:**
- An accent setting (as Spanish D5): the voice picker is the way to another accent.
- The extension's `StudiedLanguage` type, `state/profile.ts`'s `NAMES` and `packs.json` (change
  52, after change 39's D8), and what a French card's grammar says (change 51).
- A single-letter word spoken alone (« à », « y »; Spanish « y », English "I") and reading a split
  word as written in English and Spanish (`don't`, `del`): every language's behaviour, left to a
  shared follow-up (D6).
- Any change to the privacy texts, the speaker, the switches or the voice storage.

## Decisions

### D1 — France first, within a tier

`PREFERRED_REGIONS.fr = ["fr"]`. The region ranks after the tier, as English's and Spanish's do: a
French voice the reader downloaded for its quality (Apple's enhanced or premium, Chrome's
« (Enhanced) ») still comes before a compact voice of France, because the tier already honours the
reader's download. After France, the other regions keep the browser's order. No accent setting:
a reader who wants Amélie chooses her in Réglages, and the choice is kept for French
(`SpeechSettings.voices.fr`, *A voice per studied language*).

*Rejected — region before tier.* A device's super-compact voice of France would beat the voice its
reader went out of their way to install (Spanish D1's reasoning).

*Rejected — Belgium and Switzerland ranked before Canada.* None of the seven captures lists a
Belgian or a Swiss French voice on the device, so the ordering would act on no measured list. The
owner may still prefer it (Open question 1).

*Rejected — following the browser's region (a Canadian voice for `en-CA`).* The interface language
has no region; the browser's locale would make one reader's automatic voice depend on a setting
they never chose for Lingua. One rule, as Spanish has; the picker is the way out.

### D2 — `Jacques` is deprioritised with the Eloquence names

`DEPRIORITISED_NAMES` gains `Jacques` beside `Eddy` … `Shelley`. On Chrome for macOS the French
voices then group as `Thomas`, `Amélie` (ordinary) and 16 others (eight Eloquence voices of France,
eight of Canada). Without D2, D1 alone would make `Jacques` the automatic French voice there: it
is a voice of France and sorts before `Thomas` in Chrome's alphabetical list. The family pattern
already covers its identifier (`com.apple.eloquence.`) wherever a browser exposes it.

The name is matched for every language, as the other Eloquence names are. No English or Spanish
voice of the seven captures is named `Jacques` (the English and Spanish rankings are unchanged,
above), and Apple ships the name in French (France) only. A deprioritised voice still speaks when
it is the only one, and stays in the list for a reader who wants it.

### D3 — The French-speaking regions in three letters

`THREE_LETTER_REGIONS` gains `fra` → `fr`, `bel` → `be`, `che` → `ch` (`can` → `ca` is there). France
is the one D1 needs on Firefox for Android; Belgium and Switzerland let such a voice read as itself,
which keeps the browser's order among them and names it. Today `fra-FRA-default` keeps the region
`fra`, which `Intl.DisplayNames` refuses (`RangeError`, a region is two letters or three digits):
`regionName` falls back to the code, and Réglages shows « — FRA » in every interface language.
After D3 it reads « France » in French and English and « Francia » in Spanish, and Belgium and
Switzerland "Belgium" / « Bélgica », "Switzerland" / « Suiza ».

*Rejected — every French-speaking country.* No Android engine captured lists one; an unread region
keeps the browser's order and shows its code, which is the honest fallback.

### D4 — The French words in the catalogue, read through the speaker's language

`src/i18n/{fr,en,es}/languages.ts` gain a `french` entry, in each language's grammar:

| Field | fr | en | es |
|---|---|---|---|
| `name` | Français | French | Francés |
| `of` | de français | French | de francés |
| `the` | le français | French | el francés |
| `masculine` | français | French | francés |
| `feminine` | française | French | francesa |
| `windowsVoice` | Français (France) | French (France) | Francés (Francia) |
| `preview` | Voici comment sonneront tes pages quand Lingua les lira à voix haute. | (the same) | (the same) |

`windowsVoice` is the language Windows lists under « Ajouter une langue » in that interface
language, with France's region (D1). The preview is spoken in French whatever the interface, as
English's and Spanish's are in theirs, and says « tes » as the French interface does (« Choisis ton
niveau »); the owner's wording settles it (Open question 2). The French module carries the entry
because en and es are typed after it; a French-native reader cannot study French, so no French
interface shows it today — it is there for the override M2 reserves.

`language-labels.ts` keys its table by the languages the catalogue names —
`NamedLanguage = StudiedLanguage | NativeLanguage`, `{ en, es, fr }` — and the four messages the
voice block calls (`languageName`, `noVoiceInstalled`, `windowsVoiceLanguage`, `previewSentence`)
take a `NamedLanguage`. The other messages (level titles, notices, prompts) keep `StudiedLanguage`
and gain French with it in change 52. `settings-view.ts` reads the speaker's language through
`isNamedLanguage(speaker.lang)` instead of casting it to `StudiedLanguage`; a tag the catalogue does
not name (none, with the shipped pairs) leaves the sentences naming it out.

*Rejected — widening `StudiedLanguage` here.* Measured with `tsc --noEmit` on a copy of the
extension with the type widened to `en | es | fr`: two errors, the labels table (wanted) and
`wordpopup.ts:339`, where the card hands its language to the grammar renderer, typed
`StudiedLanguageCode` (`en | es`). Widening would make this change decide what a French card's
grammar says — change 51's question — and would part from change 39's D8, which leaves the type,
`NAMES` and the pairs to change 52 together. With D4, change 51 widens the renderer's code, change
52 widens `StudiedLanguage` and finds the labels table already holding French.

*Rejected — the words in change 52.* The voice block would then throw the day a speaker reads `fr`
(`words(…, "fr")` is `undefined`, its `feminine` a `TypeError`) unless change 52 remembered the
voice's three fields; and this change could not test its block in English and Spanish.

### D5 — An elided piece is heard with the word it leans on; a split word as written

M21 gives each elision piece its own highlight span (change 40), so `l'` of « l'homme » opens a card
of its own, and splits `au`/`aux` into pieces that share their word's span, as Spanish `al`/`del`.
Read alone, 61 of the corpus's 66 elided pieces are spelled or clipped (Context). When the
**speaker reads French**, the card's word button reads instead what the page writes:

1. **An elided piece** — the selection is glued to the next word: in its sentence, the character
   right after it is a letter, or an apostrophe (`'`, `’`) followed by a letter. The button reads
   from the selection's start through the letters that follow, across an apostrophe followed by a
   letter, up to a space, a hyphen, a digit or punctuation: `l'` → « l'homme », `qu'` → « Qu'est »
   (of « Qu'est-ce »), `jusqu'` → « jusqu’à », `j'` → « j’étais ». The test reads the page text after
   the selection, so it holds whether change 40's span for the piece takes its apostrophe (`l'` |
   `homme`) or leaves it to the next one (`l` | `'homme`).
2. **A piece of a split word** — the card's `written` (the range's text, `au`) differs from its
   `surface` (`à`), letter case and Unicode normalisation aside: the button reads `written`.
3. Otherwise the button reads `surface`, as today.

The button is labelled with what it reads where it already says its text: an elided piece differs
from its dictionary form (`l'`, `le`), so the card offers « ▶ L'homme », which reads « L'homme »,
then « ▶ le » — the dictionary form's button, unchanged — then the sentence button, compared with
the heard text (`sameSpokenText`). A split word whose piece is its own dictionary form (`à` of `au`)
keeps one « ▶ Word » / « ▶ Palabra » button, which reads « au ».

The card content gains `selection?: Span | null`, the selection's place in its sentence, which
`selection-card.ts` already holds for the translator (`PageHit.selection`, `SelectionInput.selection`).
The rule is a pure function of the content and the speaker's language, in `wordpopup.ts` beside
`listensFor`; without a place (a range outside any block) the button reads as today.

It is gated on the speaker's language, a plain tag (`speaker.lang === "fr"`), not on the card's
`StudiedLanguage` (D4): English and Spanish cards read exactly as before — `del` still reads its
piece, `add-lingua-dictionary-form-voice`'s non-goal. It is inert until change 40: the baseline
reads `l'homme` as one token (change 39, D2), whose sentence continues with a space.

*Rejected — reading the piece alone.* It is a letter's name (Context); a period after it does not
help (`l'.` is still « elle »).

*Rejected — reading the dictionary form instead.* `le` for `l'` loses the elision the reader
clicked, and « one form, one lemma » (M8) makes `l'` always `le`, even in « l'amie ».

*Rejected — rewriting `written` in the session.* `written` feeds the word's grammar (`wordGrammar`)
and the « forme vue » line; the heard text is the card's business only.

### D6 — Left to a shared follow-up

- **A single-letter word spoken alone** (« à », « y »; Spanish « y »; English "I"): spelled by
  Apple's voices in every language (Context), and fixed — if by appending a comma, as the
  measurements suggest — on Apple's heuristics alone, for every language's cards. It moves English
  and Spanish read-aloud, which this change must not, so it is a change of its own (Open question 3).
  With D5 the French `au` no longer reads « à » alone; the card of the word « à » itself still does.
- **A split word read as written in English and Spanish** (`don't`, `del`): the same follow-up,
  where `add-lingua-dictionary-form-voice`'s non-goal can be revisited for all three languages.
- **Homophones**: `parlent` and `parle` give two buttons that sound alike. Kept, as
  `add-lingua-dictionary-form-voice` kept « solo » / « sólo »: hearing that the written difference
  is silent is what a learner of French needs to hear.

### D7 — No French voice on the device

Every Apple capture lists `Thomas` and `Amélie`, at least in the super-compact quality the
systems ship. The case is Chrome on a Windows not set to French (its own voices are its system's
languages'; Chrome adds Google's — 19 of the 22 voices on the French Windows capture, `Google
français` among them, all remote), Chrome on Linux or ChromeOS, and Firefox for Android until the
reader allows Android's voices. Read-aloud's D8 and D9 apply as they do for English, for the
language the speaker reads: no listen row; Réglages keeps its block and says "No French voice is
installed on this device." (en) / « No hay ninguna voz francesa instalada en este dispositivo. »
(es), its ⓘ tooltip naming "French (France)" / « Francés (Francia) » for Windows and "French" /
« Francés » for macOS's Manage Voices, and offers the online voices, off. Once allowed, `Google
français` speaks, and a French voice installed later takes over.

Tested on a list derived from `chrome-windows.json` without its three local French voices: derived,
not captured. An English or Spanish Windows also lists its own English or Spanish voices, which
count for nothing in French; the owner checks on such a machine if one is at hand (D9).

### D8 — Inert until change 52

- No package lists a French pair (`packs.json`, `check:variants`), so `shippedLanguages`, the
  accepted languages and every surface's reading language exclude French, and no speaker reads
  `fr`; the stored profile's `French` is still dropped by `state/profile.ts` (change 39, D8).
- `PREFERRED_REGIONS.fr` and the French regions act only when French voices are ranked or named,
  that is for a speaker reading `fr`; `Jacques` is a name no English or Spanish voice carries.
- The `french` words are read only by the voice block for a speaker reading `fr`, and by tests.
- The elided-piece rule fires only for a speaker reading `fr`, and only on a piece change 40 makes.
- The French interface is byte for byte: no French string moves, and the French modules gain an
  entry no French-native reader can be shown.

No golden moves: `fr-en.golden`, `en-fr.golden`, `es-fr.golden`, `es-en.golden` and `en-es.golden`
are written by `crates/lingua-wasm` from the core and the packs, which this change does not touch;
the card snapshots (`word-card-es-en.txt`, `word-card-en-es.txt`, `selection-rows-fr.txt`) hold
grammar lines and gloss rows, never a listen row.

### D9 — Tests, gates and the on-device checklist

- `test/baseline/voice-ranking.txt`, recorded on `main` before any edit: for each capture, English
  and Spanish, Android's voices allowed or not and online voices allowed or not, the automatic voice
  and the ranked `voiceURI`s; a spec compares them, and it passes as committed (the 56 rankings
  measured identical).
- `test/speech.spec.ts`, a « French voice from France » block: the automatic French voice per
  capture as the Context's table says; `Jacques` apart on Chrome for macOS (2 ordinary, 16 others);
  `fra-FRA-default` named « France » / "France" / « Francia »; on synthetic lists, `fr-CA` before
  `fr-FR` gives `fr-FR`, an enhanced `fr-CA` before a compact `fr-FR` gives the enhanced one,
  `fra-CAN` before `fra-FRA` gives `fra-FRA`, `fr-BE` and `fr-CH` named in the three languages.
- `test/language-labels.spec.ts`: the four voice messages for French in the three interface
  languages, the en and es ones not French's, the preview identical in all three.
- `test/settings-language.spec.ts`: the voice block mounted with a speaker reading `fr`, in English
  and in Spanish — the voices' labels, the automatic label, and on D7's derived list the no-voice
  sentence, its tooltip and the online voices' switch, off; the existing French assertions pass
  unchanged.
- `test/wordpopup.spec.ts`: D5 with the apostrophe in the piece's span and out of it, a typographic
  apostrophe, « Qu'est-ce », « jusqu’à », the contraction `au`, the sentence button against the heard
  text, no place in the sentence, and the same contents under a speaker reading English or Spanish
  reading as before.
- Gates: `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (coverage), `yarn build`,
  `yarn check:variants`; the bundles' growth against the merge-base, per entry (an estimate before
  measuring: under 1 kB minified for the entries carrying Réglages or the card — three `french`
  entries of seven short strings, three table entries, one function); `git diff --stat
  origin/main -- crates scripts/lingua-data` empty.
- **On devices, in change 52's dogfood** (nothing reads French before it): Chrome, Firefox and
  Safari on macOS — `Thomas` automatic, `Jacques` and the Eloquence voices under "Other voices" on
  Chrome; the iPhone; Firefox for Android — the switch, « — France »; the card of `l'` in « l'homme »
  heard as « l'homme »; Chrome on an English or Spanish Windows if the owner has one — the
  no-voice sentence, the tooltip, `Google français` once allowed.

## Risks / Trade-offs

- [Apple adds French voices of other regions, or renames one] → The picker is the way out, and the
  captured lists make a change of ranking visible in review.
- [A non-Eloquence voice named `Jacques` somewhere] → It ranks lower and stays listed and choosable.
- [Change 40 draws an elision piece otherwise than D5 reads it] → D5 reads the page text after the
  selection, apostrophe in or out of the piece; if change 40 does not split, the rule never fires.
- [A host word with an inner apostrophe or a hyphen] → Letters across an apostrophe followed by a
  letter, stopping at a hyphen: « jusqu'aujourd'hui » whole, « Qu'est » of « Qu'est-ce ». None of
  the corpus's 66 pieces leans on such a chain.
- [Windows' and Android's engines on a lone elided piece] → Not measured; D5 does not depend on
  them: an elided word said with the word it leans on is how French is said.
- [No place in the sentence] → The button reads the piece as today; it happens only for a range
  outside any block.
- [The block's English and Spanish sentences] → They are change 19's sentences with French's words,
  reviewed by the owner (M9) with the French preview.

## Migration Plan

None. Nothing is stored: a French voice choice would live under `voices.fr` in the existing key,
and nothing reads French before change 52. Rollback is a revert.

## Open Questions

For the owner, none blocking:
1. France first and the other regions in the browser's order (Spanish D5's rule), or Belgium and
   Switzerland before Canada? No captured list holds a Belgian or Swiss voice.
2. The French preview: « Voici comment sonneront tes pages quand Lingua les lira à voix haute. »
3. The shared follow-up of D6 — a single-letter word spoken alone, and split words read as written
   in English and Spanish — as one change for the three languages after stage 3, or not at all.
