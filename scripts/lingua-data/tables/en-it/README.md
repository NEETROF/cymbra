# The en→it dictionary tables

The reduced tables Cymbra Lingua's English→Italian data pack is built from: English glossed in Italian,
for Italian speakers studying English — the first pair glossed in Italian (add-lingua-pack-en-it, change
36 of `docs/lingua/italian-programme.md`). They are committed so that every build makes the same pack
with no download, and so that a change to the dictionary is a pull request whose diff shows it. No
extension package carries the pack: `apps/lingua-extension/packs.json` does not list en-it until the
interface speaks Italian (change 45).

This folder holds what belongs to en-it alone: its Italian glosses, its expressions, the parts of speech
of their senses, its notice, manifest and pin. English's own tables — its forms, ranks, levels,
readings, tag pool and dictionary words — are kept once, in `../en/`, and written by en-fr's reduction
alone. en-it's reduction reads them as committed and computes nothing of them.

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → Italian gloss | the Italian Wiktionary's English section, read by the Italian edition's rules (`reduce_edition_it.py`) and en-it's own (`reduce_english_it.py`, change 37); else the Italian translations the English Wiktionary's English entries list; else the English translations the Italian Wiktionary's Italian entries list, read backwards (all CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same |
| `mwe.tsv` | expression → Italian gloss | the same sources, for multi-word headwords, keyed as en-es keys them |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources — the studied side as en-es's notice credits it | — |
| `manifest.json` | the pack's metadata: English glossed in Italian, English's analyser version, and `pack_version` (the snapshot, the rules that reduced it, and a digest of the studied tables it was built on) | — |
| `pin.json` | en-it's raw sources — three files derived from the Italian and English Wiktionaries' dumps, published as `lingua-pack-sources-en-it-2026.10.26` —, the pack these tables build with `../en/`, its base and layer, the rules that reduced them, and the sha256 of each of `../en/`'s tables they were built on | — |

## What is in them

The tables of snapshot `2026.10.26` (`pack_version` `2026.10.26+0a3353d.fb7962e`), first reduced by the
update of that day (`lingua-pack-update` run 40324469757, change 36), then reduced again at the same pin
with change 37's rules (refine-lingua-en-it-glosses, mode `reduce`, round 2; *How it is read* below),
byte for byte the local forecast:

- **Italian glosses for 21,580 lemmas** of English's 40,685 (53.0 %): 54 contractions read as pointers,
  composed by rule (change 37's D11), and as before 13,677 from the Italian
  Wiktionary's English section (55 of them through a pointer's word), 6,125 from the English
  Wiktionary's Italian translations (the direct table, in its order, at most three words per part of
  speech, its disused words and translators' notes out) and 1,724 from the Italian Wiktionary's English
  translations read backwards (the inverted table, the commonest Italian word first, each listed once).
  **13,423 expressions**: 1,380 from the section, 9,905 from the direct table, 2,138 from the inverted.
  A letter glosses no word in either direction. No pivot, no machine translation.
- **The coverage**, held by the extension check's `reduce` job to `FLOORS["en-it"]`, settled by the owner
  on 2026-10-26 before this measurement (`gloss_coverage.py --pair en-it`):

  | Lemmas | the section's definitions | + the direct table | + the inverted table | the floor |
  |---|---|---|---|---|
  | top 5,000 | 84.8 % | 90.6 % | 91.4 % | 89.4 % |
  | top 10,000 | 72.2 % | 81.1 % | 82.8 % | 80.8 % |
  | top 20,000 | 53.0 % | 67.0 % | 70.3 % | 68.3 % |
  | all 40,685 | 33.6 % | 48.7 % | 52.9 % | — |

  With change 37's contractions, **92.1 / 83.1 / 70.6 %** (53.0 % of all): 33 of the 54 among the 5,000.
  Before them, strict — a lemma glossed only through a pointer left out — 91.2 / 82.6 / 70.2 %. en-es's
  are 93.0 / 85.0 / 71.7 %: on English's dictionary words the two are level (en-it glosses 4,235 of the
  4,373 among the 5,000, 7,614 of 8,022, 12,755 of 13,930); the gap is in names, which the Italian
  Wiktionary writes fewer of, and the 24 genitives among the 5,000 (`women's`), which no pair glosses.
