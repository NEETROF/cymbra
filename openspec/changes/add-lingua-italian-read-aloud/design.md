# Design

## Context

See proposal.md — Why. Where things stand:

- `apps/lingua-extension/src/reading/speech.ts` ranks a studied language's voices: an enhanced or
  premium voice first; within a tier, `PREFERRED_REGIONS[lang]` (`en: ["us", "gb"]`, `es: ["es"]`,
  `fr: ["fr"]`), the rest in the browser's order; Apple's novelty, Eloquence and legacy voices
  deprioritised by name (`DEPRIORITISED_NAMES`, a suffixed name read by its base: `Eddy (Italian
  (Italy))` is `Eddy`) or by family (`DEPRIORITISED_FAMILY`); a voice speaks when it is on the device
  (`localService`), or, once allowed, when it is Android's own. `THREE_LETTER_LANGUAGES` knows `ita`;
  `THREE_LETTER_REGIONS` knows the English-, Spanish- and French-speaking regions, `che`
  (Switzerland) among them, not `ita`.
- The captured Italian voices (`test/fixtures/voices/`): Chrome on macOS `Alice` (it-IT, Apple
  compact), the eight Eloquence voices suffixed, `Google italiano` (not local); Chrome on Windows
  `Google italiano` alone; Firefox on macOS `Alice`; Safari on macOS, iOS and the Simulator `Alice`
  (super-compact, the default); Firefox for Android `ita-ITA-default`, `ita-ITA-f00`. No `it-CH`.
- `src/i18n/{fr,en,es}/languages.ts` name the studied languages — name, the forms the sentences
  need, `windowsVoice`, `preview` — and hold the level titles (`levelTitleEstimated`,
  `estimatedLevelsNote`) and `noVoiceInstalled`; the French module is the type the others follow.
- `src/reading/wordpopup.ts`'s `heardWord` (add-lingua-french-read-aloud D5): when the speaker reads
  French, a single word's button reads an elided piece glued to its word (`gluedWord`: from the
  selection through the letters that follow, across an apostrophe and a letter, up to a space, a
  hyphen, a digit or punctuation), else a split word's piece as the page writes it (`written`),
  never an elided piece typed alone (`ELIDED`), else the form seen. Every other language reads the
  form seen.
- Change 6: an elided piece is a token with its own span holding the apostrophe, read as its word
  (`l'` → `lo`, `dell'` → `dello`); the articulated prepositions of `a`, `da`, `in`, `su`, `con` are
  split, sharing the span; a truncation is one token spanning the apostrophe, its text the word it
  stands for (`po'` → `poco`, `da'` → `dai`). Change 10: an apostrophe for an accent is read as the
  accented word (`perche'` → `perché`), on the rest after the elisions (`c'e'` → `ci` + `è`).
- **Measured with Alice** (macOS `say -v Alice -o …`, the audio files compared by sha256 and size,
  2026-10-22): `l'` alone 8,048 B (a clipped « l »; « lo » 14,714 B); « l'uomo » = « luomo », « dell'acqua »
  = « dellacqua »; « po' » = « po », « da' » = « dà », « di' » = « di » = « dì »; « perche' » =
  « perche » ≠ « perché »; « citta' » = « citta » ≠ « città »; « piu' » = « piu » = « più »; « c'e' » =
  « ce » ≠ « c'è »; « E' vero » = « È vero »; `è` alone 49,558 B, 1.03 s, as `à`, `ò`, `ì`, `ù`
  (the letter named); « è. », « è! », « È. » 12,180 B, 0.18 s, as « e ». Google's, Android's and
  Windows' Italian voices were not measured (none on this machine); the French change measured
  Apple's alone too.

## Goals / Non-Goals

**Goals:**

- A voice of Italy by default wherever one exists; Italian voices named in the interface language.
- An Italian card heard as the page writes it, the analysis's reading where the page's writing
  misleads a voice.
- English, Spanish and French byte for byte.

**Non-Goals:**

