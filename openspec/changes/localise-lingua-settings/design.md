# Design — localise-lingua-settings

## Context

See proposal.md (Why) and change 13's design. Réglages today:

| Module | Copy |
|---|---|
| `reading/settings-view.ts` | the twelve block titles — eleven `settingBlock("…")` literals and `levelTitle(language)` (« Langues étudiées », « Barre sur la page », « Lecture à voix haute », « Livres », « Affichage », « Couleurs », « Traduction », « Raccourcis & gestes », « Compte », « Synchronisation », « Réinitialisation »), the four tabs (« Langue », « Apparence », « Pages & livres », « Données »), the reset confirmations, the voice help, the shortcut lines (`kbd()` + « — panneau latéral »), « Je connais les **3000** mots les plus courants » |
| `reading/colour-settings-view.ts` | the presets (« E-ink contrasté »…), `${name} : couleur du fond`, « Un mot <span>, un mot <span> et un mot connu. » (two swatches) |
| `reading/book-display-view.ts` | « Papier », « Réduire le texte »… |
| `reading/studied-languages-view.ts` | two texts |
| `reading/translation-setting.ts` | `COPY`, `megabytes` (« 25,8 Mo »), `costText`, `FAILURE`, `stateText` |
| `reading/account-setting.ts` | `ACCOUNT_COPY` |
| `sync/status.ts` | `lastSyncLabel` (« il y a 3 min. », `toLocaleDateString("fr-FR")`), `syncErrorCopy` |
| `test/lint-settings-hosts.spec.ts` | reads `/settingBlock\("([^"]+)"\)/` out of `settings-view.ts` (eleven titles); no other `.ts` may hold a title as a literal, no other `.html` as text |
| `#696` | adds « Rythme de révision », « Nouveaux mots par jour », « Fichier de sauvegarde », « Sources et confidentialité » and the « Langue » / « Données » tab contents; not on main when this is written |

The hosts (`popup.ts`, `sidepanel.ts`, `drawer.ts`) call `mountSettings(container, port, area,
opts)`; the popup and the side panel read `chrome.storage.local` before mounting; the drawer is
built by the reading session in the content script and reads nothing — change 14 hands it the
interface language the content script read.

## Goals / Non-Goals

**Goals:**
- Every text of Réglages comes from the catalogue, in the interface language.
- The hosts' lint still proves the titles live once.
- The French is byte for byte the same.

**Non-Goals:**
- The other surfaces (14, 16, 17); the native language's block (20).

## Decisions

### D1 — `mountSettings` takes the interface language; the hosts pass it

`mountSettings(container, port, area, opts)` gains `opts.interfaceLanguage` — absent means
French, so every existing spec mounts as before; the popup and the side panel pass the language
they read with their preferences, the drawer the one the session handed it (change 14); the view
picks the `settings` module of that language and hands each block's module its own (`colours`,
`display`, `translation`, `account-setting`, `sync`), and the language with it where a block
formats (D4). The blocks keep their signatures otherwise.

### D2 — Titles in the catalogue; the hosts' lint reads them there

`settingBlock(copy.titles.languages)` and so on. `lint-settings-hosts` imports `settings.titles`
from `src/i18n/fr/settings.ts` (the eleven literal titles; the level's stays a function of the
studied language, change 19's to name) and keeps its two checks: every host calls `mountSettings`,
and no `.ts` other than `src/i18n/fr/settings.ts`, and no `.html`, holds a French title as a
literal — the English and Spanish modules hold their own titles, which the check does not match.
The catalogue's three languages are the one place.

### D3 — Fragments as slot messages

« Je connais les **3000** mots les plus courants » is `copy.knowCommonest(n)` returning
segments `[text, {strong: "3000"}, text]` the view renders; the shortcut lines take the key
and the label; the colour explanation takes its two swatches as slots; `costText` and
`stateText` are catalogue functions of the cost and the state, French byte for byte.

### D4 — Formats

`megabytes` keeps « 25,8 Mo » in French and formats with change 13's `formatNumber` in English
and Spanish ("25.8 MB", « 25,8 MB »); `lastSyncLabel` keeps « il y a 3 min. » and
`toLocaleDateString("fr-FR")` for French, and takes change 13's `plural` for its minutes, hours
and days and `formatDate` for the others (change 13 D4).

### D5 — #696 and this change: the second one rebases

#696 is open when this change is written, and the two touch `settings-view.ts`. If #696 merges
first, this change extracts its strings with the rest. If this change merges first, #696 rebases
onto the catalogue: `settings-view.ts` is then off the baseline, so the lint refuses its new
literals until they are entries in three languages — the owner translates them with the rest of
#696, and the hosts' lint sees its new block titles the same way. Neither order needs a word of
this change to move.

## Risks / Trade-offs

- **A title duplicated by a host** → the hosts' lint, now reading the catalogue.
- **A French byte in Réglages** → the seven spec files, unchanged.
- **A Spanish Réglages that still names a language in French** → expected until change 19; the
  Spanish scenario excepts it.

## Migration Plan

One release, silent.
