# Tasks — add-lingua-wasm

## 1. Cible WASM (spec lingua-analysis — parité native/WASM)

- [ ] 1.1 Crate/feature `lingua-wasm` : bindings wasm-bindgen (analyse par lot de blocs → tokens classés, statuts, %, gloses) ; build wasm-pack `--target web`
- [ ] 1.2 Test de parité natif/WASM sur les fixtures (même `analyzer_version` ⇒ sorties identiques)
- [ ] 1.3 Lane CI : build wasm + exécution des tests de parité