- Widening the extension's studied languages (change 20).
- An accent setting; Swiss Italian beyond ranking it after Italy.
- Measuring non-Apple voices on devices this change cannot reach (task 4.1's on-device checklist).

## Decisions

### D1 — Italy first, within a tier

`PREFERRED_REGIONS.it = ["it"]`. Every captured list already gives a voice of Italy first where it
gives any; the entry pins it against a device that lists `it-CH` first (an Android engine, a future
Apple voice). A downloaded enhanced or premium voice still comes first. No deprioritised name to add:
Italian's Eloquence set is the eight suffixed names the set already holds (`Eddy` … `Shelley`), and
Chrome lists no bare-named one, unlike France's `Jacques`.

### D2 — Italy in three letters

`THREE_LETTER_REGIONS.ita = "it"`: `ita-ITA-default` is placed in Italy and named after it — the
region names come from `Intl.DisplayNames`, as the others' do. Switzerland (`che`) is there; San
Marino and the Vatican have no voice and are not added.

### D3 — The Italian words in the catalogue, read through the speaker's language

`src/i18n/fr/languages.ts` gains `italian`: `name` « italien », `of` « d'italien », `the`
« l'italien », `masculine` « italien », `feminine` « italienne », `windowsVoice` « Italien
(Italie) », `preview` « Ecco come suoneranno le tue pagine quando Lingua le leggerà ad alta voce. »
(in Italian whatever the interface, *tu* as the other previews, I6); `en` « Italian » for every form,
"Italian (Italy)"; `es` « italiano », « italiano », « el italiano », « italiano », « italiana »,
« Italiano (Italia) ». The voice block reads them through the speaker's language, as French's D4
does, without widening `StudiedLanguage`; the same entry gives the level titles « Niveau d'italien
estimé », "Estimated Italian level", « Nivel de italiano estimado » that change 14's estimated
levels need. `ownNames` (the languages' own names) is not touched: « Italiano » as a native or
interface language is changes 32 and 33's.

### D4 — An Italian piece is heard as the page writes it, the analysis's reading where the page misleads

`heardWord` opens its gate to Italian (`fr` or `it`) and, for Italian, reads in order:
1. **an elided piece glued to its word** — `gluedWord`, French's (« l'uomo », « dell'acqua », « un'amica »,
   « Quest'anno »), measured right with Alice; when the word it leans on is a spelling read for an
   accent (change 10), the glued text ends with that token's accented text (« c'e' » → « c'è »:
   measured, « c'e' » is « ce »);
2. **a split articulated preposition's piece** — the word as written (`written`: « al », « nel »),
   French's `au`;
3. **a truncation** — as written (« po' », « da' », « fa' », « va' », « sta' », « di' », « mo' »,
   « be' »): the token's text (`poco`, `dai`) is another word on the page; measured, « po' » is
   « po » and « da' » « dà »;
4. **a word read for an accent** — the token's text (`perché`, `città`, `è`): written, « perche' »
   and « citta' » are read as the bare words with the wrong stress (measured);
5. **a single accented vowel** — the text followed by a full stop (« è. »): alone, Alice names the
   letter (1.03 s); with the stop it reads the vowel (0.18 s, as « e »); the label stays « è »;
6. else the form seen.

`ELIDED` is any word ending in an apostrophe, so it already keeps an Italian elided piece typed alone
(« l' uomo » reads `lo`, as French's `l’ homme` reads `Le`) — and it would also catch a truncation:
the Italian branch tells a truncation apart first, by change 6's eight truncations (`po'`, `mo'`,
`fa'`, `va'`, `da'`, `sta'`, `di'`, `be'`, compared in lowercase with the typographic apostrophe
read as the straight one), and a spelling read for an accent by its token's text ending in an
accented vowel where the written word ends in the bare vowel and an apostrophe, before `ELIDED` is
read. A card read in English, Spanish or French reads byte for byte as before.

*Rejected — reading every Italian card as written.* « perche' » and « citta' » lose their stress,
« c'e' » becomes « ce », and a lone `è` is a letter's name — the four cases the analysis already
reads right.

### D5 — What cannot move

The voice-ranking snapshot (`voice-ranking.txt`) pins English's, Spanish's and French's rankings over
every captured list and setting before the change; it passes unchanged, and gains Italian's.
`heardWord`'s French branch and every other language's are untouched, held by their tests.

### D6 — Inert until change 20

No surface reads Italian before `enable-lingua-italian` (change 20) lists an Italian pair and widens
the studied languages: the ranking, the names, the catalogue and the card's rule are exercised by
unit tests and snapshots only. Change 20's dogfood hears them on real devices.

### D7 — Tests and the on-device checklist

- `test/speech.spec.ts`: Italian's ranking over the seven captures (Alice first on Chrome for macOS,
  the Eloquence voices under other voices, `Google italiano` not offered; `ita-ITA-default` placed
  in Italy); a synthetic list with `it-CH` before `it-IT`; the voice-ranking snapshot.
- `test/wordpopup.spec.ts` (or where French's D5 is tested): each case of D4 — `l'` in « L'uomo »,
  `a` of « al », « po' », « perche' », `c'` of « c'e' », « è », an elided piece typed alone, and
  English, Spanish and French cards unchanged.
- `test/i18n` checks: the three `languages` modules hold `italian`, every form filled, the preview
  Italian.
- On-device checklist for change 20's dogfood: the measured cases heard with Alice on macOS and iOS,
  with Google's voice on Chrome, Android's on Firefox for Android, and an installed Windows Italian
  voice on Chrome for Windows.

## Risks / Trade-offs

- [Non-Apple voices read differently] → measured on Apple's alone, as French's; change 20's dogfood
  hears the others; D4's rules each read the analysis's text or the page's, which every voice reads.
- [« è. » heard with a pause] → 0.18 s, the vowel's length; the label shows « è ».
- [A truncation written for an accented word] (`da'` for *dà*) → read as written, which the voice
  reads as « dà » (measured), so the page's intent is heard either way.

## Migration Plan

Nothing deploys: the change ships with the extension's next release and is inert until change 20.
Rollback is a revert.

## Effort

1–1.5 ideal days, against the programme's 1–2: the ranking and the region 0.25; the catalogue 0.25;
`heardWord`'s Italian branch 0.25–0.5; their tests and the snapshot 0.25–0.5.
