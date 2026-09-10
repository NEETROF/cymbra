# Tasks — add-lingua-extension-reading

## 1. Browser extension — reading (spec lingua-browser-extension)

- [ ] 1.1 Scaffold `apps/lingua-extension`: MV3, framework-free TS, Yarn, vitest, esbuild/vite build; manifest with `activeTab` + `optional_host_permissions <all_urls>` + `storage` + keyboard commands; unit added to the `ci-units` filter with its vitest/lint lane
- [ ] 1.2 `AnalyzerPort` (messages) + lazy WASM instantiation in the content script; per-form memoisation
- [ ] 1.3 Highlight engine: TreeWalker → Ranges → two Highlight registries; `::highlight()` styles; exclusions (script/style/editable/extension UI hosts)
- [ ] 1.4 Per-mutated-subtree re-scan (debounced MutationObserver) + IntersectionObserver to prioritise what is visible; manual test on 5 heavy SPAs, documented
- [ ] 1.5 Word popup (closed shadow DOM): dictionary form, form as seen, pack gloss, plain-language rarity, status actions; cross-tab propagation via `storage.onChanged`; NO occurrence of the word "lemma" (UI-string lint test)
- [ ] 1.6 Selection capture on the shortcut: source-sentence extraction, phrase card
- [ ] 1.7 Per-tab percentage badge + icon popup (stats, due-card counter, calibration slider, reset)
- [ ] 1.8 Cymbra identity: `tokens.css` mirroring `CymbraColors` (precedent: `apps/back-office/src/styles.css`), applied to the icon popup, the word popup and the extension pages (the review surfaces from `add-lingua-extension-review` will consume the same sheet); highlight tints derived from the palette's amber/coral; the "no hex outside tokens.css" lint wired into the vitest/lint lane from 1.1
