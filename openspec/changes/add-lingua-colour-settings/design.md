## Context

The colours the extension paints with live in one token sheet, `src/styles/tokens.css`,
checked by `test/lint-hex.spec.ts`: no hex outside it. The same sheet holds the two
`::highlight()` rules:
- *learning*: amber fill at 28 %, dotted amber underline;
- *unknown*: coral fill at 30 %, solid coral underline.

The reading session adopts the sheet into every document it reads (`injectPageStyles`,
`reading/session.ts`): a web page for the content script, each section of a book for the reader.
The reader's page colours come from the same tokens: `--cymbra-lingua-paper` behind the book, and
`--cymbra-lingua-night*`, read by `nightColoursOf` and written into each section's sheet by
`bookStyles`. The reader's display (text size, paper or dark) is a `chrome.storage.local`
preference, `cymbra-lingua-reader-display`. It is edited by one builder
(`reading/book-display-view.ts`) rendered in two places, and followed through
`storage.onChanged`.

The trigger is a Boox Go Color 7 Gen 2 (Kaleido 3). The colour filter over the e-ink panel
washes pastels out and draws colour at 150 ppi against 300 ppi for black. So the unknown fill
vanishes and the default underline is a hairline.

Only four properties apply in a `::highlight()`: `color`, `background-color`, `text-decoration`
(with its longhands) and `text-shadow`. Weight, size and borders are not available, so every
preset and setting is built from those four.

## Goals / Non-Goals

**Goals:**
- Presets that make the highlight readable on a monochrome and on a colour e-ink panel.
- A free setting for each part a highlight can paint, per status.
- The paper and dark page colours of the reader.
- One preference, applied at once to every open page and book.
- The default path unchanged — no extra sheet, no extra paint, for a reader who never opens
  the block.

**Non-Goals:**
- Syncing the choice across devices. It is a screen's preference: the e-ink preset is wrong on
  the laptop. It stays on the device, like the reader's display.
- Restyling web pages: their background and text are the site's. Only the highlights change
  there.
- Recolouring the extension's own chrome (popup, drawer, panels): the identity stays.
- Marking known words.
- A contrast checker. A warning covers the one failure the spec names: two statuses
  indistinguishable without colour.

## Decisions

### D1. One preference: a preset, or a full custom set

`cymbra-lingua-colours` in `chrome.storage.local`:

```ts
type ColourPreset = "cymbra" | "eink-mono" | "eink-colour";
interface StatusColours {
  fill: { colour: Hex; intensity: "none" | "light" | "strong" };
  underline: { colour: Hex; style: "solid" | "dotted" | "dashed" | "wavy" | "double" | "none"; thickness: "thin" | "thick" };
  text: Hex | null;               // null: the page's own colour
}
interface PageColours { background: Hex; text: Hex | null } // paper: null keeps the book's text
interface Colours { unknown: StatusColours; learning: StatusColours; paper: PageColours; dark: PageColours & { text: Hex } }
type ColourPreference = { preset: ColourPreset } | { preset: "custom"; colours: Colours };
```

`colourPreferenceOf(value)` makes a stored value safe, like `readerDisplayOf`: an unknown preset
or a malformed colour (not `#rrggbb`) falls back to the Cymbra preset. A custom set is stored
whole, not as a diff over a preset, so a later retune of a preset never changes what a reader
set by hand.

Alternative: one key per setting. That gives many `onChanged` events for one gesture, and no
single "customised" state to show.

### D2. Presets are tokens; a custom colour is the reader's data

The two e-ink presets are declared in `tokens.css` as `--cymbra-lingua-eink-*` tokens, next to
the Cymbra tints, so the "no colour outside the token sheet" rule holds. The sheet a preset
produces references them through `var()`. A custom set carries the reader's hex values: they
are data, not code. The Cymbra preset produces **no** sheet at all (Goals: the default path is
unchanged).

When the reader edits a setting while a preset is on screen, the editor fills itself from that
preset's resolved values (`getComputedStyle` on the settings document, as `nightColoursOf`
already does for the night colours). The result is stored as `custom`.

### D3. A second sheet, generated, adopted after the token sheet

`colourCss(preference): string` is a pure function. It returns:
- `:root, :host { … }` overriding the **page tokens** `--cymbra-lingua-page-paper`,
  `--cymbra-lingua-page-night`, `--cymbra-lingua-page-night-ink`, and setting
  `--cymbra-lingua-page-paper-ink` (unset in the token sheet: the book keeps its own text on
  paper). The token sheet defaults each page token to the identity's colour
  (`--cymbra-lingua-page-paper: var(--cymbra-lingua-paper)`), and the reader page and the book
  read only the page tokens. *As built:* the sheet first overrode the identity tokens themselves,
  but the Cymbra preset is *made of* those tokens, so editing from Cymbra after a custom choice
  read the custom values back. The identity tokens are never overridden;
- the two `::highlight()` rules in full. At equal specificity a later sheet wins, so these
  replace the token sheet's for every property they set. `color` is emitted only when a text
  colour is chosen, `text-decoration-thickness` only for "thick".