- **Where the rows of the commonest come from** (5,000 / 10,000 / 20,000): definitions 4,229 / 7,200 /
  10,565, a pointer's word 10 / 18 / 28, the direct table 292 / 895 / 2,807, the inverted table 39 / 163
  / 665, unglossed 430 / 1,724 / 5,935. **The table share**: of the 8,276 lemmas glossed among the
  10,000 commonest, 12.8 % take their gloss from a table (895 direct, 163 inverted); en-es's, 26.0 %.
- **The pack**: 1,550,149 B (1,553,528 B before change 37); its base, English's, 935,260 B (en-es's byte
  for byte), its layer 615,114 B (618,493 B).

## How it is read: change 37's rules

Change 37 (`refine-lingua-en-it-glosses`) reads the same sources with en-it's own rules,
`reduce_english_it.py`, which no other pair loads; the shared rules and the Italian edition's do not
move. Each rule measured alone against change 36's rows — rows changed among the ranks 1–1,000 /
1,001–5,000 / 5,001–20,000 / beyond, and how many change their first sense:

| Rule | Rows | First sense | Example |
|---|---|---|---|
| D6, names' notes off common words, given names short | 73 (7 / 35 / 29 / 2) | 55 | `summer` « estate », `john` « Giovanni (nome) » |
| D8, heraldry anywhere, tinctures | 60 (12 / 19 / 22 / 7) | 5 | `side` « lato; faccia di una medaglia; allearsi; … », `or` « o, ovvero » |
| D9, a wiki's text | 1 | 0 | `sandbox` « sabbionaia » |
| D13, a state's code beside another sense | 3 (0 / 3 / 0 / 0) | 2 | `ma` « media mobile » |
| D9, an abbreviation's English expansion | 72 (2 / 17 / 35 / 18) | 70 | `uk` « Regno Unito »; `cqc` unglossed alone, kept with D7 |
| D7, a pointer's wording | 11 (2 / 2 / 6 / 1), 2 expressions | 7 | `cellphone` « telefono cellulare », `gun powder` « polvere pirica » |
| D5, a frame before the translation | 59 (19 / 20 / 19 / 1), 7 expressions | 55 | `my` « il mio, la mia, i miei, le mie », `yes` « sì » |
| D10b, split forms | 1 | 1 | `mx` « signor/a » |
| D11, contractions read as pointers | 55 (15 / 19 / 15 / 6), 54 gained | 55 | `you're` « contrazione di you are », `doesn't` « contrazione di does not » |
| D2, the part of speech two signals agree on | 199 (33 / 80 / 69 / 17) | 199 | `make` « fare, creare, fabbricare; …; marca » |
| D3a, the missing main part of speech | 76 (2 / 10 / 46 / 18) | 76 | `rescue` « salvare; salvataggio », `treat` « trattare; … » |
| D3b, a row of descriptions | 45 (2 / 6 / 27 / 10) | 45 | `its` « suo; possessivo di it », `should` « dovere; … » |
| D4, D12, the reviewed rows (round 1: `FIRST_RUN` 34, `CORRECTIONS` 23) | 55 (35 / 13 / 6 / 1) | 49 | `that` « quello, quella; che, il quale… », `can` « potere; latta… », `some` « qualche, alcuni, alcune; del, della » |
| D10a, repeats | 369 (37 / 104 / 141 / 87) | 13 | `national` « nazionale » |

**Round 2** (the specialist's review of the gloss diff, the owner's, 2026-10-26: 15 of the 468 changed
rows of the 5,000 commonest read worse): a reviewed row is never moved by D2 or D3; `FIRST_RUN` gains 13
rows (`cop` « poliziotto », `shine` « brillare, splendere », `cope` « affrontare, far fronte; resistere,
fronteggiare », `express` « esprimere », `joint` « articolazione, giuntura », `stir` « mescolare,
agitare », `forum`, `whenever`, `flash`, `epic`, `piece`, `graph`, `whatsoever`) and `CORRECTIONS` 2
(`spare` « di scorta, di ricambio; risparmiare; ricambio », `intense` « intenso »), every word a
source's; D6 keeps a given name on a word that only names itself (`henry` « Enrico (nome); henry »), D8
reads « armi parlanti » as heraldry (`ash`), D5 « rafforzativo » as a frame (`so`). No narrowing of D2
passed the criteria; D2 is as measured. Against round 1, 20 rows move (4 / 15 / 1 / 0), none for the
worse.

