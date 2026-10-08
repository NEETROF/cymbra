# The interface's copy — one catalogue, three languages

Every text the extension shows lives here, once per language: `fr/<surface>.ts`, `en/<surface>.ts`,
`es/<surface>.ts`, one module per surface (`popup`, `hud`, `drawer`, `card`, `selection`,
`sidepanel`, `review`, `stats`, `settings`, `studied-languages`, `native-language`, `colours`,
`display`, `translation`, `account-setting`, `account`, `account-errors`, `onboarding`, `reader`,
`sync`, `languages`, `grammar`). The French module is the source;
the English and Spanish ones are typed after it (`export const popup: typeof fr = { … }`), so a key
missing in a translation does not compile (`yarn typecheck`). `test/i18n.spec.ts` checks the rest at
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
English, « MCER » in Spanish (M19).

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
infinitive is, which moods are — is the description's, beside `finiteKey`. The rule that a draft is typed after the
French covers code here: `en/grammar.ts` and `es/grammar.ts` are `typeof fr`, so the compiler checks
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
  `Intl.PluralRules`. French and Spanish have `many` (a round million); English has not, and a
  missing `many` falls back to `other`. In French every form carries the current string
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

## What is the same everywhere

Names and symbols need no translation and are allowed to equal the French: « Cymbra Lingua », the
icons (« — », « ⚙ », « ✕ »…), the separators, « Aa », the key names that are the same in the
language (« Alt », « S »), « Google », « Apple », the voice previews (spoken in the studied language),
the native languages named each in its own language (« Français », “English”, « Español »:
`languages.ownNames`, the choice of add-lingua-native-language-choice).
`test/i18n.spec.ts` lists them, and refuses any other translation equal to its French.
