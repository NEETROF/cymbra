# lingua-analysis — cible WASM et parité

## ADDED Requirements

### Requirement: Parité native/WASM
Le cœur SHALL être compilable en module WASM (build wasm-pack `--target web`) exposant l'analyse par lot de blocs (tokens classés, statuts, pourcentage, gloses), et SHALL produire, à `analyzer_version` égale et pack égal, des sorties identiques octet pour octet entre la cible native et la cible WASM sur le corpus de fixtures. Une lane CI SHALL construire le module WASM et exécuter les tests de parité.

#### Scenario: Parité sur le corpus de fixtures
- **WHEN** le même texte de fixture est analysé par le binaire natif et par le module WASM à la même `analyzer_version` et avec le même pack
- **THEN** les sorties (token, lemme, classement, pourcentage) sont identiques octet pour octet

#### Scenario: Divergence bloquée en CI
- **WHEN** une modification du cœur fait diverger la sortie WASM de la sortie native sur une fixture
- **THEN** la lane CI de parité échoue