The intensity maps to an alpha: none 0, light 0.3, strong 0.65. For a preset, the fill is a
token that already carries its alpha.

`applyColourSheet(doc, css)` sits next to `injectPageStyles`. It keeps a second constructable
sheet per document (same `WeakMap` pattern, same `<style>` fallback) and `replaceSync`s it on
every change. That is one sheet per document, adopted after the token sheet, which also keeps
it safe from SPA morphing (see `injectPageStyles`).

### D4. The session owns applying it; the reader page follows for its own page

`ReadingSession` loads the preference at `start` and applies it wherever it injects the tokens
today (both call sites of `injectPageStyles`). It re-applies to every attached document on
`storage.onChanged` for the key, through the same `AsyncStorageArea` seam plus a
`watchColours` callback, so tests need no `chrome`.

The reader page applies the sheet to its own document, for the page tokens behind the book. It also calls `renderer.setDisplay` again, so that
`bookStyles` re-reads the page colours.

`bookStyles` gains the paper text: when `--cymbra-lingua-page-paper-ink` resolves to a colour, the
paper page's sheet sets `html, body { color: … !important }`, as the dark page does. Otherwise
it keeps the book's colours. `NightColours` becomes `PageColours`, read the same way.

### D5. One builder, in Réglages

`reading/colour-settings-view.ts` renders the block, like `book-display-view.ts`. The settings
builder (`settings-view.ts`, the one builder of every Réglages surface) mounts it as a
**Couleurs** block. The block contains, in order:
- the preset (radio: Cymbra / E-ink contrasté / E-ink couleur / Personnalisé);
- the preview;
- two fieldsets, *Mots inconnus* and *Mots en cours*: fill colour + intensity, underline
  colour + style + thickness, text colour ("celle de la page" or a picker);
- *Page du lecteur*: paper background + text ("celle du livre" or a picker), dark background +
  text;
- *Rétablir les couleurs Cymbra*.

Colours use `<input type="color">`, which exists on every target (Safari ≥ 14.1, Firefox
Android). Selects handle the rest: a tap per choice, one redraw on e-ink.

The **preview** is painted with inline styles computed by the same function as the sheet
(`statusStyle(colours, status)`), not with `CSS.highlights`. Rendered in the content-script
drawer, a registry entry under the extension's highlight names would replace the page's own
highlights of that name.

The **warning** appears under the fieldsets when the two underline styles are equal, or both
"none".

### D6. Measured, not assumed: what a highlight paints on each engine

`text-decoration-thickness` and the `wavy` / `double` styles inside `::highlight()` were not
known to be uniformly implemented. A spike (task 1.1) painted each property in a highlight on
Chromium, Firefox and WebKit through Playwright and compared the pixels with an unpainted word,
and with a plain solid underline.

**Measured 2026-10-01** — Chromium 151, Firefox 153, WebKit 26.5 (Safari's engine): every
engine paints `background-color`, `color`, `text-decoration` in every style (solid, dotted,
dashed, wavy, double), `text-decoration-thickness` and `text-underline-offset` inside a
highlight. Firefox draws the default underline thinner than the others (a hairline, which is
the Kaleido problem itself), so the thickness setting matters most there.

So nothing is compiled out per variant: no `__HIGHLIGHT_*__` constants. The rule stands for a
future engine (spec: "A setting that the browser cannot paint … SHALL NOT be offered"); none of
today's needs it.

### D7. The presets, provisional until the Boox

| | Unknown | Learning | Paper page |
|---|---|---|---|
| **Cymbra** | today's | today's | today's |
| **E-ink contrasté** | fill mid-grey, strong; solid black underline, thick | no fill; dotted black underline, thick | white, black text |
| **E-ink couleur** | text dark red; solid dark-red underline, thick; light fill | text dark blue; dotted dark-blue underline, thick | white, book's text |

Kaleido panels render saturated dark colours best and wash out yellows and pastels, hence ink
on the word rather than a tint behind it. The values are tuned on the Go Color 7 Gen 2 (task
5.2) before release, and the tuned values are recorded here.

## Risks / Trade-offs

- [A reader picks colours that hide the text (white on white)] → the preview shows it before
  the book does, and *Rétablir* is one tap away. There is no contrast checker (Non-Goals).
- [A dark page chosen with a light text colour that the paper preset then inherits] → paper and
  dark have separate colours; a preset sets both.
- [`onChanged` fires in every tab for every edit of a picker] → the picker writes on `change`
  (release), not `input` (drag), so one edit is one write.
- [The token names become a contract between the sheet and `colourCss`] → a test asserts that
  every variable `colourCss` writes exists in `tokens.css`.
- [Safari's `<input type="color">` opens a system sheet over the drawer] → accepted; checked on
  the iPad (task 5.3).

## Migration Plan

None. With no stored preference the extension behaves as today (the Cymbra preset, no sheet).
It ships with the next extension release, after `add-lingua-reader` (its task 7.4). This change
archives after it.

## Open Questions

- ~~Which highlight properties each engine paints (D6)~~ — answered: all of them (task 1.1).
- The e-ink presets' exact values (D7) — answered on the Boox.
