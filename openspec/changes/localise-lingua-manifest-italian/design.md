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
- Rewriting `_locales/fr`, `en` or `es` (changes 20, 25 and 30 name Italian in them).

## Decisions

### D1 — Four messages, drafted in Italian, in the interface's words

`_locales/it/messages.json` holds the four keys of the other folders, each with an English `description`
for translators, as theirs. The text addresses the reader as « tu » (I6); commands are worded in the
imperative, as Italian interfaces word them, with the nouns the Italian interface gives the same
commands in Réglages' shortcut lines (`src/i18n/it/settings.ts`, change 33) — the browser's page and
Réglages name one command alike, as French and Spanish do. Drafts (M9's rule for drafts, I2's reviewer):

| Key | Draft | Characters |
|---|---|---|
| `extensionDescription` | « Leggi l'inglese sul web: parole sconosciute evidenziate, percentuale onesta. Offline e privato. » | 95 |
| `commandCaptureSelection` | « Cattura la selezione (parola o espressione) con la sua frase » | 60 |
| `commandToggleDrawer` | « Apri o chiudi il pannello di ripasso nella pagina » | 49 |
| `commandSidePanel` | « Apri il pannello laterale di ripasso » | 36 |

The description names English alone: en-it is the only pair glossed in Italian that change 45 ships,
and the check refuses a summary naming a language no shipped pair studies. Changes 50 and 54 rewrite it;
the shape that names all three fits — « Il web in inglese, francese o spagnolo: parole sconosciute
evidenziate, percentuale onesta. Offline e privato. », 110 characters, the shape change 20 recommends
for French. Italian typography as change 33 writes it: no space before « : », the apostrophe as the
interface writes it. The brand stays « Cymbra Lingua »; nothing else of the manifest is localised.

### D2 — The check learns Italian's words

`LANGUAGE_NAMES.it` holds Italian's names for the four studied languages exactly as
`src/i18n/it/languages.ts` names them (expected « Inglese », « Francese », « Spagnolo », « Italiano »,
capitalised as the other rows); `check-version.spec.ts`'s `catalogue()` gains `it`, so the row and the
catalogue stay equal. `namedLanguages` already matches a name whole and case-free, so « l'inglese »
names English. No other line of `check_version.mjs` changes.

### D3 — No build change; tests prove the path

`tool/manifests.mjs` and `tool/packs.mjs` are not edited. `test/manifests.spec.ts` gains, for each
target, a pair list holding en-it beside today's: the package carries `_locales/it` with the other
shipped natives' folders, `default_locale` stays `en`, and every `__MSG_` reference resolves in
`_locales/it`; with today's list, no package carries it. `test/check-version.spec.ts` gains: with en-it
listed, the committed Italian description passes `summaryProblems`; one naming « francese » fails,
naming the Italian description and French. `check_variants`' rule that a localised manifest's folders
are exactly the shipped natives already covers the packages.

### D4 — Reviewed by the Italian reviewer agent

The four messages are reviewed in the implementation pull request by the agent change 33 defines
(`.claude/agents/lingua-italian-reviewer.md`), its findings applied or answered there (I2); the owner is
not asked to review Italian. Change 44 rereads them with the rest of the Italian wording.

### D5 — Nothing ships before change 45

While `packs.json` lists no pair glossed in Italian, `shippedNatives` holds no `it`, nothing copies
`_locales/it`, and every package is byte for byte what it was. Apple validates a localised archive's
shape, which `_locales/{fr,en,es}` already passed (`localise-lingua-manifest`'s Impact: a folder of the
same shape needs no second validation). Once change 45's package is submitted, the Chrome Web Store and
addons.mozilla.org show its Italian summary to Italian browsers; it is submitted with change 46's
Italian listings, the owner deciding (I12).

## Risks / Trade-offs

- **Italian written without a native reader** → the reviewer agent (I2), change 44's pass, and change 55
  for dogfood corrections; the four strings are 240 characters in all.
- **The interface's words change after this lands** → the drafts are held to Réglages' nouns at
  implementation; a later change to either carries both, as French's two homes are held equal.
- **A summary that outgrows a pair** → `summaryProblems` fails changes 50 and 54 until they rewrite it.

## Migration Plan

Nothing deploys; no package moves. Rollback is a revert.
