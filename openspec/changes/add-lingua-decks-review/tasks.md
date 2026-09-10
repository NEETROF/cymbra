# Tasks — add-lingua-decks-review

## 1. lingua-core — decks et révision (spec lingua-decks-review)

_Tests sur mini-fixtures synthétiques (comme les étages précédents du cœur) ; le vrai pack arrive avec `add-lingua-data-pack`._

- [ ] 1.1 Schéma de carte (lemme, forme, phrase, source, glose, `media` optionnel avec `source`/`sync_policy`, état FSRS) sérialisable versionné
- [ ] 1.2 Intégration FSRS : notation `again/hard/good/easy`, échéances, compteur de dues ; version du crate épinglée, paramètres stockés sur l'état
- [ ] 1.3 « Je connais » en révision → statut `known` provenance `srs`, carte conservée hors file ; tests
- [ ] 1.4 Sauvegarde/restauration : export complet de l'état (cartes, statuts, calibration, paramètres FSRS) en fichier versionné + restauration à l'identique ; tests d'aller-retour sans perte (round-trip champ par champ)
