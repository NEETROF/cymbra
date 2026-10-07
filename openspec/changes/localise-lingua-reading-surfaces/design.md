# Design — localise-lingua-reading-surfaces

## Context

See proposal.md (Why), and change 13's design: the catalogue's shape (D1), slot messages and
plurals (D2), the interface-language key (D3), the formats (D4), the lint and its baseline (D5).
The reading surfaces today (the inventory of 2026-10-07):

| Surface | Files | Copy |
|---|---|---|
| Popup | `popup/popup.ts` (8 literals), `popup.html` (26 text nodes, aria/title « Réglages », « Retour ») | status, level call to action (`chooseLevelPrompt`, change 19's seam), counts (`${pct}%`), account line, « Réviser (n) » assembled in the HTML |
| HUD | `reading/hud.ts` (6) | aria « Mots connus sur la page — ouvrir les actions », « Réviser », « Choisis ton niveau » |
| Drawer | `reading/drawer.ts` (5) | « Révision », « Fermer », the session-expired line |
| Word card | `reading/wordpopup.ts` (18 + 3 templates), `gloss-pages.ts` (none) | « Mot à mot — … », waiting, no gloss, page labels, « forme vue : « … » », the listen labels and their aria, actions, « Fermer » |
| Selection card | `reading/selection-card.ts` (3 + 5) | kind labels, the rarity bands with `toLocaleString("fr-FR")` |
| Side panel | `sidepanel.html` (4) | tab names |
| Reader | `reader/copy.ts` (28 + 4, already an object), `app.ts`, `library.ts` (1), `reader.html` (title) | library, import, remove confirmation, page titles, `${n} %` |

Every surface reads `chrome.storage.local` preferences before it renders (the HUD state, the
colours, `loadEnabled`); the content script's `ReadingSession` builds the HUD and the card after
`resolveContentPort`.

## Goals / Non-Goals

**Goals:**
- Every text of these surfaces comes from the catalogue, in the interface language.
- The French is byte for byte the same; the spec files are the proof.
- The pages and the injected hosts say their language.

**Non-Goals:**
- Any other surface (15–17); the grammar lines (18); the languages' names (19).
- Changing a page's structure beyond ids on its text nodes.

## Decisions

### D1 — A surface reads its copy once, picked by the interface language

Each surface module imports its three catalogue modules and holds `copy = copyOf(language)`
(`{fr, en, es}[language]`) chosen from the interface language read with its preferences — the
content script reads it once in `ReadingSession`'s start and hands it to the HUD, the card and
the selection card; the popup, the side panel and the reader read it with their first storage
read. The surfaces keep their shapes; a literal becomes `copy.key`, a fragment sentence becomes a
slot message rendered by the surface (`copy.knownOnPage(n)` returns segments the surface wraps),
a count a plural form.

### D2 — The pages' static text is filled at mount

`popup.html`, `sidepanel.html` and `reader.html` keep their skeletons; their text nodes and
`aria-label`/`title` attributes carry an id or a `data-copy="key"` attribute, and a `fillPage(copy)`
helper (change 13's `index.ts`) sets them from the catalogue before the page is shown, with
`document.documentElement.lang`. The French text leaves the HTML, so the lint's baseline drops the
pages; a page shows nothing before its script runs — it is hidden by its stylesheet until filled,
as the popup already hides sections until `loadEnabled` answers.

### D3 — Injected hosts carry `lang`

`#cymbra-lingua-host`, `#cymbra-lingua-hud-host` and `#cymbra-lingua-drawer-host` get `lang` from
the interface language at creation (the card's studied-language words are change 18's).

### D4 — Formats

`rarityText`'s `toLocaleString("fr-FR")` and the reader's `${n} %` go through the catalogue's
`formatNumber(language, n)` and the French entries' own strings, so the French output is the same
bytes (change 13 D4).

### D5 — Tests

The existing spec files render these surfaces with the default (French) copy and assert French:
they pass unchanged, and that is the byte-for-byte check. Each surface gains one test rendering
it with `en` and asserting three texts, so the plumbing is exercised in every language; the
`lang` attributes are asserted. `lint-copy`'s baseline loses the files, and `lint-language-labels`
is unchanged (the languages' names still come through the seam, change 19).

## Risks / Trade-offs

- **A text the inventory missed** → the lint fails when the file leaves the baseline, naming it.
- **A page flashing empty** → hidden until filled, as today's popup; measured by eye on the five
  targets in the pull request.
- **The popup's « Réviser (n) » assembled in the HTML** → a slot message rendered into the node.

## Migration Plan

One release, silent. No stored state, no wire.
