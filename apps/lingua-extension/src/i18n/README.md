# The interface's copy — one catalogue, one module per surface and language

Every text the extension shows lives here, once per language: `fr/<surface>.ts`, `en/<surface>.ts`,
`es/<surface>.ts`, `it/<surface>.ts`, one module per surface — 24 of them: `popup`, `hud`, `drawer`,
`card`, `selection`, `sidepanel`, `review`, `stats`, `settings`, `studied-languages`,
`native-language`, `colours`, `display`, `translation`, `account-setting`, `account`,
`account-errors`, `onboarding`, `reader`, `sync`, `languages`, `grammar`, `app`, `badge` (`SURFACES`,
`index.ts`). Italian's renderer of readings, `it/grammar.ts`, is add-lingua-italian-card-wording's
(the Italian programme's change 34). The French module is the source;
the English, Spanish and Italian ones are typed after it (`export const popup: typeof fr = { … }`), so
a key missing in a translation does not compile (`yarn typecheck`). `test/i18n.spec.ts` checks the rest at
runtime: no empty entry, every slot taken, nothing left in French outside the texts that are the same
in every language. `test/lint-copy.spec.ts` refuses a French literal in `src/` outside this
directory, except in the files its baseline names — the surfaces still holding their copy, which the
changes moving them took off the list: `localise-lingua-reading-surfaces` (14),
`localise-lingua-settings` (15), `localise-lingua-review-stats` (16),
`localise-lingua-account-onboarding` (17), `generalise-lingua-card-wording` (18,
`reading/grammar-labels.ts`) and `add-lingua-native-language-labels` (19,
`analyzer/language-labels.ts`). The baseline is empty since.

A language's name — « anglais », "Spanish", « español » and their inflections, in any interface language — is
named in `<language>/languages.ts` alone, and the surfaces ask `src/analyzer/language-labels.ts` for it in the
interface language they read (`add-lingua-native-language-labels`); `test/lint-language-labels.spec.ts` refuses
one in a string literal anywhere else in `src/`, the other catalogue modules included, the engine's enum names
(`state/profile.ts`) being its one exception. The level scale is `languages.levelScale`: « CEFR » in French and
English, « MCER » in Spanish (M19), « QCER » in Italian (I6).

A copy site with no French literal is beyond the lint: a format whose output is French. The one the
inventory found, `src/reading/speech.ts`'s `voiceLabel` — a voice's region named through
`Intl.DisplayNames(["fr"])` — now goes through `regionName(language, code)` and the
`settings.voiceLabel(name, place)` slot message (`localise-lingua-settings`), so a voice is named in
the interface language; a new format goes through a helper of `index.ts` the same way.

The interface language is the reader's native language (`language.ts`: `interfaceLanguage(area)`,
`fr` when the key is absent, and when it cannot be read — it never rejects); `index.ts` holds the
helpers — `plural`, `formatNumber`, `formatCount`, `formatPercent`, `formatDate`, `regionName`,
`fillPage`, `fillPageInLanguage`, `renderAround`, `slot` and `fillSlots` — and maps no surface, so
importing it costs an entry nothing. A surface imports its own three modules and picks by the
language; Réglages, mounted by three hosts, picks its seven (`settings`, `studied-languages`,
`colours`, `display`, `translation`, `account-setting`, `sync`) in `reading/settings-copy.ts` and hands
each block its own, which holds its French module as its default. The native language's choice picks
`native-language` in its own view (`reading/native-language-view.ts`), in Réglages as in the onboarding
and the popup: a bundle built without the choice carries none of it.

## The grammar renderers

