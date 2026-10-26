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
| `gloss.tsv` | lemma → Italian gloss | the Italian Wiktionary's English section, read by the Italian edition's rules (`reduce_edition_it.py`); else the Italian translations the English Wiktionary's English entries list; else the English translations the Italian Wiktionary's Italian entries list, read backwards (all CC BY-SA 4.0 + GFDL, through kaikki) |
| `senses.tsv` | lemma → part of speech of each run of its gloss's senses | the same |
| `mwe.tsv` | expression → Italian gloss | the same sources, for multi-word headwords, keyed as en-es keys them |
| `NOTICE` | the attribution stack, embedded in the pack: both sides' sources — the studied side as en-es's notice credits it | — |
| `manifest.json` | the pack's metadata: English glossed in Italian, English's analyser version, and `pack_version` (the snapshot, the rules that reduced it, and a digest of the studied tables it was built on) | — |
| `pin.json` | en-it's raw sources — three files derived from the Italian and English Wiktionaries' dumps, published as `lingua-pack-sources-en-it-2026.10.26` —, the pack these tables build with `../en/`, its base and layer, the rules that reduced them, and the sha256 of each of `../en/`'s tables they were built on | — |

## What is in them

The tables of snapshot `2026.10.26` (`pack_version` `2026.10.26+8612b67.fb7962e`), reduced by the update
of that day (`lingua-pack-update` run 40324469757), byte for byte the forecast the floor was fixed on
(design D6, *Implementation*):

- **Italian glosses for 21,526 lemmas** of English's 40,685 (52.9 %): 13,677 from the Italian
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
  | top 5,000 | 84.8 % | 90.6 % | **91.4 %** | 89.4 % |
  | top 10,000 | 72.2 % | 81.1 % | **82.8 %** | 80.8 % |
  | top 20,000 | 53.0 % | 67.0 % | **70.3 %** | 68.3 % |
  | all 40,685 | 33.6 % | 48.7 % | 52.9 % | — |

  Strict — a lemma glossed only through a pointer left out — 91.2 / 82.6 / 70.2 %. en-es's are 93.0 /
  85.0 / 71.7 %: on English's dictionary words the two are level (en-it glosses 4,235 of the 4,373 among
  the 5,000, 7,614 of 8,022, 12,755 of 13,930); the gap is in contractions (24 of the 81 among the 5,000)
  and names, which the Italian Wiktionary writes fewer of.
- **Where the rows of the commonest come from** (5,000 / 10,000 / 20,000): definitions 4,229 / 7,200 /
  10,565, a pointer's word 10 / 18 / 28, the direct table 292 / 895 / 2,807, the inverted table 39 / 163
  / 665, unglossed 430 / 1,724 / 5,935. **The table share**: of the 8,276 lemmas glossed among the
  10,000 commonest, 12.8 % take their gloss from a table (895 direct, 163 inverted); en-es's, 26.0 %.
- **The pack**: 1,553,528 B; its base, English's, 935,260 B (en-es's byte for byte), its layer 618,493 B.

Known defects, for change 37 (`refine-lingua-en-it-glosses`, design D10): names' notes on common words
(`summer` « estate; nome proprio di persona inglese femminile »), US state codes on two-letter tokens,
function words described rather than glossed (`my`, `would`), a label before a colon, capital openings,
heraldic and obsolete senses shown last, Italian inflected forms in the tables (`polar` « polare;
polari »), unglossed contractions, pages copied from another word in the source (`some`, `trash`).

## A sample

100 glossed lemmas of the 20,000 commonest, drawn by the Italian pairs' seed (20261024), each marked by
its source: 77 definitions, 21 from the direct table, 2 from the inverted table.

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
| 1373 | `connection` | definition | « congiuntura, coincidenza » |
| 1559 | `behavior` | definition | « comportamento » |
| 1797 | `perfectly` | definition | « perfettamente, alla perfezione; completamente, totalmente » |
| 1819 | `newspaper` | definition | « giornale, quotidiano » |
| 1873 | `heaven` | definition | « paradiso; cielo » |
| 1996 | `interior` | direct table | « interiore, interno » |
| 2296 | `occur` | definition | « accadere; succedere; capitare; trovarsi » |
| 2378 | `mystery` | definition | « mistero » |
| 2388 | `rome` | definition | « Roma » |
| 2753 | `austin` | definition | « nome proprio di persona maschile, corrispondente all'italiano Agostino » |
| 2967 | `sufficient` | definition | « sufficiente » |
| 3023 | `stability` | definition | « stabilità » |
| 3078 | `awkward` | definition | « goffo, impacciato, sgraziato; delicato, difficile, problematico, complicato, arduo; imbarazzante, scomodo, difficile; maldestro; a disagio; incapace, inetto; scomodo, poco maneggevole; pericoloso » |
| 3098 | `framework` | definition | « struttura » |
| 3182 | `corps` | definition | « forza armata » |
| 3443 | `climb` | definition | « arrampicata, scalata, salita; scalare; arrampicarsi, salire, montare; salire, andare verso l'alto (anche in senso figurato); arrampicare (praticare l'arrampicata sportiva) » |
| 3716 | `rival` | definition | « avversario, rivale, antagonista, della concorrenza, opposto, contrastante; rivale, avversario, antagonista, concorrente; rivaleggiare, competere; rivaleggiare con, essere alla pari di, uguagliare » |
| 3837 | `preliminary` | direct table | « preliminare » |
| 4090 | `ridge` | definition | « cresta » |
| 4104 | `babe` | definition | « infante; ragazza » |
| 4254 | `skull` | definition | « teschio, cranio; nell'araldica inglese il teschio usato come cimiero in un achievement indica che il defunto era l'ultimo della linea di discendenza » |
| 4539 | `tide` | definition | « marea, bassa; tempo (oggi usato solo nella liturgia per le feste cristiane, ad esempio Christmastide e Eastertide) » |
| 4587 | `o'clock` | definition | « in punto; utilizzato per indicare che è il momento giusto per un determinato atto o evento » |
| 4617 | `bennett` | definition | « nome proprio di persona inglese maschile » |
| 4745 | `possess` | direct table | « possedere, avere » |
| 4774 | `caroline` | definition | « nome proprio di persona femminile, corrispondente all'italiano Carolina » |
| 5061 | `berry` | definition | « bacca » |
| 5173 | `oath` | definition | « giuramento; bestemmia, imprecazione » |
| 5215 | `adrian` | definition | « nome proprio di persona maschile, corrispondente all'italiano Adriano » |
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
| 12160 | `duff` | definition | « nome proprio di persona inglese maschile; fondoschiena; polvere di carbone » |
| 12222 | `payload` | direct table | « carico, carico utile » |
| 12568 | `mx` | direct table | « signorə, signor, a » |
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
en-it <out>` with Python 3.12 and `requirements-reduce.txt`. An update of en-fr that moves `../en/`
brings en-it along, reduced again from its own pin after en-fr. Re-bless `crates/lingua-wasm/tests/
baseline/en-it.golden` and the extension's `test/baseline/word-card-en-it.txt` in the same pull request.
