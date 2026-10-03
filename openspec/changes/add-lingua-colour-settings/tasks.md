## 1. What a highlight can paint

- [x] 1.1 Spike: paint `text-decoration-thickness`, each `text-decoration-style` and `color` inside `::highlight()` on Chromium, Firefox and WebKit (Playwright, pixel sampling); record the results in design D6
  - 2026-10-01: Chromium 151, Firefox 153, WebKit 26.5 paint all of them (design D6).
- [x] 1.2 `build.mjs`: `__HIGHLIGHT_THICKNESS__` / `__HIGHLIGHT_STYLES__` per variant from 1.1, declared in `env.d.ts`
  - Not needed: 1.1 found every engine paints every property (design D6), so nothing is compiled out.

## 2. The preference and its sheet

- [x] 2.1 `state/storage.ts`: `COLOURS_KEY`, the `ColourPreference` model, `colourPreferenceOf` (safe fallback to Cymbra), load and save (design D1)
- [x] 2.2 `styles/tokens.css`: the e-ink presets' tokens and the page tokens `--cymbra-lingua-page-*` (design D2, D3, D7)
- [x] 2.3 `reading/colours.ts`: `colourCss(preference)` (empty for Cymbra), `statusStyle(colours, status)` for the preview, `resolvePreset(doc, preset)` from computed tokens (design D2, D3)
- [x] 2.4 `reading/highlight.ts`: `applyColourSheet(doc, css)` — a second adopted sheet per document, `<style>` fallback (design D3)
- [x] 2.5 Tests: validation, `colourCss` per preset and for a custom set, every variable it writes exists in `tokens.css`, `applyColourSheet` adopt/replace/fallback

## 3. Applied everywhere

- [x] 3.1 `ReadingSession`: load at start, apply wherever the tokens are injected, re-apply to attached documents on change (design D4)
- [x] 3.2 Reader page: apply to its own document; `bookStyles` reads `PageColours` and forces the paper text colour only when chosen; re-render the display on change
- [x] 3.3 Tests: a read document (book section) repainted on change without reload, no sheet for Cymbra; the reader page restyles the open book; paper text kept unless chosen; the dark page's text replaced

## 4. The Couleurs block

- [x] 4.1 `reading/colour-settings-view.ts`: presets, preview, the two status fieldsets, the page fieldset, reset, the warning; editing from a preset stores a custom set (design D5)
- [x] 4.2 Mount it in `settings-view.ts` (every Réglages surface); French copy
- [x] 4.3 Tests: preset choice, a setting changed from a preset, every control, the warning, reset, refresh (no option is compiled out: design D6)
- [x] 4.4 Lint, format, typecheck, tests and coverage of `apps/lingua-extension` green

## 5. On the devices

- [x] 5.1 Chromium (built extension): preset change repaints an open web page and an open book
  - 2026-10-01, `dist-chromium` under Playwright: an open book is repainted at once by E-ink contrasté then E-ink couleur (the section's colour sheet adopted last); the Couleurs block renders in the side panel with its preview. A web page is still to see by hand — the content script needs the toolbar click on Chromium.
- [x] 5.2 Boox Go Color 7 Gen 2 (Firefox for Android): tune the two e-ink presets until unknown and learning words read at a glance; record the values in design D7
  - 2026-10-03, Boox Go 10.3 Lumi (monochrome), Firefox 157: Guillaume validates *E-ink contrasté* as shipped. *E-ink couleur* still to see on the Go Color 7, whose colour layer this panel does not have.
  - 2026-10-03: no colour e-ink panel is available. Guillaume accepts *E-ink couleur* as shipped (design D7).
- [x] 5.3 Safari iPad: the colour pickers from the drawer; the preview
- [x] 5.4 Firefox desktop: the block in the sidebar, a web page repainted
  - 2026-10-03: 5.1, 5.3 and 5.4 validated by Guillaume, with the surfaces' themes and text size (tasks 7.x).

## 7. The surfaces on a large e-ink screen (dogfood, Boox Go 10.3 Lumi, 2026-10-03)

- [x] 7.1 `styles/tokens.css`: the surfaces' light and e-ink themes on `:host([data-cymbra-lingua-ui])` and `:root[data-cymbra-lingua-ui]`, and `--cymbra-lingua-warn`, `-ok`, `-danger` for what the surfaces say in colour (design D8)
- [x] 7.2 `reading/surface-look.ts`: the palette from the preset and the page theme, the scale from the text size, followed by the word card, the drawer, the pill and the six extension pages (design D8, D9)
- [x] 7.3 Every surface stylesheet: font sizes and the main widths multiply `--cymbra-lingua-ui-scale`, clamped to the viewport; notices, successes and dangers through the themed tokens (design D9)
- [x] 7.4 Réglages: the text size and the theme move to an *Affichage* block before *Couleurs*; the page row is labelled *Thème* (design D9)
- [x] 7.5 Tests: the palette per preset and theme, a root painted then following every change, the three hosts, no fixed font size and no preset colour for messages in any stylesheet, every surface token re-pointed by both themes, the *Affichage* block
- [x] 7.6 Boox Go 10.3 Lumi: at the reader's size, the word card, the drawer, the toolbar, the library and the pill read comfortably; with an e-ink preset they are black on white
  - 2026-10-03, Firefox 157 on the Boox: validated by Guillaume, the colours and the text size changing every surface.

## 6. Release

- [ ] 6.1 Shipped with the next extension release, after `add-lingua-reader` (its task 7.4) — not before Guillaume's go-ahead
