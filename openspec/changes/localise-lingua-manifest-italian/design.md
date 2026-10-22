# Design — localise-lingua-manifest-italian

## Context

Change 39 of the [Italian programme](../../../docs/lingua/italian-programme.md); its precedent is the
matrix programme's change 27, `localise-lingua-manifest`, whose machinery serves any native language:

| Where | What it does today |
|---|---|
| `_locales/{fr,en,es}/messages.json` | `extensionDescription`, `commandCaptureSelection`, `commandToggleDrawer`, `commandSidePanel`; each with an English `description` for translators |
| `tool/manifests.mjs` | for every native a shipped pair is glossed in (`shippedNatives`, `tool/packs.mjs`), copies `_locales/<native>/` and writes `__MSG_` references; refuses to build when a shipped native has no `messages.json` or lacks a referenced key; `default_locale` `en` when an English-glossed pair ships |
| `tool/check_version.mjs` | `localeProblems`: every committed description within Apple's 112 characters; `_locales/fr` held equal to `manifest.json`'s literal. `summaryProblems` (`add-lingua-french-listings` D6): for each shipped native, its description names exactly the languages its shipped pairs study, matched through `LANGUAGE_NAMES[native]` — a shipped native with no row fails, naming `LANGUAGE_NAMES` |
| `LANGUAGE_NAMES` | rows `fr`, `en`, `es`; change 20 (`enable-lingua-italian`) adds Italian to each (« Italien », "Italian", « Italiano »); no `it` row |
| `test/check-version.spec.ts` | « names each language as the interface's catalogue does »: each row held equal to `src/i18n/<native>/languages.ts`'s names |

The first pair glossed in Italian is en-it, shipped by change 45; fr-it and es-it follow in changes 50
and 54. Change 32 lets the extension's native type hold `it`; change 33 writes `src/i18n/it/`, its
`languages.ts` included, and defines the Italian reviewer agent (I2).

## Goals / Non-Goals

**Goals:**
- The day en-it is listed, the packages build and `yarn check:version` passes, with nothing more to write.
- An Italian browser shows an Italian description naming what Italian readers can study.

**Non-Goals:**
- The Italian store listings (change 46, I9); the Apple host's own text (change 40).
- Rewriting `_locales/fr`, `en` or `es`: changes 20 and 25 name Italian in them, and change 30 if it
  lists it-es.

## Decisions

### D1 — Four messages, drafted in Italian, in the interface's words

`_locales/it/messages.json` holds the four keys of the other folders, each with an English `description`
for translators, as theirs. The text addresses the reader as « tu » (I6). Commands are worded in the
imperative, as the browsers' own Italian pages word theirs: Chrome's generic extension command is
« Attiva l'estensione » where its French and Spanish are « Activer l'extension » and « Activar la
extensión » (`IDS_EXTENSION_COMMANDS_GENERIC_ACTIVATE`), and Firefox's Italian shortcuts page has
« Attiva/disattiva barra laterale » (`aboutAddons.ftl`). So the French and Spanish infinitives and the
Italian imperative each follow their browser; nobody should align one with the others.

Each command's description contains the words the Italian Réglages gives that command
(`src/i18n/it/settings.ts`'s `shortcutCapture`, `shortcutDrawer` and `shortcutSidePanel`, change 33), so
that the browser's page and Réglages name one command alike. That is a test (D3), and it already holds
for French, English and Spanish. It also means Réglages' Italian lines are imperative
(« cattura la selezione »). Change 33 is implemented first and its design does not fix the verb's
form; if it lands an infinitive, this change's implementation edits `src/i18n/it/settings.ts` so that
the test passes.

Drafts, under M9's rule for drafts and reviewed under I2:

| Key | Draft | Characters |
|---|---|---|
| `extensionDescription` | « Leggi il web in inglese: parole sconosciute evidenziate, percentuale reale. Offline e senza tracciamento. » | 105 |
| `commandCaptureSelection` | « Cattura la selezione (parola o espressione) insieme alla frase » | 62 |
| `commandToggleDrawer` | « Apri o chiudi il pannello di ripasso nella pagina » | 49 |
| `commandSidePanel` | « Apri il riquadro laterale di ripasso » | 36 |

- **« percentuale reale »**, not « onesta »: with a quantity, *onesto* reads "fair, reasonable"
  (*prezzo onesto*), not "truthful".
- **« senza tracciamento »**, not « privato », which reads "not public" in Italian (Music's `app_it.arb`:
  « Profilo pubblico o privato ») and would agree with no noun.
- **« riquadro laterale »**: Chrome's Italian word for its side panel (26 strings against 2 with
  « pannello laterale »). The side panel is Chromium's alone.

