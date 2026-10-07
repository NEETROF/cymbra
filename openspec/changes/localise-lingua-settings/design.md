# Design — localise-lingua-settings

## Context

See proposal.md (Why) and change 13's design. Réglages today:

| Module | Copy |
|---|---|
| `reading/settings-view.ts` | the twelve `settingBlock("…")` titles (« Langues étudiées », `levelTitle(language)`, « Barre sur la page », « Lecture à voix haute », « Livres », « Affichage », « Couleurs », « Traduction », « Raccourcis & gestes », « Compte », « Synchronisation », « Réinitialisation »), the four tabs (« Langue », « Apparence », « Pages & livres », « Données »), the reset confirmations, the voice help, the shortcut lines (`kbd()` + « — panneau latéral »), « Je connais les **3000** mots les plus courants » |
| `reading/colour-settings-view.ts` | the presets (« E-ink contrasté »…), `${name} : couleur du fond`, « Un mot <span>, un mot <span> et un mot connu. » |
| `reading/book-display-view.ts` | « Papier », « Réduire le texte »… |
| `reading/studied-languages-view.ts` | two texts |
| `reading/translation-setting.ts` | `COPY`, `megabytes` (« 25,8 Mo »), `costText`, `FAILURE`, `stateText` |
| `reading/account-setting.ts` | `ACCOUNT_COPY` |
| `sync/status.ts` | `lastSyncLabel` (« il y a 3 min. », `toLocaleDateString("fr-FR")`), `syncErrorCopy` |
| `test/lint-settings-hosts.spec.ts` | reads `/settingBlock\("([^"]+)"\)/` out of `settings-view.ts`; no other `.ts` may hold a title as a literal, no other `.html` as text |
| `#696` | adds « Rythme de révision », « Nouveaux mots par jour », « Fichier de sauvegarde », « Sources et confidentialité » and the « Langue » / « Données » tab contents |

The hosts (`popup.ts`, `sidepanel.ts`, `drawer.ts`) call `mountSettings(container, port, area,
opts)`; each reads `chrome.storage.local` before mounting.

## Goals / Non-Goals

**Goals:**
- Every text of Réglages comes from the catalogue, in the interface language.
- The hosts' lint still proves the titles live once.
- The French is byte for byte the same.

**Non-Goals:**
- The other surfaces (14, 16, 17); the native language's block (20).

## Decisions

### D1 — `mountSettings` takes the copy; the hosts pass it

`mountSettings(container, port, area, opts)` gains `opts.language`, the interface language the
host read with its preferences; the view picks `settingsCopy[language]` and hands each block's
module its own (`colours`, `display`, `translation`, `account-setting`, `sync`). The blocks keep
their signatures otherwise.

### D2 — Titles in the catalogue; the hosts' lint reads them there

`settingBlock(copy.titles.languages)` and so on. `lint-settings-hosts` reads the titles from
`src/i18n/fr/settings.ts` (the `titles` object, by a regex on its source as today, or by
importing it) and keeps its two checks: every host calls `mountSettings`, and no other `.ts` or
`.html` holds a title as a literal. The catalogue's three languages are the one place.

### D3 — Fragments as slot messages

« Je connais les **3000** mots les plus courants » is `copy.knowCommonest(n)` returning
segments `[text, {strong: "3000"}, text]` the view renders; the shortcut lines take the key
and the label; the colour explanation takes its three swatches as slots; `costText` and
`stateText` are catalogue functions of the cost and the state, French byte for byte.

### D4 — Formats

`megabytes` keeps « 25,8 Mo » in French and formats with the locale in English and Spanish;
`lastSyncLabel` keeps « il y a 3 min. » and `toLocaleDateString("fr-FR")` for French, the
locale's for the others (change 13 D4).

### D5 — #696 first

Its strings are in `settings-view.ts` by the time this change extracts, so the French catalogue
holds them; if #696 were still open, this change would be rebased after it, as the programme
rules.

## Risks / Trade-offs

- **A title duplicated by a host** → the hosts' lint, now reading the catalogue.
- **A French byte in Réglages** → the six spec files, unchanged.

## Migration Plan

One release, silent.
