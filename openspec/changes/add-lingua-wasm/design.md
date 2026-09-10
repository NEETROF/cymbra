# Design — add-lingua-wasm

## Context

Cinquième étage de la pile Lingua : le cœur (`add-lingua-analysis`) est déterministe à `analyzer_version` donnée et dispose d'un corpus de fixtures de non-régression ; le pack (`add-lingua-data-pack`) est versionné et compatible-gaté. Ce change ajoute la seconde cible de compilation du même cerveau. Décisions héritées et non rediscutées : déterminisme contractuel et fixtures (`add-lingua-analysis`), format et budget 5 Mo du pack (`add-lingua-data-pack`).

## Decisions

### D1 — Cible WASM : wasm-pack `--target web`, bindings minces, logique dans le cœur
`lingua-core` compile en WASM via wasm-pack `--target web` — le format consommable tel quel par un content script ou une event page d'extension, sans bundler dédié. Les bindings wasm-bindgen (crate/feature `lingua-wasm`) sont une couche mince : analyse **par lot de blocs** → tokens classés, statuts, %, gloses — aucune logique dedans ; la logique reste dans `lingua-core`, host-testée (convention du monorepo : le glue de binding est exclu du coverage, comme le glue frb de music, la logique jamais). Un seul artefact WASM : ce qui varie par navigateur (où il s'instancie) est confiné derrière l'`AnalyzerPort`, décision portée par `add-lingua-extension-reading`.

### D2 — La parité est un contrat testé en CI, pas une promesse
Déterminisme contractuel hérité d'`add-lingua-analysis`, étendu à la cible croisée : à `analyzer_version` égale et pack égal, sortie identique octet pour octet entre natif et WASM, testée en CI sur le corpus de fixtures (fixtures croisées natif/WASM). La lane de parité échoue sur toute divergence — une divergence silencieuse entre cibles fausserait le % affiché par l'extension sans qu'aucun test natif ne le voie. C'est la garantie « un seul cerveau » sur laquelle reposent tous les changes de surface suivants.

## Risks / Trade-offs

- [Divergences natif/wasm32 (flottants, tailles d'entiers, ordre d'itération)] → sorties canonicalisées (représentations déterministes, collections ordonnées) ; la lane de parité est le filet — elle transforme une classe de bugs indétectables en échec de CI.
- [Taille du module (~1 Mo de code wasm + pack ≤ 5 Mo)] → budget du pack posé par `add-lingua-data-pack` ; instanciation lazy, mémoïsation et coût par onglet relèvent d'`add-lingua-extension-reading` (risque géré là-bas, cible < 50 ms d'init).
- [WASM dans les content scripts Firefox (CSP)] → hors périmètre ici : spike jour 1 du port Firefox (`add-lingua-firefox`) ; la cible `--target web` reste valable dans les deux emplacements (content script ou event page).
