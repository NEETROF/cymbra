# The marks measurement

How often the translated sentence marks the right words for the reader's selection, per pair: en-fr
and es-fr (release-lingua-spanish-translation), es-en and en-es
(measure-lingua-translation-matrix-marks), fr-en and fr-es (add-lingua-french-translation), it-fr
(measure-lingua-it-fr-translation-marks), it-en (measure-lingua-it-en-translation-marks), it-es
(measure-lingua-it-es-translation-marks). es-fr, fr-es, it-fr and it-es go through English — es-en
then en-fr, fr-en then en-es, it-en then en-fr, it-en then en-es — so their marks cross two
alignments. A measurement is a pair's — es-fr's says nothing of es-en, measured on its own route and
judged in English (generalise-lingua-translation-routes-by-pair D6), nor fr-es's of fr-en's, nor
it-fr's or it-es's of it-en, the it-en model alone, measured on its own and judged in English.
Decision D2 of the Spanish programme fixed what the result decides before anything was measured:

- **≥ 90 % correct and ≤ 25 % withheld:** marked;
- **75–90 %:** unmarked;
- **below that:** withdrawn.

The corpus is per **studied language** — a selection is of the text it was made in, and es-en is
measured on the same Spanish selections as es-fr, en-es on the same English ones as en-fr, fr-es on
the same French ones as fr-en, it-en and it-es on the same Italian ones as it-fr — while the results and the
judgments are filed per **pair**, and the judge reads the pair's **native language**, the one its
route translates into. Every studied language's selections come from the same sentences by one rule
(add-lingua-french-translation D4): French, added, moved no English or Spanish selection, nor any
committed result; Italian, added after it (measure-lingua-it-fr-translation-marks D1), moved no
English, Spanish or French selection, nor any committed result.

| File                   | What it holds                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `select_corpus.mjs`    | The selection rule: every tenth PUD sentence, the same in every language, one word each (noun, verb, noun, adjective in turn)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `pud.mjs`              | PUD English, Spanish, French and Italian (French at add-lingua-french-forms-tables' pin, Italian at `master`'s head on 2026-10-22, the corpus's own pin), fetched at pinned commits and checked by sha256 (CC BY-SA, never committed)                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `corpus.json`          | The 400 selections, 100 per studied language (`lang`: en, es, fr, it), each step's four from one sentence: sentence id, token, word, offsets                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `engine.mjs`           | The pinned engine in Node with a route's models built as `engine-worker.ts` builds them: one markup string in, its translation out — shared by the harness and the soak                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `measure.mjs`          | The loop: each selection marked exactly as `relay.ts` marks it — tagged sentence, fragment alone, reconciled — and the experiment's mark; a trap answered per request, as the extension answers it                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `stop-words.mjs`       | The experiment's stop words per native language (fr, en, es): the function words skipped when the gloss is looked for in the sentence; the harness picks the route's native language's                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `../measure_marks.mjs` | The harness: the pinned engine and the catalogue's models, the corpus of a pair's studied language through `measure.mjs` over that pair's route; writes `results-<pair>.jsonl`                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `../soak_engine.mjs`   | The soak (harden-lingua-translation-engine D4): a pair's route over the same corpus, reporting what trapped, the time per sentence and the memory high-water mark                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `results-<pair>.jsonl` | Per selection: the translated sentence (`translation`) with its marks bracketed (`shown`), the fragment's own translation (`alone`, null when it trapped twice, `""` for an empty fragment or one the route translated as nothing), and the experiment's mark (`gloss`, null when the pair's `tables/<pair>/gloss.tsv` is not committed or holds no gloss); a sentence the engine trapped on twice is `trapped: true` with no translation — `results-en-fr.jsonl`, `results-es-fr.jsonl`, `results-es-en.jsonl`, `results-en-es.jsonl`, `results-fr-en.jsonl`, `results-fr-es.jsonl`, `results-it-fr.jsonl`, `results-it-en.jsonl`, `results-it-es.jsonl` |
| `judged-<pair>.tsv`    | Every mark judged correct, wrong or withheld, with a reason for each wrong one — `judged-en-fr.tsv`, `judged-es-fr.tsv`, `judged-es-en.tsv`, `judged-en-es.tsv`, `judged-fr-en.tsv`, `judged-fr-es.tsv`, `judged-it-fr.tsv`, `judged-it-en.tsv`, `judged-it-es.tsv`                                                                                                                                                                                                                                                                                                                                                                                       |
| `tier.mjs`             | D2's first tier and the count of a judged file, as read above; `test/translate-marks.spec.ts` holds `MARKED_PAIRS` to it                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |

## Running it again

One pair per run, named as the catalogue routes it; the corpus is its studied language's, the stop
words its native language's. The catalogue routes twelve pairs — the six of the matrix, and Italian's
six since `add-lingua-italian-translation-models` (it-en, en-it, it-fr, it-es, fr-it, es-it) — and
the harness measures the nine below: the Italian selections are in the corpus since the Italian
programme's row 19 (`measure-lingua-it-fr-translation-marks`), which measured it-fr on them; row 24
(`measure-lingua-it-en-translation-marks`) measured it-en on the same selections, row 29
(`measure-lingua-it-es-translation-marks`) it-es, and the pairs glossed in Italian need the Italian stop words, row 38's
(`measure-lingua-en-it-translation-marks`) — each Italian pair is measured by its own row (19, 24, 29,
38, 49, 53) before it ships:

```bash
node tool/assemble_model_site.mjs /tmp/models                                                  # or read models.cymbra.app: omit --models
node --experimental-strip-types tool/measure_marks.mjs --pair en-fr --models /tmp/models      # ~15 s, writes results-en-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-fr --models /tmp/models      # writes results-es-fr.jsonl
node --experimental-strip-types tool/measure_marks.mjs --pair es-en --models /tmp/models      # the Spanish selections, judged in English
node --experimental-strip-types tool/measure_marks.mjs --pair en-es --models /tmp/models      # the English selections, judged in Spanish
node --experimental-strip-types tool/measure_marks.mjs --pair fr-en --models /tmp/models      # the French selections, judged in English
node --experimental-strip-types tool/measure_marks.mjs --pair fr-es --models /tmp/models      # the same, through English, judged in Spanish
node --experimental-strip-types tool/measure_marks.mjs --pair it-fr --models /tmp/models      # the Italian selections, through English, judged in French
node --experimental-strip-types tool/measure_marks.mjs --pair it-en --models /tmp/models      # the same, through the it-en model alone, judged in English
node --experimental-strip-types tool/measure_marks.mjs --pair it-es --models /tmp/models      # the same, it-en then en-es, judged in Spanish
node tool/marks/select_corpus.mjs                                                             # only to rebuild corpus.json; deterministic
```

