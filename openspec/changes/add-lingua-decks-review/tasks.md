# Tasks — add-lingua-decks-review

## 1. lingua-core — decks et révision (spec lingua-decks-review)

_Tests sur mini-fixtures synthétiques (comme les étages précédents du cœur) ; le vrai pack arrive avec `add-lingua-data-pack`._

- [ ] 1.1 Schéma de carte (lemme, forme, phrase, source, glose, `media` optionnel avec `source`/`sync_policy`, état FSRS) sérialisable versionné
- [ ] 1.2 Intégration FSRS : notation `again/hard/good/easy`, échéances, compteur de dues ; version du crate épinglée, paramètres stockés sur l'état
- [ ] 1.3 « Je connais » en révision → statut `known` provenance `srs`, carte conservée hors file ; tests
- [ ] 1.4 Export Anki CSV (une colonne par champ du schéma, colonnes vides pour les champs non peuplés) avec tests de fidélité des champs
