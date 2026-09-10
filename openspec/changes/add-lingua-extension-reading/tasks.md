# Tasks — add-lingua-extension-reading

## 1. Extension navigateur — lecture (spec lingua-browser-extension)

- [ ] 1.1 Scaffold `apps/lingua-extension` : MV3, TS sans framework, Yarn, vitest, esbuild/vite build ; manifest `activeTab` + `optional_host_permissions <all_urls>` + `storage` + commandes clavier ; unité ajoutée au filtre `ci-units` avec sa lane vitest/lint
- [ ] 1.2 `AnalyzerPort` (messages) + instanciation lazy du WASM dans le content script ; mémoïsation par forme
- [ ] 1.3 Moteur de surlignage : TreeWalker → Ranges → deux registres Highlight ; styles `::highlight()` ; exclusions (script/style/éditable/hôtes d'UI)
- [ ] 1.4 Re-scan par sous-arbre muté (MutationObserver débouncé) + IntersectionObserver pour prioriser le visible ; test manuel sur 5 SPA lourdes documenté
- [ ] 1.5 Popup de mot (shadow DOM fermé) : forme du dictionnaire, forme vue, glose du pack, rareté vulgarisée, actions statuts ; propagation cross-onglets via `storage.onChanged` ; AUCUNE occurrence du mot « lemme » (test de lint des chaînes UI)
- [ ] 1.6 Capture de sélection au raccourci : extraction de la phrase d'origine, carte d'expression
- [ ] 1.7 Badge % par onglet + popup d'icône (stats, compteur de cartes dues, curseur de calibration, reset)
- [ ] 1.8 Charte Cymbra : `tokens.css` mirrorant `CymbraColors` (précédent : `apps/back-office/src/styles.css`), appliquée au popup d'icône, au popup de mot et aux pages d'extension (les surfaces de révision d'`add-lingua-extension-review` consommeront la même feuille) ; surlignages dérivés de l'ambre/corail de la palette ; lint « aucun hex hors tokens.css » branché sur la lane vitest/lint de 1.1