**Together: 1,027 rows change** (152 / 308 / 402 / 165), **613 in their first sense** (103 / 192 / 243 /
75), 69 of the 300 commonest; 54 lemmas gained, none lost; 9 expressions (`dvd players` « lettori DVD »,
`south african` « sudafricano; … »). The reviewed rows' words are their sources' — the section's, a
table's, an expression's —, each op checked against the source it names and named on the reduction's
summary line when it finds nothing (none did). Two kinds of gloss are not a source's, each listed:
- **written by a person**: `lot`'s « (a lot) », after the expression *a lot*'s « molto, parecchio » (task
  0.2, the owner's, 2026-10-26) — `lot` « molto, parecchio (a lot); lotto; destino; … »;
- **generated by rule** (task 0.1, the owner's): the 55 contractions D11 writes, the frame « contrazione
  di » the section's own, the English full form the rule's — `could've`, `doesn't`, `hadn't`, `hasn't`,
  `he'd`, `he'll`, `he's`, `how'd`, `how's`, `i'd`, `i'll`, `it'd`, `it'll`, `let's`, `might've`,
  `must've`, `she'd`, `she'll`, `she's`, `should've`, `that'd`, `that'll`, `that's`, `there'd`,
  `there'll`, `there're`, `there's`, `they'd`, `they'll`, `they're`, `they've`, `this'll`, `we'd`,
  `we'll`, `we're`, `we've`, `what'd`, `what'll`, `what're`, `what's`, `when's`, `where'd`, `where's`,
  `who'd`, `who'll`, `who're`, `who's`, `who've`, `why'd`, `why's`, `would've`, `you'd`, `you'll`,
  `you're`, `you've`.

## Known defects

Change 36's list (its D10), after change 37:
- **Fixed**: names' notes on common words, state codes beside another sense, function words described
  (`my`, `its`, `should`; `will` opens on its auxiliary's description, partly right), labels before a
  colon (frames; `so` in round 2), heraldic senses anywhere (`ash`'s canting arms in round 2), English
  text (`intense` in round 2), unglossed contractions, the pages copied from another word
  (`some`, `trash`), the false friends the review named, nine misspellings, English expansions and text
  (`uk`, `media`), repeats and split forms.
- **Left, each with its reason** (change 37's D13): the 18 state codes that are the code's only sense
  (a code's meaning); capital openings (`Lei`, brands and acronyms are right); Italian inflected forms in
  the tables (`polar` « polare; polari »: telling them apart needs `tables/it/forms.tsv`, which en-it's
  pin does not record); `ashamed` « confuso », `reckless` « impavido; spericolato », `sahara`
  « sahariano », `mr`'s description, `oh`, `aw`; `hurt`'s « torta (di azzurro) », a heraldic sense
  neither tagged nor worded so; `very`'s suffixes among its items.
- **Read worse by a rule**, within its measured 4 to 1: the specialist's 15 of the 5,000 are reviewed
  rows since round 2 (`former`, `piece`, `henry`, `joint`, `express`, `whenever`, `flash`, `forum`,
  `cop`, `epic`, `spare`, `shine`, `graph`, `cope`, `whatsoever`, and the sampled `stir`); left past
  them: D2's `bugger`; D3's `queer`, `woodland`, `flirt`, `nag`, `woof`, `jive`, `swat`, `nigga`; D9's
  `maga`, and `bbc`, `cnn`, `cbs`, `nbc`, `espn`, `hbo`, `noaa` keeping only a description.
- **Left after round 2**: `play`, `kind`, `party`, `early`, `court` open on a minor sense of the section's
  order; the words no source gives, not written (the owner's): `cope`'s « farcela », `spare`'s « libero »,
  the contractions' Italian.
- **The misspelling census** (`../../measure/en_it_misspellings.py`, against `tables/it/forms.tsv`,
  under 1.5 Zipf): 312 words of 231 rows of the 5,000 before change 37, 289 of 217 after, none new —
  nearly all real Italian, which no rule may take.

## A sample

100 glossed lemmas of the 20,000 commonest, drawn by the Italian pairs' seed (20261024), each marked by
its source: 77 definitions, 21 from the direct table, 2 from the inverted table. The same lemmas as
change 36 drew them, their glosses as change 37 reads them (nine rows moved: `connection`, `rival`,
`skull`, `tide`, `mx`, `duff` without its name's note, the given names `austin`, `caroline`, `adrian` short).

| Rank | Lemma | Source | Gloss |
|---:|---|---|---|
| 124 | `while` | definition | « mentre; momento, po' di tempo » |
| 219 | `everything` | definition | « ogni cosa, tutto » |
| 421 | `political` | definition | « politico » |
| 539 | `star` | definition | « corpo celeste: astro, stella; personaggio famoso: divo, stella, star, vedette » |
| 918 | `movement` | definition | « movimento; variazione » |
| 966 | `anymore` | definition | « (non) più; al giorno d'oggi » |
| 1089 | `agreement` | definition | « accordo, patto, contratto; accordo, concordia, comunione d'intenti, opinione condivisa; concordanza grammaticale (ad es. fra le varie parti del discorso) » |
| 1347 | `copy` | definition | « copia, riproduzione; copia, imitazione; copia (singola pubblicazione stampata); copiare, riprodurre, fare una copia; copiare, imitare » |
| 1373 | `connection` | definition | « connessione, collegamento, coincidenza; congiuntura, coincidenza » |
| 1559 | `behavior` | definition | « comportamento » |
| 1797 | `perfectly` | definition | « perfettamente, alla perfezione; completamente, totalmente » |
| 1819 | `newspaper` | definition | « giornale, quotidiano » |
| 1873 | `heaven` | definition | « paradiso; cielo » |
| 1996 | `interior` | direct table | « interiore, interno » |
| 2296 | `occur` | definition | « accadere; succedere; capitare; trovarsi » |
| 2378 | `mystery` | definition | « mistero » |
| 2388 | `rome` | definition | « Roma » |
| 2753 | `austin` | definition | « Agostino (nome) » |
| 2967 | `sufficient` | definition | « sufficiente » |
| 3023 | `stability` | definition | « stabilità » |
| 3078 | `awkward` | definition | « goffo, impacciato, sgraziato; delicato, difficile, problematico, complicato, arduo; imbarazzante, scomodo, difficile; maldestro; a disagio; incapace, inetto; scomodo, poco maneggevole; pericoloso » |
| 3098 | `framework` | definition | « struttura » |
| 3182 | `corps` | definition | « forza armata » |
| 3443 | `climb` | definition | « arrampicata, scalata, salita; scalare; arrampicarsi, salire, montare; salire, andare verso l'alto (anche in senso figurato); arrampicare (praticare l'arrampicata sportiva) » |
| 3716 | `rival` | definition | « rivale, avversario, antagonista, concorrente; avversario, rivale, antagonista, della concorrenza, opposto, contrastante; rivaleggiare, competere; rivaleggiare con, essere alla pari di, uguagliare » |
| 3837 | `preliminary` | direct table | « preliminare » |
| 4090 | `ridge` | definition | « cresta » |
| 4104 | `babe` | definition | « infante; ragazza » |
| 4254 | `skull` | definition | « teschio, cranio » |
| 4539 | `tide` | definition | « marea; tempo (oggi usato solo nella liturgia per le feste cristiane, ad esempio Christmastide e Eastertide) » |
| 4587 | `o'clock` | definition | « in punto; utilizzato per indicare che è il momento giusto per un determinato atto o evento » |
| 4617 | `bennett` | definition | « nome proprio di persona inglese maschile » |
| 4745 | `possess` | direct table | « possedere, avere » |
| 4774 | `caroline` | definition | « Carolina (nome) » |
| 5061 | `berry` | definition | « bacca » |
| 5173 | `oath` | definition | « giuramento; bestemmia, imprecazione » |
| 5215 | `adrian` | definition | « Adriano (nome) » |
| 5370 | `zombie` | definition | « (occultismo) zombi; zombi, morto che cammina; babbeo, zuccone » |
| 5657 | `hostage` | direct table | « ostaggio » |
| 5800 | `choir` | definition | « coro; coro (porzione della chiesa dove siedono i cantori); coro, schiera angelica » |
| 5803 | `confession` | definition | « confessione, ammissione (di una colpa o simili); confessione (sacramento); confessione, credo religioso » |
| 6245 | `mafia` | definition | « mafia » |
| 6281 | `vodka` | definition | « vodka » |
| 6612 | `mechanic` | definition | « meccanico » |
| 6950 | `volcano` | definition | « vulcano » |
| 7230 | `oriental` | definition | « orientale » |
| 7275 | `yen` | definition | « yen » |
| 7500 | `theorem` | definition | « teorema » |
| 7564 | `gravel` | definition | « ghiaia; renella, calcolosi » |
| 7858 | `dammit` | definition | « accidenti, maledizione, dannazione » |
| 8061 | `spur` | definition | « sprone; sperone » |
| 8144 | `jp` | definition | « abbreviazione di justice of the peace (giudice di pace) » |
| 8425 | `sis` | definition | « sorella » |
| 8479 | `ebola` | direct table | « ebola » |
| 8690 | `waiter` | definition | « cameriere » |
| 8957 | `announcer` | definition | « annunciatore » |
| 9012 | `installment` | definition | « puntata; rata » |
| 9539 | `lax` | direct table | « lasso, lassista, allentato » |
| 9580 | `seismic` | direct table | « sismico » |
| 9693 | `quota` | direct table | « quota, contingente » |
| 9750 | `ashore` | direct table | « a riva, verso riva » |
| 9796 | `fauna` | definition | « fauna » |
| 9811 | `harden` | direct table | « indurirsi » |
| 9891 | `wrestler` | definition | « wrestler » |
| 10130 | `regression` | direct table | « regressione » |
| 10648 | `groundbreaking` | definition | « rivoluzionario, pionieristico » |
| 10724 | `tram` | definition | « tram; trasportare in un tram o su una linea tranviaria » |
| 11136 | `obsessive` | definition | « ossessivo » |
| 11140 | `pendant` | definition | « pendente, ciondolo, pendaglio, pendant; pennone » |
| 11201 | `biotechnology` | definition | « biotecnologie » |
| 11219 | `cosplay` | definition | « cosplay » |
| 11598 | `sahara` | inverted table | « sahariano » |
| 12160 | `duff` | definition | « fondoschiena; polvere di carbone » |
| 12222 | `payload` | direct table | « carico, carico utile » |
| 12568 | `mx` | direct table | « signor/a » |
| 13793 | `martini` | direct table | « martini » |
| 13799 | `misinformation` | inverted table | « disinformazione » |
| 13966 | `hover` | direct table | « librarsi, volteggiare, aggirarsi » |
| 14557 | `emoji` | direct table | « emoji » |
| 14728 | `coronavirus` | direct table | « coronavirus » |
| 14965 | `gallant` | definition | « coraggioso, valoroso, eroico; onorabile, nobile; galante, cavalleresco, cortese; sfarzoso, splendido, lussuoso » |
| 15158 | `ethereal` | definition | « etereo » |
| 15506 | `abstinence` | definition | « astinenza » |
| 15514 | `arduous` | definition | « arduo, duro, faticoso » |
| 15527 | `boa` | direct table | « boa » |
| 15745 | `custard` | definition | « crema » |
| 16388 | `crossword` | definition | « cruciverba » |
| 16558 | `blunder` | definition | « errore grossolano » |
| 16705 | `tantrum` | direct table | « bizza, capriccio » |
| 16803 | `femininity` | definition | « femminilità » |
| 17063 | `momentous` | definition | « fondamentale, cruciale; epocale, assai importante » |
| 17171 | `audiobook` | direct table | « audiolibro » |
| 17278 | `irc` | definition | « Internet Relay Chat » |
| 17532 | `fresco` | definition | « affresco » |
| 17533 | `frigate` | direct table | « fregata » |
| 17822 | `morgue` | definition | « obitorio » |
| 18628 | `standstill` | definition | « blocco, arresto » |
| 19586 | `mutton` | definition | « carne di montone » |
| 19722 | `authoritarianism` | definition | « autoritarismo » |
| 19757 | `conundrum` | definition | « enigma » |

## Updating

Never by hand: `lingua-pack-update` (mode `update`, pair `en-it`; `keep=en` reuses the English edition's
recorded file on a routine update, sparing the 3 GB dump), or `scripts/lingua-data/build.sh --update
en-it <out>` with Python 3.12 and `requirements-reduce.txt`. After a change to en-it's rules, mode `reduce`
reads the pinned sources again; each rule of `reduce_english_it.py` can be measured alone with
`reduce-en-it.py --rules <rule>`. An update of en-fr that moves `../en/`
brings en-it along, reduced again from its own pin after en-fr. Re-bless `crates/lingua-wasm/tests/
baseline/en-it.golden` and the extension's `test/baseline/word-card-en-it.txt` in the same pull request.