`grammar` is the one surface whose modules are code, not texts (generalise-lingua-card-wording D2):
the word card's grammar lines are built from parts — names, articles, elisions, a joining, an order
of tenses — that no slot message can carry. `src/reading/grammar-description.ts` describes a form in
no language (the readings as the engine's tags, merged by tag; what a card names at all; how a line
is composed), and each language's `grammar.ts` exports `grammar: GrammarRenderer` (`index.ts`) that
words it — its tables private to it, keyed by studied language where a name depends on it (`TENSES`,
listed in the order the card names them, and the gerund's name); what is named at all — whether the
infinitive, the present participle and the gerund are, whether the indicative and the subjunctive of
one tense are said once — is the description's, one table keyed by studied language (`CARD_NAMES`),
beside `finiteKey`. The studied languages the renderers name are `StudiedLanguageCode`'s: English,
Spanish, French (add-lingua-french-word-card D1) and Italian (add-lingua-italian-word-card D1), French
ahead of the extension's `StudiedLanguage`, which change 52 (enable-lingua-french) widened when it
listed the French pairs, and Italian ahead of it until the Italian programme's change 20
(enable-lingua-italian) widened it when it listed it-fr: since then an Italian card is opened for a
French-native reader, since its change 25 (enable-lingua-italian-for-english-speakers) listed it-en
for an English-native one too, and since its change 30 (enable-lingua-italian-for-spanish-speakers)
listed it-es for a Spanish-native one; the renderers' Italian rows are held by their unit tests and the
it-fr, it-en and it-es snapshots (`test/word-card-it-fr.spec.ts`, `test/word-card-it-en.spec.ts`,
`test/word-card-it-es.spec.ts`). Every renderer
has a table for every studied language: the French renderer's French, which no French-native reader
is shown, serves the interface override M2 reserves and the enumeration of what each renderer names,
as the English renderer's English and the Spanish renderer's Spanish do; its Italian is the first
table of a studied language whose moods merge that a French-native reader is shown. The rule that a
draft is typed after the French covers code here: `en/grammar.ts`, `es/grammar.ts` and `it/grammar.ts` are `typeof fr`, so the compiler checks
the functions; `test/grammar-description.spec.ts` checks that every renderer names what the French
names, no more, and `test/i18n.spec.ts` checks the drafts as renderers rather than as texts. The
French renderer is the card's wording before the catalogue, byte for byte; `reading/grammar-labels.ts`
keeps its API over it and holds no literal.

## Pages

An HTML page holds no text of its own: a node carries `data-copy="key"` (its whole text is
replaced) and an attribute `data-copy-aria-label="key"`, `data-copy-title`, `data-copy-placeholder`
or `data-copy-alt` (no other attribute is ever written from a module). The page's script starts with
`fillPageInLanguage(document, area, moduleOf)`, with its first storage read, before it builds
anything that shows copy: it reads the interface language, sets the page's `lang`
(`setDocumentLanguage`), then calls `fillPage(document, module)` with the module for that language,
and hands back the language and the module. Until then `<html>` carries `data-copy-pending`, under
which the page's stylesheet hides `body` (`html[data-copy-pending] body { visibility: hidden }`):
`fillPage` removes it. The rule also reveals the body after 1.5 s on its own, so a script that dies
before the fill leaves an unfilled page rather than a blank one. A text assembled around a figure
(« Réviser (3) ») is a slot message the script renders into the node; one around a node of its own
(« Niveau d'anglais : <b>B1</b> ») is called with `NODE_SLOT` and rendered by `renderAround`.

Every reader surface reads its module the same way (localise-lingua-reading-surfaces): the content
script and the reader page read the key before they build the reading session, which hands the HUD,
the drawer, the word card and the selection card their module and the language at construction, and
each injected host says it in `lang`. The words of the document inside them — the card's headword,
its form seen and its word-by-word forms, a review card's headword and sentence — say the studied
language in a `lang` of their own.

## Shape

- **A plain text** is a `string`. No `as const`: the French value must be typed `string`, or a
  translation would have to equal it.
- **A sentence built from parts** is one function of its parts — `review.sources(names)`,
  `colours.preview(unknown, learning)` — so a translation may put the parts elsewhere. A part the
  surface renders apart (a bold number, a painted word) is passed as a sentinel and the result
  rendered around it; the catalogue holds the sentence, not the markup. `slot(i)` is the `i`-th
  part's sentinel and `fillSlots(message, parts)` the nodes to append, each part where the language
  put it and a part the message dropped appended after it (« Je connais les **3000** mots les plus
  courants », Réglages' shortcut lines and colour preview). `NODE_SLOT` is `slot(0)` and
  `renderAround(element, message, node)` is `fillSlots` with that one part: one mechanism, so a
  message renders through either helper.
- **A count** is a `PluralForms` object, `{ one, many?, other }`, each form a function of the
  number _as the language writes it_: `plural(language, n, forms)` picks the form through
  `Intl.PluralRules`. French, Spanish and Italian have `many` (a round million); English has not,
  and a missing `many` falls back to `other`. In French every form carries the current string
  (« 3 carte(s) à revoir »): the French module writes it under `one` and `other`, through
  `pluralForms(…)` so that a translation may add `many`.
- **Keys are named by meaning, in English** — `review.reveal`, not `afficherLaReponse` — so a
  French text can change without a key moving. A key for an icon ends in `Icon`; a key for an
  accessible name ends in `Label`; a key for one of several alternatives names the alternative
  (`fromDeclaredLevel`), the message taking it says which key it expects.

## The French — byte for byte

The French catalogue is what the interface showed before it existed (M23): the same spaces before
« : ; ? » (plain ones), the same apostrophes (ASCII in most places, ’ where the source had it), the
same « » with plain inner spaces, the same abbreviations (« min. », « Mo »), the same raw counts.
Where a surface wrote a no-break space as an escape (`stats/ladder.ts`: ` `, ` `), the
module writes the same escape. A typography pass on the French, if ever, is a change of its own.

Numbers follow the surface: a count in a plural form is the raw figure (`plural` passes `String(n)`
in French — never a grouped « 1 234 »); a figure the surface already formatted (« 20 000 » with its
narrow no-break space) is `formatNumber("fr", n)`, which is `toLocaleString("fr-FR")` as before;
a percentage keeps each surface's form, `formatPercent("fr", n, "tight" | "spaced")` — the popup
writes « 45% », the reader's text size « 110 % ».

## English

US English (M10): _analyze_, _color_, _license_, _canceled_. Register: direct, second person, no
exclamation marks where the French had none. Curly quotes (“ ”) where the French has « ». An em dash
without spaces is fine, but the drafts keep the French's spaced « — » where it parts two clauses, so
the sentences keep their shape. No space before a colon. Numbers through `en-US` (`20,000`; `25.8`);
dates through `en-US` (`10/4/2026`, `October 4, 2026`); a percentage `96%`.

Terms: Settings (Réglages), deck, card, review, known / unknown words, words being learned (« mots en
cours »), highlighting (surlignage), level, CEFR, side panel, drawer (the in-page panel), library,
sign in / sign out, username (pseudo), connected accounts, sync, pill (pastille), pack (the
dictionary pack), model (the translation model), _-ing form_. Grammar: _simple past_, _simple present_,
_past participle_, _gerund_; the Spanish tenses by their usual English names (_present indicative_,
_imperfect indicative_, _preterite indicative_, _future indicative_, _conditional_, _present
subjunctive_, _imperfect subjunctive_, _future subjunctive_, _imperative_).

## Spanish

Tú, never usted, never vosotros (M19): the imperative is the tú form (_elige_, _reintenta_, _vuelve a
iniciar sesión_); a plural address is avoided rather than conjugated. Neutral, not markedly
peninsular, but Spain's conventions where one must be picked (« Ajustes », « ordenador » is avoided
by « dispositivo »). The reader is addressed without a gender: « Has iniciado sesión », never
« conectado/a ». Guillemets « » with no inner space; ¿ ¡ open every question and exclamation; no space
before a colon; the French's « — » between two clauses becomes a colon or a full stop, as Spanish
writes it — but a page or window title keeps its dash (« Cuenta — Cymbra Lingua », « Cymbra Lingua —
repaso »), since it parts two names, not two clauses. Numbers as the RAE writes them
(`formatNumber("es", n)`): no grouping below ten thousand (« 5000 »), then groups of three parted by a
narrow no-break space (« 20 000 »); a comma before the decimals (« 25,8 »). The helper writes that
grouping itself, so a caller's `useGrouping` is overridden in Spanish: the RAE's rule is the only
grouping Spanish gets. A percentage takes a narrow no-break space before the sign (« 96 % », U+202F) —
the project's convention, where `Intl` would put a no-break space (U+00A0). Dates through `es-ES`
(`4/10/2026`, `4 de octubre de 2026`). Ordinals with the period: 1.ª, 2.ª, 3.ª.

Terms: Ajustes (Réglages), mazo (deck), tarjeta (card), repaso / repasar, palabras conocidas /
desconocidas, palabras en aprendizaje (« mots en cours »), resaltado (surlignage), nivel, MCER
(CEFR), panel lateral, panel (the in-page drawer), biblioteca, iniciar / cerrar sesión, nombre de
usuario (pseudo), cuentas vinculadas, sincronización, pastilla, paquete (the dictionary pack), modelo,
« forma en -ing ». Grammar: the RAE's tense names — _presente de indicativo_, _pretérito imperfecto de
indicativo_, _pretérito perfecto simple de indicativo_, _futuro simple de indicativo_, _condicional
simple_, _presente de subjuntivo_, _pretérito imperfecto de subjuntivo_, _futuro de subjuntivo_,
_imperativo_; _participio_, _gerundio_, _infinitivo_. For English forms, the names Spanish-language
teaching of English gives them, never a Spanish tense's: _pasado simple_, _presente simple_,
_participio pasado_ (the Spanish Wiktionary's English form-of wording) and « forma en -ing ».

## Italian

Written before it was shown (add-lingua-italian-interface): `INTERFACE_LANGUAGES` gained `it` with
the Italian programme's change 45 (enable-lingua-italian-speakers), which ships en-it and shows it to
a reader whose native language is Italian. Drafted by an agent and reviewed by the dedicated Italian
reviewer (`.claude/agents/lingua-italian-reviewer.md`, I2), the owner reading it on the five targets
in that release's dogfood.

Tu, never Lei (I6): the imperative is the tu form (_scegli_, _riprova_, _accedi di nuovo_). The reader
is addressed without a gender: « Accesso effettuato », never « connesso/a »; « Ti diamo il
benvenuto », never « Benvenuto/a ». Guillemets « » with no inner space; no space before « : ; ? ! »;
the French's « — » between two clauses becomes a colon, as in Spanish — but a page or window title
keeps its dash (« Account — Cymbra Lingua »). Numbers through `it-IT` (`formatNumber("it", n)`):
« 20.000 », « 1.234.567 », a comma before the decimals (« 25,8 »), and no grouping under five digits
(« 5000 », the owner's answer of 2026-10-22), pinned with `useGrouping: "min2"` so that no browser's
ICU groups it. A percentage with no space (« 96% »), in every surface's form. Dates through `it-IT`
(`04/10/2026`, `4 ottobre 2026`), hours « 14:05 ». A round million is `many` and takes « di »
(« 1.000.000 di carte »); 0 is `other`. Ordinals « 7º », « 2ª ».

The word card's grammar (`it/grammar.ts`, add-lingua-italian-card-wording): Italian's forms by
Treccani's names, the mood before the tense (« indicativo imperfetto », « congiuntivo presente »,
« passato remoto », « futuro semplice », « condizionale presente »); French's and Spanish's by the
Italian name of the same tense (the passé simple and the pretérito perfecto simple are « passato
remoto »; Spanish's future subjunctive « congiuntivo futuro (futuro de subjuntivo) »); English's as the
Italian Wiktionary names them (« presente semplice », « passato semplice », « participio passato ») and
« forma in -ing ». Persons read « terza persona singolare », several of one tense named once (« prima e
terza persona singolare »), and a tense read in both numbers named once too (« prima persona singolare e
terza persona plurale dell'indicativo presente », review-lingua-italian-wording, through
`ReadingWords.bothNumbers`); a list puts a comma, not a second « e », before a name that joins two
itself (« plurale, maschile e femminile plurale di grande »); « di » contracts with a tense's article (« del passato remoto »,
« dell'indicativo imperfetto ») and never elides before a word of the page (« di eat »); « può anche
essere » takes each name's article (« il passato semplice », « l'infinito », « la forma in -ing »).
Pinned on the real packs' readings by `test/word-card-grammar-it.spec.ts`, grammar lines only.

Elision and contraction, where a message is built from parts: the languages carry their article
(« l'inglese », « lo spagnolo », « il francese », « l'italiano ») and `languages.ts`'s private `da()`
contracts « da » with it (« dall'inglese », « dallo spagnolo », « dal francese »), as Spanish's `de()`
does « de »; a date is written without an article, which would elide before « 8 » and « 11 »
(« Collegato: 8 ottobre 2026 », « Ultima sincronizzazione: 04/10/2026 »); an hour follows « alle
ore », whose article never elides (« alle ore 13:05 »); a count follows the feminine plural article
« le », which never elides (« le 20.000 parole »). The word card's language line agrees with
« parola » (« Parola spagnola »: `READ_AS_NOUN.it` is feminine).

Terms, from Cymbra Music's Italian (`apps/music/lib/l10n/app_it.arb`) where it has them:
Impostazioni (Réglages), Accedi / Esci, Annulla, Riprova, account, Email, Password; and mazzo (deck),
carta (card — « scheda » is the browser's tab), ripasso / ripassare, parole conosciute / sconosciute,
parole in apprendimento (« mots en cours »), evidenziazione (surlignage), livello, QCER (CEFR),
pannello di ripasso sulla pagina (the drawer), riquadro laterale (the browser's side panel, Chrome's
own word — « pannello » stays for Lingua's panels), acquisisci la selezione (capturer la sélection),
biblioteca, nome utente (pseudo), account collegati, sincronizzazione, badge (pastille), pacchetto
(the dictionary pack), modello, traduzione estesa / online. One word per concept across every Italian
surface: the glossary, `test/italian-glossary.json` (presented by `docs/lingua/italian-glossary.md`,
review-lingua-italian-wording), names each concept's accepted and refused forms, and
`test/italian-glossary.spec.ts` holds this catalogue, `_locales/it` and the Safari app's activation
page to it. The voice help's Windows and macOS menu paths, Firefox's consent name
(`it/settings.ts`) and the onboarding's « Fissa » (`it/onboarding.ts`) are unverified. Choices of the
owner (2026-10-26), kept by change 44's pass and checked in the dogfood of the release that shows the
interface (change 45): « nome utente », where Music says « Handle »; « Stats » in the HUD and the
drawer, where the French writes « Stats », as the Spanish; the unverified paths stay marked until that
dogfood checks them.

## What is the same everywhere

Names and symbols need no translation and are allowed to equal the French: « Cymbra Lingua », the
icons (« — », « ⚙ », « ✕ »…), the separators, « Aa », the key names that are the same in the
language (« Alt », « S »), « Google », « Apple », the voice previews (spoken in the studied language),
the native languages named each in its own language (« Français », “English”, « Español »:
`languages.ownNames`, the choice of add-lingua-native-language-choice, « Italiano » among them).
`test/i18n.spec.ts` lists them, and refuses any other translation equal to its French.
