## 1. What a highlight can paint

- [ ] 1.1 Spike: paint `text-decoration-thickness`, each `text-decoration-style` and `color` inside `::highlight()` on Chromium, Firefox and WebKit (Playwright, pixel sampling); record the results in design D6
- [ ] 1.2 `build.mjs`: `__HIGHLIGHT_THICKNESS__` / `__HIGHLIGHT_STYLES__` per variant from 1.1, declared in `env.d.ts`

## 2. The preference and its sheet

- [ ] 2.1 `state/storage.ts`: `COLOURS_KEY`, the `ColourPreference` model, `colourPreferenceOf` (safe fallback to Cymbra), load and save (design D1)
- [ ] 2.2 `styles/tokens.css`: the e-ink presets' tokens and `--cymbra-lingua-paper-ink` (design D2, D7)
- [ ] 2.3 `reading/colours.ts`: `colourCss(preference)` (empty for Cymbra), `statusStyle(colours, status)` for the preview, `resolvePreset(doc, preset)` from computed tokens (design D2, D3)
- [ ] 2.4 `reading/highlight.ts`: `applyColourSheet(doc, css)` — a second adopted sheet per document, `<style>` fallback (design D3)
- [ ] 2.5 Tests: validation, `colourCss` per preset and for a custom set, every variable it writes exists in `tokens.css`, `applyColourSheet` adopt/replace/fallback

## 3. Applied everywhere

- [ ] 3.1 `ReadingSession`: load at start, apply wherever the tokens are injected, re-apply to attached documents on change (design D4)
- [ ] 3.2 Reader page: apply to its own document; `bookStyles` reads `PageColours` and forces the paper text colour only when chosen; re-render the display on change
- [ ] 3.3 Tests: a web page and a book section repainted on change without reload; paper text kept unless chosen; the dark page's text replaced

## 4. The Couleurs block

- [ ] 4.1 `reading/colour-settings-view.ts`: presets, preview, the two status fieldsets, the page fieldset, reset, the warning; editing from a preset stores a custom set (design D5)
- [ ] 4.2 Mount it in `settings-view.ts` (every Réglages surface); French copy
- [ ] 4.3 Tests: preset choice, a setting changed from a preset, the warning, reset, options absent where the build has no support
- [ ] 4.4 Lint, format, typecheck, tests and coverage of `apps/lingua-extension` green

## 5. On the devices

- [ ] 5.1 Chromium (built extension): preset change repaints an open web page and an open book
- [ ] 5.2 Boox Go Color 7 Gen 2 (Firefox for Android): tune the two e-ink presets until unknown and learning words read at a glance; record the values in design D7
- [ ] 5.3 Safari iPad: the colour pickers from the drawer; the preview
- [ ] 5.4 Firefox desktop: the block in the sidebar, a web page repainted

## 6. Release

- [ ] 6.1 Shipped with the next extension release, after `add-lingua-reader` (its task 7.4) — not before Guillaume's go-ahead