A request that traps the engine (a `WebAssembly.RuntimeError`, `isTrap`) is asked once more on a
fresh engine, as the extension asks it after a respawn (harden-lingua-translation-engine D2): the
instance that trapped is poisoned and put down. A sentence that traps twice is recorded
`trapped: true` — the reader would get no translation — and judged `withheld`, `trapped twice`; a
fragment that traps twice leaves the sentence's own marks unreconciled, as the extension shows
them (so does an empty fragment, never asked: `alone` is `""` for it, null for one that trapped
twice; and so does a fragment the route translates as nothing, `""` too, which the run counts apart
by its fragment — it-es's k 58). The run says how many engines it built and which ids trapped. The fresh-engine retry was
exercised once against the real engine on 2026-10-08 — a factory throwing a trap-shaped error from
the first instance's first request made the es-en run build a second engine, which answered every
request to the committed figures, byte for byte — and remains fake-tested in the suite
(`test/translate-measure.spec.ts`).

## Soaking a route

Beside the measurement, by hand and never in CI (the programme's M25): the real engine through a
pair's route over the same corpus, to find the inputs that trap it — measured through the en-es
model in the study (2026-10-06), a trap poisons the instance for every model built after it — and
what a run costs in time and memory. It is how en-es is tried before it ships (change 35), now
that change 25 pins the en-es route in the catalogue, and fr-en and fr-es before change 52, now that
change 50 routes them, and the en-it model through en-it, fr-it and es-it, now that the Italian
programme's change 2 (`add-lingua-italian-translation-models`) pins it-en 2.0 and en-it 2.1 and
routes Italian's six pairs, and the it-en model through it-en and it-fr, over the Italian selections
row 19 (`measure-lingua-it-fr-translation-marks` D2) added to the corpus, which closes change 2's
D4, and it-es's chain, it-en's English fed to en-es, over the same selections (row 29,
`measure-lingua-it-es-translation-marks` D1) — their runs are recorded in `TRANSLATION.md`; a pair the catalogue does not route is refused.
A studied language the corpus held no selection of would be reported, not refused: « 0 selections
run » and « tool/marks/corpus.json holds no … selection: nothing was soaked », exit status 0.

```bash
node --experimental-strip-types tool/soak_engine.mjs --pair en-fr --models /tmp/models            # ~15 s
node --experimental-strip-types tool/soak_engine.mjs --pair es-fr --models /tmp/models            # two models, through English
node --experimental-strip-types tool/soak_engine.mjs --pair es-fr --models /tmp/models --limit 10 # the first ten selections
node --experimental-strip-types tool/soak_engine.mjs --pair en-es --models /tmp/models --isolate  # each sentence in a child process, ~45 s
node --experimental-strip-types tool/soak_engine.mjs --pair fr-es --models /tmp/models --isolate  # the French selections, through English
node --experimental-strip-types tool/soak_engine.mjs --pair en-it --models /tmp/models --isolate  # the English selections, through en-it
node --experimental-strip-types tool/soak_engine.mjs --pair fr-it --models /tmp/models --isolate  # the French selections, fr-en then en-it
node --experimental-strip-types tool/soak_engine.mjs --pair es-it --models /tmp/models --isolate  # the Spanish selections, es-en then en-it
node --experimental-strip-types tool/soak_engine.mjs --pair it-en --models /tmp/models --isolate  # the Italian selections, through it-en
node --experimental-strip-types tool/soak_engine.mjs --pair it-fr --models /tmp/models --isolate  # the same, it-en then en-fr
node --experimental-strip-types tool/soak_engine.mjs --pair it-es --models /tmp/models --isolate  # the same, it-en then en-es
```

Each selection goes through the engine as `relay.ts` sends it: the sentence with the selection
tagged — timed, since it is what the card waits for — then the fragment alone. The report says how
many were translated, which trapped (by corpus id, never by text), the median, mean and maximum
time per sentence, and the process's resident high-water mark (`process.resourceUsage().maxRSS`).
In Node a trap throws a `WebAssembly.RuntimeError` the tool catches, but the instance is poisoned
from then on, so without `--isolate` the run **stops at the first trap and says so**; with it, each
sentence runs in a child process of its own and the run goes on to the end — slower, since each
child loads the engine and the route again. A child that ends without its report (a trap while the
route is built, a crash, a signal) counts as trapped for its id; one past 120 s is counted apart,
as timed out. The exit status is 1 when anything trapped or timed out.

What the memory figure is, and is not. It is Node's RSS — the parent's, or the highest child's
under `--isolate`. The ≈ 322 MiB of es-fr's two models (≈ 195 MiB for one) was measured in the
browser's worker, and Node's figure is not like for like: read it for a run's order of magnitude
and for growth across the corpus, not against the worker's. The soak does not check the two-model
bound of `engine-worker.ts` either: a run loads one route and deletes nothing — and a deletion
would not show in RSS anyway, since a wasm instance's linear memory never shrinks; the bound caps
growth, with the freed blocks reused by the next model built. The catalogue lists six models
(en-fr, es-en, en-es, fr-en, it-en, en-it). The matrix's shipped pairs need two at most per native
language — en-fr and es-en for French, es-en and fr-en for English, en-es and fr-en for Spanish — and
the eviction runs when the native language changes while the worker lives
(add-lingua-french-translation D3), which production meets since changes 34, 35 and 52 shipped the
matrix's pairs. With Italian's six routes, the pairs glossed in each native language need three —
en-fr, es-en and it-en for French; es-en, fr-en and it-en for English; en-es, fr-en and it-en for
Spanish; en-it, fr-en and es-en for Italian — still loaded two at a time, a route's: a reader who keeps
their native language makes the worker delete a model only when a third pair's route needs the room
(add-lingua-italian-translation-models D2), which a French-native reader of English, Spanish and
Italian meets since the Italian programme's change 20 (enable-lingua-italian) shipped it-fr.

## Judging

The criteria were written before the run (release-lingua-spanish-translation D3, then
measure-lingua-translation-matrix-marks D4 for any native language; add-lingua-french-translation D6
added the French examples, before fr-en and fr-es were run, and moved no verdict;
measure-lingua-it-fr-translation-marks D3 added the Italian examples below, taken from the corpus,
before it-fr was run, and moved no verdict; measure-lingua-it-en-translation-marks D2 added their
English counterparts, anchored on verdicts already committed, before it-en was run, and moved no
verdict; measure-lingua-it-es-translation-marks D3 added their Spanish counterparts, before it-es was
run, and moved no verdict), and a pair is judged in its **native language** — French for en-fr, es-fr
and it-fr, English for es-en, fr-en and it-en, Spanish for en-es, fr-es and it-es. Through English,
fr-es's mark crosses two alignments, as es-fr's does, and so do it-fr's, it-en then en-fr, and
it-es's, it-en then en-es: each is judged against the rendering the reader sees, in the pair's native
language, whatever the English one in between. it-en's mark crosses one, the it-en model's alone:
Italian is judged in English as French is (fr-en).

- **Correct:**
  - the marks cover the rendering, in the pair's native language, of the selected word in that
    sentence;
  - they may include the article, preposition or auxiliary it carries, and may be split;
  - an expression's rendering counts, as « sin embargo » → « Cependant » (es-fr), « sin embargo »
    → « However » (es-en), « gave up » → « se rindió » (en-es), « A travers » → « Throughout »
    (fr-en), « Par conséquence » → « Como resultado » (fr-es).
- **Wrong:** the marks cover another word, only a function word, or only part of a compound; or a
  word the translation omitted, so that the marks cover something else.
- **Withheld:** no mark — a sentence the engine trapped on twice included (`withheld`,
  `trapped twice`).

**Italian selections, judged in French** (measure-lingua-it-fr-translation-marks D3): the rules above,
with these examples from the corpus's Italian steps, written before it-fr was run.

- **A one-word past rendered as a compound one:** Italian's passato remoto is one word, French's
  rendering through English often a passé composé — steps 37 « vidi », 65 « conobbe », 69
  « rilasciò », 81 « accusò ». For step 65, a mark on « a connu » or « connu » is correct, on « a »
  alone it is not.
- **A subject the route adds:** Italian drops its subject, English and French write one — step 37's
  « A Pyongyang vidi alcune persone » alone: a mark on « ai vu » or « vu » is correct, on « j' » alone
  it is not.
- **A word after a fused preposition:** step 6's « Nel corso della storia » (also 22, 82, 91): a mark
  on « cours » or « Au cours » is correct, on « Au » alone it is not; an expression's rendering
  (« Tout au long de ») counts.
- **A word after an elided piece:** step 86's « l'unico nel Governo », the only Italian selection of
  its kind (French had 8): the mark renders `unico` (« le seul »), never the article alone.
- **Gender or number English lost:** a French rendering whose gender or number the route lost still
  counts as the word's rendering — step 79 « nemiche », step 71 « stereotipata ».

**Italian selections, judged in English** (measure-lingua-it-en-translation-marks D2): the rules
above, with these examples from the same Italian steps, anchored on verdicts already committed,
written before it-en was run.

- **A subject the model adds:** step 25 « Ha detto all'Andrew Marr Show »: a mark on « told »,
  « said » or « He told » is correct, on « He » alone it is not — one of es-en's two wrong marks, k 25,
  marked « he » for « said »; es-fr's k 25 « il a déclaré » was judged correct for carrying the subject
  the rendering adds.
- **A compound past rendered as a simple one:** the passato remoto is one word in English as in
  Italian (es-en: k 65 « met », k 69 « released », k 81 « accused »), so the French case above does
  not arise; the reverse does — an Italian compound past's participle (steps 5, 13, 17, 25, 31) often
  renders as one English simple past: a mark on that word is correct, on « has » or « were » alone it
  is not.
- **A word after a fused preposition:** step 6's « Nel corso della storia » (also 22, 82, 91): a mark
  on « course » or « In the course » is correct, on « In » alone it is not; an expression's rendering
  (« Throughout ») counts.
- **A word after an elided piece:** step 86's « l'unico nel Governo »: a mark on « only », « only
  one » or « the only (one) » is correct — the article comes from « l' » (es-en k 86 « the only
  one ») —, on « the » alone it is not.
- **A clitic pronoun beside the selected verb** (steps 9, 31, 33, 97; none reflexive): step 33 « Gli
  viene riconosciuto » → « He is credited »: « credited » or « is credited » is correct, « He » alone
  is not (es-en k 33 marked « credited »); step 31 « l'ho scelta » → « I chose it »: « chose » is
  correct, « it », « her » or « I » alone is not; step 97 « si incontrano » → « met »: « each other »
  alone is not; step 9 « ci fossero » → « there were »: « were » or « be » is correct, « there » alone
  is not.
- **One Italian word, an English compound:** step 54 « Il capodanno cinese » → « Chinese New Year »
  and step 78 « sveglia » → « alarm clock »: the whole compound is correct; « Year », « alarm » or
  « clock » alone is part of a compound, which the criteria count wrong — es-en's k 54 (« Year »
  judged correct) does not carry over, its Spanish selection being « Año » alone.
