# lingua-browser-extension — variante Firefox (delta)

## ADDED Requirements

### Requirement: Variante Firefox
L'extension SHALL être livrée sur Firefox (desktop et Android) comme variante de build construite depuis la même source que la variante chromium : event page (`background.scripts` déclaré à côté du `service_worker`), analyse WASM chargée dans l'event page et consommée par le content script via l'`AnalyzerPort`, host permissions optionnelles demandées à l'install, panneau via `sidebar_action` (la même page que le side panel Chromium), et publication AMO desktop + Android avec le même zip.

#### Scenario: Un build, deux artefacts
- **WHEN** le build de release s'exécute
- **THEN** il produit les variantes chromium et firefox depuis la même source, ne différant que par le manifest et l'implémentation de l'`AnalyzerPort` (la variante safari rejoindra ce build avec `add-lingua-apple`)

#### Scenario: Permissions Firefox à l'installation
- **WHEN** l'utilisateur installe l'extension sur Firefox
- **THEN** l'extension fonctionne en mode « surligner cette page » et propose le grant global via le prompt de permissions optionnelles
