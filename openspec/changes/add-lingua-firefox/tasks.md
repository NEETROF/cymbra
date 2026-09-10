# Tasks — add-lingua-firefox

## 1. Port Firefox (desktop + Android)

- [ ] 1.1 Spike jour 1 : WASM dans un content script Firefox (CSP) — verdict documenté ; le design assume le repli event page quel que soit le résultat
- [ ] 1.2 Système de variantes de manifest introduit dans le build : variante `firefox` générée à côté de `chromium` (event page `background.scripts` déclaré à côté du `service_worker`, CSP `wasm-unsafe-eval` explicite, `browser_specific_settings`)
- [ ] 1.3 Impl `AnalyzerPort` event page : WASM chargé dans l'event page, requêtes par lots, mémoïsation par forme côté content script
- [ ] 1.4 Permissions optionnelles à l'install (détection `permissions.contains` + prompt) et panneau via `sidebar_action` (même page que le side panel)
- [ ] 1.5 Parcours manuel Firefox desktop + Android (`web-ext run` / adb) documenté ; publication AMO (desktop + Android, même zip)