- **Gender or number English does not mark, and a numeral beside the word:** step 79 « le due fazioni
  nemiche » → « the two enemy factions »: a mark on « enemy » (or « hostile », « rival ») is correct,
  on « two » it is not, as es-en's k 79 was judged; step 71 « stereotipata » → « stereotypical » or
  « stereotyped ».

**Italian selections, through English, judged in Spanish** (measure-lingua-it-es-translation-marks
D3): the rules above, with these examples from the same Italian steps, written before it-es was run.
it-es goes through it-en then en-es, as fr-es goes through fr-en then en-es. Each line is read against
three references: it-fr's verdict and it-en's verdict on the same selection, and `results-en-es.jsonl`'s
Spanish for the English sentence — a reference on idiom only, which holds only where the Italian word
means what the English one does (not on the 34 steps where the Italian selection is another word, nor
on 2 « volta », 32 « punto di partenza », 54 « capodanno », 70 « sceneggiatore », 82 « agli
sgoccioli », 98 « sospensione »). A verdict departing from both it-fr's and it-en's gives its reason.

- **A word the route leaves untranslated:** a mark on a word left in English or copied through from
  Italian is wrong (« left in English », « left in Italian »), unless Spanish uses the word too (a
  loanword such as « Premium »); a subject pronoun the route adds, marked alone, is wrong.
- **A compound past rendered by one Spanish preterite** (steps 5, 13, 17, 25, 31): 5 « aumentaron »,
  « crecieron »; 13 « publicaron », « se publicaron », « fueron publicadas »; 17 « comentó »; 25
  « dijo », « Le dijo », « declaró »; 31 « elegí », « la elegí » — and a Spanish compound past, with or
  without its « ha » — are correct; « ha », « han », « fueron », « se » or « Le » alone, or an added
  « Él » or « Yo » alone, is not (en-es k 13, 17, 25 correct; es-en k 25 « he » wrong).
- **The passato remoto, the same rule:** 37 « vi », 65 « conoció », 69 « lanzó » or « publicó », 81
  « acusó »; the tense never decides (fr-es k 65 « conoce » counted correct).
- **A word after an articulated preposition** (6, 22, 82, 91): step 6 « Nel corso della storia »:
  « curso », « el curso », « transcurso » or « A lo largo (de) » is correct, « En », « A » or « lo »
  alone, or « historia », is not; 22 « Nel migliore dei casi »: « mejor » or « A lo sumo », not « En »,
  « el » or « casos »; 82 « agli sgoccioli »: « (en las) últimas », « últimos coletazos » or « camino
  (de salir) », not « en », « las » or « está »; 91 « degli ultimi »: « (los) últimos », not « de los »
  or « sus » (fr-es k 6, 91; en-es k 82 « camino »).
