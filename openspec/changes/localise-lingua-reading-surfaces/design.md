# Design — localise-lingua-reading-surfaces

## Context

See proposal.md (Why), and change 13's design: the catalogue's shape (D1: one module per surface
and language, every French literal extracted — the `popup`, `hud`, `drawer`, `card`,
`selection`, `sidepanel` and `reader` modules exist after it), slot messages and plurals (D2),
the interface-language key (D3), the formats (D4: `formatNumber`, `formatPercent`), the lint and
its baseline (D5). The reading surfaces today (the inventory of 2026-10-07):

| Surface | Files | Copy |
|---|---|---|
| Popup | `popup/popup.ts` (8 literals), `popup.html` (26 text nodes, aria/title « Réglages », « Retour »; sections hidden by `hidden` attributes and revealed by `render()`; aria « Réglages », « Surlignage activé », « Bibliothèque (livres EPUB) », « Statistiques d'apprentissage » paint before any script; Safari sizes its popover from the content) | status, level call to action (`chooseLevelPrompt`, change 19's module), counts (`${pct}%`), account line, « Réviser (n) » and « Niveau : » assembled in the HTML |
| HUD | `reading/hud.ts` (6) | aria « Mots connus sur la page — ouvrir les actions », « Réviser », « Choisis ton niveau », `${pct}%` |
| Drawer | `reading/drawer.ts` (5) | « Révision », « Fermer », the session-expired line |
| Word card | `reading/wordpopup.ts` (18 + 3 templates; the page labels at :115-117) | « Mot à mot — … », waiting, no gloss, page labels, « forme vue : « … » », the listen labels and their aria, actions, « Fermer » |
| Selection card | `reading/selection-card.ts` (3 + 5) | kind labels, the rarity bands with `toLocaleString("fr-FR")` |
| Side panel | `sidepanel.html` (4) | tab names |
| Reader | `reader/copy.ts` (28 + 4, already an object), `app.ts`, `library.ts` (« Livre sans titre », saved into the book record at import), `reader.html` (title) | library, import, remove confirmation, page titles, `${n} %` |
| The content script | `content.ts` → `new ReadingSession(…)` whose constructor builds the HUD, the drawer, the word card and the selection cards with their labels; `start()` reads the preferences afterwards |
| Tests | `wordpopup`, `selection-card`, `rarity-text`, `hud`, `reader-app` specs assert French; no spec renders the popup, the drawer or the side panel (`popup.ts`, `sidepanel.ts`, `drawer.ts` are excluded from coverage), and none asserts the HTML pages' text |

## Goals / Non-Goals

**Goals:**
- Every text of these surfaces comes from the catalogue, in the interface language.
- The French is byte for byte the same, with a check for every file — the existing spec files
  where they exist, new ones where none does.
- The pages and the injected hosts say their language.

**Non-Goals:**
- Any other surface (15–17); the grammar lines (18, which owns the studied words' `lang`); the
  languages' names (19, whose module these surfaces call with the language).
- Changing a page's structure beyond ids on its text nodes.
- What happens when the native language changes while a surface is open: change 20 defines the
  rebuild and the re-labelling; here a surface reads the key once before it builds its copy.

## Decisions

### D1 — The key is read before a surface is built; the copy is handed in

The content script reads the interface language with its first storage read, before
`new ReadingSession(…)`, and hands the session the copy of the HUD, the drawer, the word card and
the selection card; the session passes each its module at construction, so the labels set in the
constructors are the catalogue's from the first paint. The popup, the side panel and the reader
read the key with their first storage read, before they build anything that shows copy. A
literal becomes `copy.key`; a fragment sentence becomes a slot message rendered by the surface
(`copy.knownOnPage(n)` returns segments the surface wraps); a count a plural form; the languages'
names come from change 19's module, called with the language. Re-labelling an open surface when
the key changes is change 20's (D3 there).

### D2 — The pages' static text is filled at mount, and nothing shows before

`popup.html`, `sidepanel.html` and `reader.html` keep their skeletons; their text nodes and
`aria-label`/`title` attributes carry a `data-copy="key"` attribute, and `fillPage(copy)` (change
13's `index.ts`) sets them before the page is shown, with `document.documentElement.lang`. Until
then the page's `<html>` carries `data-copy-pending`, whose rule in each page's stylesheet hides
`body`; the script removes it after filling. The French text leaves the HTML, so the lint's
baseline drops the pages. Safari sizes its popover from the content: the fill happens in the
same task as the first storage read's resolution, before the popover is measured — checked by
eye on Safari in the pull request.

### D3 — Injected hosts carry `lang`

`#cymbra-lingua-host`, `#cymbra-lingua-hud-host` and `#cymbra-lingua-drawer-host` get `lang` from
the interface language at creation; this change owns the hosts' `lang`, change 18 the studied
words' inside a line.

### D4 — Formats

`rarityText`'s `toLocaleString("fr-FR")`, the reader's `${n} %`, the popup's and the HUD's
`${pct}%` go through the catalogue's `formatNumber` and `formatPercent` (change 13 D4: French
keeps each surface's current form — the popup's "45%", the reader's "45 %" — Spanish follows the
RAE, « 96 % » and « 20 000 », English "96%" and "20,000"), so the French output is the same bytes.

### D5 — Tests

The spec files that render the word card, the selection card, the HUD and the reader with the
French copy pass unchanged: that is the byte-for-byte check for those files. The popup, the
drawer and the side panel have no spec: this change adds one for each (`popup.spec.ts`,
`drawer.spec.ts`, `sidepanel.spec.ts`), rendering the surface in jsdom with the French catalogue
and asserting the text the HTML pages and the modules held before (the assertions are the
inventory's literals), and the pages' text through `fillPage`. Each surface gains one test in
English asserting three texts and the `lang` attributes. `lint-copy`'s baseline loses the files.

## Risks / Trade-offs

- **A text the inventory missed** → the lint fails when the file leaves the baseline, naming it.
- **A page flashing empty** → hidden until filled by `data-copy-pending`; measured by eye on the
  five targets in the pull request.
- **The Safari popover sized before the fill** → the fill is synchronous after the first read;
  checked by eye.
- **The popup's « Réviser (n) » and « Niveau : » assembled in the HTML** → slot messages rendered
  into the nodes.

## Migration Plan

One release, silent. No stored state, no wire.
