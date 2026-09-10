# lingua-browser-extension — variante Safari (delta)

## ADDED Requirements

### Requirement: Variante Safari
L'extension SHALL être livrée sur Safari macOS et iOS comme variante `safari` du build multi-cibles (même source que les variantes chromium et firefox), convertie via `safari-web-extension-converter` et hébergée par l'app conteneur Apple : l'analyse SHALL transiter en natif via nativeMessaging derrière l'`AnalyzerPort` (aucun WASM dans les contextes d'extension Safari), et le panneau injecté (drawer) SHALL porter seul la révision dans le navigateur, Safari n'ayant pas d'API de panneau. Les canaux tiers (Edge Canary Android, stores curés, forks Chromium) ne SHALL PAS être promis ni testés.

#### Scenario: Un build, trois artefacts
- **WHEN** le build de release s'exécute
- **THEN** il produit les variantes chromium, firefox et safari depuis la même source, ne différant que par le manifest et l'implémentation de l'`AnalyzerPort`

#### Scenario: Révision dans Safari sans API de panneau
- **WHEN** l'utilisateur ouvre la révision dans Safari
- **THEN** le drawer injecté (shadow DOM) porte la session de révision, sur le même état local que le reste de l'extension