- **A word before a fused « della / del / dei / dell' »** (0, 3, 11, 16, 40, 50, 51, 55, 63, 74, 76,
  87, 96, 98): the mark may carry the « de (la) » or « del » beside it, never only that, never the next
  noun — 0 « parte » or « gran parte », not « transición » or « de la » (a withheld is likely, as
  fr-en's and fr-es's k 0); 3 « compañía », « empresa » or « sociedad », not « surcoreana »; 50
  « caída », not « dinastía »; 96 « parte(s) » (before « dell' »), not « del »; 76 « (los) agentes »,
  not « del ».
- **A word after an elided piece:** step 86 « l'unico nel Governo »: « único » or « el único », not
  « el » alone or « Gobierno » (a withheld is likely, es-en k 86); step 29, after the truncation
  « Fa' »: « caer », « Deja caer », « Suelta » or « Tira », not « Deja » alone or « micrófono »
  (fr-es k 29).
- **A clitic beside the selected verb** (9, 31, 33, 97): 9 « ci fossero »: « hubiera », « hubiese »,
  « haya » or « existieran », not « le gustaría » or « que » alone; 33 « Gli viene riconosciuto »:
  « atribuye », « reconoce » or « Se le atribuye », not « Se », « le » or « Se le » alone, nor
  « mérito »; 97 « si incontrano »: « se conocieron », « se reunieron » or « se encuentran », never
  « se » alone (en-es and fr-es k 97); a Spanish enclitic (« enseñarlo ») is one word.
- **An Italian « si »:** step 61's selection is « riguarda », beside an impersonal « si considera »:
  « En cuanto a », « En lo que respecta (a) » or « Con respecto a » is correct, « (se) considera » is
  not; the « se » Spanish adds to a passive is carried with the verb, never marked alone.
- **Gender and number never decide:** 79 « enemigas », « enemigos » or « rivales », not « dos »
  (es-en's « two » and es-fr's « deux » were wrong); 71 « estereotipada »; 93 « capitaneado » or
  « comandado » (fr-es k 93); 27 « idénticos »; 67 « similar(es) »; 96 « partes »; 59 « buenos » or
  « bien », not « resultados »; 15 « principal » or « mayor », not « su » alone (fr-en k 15).
- **One Italian word, a Spanish compound or expression:** 54 the whole « Año Nuevo » (en-es's « Nuevo »
  alone, judged correct there, does not carry over); 78 « despertador » or « alarma »; 83 « canción »,
  not « de éxito »; 57 « conforman » or « constituyen », not « que »; 77 « avanza » or « anda »; 89
  « ganar » or « vencer », not « pudo » alone; and the expressions — 58 « luego », 42 « Debido a », 80
  « A diferencia », 46 « En realidad » or « De hecho » (« actualmente » doubtful), 60 « momento », 30
  « vez » or « la última vez » (« la » or « última » alone is wrong, fr-es k 30), 2 « Una vez ».
- **A loose rendering the mark follows:** correct, flagged doubtful — en-es's k 18, 70 and 77 as a
  class, not as verdicts. The Italian words differ from the English ones there, so 70 « guionista »
  (« sceneggiatore ») and 98 « suspensión » (« sospensione ») are expected plainly correct.

A doubtful it-es line says so in `engine_reason`; where counting the doubtful lines wrong would take
it-es below the first tier, they are committed `wrong` with their reason, so the committed figures are
the strict ones wherever it matters (measure-lingua-it-es-translation-marks D4).

One judge reads a pair, as the shipped pairs were judged — it-es's too, checked by a blind second
judge where its figures fell under 96 % (measure-lingua-it-es-translation-marks, Risks); a doubtful
line says so in `engine_reason`, and the owner may re-judge any line in the pull request — the figures are
recomputed from the committed judgments (`tier.mjs`). The rates read D2 as the study did:

- correct is the share of the shown marks;
- withheld is the share of all 100 selections.

## Results

Engine pinned by `engine-pin.json`; models `en-fr` base-memory 2.0, `es-en` 2.0, `en-es` 2.1,
`fr-en` 2.0 and `it-en` 2.0 (`model-manifest.json`). The pairs listed in `MARKED_PAIRS`
(`src/translate/markup.ts`) are the judged pairs on the first tier here, and
`test/translate-marks.spec.ts` holds the list to the judged files: every listed pair judged, a judged
pair listed exactly when its committed judgment reaches the tier (measure-lingua-it-fr-translation-marks
D4). en-fr and es-fr were measured on 2026-10-05, judged in French; es-en and en-es on 2026-10-08
(measure-lingua-translation-matrix-marks), judged in English and in Spanish; fr-en and fr-es on
2026-10-09 (add-lingua-french-translation), judged in English and in Spanish; it-fr on 2026-10-23
(measure-lingua-it-fr-translation-marks), judged in French; it-en on 2026-10-25
(measure-lingua-it-en-translation-marks), judged in English; it-es on 2026-10-25
(measure-lingua-it-es-translation-marks), judged in Spanish — one judge each, it-es's checked by a
blind second one.

|                                      | Correct (of shown marks) | Withheld | D2                                                                              |
| ------------------------------------ | ------------------------ | -------- | ------------------------------------------------------------------------------- |
| **es-fr, engine (pivot es→en→fr)**   | **89 / 90 — 98.9 %**     | **10 %** | **First tier: marked**                                                          |
| en-fr, engine                        | 96 / 97 — 99.0 %         | 3 %      | (reference; the study found 83 / 87)                                            |
| **es-en, engine, judged in English** | **94 / 96 — 97.9 %**     | **4 %**  | **First tier: marked (shipped since change 34)**                                |
| **en-es, engine, judged in Spanish** | **96 / 96 — 100 %**      | **4 %**  | **First tier: marked (shipped since change 35)**                                |
| **fr-en, engine, judged in English** | **95 / 96 — 99.0 %**     | **4 %**  | **First tier: marked (shipped since change 52)**                                |
| **fr-es, engine (pivot fr→en→es)**   | **90 / 91 — 98.9 %**     | **9 %**  | **First tier: marked (shipped since change 52)**                                |
| **it-fr, engine (pivot it→en→fr)**   | **93 / 97 — 95.9 %**     | **3 %**  | **First tier: marked (shipped since the Italian programme's change 20)**        |
| **it-en, engine, judged in English** | **94 / 97 — 96.9 %**     | **3 %**  | **First tier: marked (shipped since the Italian programme's change 25)**        |
| **it-es, engine (pivot it→en→es)**   | **90 / 95 — 94.7 %**     | **5 %**  | **First tier: marked (listed, inert until the Italian programme's change 30)**  |
| es-fr, gloss-located (experiment)    | 76 / 78 — 97.4 %         | 22 %     | —                                                                               |
| en-fr, gloss-located (experiment)    | 76 / 79 — 96.2 %         | 21 %     | —                                                                               |
| es-en, gloss-located (experiment)    | 75 / 78 — 96.2 %         | 22 %     | —                                                                               |
| en-es, gloss-located (experiment)    | —                        | —        | left empty: no `tables/en-es/gloss.tsv` yet (22)                                |
| fr-en, gloss-located (experiment)    | —                        | —        | left empty: no French gloss table yet (48)                                      |
| fr-es, gloss-located (experiment)    | —                        | —        | left empty: no French gloss table yet (49)                                      |
| it-fr, gloss-located (experiment)    | —                        | —        | left empty: no `tables/it-fr/gloss.tsv` at the run (the Italian programme's 16) |
| it-en, gloss-located (experiment)    | —                        | —        | left empty: no `tables/it-en/gloss.tsv` yet (the Italian programme's 22)        |
| it-es, gloss-located (experiment)    | 69 / 70 — 98.6 %         | 30 %     | —                                                                               |

What these numbers say:

- **es-fr is on D2's first tier.** One wrong mark in 90 (« dos facciones enemigas » marked « deux »
  instead of « ennemies »).
- **It holds under a stricter reading:** if the five expression marks counted as wrong, es-fr would
  still score 84 / 90, 93 %.
- **es-en and en-es are on the first tier too**, at the figures the study predicted (94 / 96 and
  96 / 96). es-en's two wrong marks, by the TSV's `k`: « declaró » marked « he » instead of « said »
  (k 25), and « enemigas » marked « two » instead of « enemy » (k 79) — the mark es-fr inherited
  through the pivot. Its four withheld are three expressions the translation reshaped (« dejar
  caer », « dar por hecho », « consiguió vencer ») and one hyphenated compound
  (« franco-monegasco »). en-es has no wrong mark; its four withheld are two of en-fr's own
  (« officer », « the only one » — the same English selections), « alarm clock » → « despertador »
  and « captained », unmarked. Three en-es lines (k 18, k 70, k 77) follow a loose rendering, e.g.
  « edge » → « ventaja »: the mark sits on the rendering the reader sees, so they count correct and
  are flagged `doubtful:` in `engine_reason`.
- **No request trapped** on es-en or en-es: one engine built per run, no `trapped: true` line and no
  unreconciled fragment — the trapped ids this README would list are none (`TRANSLATION.md`, where
  change 25 recorded the en-es soak, says the same).
- **fr-en and fr-es are on the first tier**, at the figures the study predicted within its ± 6 %
  (94 / 96 and 90 / 91). fr-en's one wrong mark: « intérêt principal » marked « their » instead of
  « main » (k 15); its four withheld are « la plus grande partie de » → « most of » (k 0),
  « Les orages » (k 48), « Autrement dit » → « In other words » (k 57) and « Pour l'instant » →
  « For now » (k 79). fr-es's one wrong mark: « la dernière fois » marked « la » instead of « vez »
  (k 30); its nine withheld are fr-en's four and five the pivot loses (« discordance » rendered
  « dispensación », « au mieux », « Au troisième siècle » → « En el siglo III », « stéréotypé »,
  « représente »), as es-fr withholds 10 where es-en withholds 4.
- **The fragment alone mattered on both routes:** it moved the engine's mark to the right word on 2
  fr-en lines (« last » → « time », « hit » → « song ») and 4 fr-es lines (« su » → « principal »,
  « protección » → « limitada », « es … de » → « canción », « , se » → « se reunieron »), and trimmed
  it on 3 and 4 others (« would want » → « want »).
- **Three lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`: fr-en
  « limitée » → « for limited » (k 39) and « traces » → « The blood trails » (k 92), fr-es
  « voudrait » → « el … querría” » (k 25). Counted wrong, fr-en is 93 / 96 and fr-es 89 / 91: the
  tier holds either way.
- **The 8 French selections right after an elided piece** (`d'enseigner`, `l'armée`, …) are correct
  on both pairs but « l'instant », withheld on both. No request trapped on either French pair: one engine
  built per run, every fragment reconciled.
- **it-fr is on the first tier** through two models, it-en then en-fr: 93 of its 97 shown marks
  correct, 3 withheld — listed in `MARKED_PAIRS`, inert until the Italian programme's change 20
  (`enable-lingua-italian`) listed it-fr in `packs.json`, marked since. Its four wrong marks, by `k`: « isole »
  marked « granit » instead of « îles » (k 38), « battaglia » marked « au » alone instead of
  « combat » (k 52), « comprendono » marked « qui » (k 57, as en-fr's « make » was) and « rilasciò »
  marked the auxiliary « a » alone instead of « a sorti » (k 69). Its three withheld: « gran parte della »
  → « une grande partie de » (k 0), « Per quanto riguarda » → « Quant à » (k 61) and « stereotipata »,
  rendered « stéréotypée » unmarked (k 71).
- **The criteria's Italian cases** (Judging): the one-word pasts are marked on their compound
  rendering — « ai vu » (k 37, carrying the « j' » the route adds), « a rencontré » (k 65), « a
  accusé » (k 81) — but for k 69's auxiliary alone; after a fused preposition, « Tout | long »
  (k 6, « Tout au long de »), « mieux » (k 22) and « derniers » (k 91) are correct, k 82 doubtful;
  after the one elided piece, « seul » (k 86); gender or number never decided a verdict
  (« ennemies », k 79).
- **Four it-fr lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`:
  « discordanza » → « discorde » (k 14) and « coppia » → « paire » (k 20), loose renderings the
  mark follows; « Fa' cadere » → « Lâche le » (k 29), the mark carrying « micro »'s article; and
  « agli sgoccioli » rendered « son infarest" » (k 82), a garbled non-word the mark follows, as
  es-fr's k 15 (« indifféronteurs ») was judged. Counted wrong, it-fr is 89 / 97, 91.8 %: the tier
  holds either way.
- **No request trapped on it-fr**: one engine built, every fragment reconciled. The fragment alone
  moved the engine's mark to the right word once (« dilemmes » → « similaires », k 67) and trimmed
  it twice (k 7, k 96). Run twice, the results were byte for byte the same.
- **it-en is on the first tier** through its one model: 94 of its 97 shown marks correct, 3 withheld —
  listed in `MARKED_PAIRS`, inert until the Italian programme's change 25
  (`enable-lingua-italian-for-english-speakers`) listed it-en in `packs.json`, marked since. Its three wrong marks, by
  `k`: « comprendono », rendered « understand » in the wrong sense, marked « who » (k 57, as it-fr's
  « qui » and en-fr's « make » were); « sveglia » marked « alarm » alone, half of « alarm clock »
  (k 78); and « agli sgoccioli », rendered « in its infarest » — a garbled non-word —, marked « its »
  alone, a possessive the model added, not the rendering (k 82). Its three withheld: « gran parte della » → « much of »
  (k 0), « isole » → « islands » unmarked (k 38) and « Per quanto riguarda » → « As for » (k 61).
- **The criteria's Italian cases in English** (Judging): the subject the model adds is left out on
  « told » (k 25) and carried by « I saw » (k 37) and « he played » (k 73); the compound pasts are
  marked on their one English word — « grew » (k 5), « posted » (k 13), « commented » (k 17),
  « told » (k 25), « chose » (k 31); after a fused preposition, « Throughout » (k 6), « best » (k 22,
  « At best ») and « last » (k 91) are correct, k 82 wrong; after the elided piece, « only one »
  (k 86); beside a clitic, « to be » (k 9, « there » left out), « chose » (k 31), « credited » (k 33)
  and « meet » (k 97); of the English compounds, « New Year » (k 54) is whole and « alarm » (k 78) is
  not; and « enemy » (k 79) is marked where es-en marked « two ».
- **Two it-en lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`:
  « discordanza » → « discord » (k 14) and « coppia » → « pair » (k 20), loose renderings the mark
  follows — it-fr's k 14 and k 20 too. Counted wrong, it-en is 92 / 97, 94.8 %: the tier holds either
  way.
- **No request trapped on it-en**: one engine built, every fragment reconciled. The fragment alone
  moved the engine's mark to the right word once (« in » → « battle », k 52) and trimmed it once
  (« originally from » → « originally », k 7). Run twice, the results were byte for byte the same.
- **it-es is on the first tier** through two models, it-en then en-es: 90 of its 95 shown marks
  correct, 5 withheld — listed in `MARKED_PAIRS`, inert until the Italian programme's change 30
  (`enable-lingua-italian-for-spanish-speakers`) lists it-es in `packs.json`. Its five wrong marks, by
  `k`: « fossero » marked the personal « a » of « regula a los agentes » instead of « hubiera » (k 9);
  « volta » marked « última » instead of « vez » (k 30, as fr-es's « la » was); « comprendono »,
  rendered « entienden » in the wrong sense, marked « que » (k 57, as it-fr's « qui » and it-en's
  « who »); « agli sgoccioli », rendered « en su infarest », marked « su » alone (k 82, as it-en's
  « its »); and « si incontrano » marked the name « Nicolai » instead of « se reúnen » (k 97). Three
  of the five — k 9, 30 and 97 — are the pivot's: the tag lands on a neighbour in Spanish, and the
  fragment alone (« Fueron », « Tiempo », « Conocer ») is not in the sentence to move it, where it-fr
  and it-en were right. Its five withheld: it-fr's and it-en's « gran parte della » (k 0) and « Per
  quanto riguarda » (k 61), it-en's « isole » (k 38), and two the pivot loses — « Gli viene
  riconosciuto » → « Se le atribuye » (k 33) and « sveglia » → « despertador » (k 78, as en-es
  withholds « alarm clock » → « despertador »).
- **The criteria's Italian cases in Spanish** (Judging): the compound pasts are marked on their one
  preterite — « crecieron » (k 5), « publicaron » (k 13), « comentó » (k 17), « dijo » (k 25, without
  « Le »), « elegí » (k 31) — and so is the passato remoto — « vi » (k 37), « conoció » (k 65),
  « lanzó » (k 69), « acusó » (k 81); after an articulated preposition, « lo largo » (k 6, « A lo
  largo de »), « mejor » (k 22) and « últimos » (k 91) are correct, k 82 wrong; before a fused one,
  « compañía » (k 3), « caída » (k 50), « los agentes » (k 76) and « parte de » (k 96), never the next
  noun, k 0 withheld; after the elided piece, « único » (k 86), and after the truncation « Deje caer »
  (k 29); beside a clitic, k 9 and k 97 are wrong and k 33 withheld; « Año Nuevo » (k 54) is whole and
  « despertador » (k 78) unmarked; gender never decided (« enemigas », k 79; « capitaneado », k 93).
  No word was left in English or Italian, and no added subject was marked alone.
- **Four it-es lines are doubtful**, counted correct and flagged `doubtful:` in `engine_reason`:
  « discordanza » → « discordia » (k 14) and « coppia » → « par » (k 20), loose renderings the mark
  follows — it-fr's and it-en's k 14 and k 20 too; « Nel corso » → « A lo largo de » marked « lo
  largo » without « A » (k 6); and « agenti » → « oficiales », a calque of « officers » (k 75). Counted
  wrong, it-es is 86 / 95, 90.5 %: the tier holds either way.
- **A blind second judge** read `results-it-es.jsonl` and these criteria, without the first judge's
  verdicts, at the results' commit: the two agreed on all 100 lines, engine and experiment alike; k 6
  and k 75 are the second judge's doubts, taken up.
- **No request trapped on it-es**: one engine built. One fragment came back as nothing — « seguito »
  (k 58), asked alone, translated as an empty string — so its sentence's marks stand unreconciled,
  « Más ⟦tarde⟧ », as the extension would show them; every other fragment was reconciled. The fragment
  alone moved the engine's mark to the right word three times (« su » → « mayor », k 15; « en » →
  « batalla », k 52; « donde » → « interpretó », k 73). Run twice, the results were byte for byte the
  same.
- **Sampling error:** about ±6 % at 100 selections.

**The experiment** marks the first word of the sentence that matches the pack's gloss of the
selected word, in the pair's native language, its stop words left out (`stop-words.mjs`).

- It is about as precise as the engine, in English as in French (es-en: 75 / 78).
- It withholds about one selection in five: a translation often picks a synonym the gloss does not
  list.
- Where the engine withholds, it would have found 7 of Spanish's 10 missing marks for es-fr, and
  2 of es-en's 4.
- fr-en's and fr-es's columns are empty: change 43 commits fr-en's gloss table empty, and the harness
  reads a table only when it holds a gloss. Once changes 48 and 49 commit French's glosses, a later
  run may fill them without touching the engine's columns.
- it-fr's columns are empty: no `tables/it-fr/gloss.tsv` was committed at the run (the Italian
  programme's change 16 commits it); a later run may fill them the same way.
- it-en's columns are empty too: no `tables/it-en/gloss.tsv` was committed at the run (the Italian
  programme's change 22 commits it).
- it-es's columns are filled, from `tables/it-es/gloss.tsv` and `tables/it/forms.tsv` as change 28
  left them (`e63e7d7f4`): 69 of 70 correct, 30 % withheld — its one wrong mark « Año » alone, half of
  « Año Nuevo » (k 54). Where the engine withholds, it would have found 3 of it-es's 5 missing marks
  (k 0, 38, 78).

A later change may try it as a fallback; nothing here ships it.
