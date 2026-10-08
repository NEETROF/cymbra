# Design — localise-lingua-reading-surfaces

## Context

See proposal.md (Why), and change 13's design: the catalogue's shape (D1: one module per surface
and language, every French literal extracted — the `popup`, `hud`, `drawer`, `card`, `selection`,
`sidepanel` and `reader` modules exist after it; its implementation gave the drawer and the side
panel modules of their own), slot messages and plurals (D2), the interface-language key (D3), the
formats (D4: `formatNumber`, `formatPercent`, `formatDate`), the lint and its baseline (D5). The reading surfaces today (the inventory of 2026-10-07):

| Surface | Files | Copy |
|---|---|---|
| Popup | `popup/popup.ts` (8 literals), `popup.html` (26 text nodes, aria/title « Réglages », « Retour »; sections hidden by `hidden` attributes and revealed by `render()`; aria « Réglages », « Surlignage activé », « Bibliothèque (livres EPUB) », « Statistiques d'apprentissage » paint before any script; Safari sizes its popover from the content) | status, level call to action (`chooseLevelPrompt`, change 19's module), counts (`${pct}%`), account line, « Réviser (n) » and « Niveau : » assembled in the HTML |
| HUD | `reading/hud.ts` (6) | aria « Mots connus sur la page — ouvrir les actions », « Réviser », « Choisis ton niveau », aria « Réglages » and « Réduire », `${pct}%` |
| Drawer | `reading/drawer.ts` (5) | « Révision », « Fermer », the session-expired line |
| Word card | `reading/wordpopup.ts` (18 + 3 templates; the page labels at :115-117) | « Mot à mot — … », waiting, no gloss, page labels, « forme vue : « … » », the listen labels and their aria, actions, « Fermer » |
| Selection card | `reading/selection-card.ts` (3 + 5) | kind labels, the rarity bands with `toLocaleString("fr-FR")` |
| Side panel | `sidepanel.html` (4) | tab names |
| Reader | `reader/copy.ts` (28 + 4, already an object), `app.ts`, `library.ts` (« Livre sans titre », saved into the book record at import: stored data, written once in the interface language of the import, not display-time copy), `reader.html` (title) | library, import, remove confirmation, page titles, `${n} %` |
| The content script and the reader | `content.ts` and `reader/reader.ts` (the book's session, `surface: "book"`) → `new ReadingSession(…)` whose constructor builds the HUD, the drawer, the word card and the selection cards with their labels; `start()` reads the preferences afterwards |
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
`new ReadingSession(…)`, and hands the session both the interface language and the copy of the
HUD, the drawer, the word card and the selection card; the session passes each its module and the
language at construction, so the labels set in the constructors are the catalogue's from the
first paint, and the surfaces that need the language itself (the grammar renderer of change 18,
the languages' names of change 19) have it. `reader/reader.ts` builds the book's session the same way and hands it the same. The popup,
the side panel and the reader read the key with their first storage read, before they build
anything that shows copy. Every constructor and function that gains the copy or the language
takes it as an optional trailing parameter or option, absent meaning French, so the existing
specs mount as before. A literal becomes `copy.key`; « Réviser (n) » and « Niveau : … » are
change 13's slot messages `popup.review(due)` and `popup.levelLine(title, level)`, rendered into
the nodes; a count a plural form; the languages' names come from change 19's module, called with
the language. Re-labelling an open surface when
the key changes is change 20's: it rebuilds the session and reloads the pages.

### D2 — The pages' static text is filled at mount, and nothing shows before

`popup.html`, `sidepanel.html` and `reader.html` keep their skeletons; their text nodes and
`aria-label`/`title` attributes carry a `data-copy="key"` attribute, and `fillPage(document,
copy)` — added to change 13's `index.ts` by this change — sets them before the page is shown,
and `setDocumentLanguage(document, language)` (change 13) sets the page's `lang`. Until
then the page's `<html>` carries `data-copy-pending`, whose rule in each page's stylesheet hides
`body`; the script removes it after filling. The three pages take this step through one helper,
`fillPageInLanguage(document, area, moduleOf)`, and it cannot be left hanging on a failed read:
`interfaceLanguage` answers French when the storage read throws, and the pending rule reveals the
body by itself after 1.5 s should a script die before the fill. The French text leaves the HTML,
so the lint's baseline drops the pages. Safari sizes its popover from the content, at open and
again as the content grows (`popup.css` already relies on that re-measure for the account block):
the fill happens in the same task as the first storage read's resolution, so the popover grows to
the filled page as it does to the account block — checked by eye on Safari in the pull request.

### D3 — Injected hosts carry `lang`

`#cymbra-lingua-host`, `#cymbra-lingua-hud-host` and `#cymbra-lingua-drawer-host` get `lang` from
the interface language at creation; this change owns the hosts' `lang`, change 18 the studied
words' inside a line. The host speaking the interface language, the words of the document a
surface shows whole say the studied language in a `lang` of their own: the word card's headword,
its form seen and its word-by-word forms (from the card's `language`, English without one), and a
review card's headword and sentence (from the card's).

### D4 — Formats

`rarityText`'s `toLocaleString("fr-FR")` goes through change 13's `formatNumber` (French as
today; Spanish per the RAE, « 20 000 » — change 13's implementation chose the RAE over `Intl`'s
« 20.000 », and its tests pin it); the reader's `${n} %` and the popup's and the HUD's `${pct}%`
go through change 13's `formatPercent(language, n, form)`: in French the form the surface writes
today (`"tight"`, "45%", for the popup and the HUD; `"spaced"`, "45 %", for the reader), in
English "45%", in Spanish « 45 % » with a narrow no-break space. A count — the popup's figures
and « Réviser (n) », the word card's page count — goes through `formatCount(language, n)`: French
the bare figure it writes today, English and Spanish through `formatNumber`. The French output is
the same bytes.

### D5 — Tests

The spec files that render the word card, the selection card, the HUD and the reader with the
French copy pass unchanged: that is the byte-for-byte check for those files. No spec asserts the
popup's, the drawer's or the side panel's text (`surface-look.spec.ts` builds a drawer and asserts
no text): this change adds one for each: `popup.spec.ts` and
`sidepanel.spec.ts` load the page's HTML skeleton in jsdom, call `fillPage` with the French
catalogue and assert that every `data-copy` node and attribute holds the text the page held and
that `data-copy-pending` is gone (`popup.ts` and `sidepanel.ts` are entry scripts with no exported
render; their own literals are guarded by their exit from the lint's baseline); `drawer.spec.ts`
builds a `Drawer` and asserts its tabs, its close label and the session-expired line. Each surface gains one test in
English asserting three texts and the `lang` attributes. `lint-copy`'s baseline loses the files.

## Risks / Trade-offs

- **A text the inventory missed** → the lint fails when the file leaves the baseline, naming it.
- **A page flashing empty** → hidden until filled by `data-copy-pending`; measured by eye on the
  five targets in the pull request.
- **The Safari popover sized before the fill** → Safari re-measures as the content grows, and the
  fill is synchronous after the first read; checked by eye.
- **The popup's « Réviser (n) » and « Niveau : » assembled in the HTML** → change 13's
  `popup.review(due)` and `popup.levelLine(title, level)` rendered into the nodes.

## Migration Plan

One release, silent. No stored state, no wire.