The description names English alone, since en-it is the only pair glossed in Italian that change 45
ships. A later change listing fr-it (change 50) or es-it (change 54, if es-it ships under I4's floor)
rewrites it, as `summaryProblems` requires. The shape naming three languages is tight:
« Il web in inglese, francese o spagnolo: parole sconosciute evidenziate, percentuale reale. » leaves
22 characters for the tail. « Senza tracciamento. » fits (110); « Offline e senza tracciamento. » does
not (120). That change chooses the tail with the reviewer agent.

Typography: no space before « : », and the apostrophe U+0027, as `_locales/fr` writes it. The brand
stays « Cymbra Lingua », and nothing else of the manifest is localised. The translators' notes name the
pairs they follow (en-it; fr-it and es-it to come), not programme change numbers, since
`messages.json` ships inside the package.

### D2 — The check learns Italian's words

`LANGUAGE_NAMES.it` holds Italian's names for the four studied languages exactly as
`src/i18n/it/languages.ts` names them: expected « Inglese », « Francese », « Spagnolo », « Italiano »,
capitalised like the other rows. Change 20's `catalogue()` already gains `it` (Italian's name in each
native's words). This change adds a fourth expected row, `it: catalogue(itLanguages)`, importing
`../src/i18n/it/languages`, so that the row and the catalogue stay equal. The comment at
`check_version.mjs`'s `LANGUAGE_NAMES`, which names `src/i18n/{fr,en,es}`, gains `it`.
`namedLanguages` already matches a name whole and case-free, so « l'inglese » names English. No other
line of `check_version.mjs` changes.

### D3 — No build change; tests prove the path

`tool/manifests.mjs` and `tool/packs.mjs` are not edited. The tests are:
- **`test/manifests.spec.ts`**:
  - for each target, en-it listed beside today's pairs: the package carries `_locales/it` with the other
    shipped natives' folders, `default_locale` stays `en`, and every `__MSG_` reference resolves in
    `_locales/it`;
  - with today's list, no package carries it;
  - the test at its top that expects the committed folders to be exactly `["en", "es", "fr"]` becomes
    `["en", "es", "fr", "it"]`, its title with it;
  - the Italian line of the test that reads what each description names its own readers study names
    « inglese », not « italiano ».
- **`test/check-version.spec.ts`**:
  - its expected committed set (`["en", "es", "fr"]`) gains `it`;
  - with en-it listed, the committed Italian description passes `summaryProblems`;
  - one naming « francese » fails, naming the Italian description and « Francese ».
- **Settings and shortcuts alike**, a new `test/locales-shortcuts.spec.ts`: for each of fr, en, es and
  it, each Réglages shortcut line's text after the keys is contained, case-insensitively, in the
  matching `_locales/<language>` command's message.

`check_variants`' rule that a localised manifest's folders are exactly the shipped natives already
covers the packages.

No en-it pack exists before change 36, so no package can be loaded in an Italian browser here. The
precedent's manual check (chrome://extensions/shortcuts and about:addons in the browser's language,
no `__MSG_` text) is done for Italian in change 45's dogfood, and its task names it.

### D4 — Reviewed by the Italian reviewer agent

The four messages are reviewed in the implementation pull request by the agent change 33 defines
(`.claude/agents/lingua-italian-reviewer.md`), its findings applied or answered there (I2). The owner is
not asked to review Italian. Change 44 rereads them with the rest of the Italian wording; its row names
`_locales/it`.

### D5 — Nothing ships before change 45

While `packs.json` lists no pair glossed in Italian, `shippedNatives` holds no `it`, nothing copies
`_locales/it`, and every package is byte for byte what it was. Apple has accepted packages carrying
`_locales`: lingua-apple 1.6.0 shipped `_locales/{fr,en,es}` (#790), and 1.7.0 and 1.8.1 were accepted
with them. An `it` folder has the same shape.

Once change 45's package is submitted, the Chrome Web Store takes a summary per `_locales` folder.
What addons.mozilla.org does with it is recorded by change 46's dashboard procedure, as it was for
`es` (`apps/lingua-extension/STORE-LISTING.md`). That package is submitted with change 46's Italian
listings, the owner deciding (I12).

## Risks / Trade-offs

- **Italian written without a native reader** → the reviewer agent (I2), change 44's pass, and change 55
  for dogfood corrections; the four strings are 252 characters in all.
- **The interface's words change after this lands** → the containment test (D3) fails until both
  move, as French's two homes are held equal.
- **A summary that outgrows a pair** → `summaryProblems` fails changes 50 and 54 until they rewrite it.

## Migration Plan

Nothing deploys; no package moves. Rollback is a revert.
