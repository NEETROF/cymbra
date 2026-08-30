# lingua-data-packs — packs de données linguistiques

## ADDED Requirements

### Requirement: Pack conteneur versionné par paire de langues
Un pack SHALL être un conteneur unique versionné, clé par paire (langue étudiée → langue maternelle), contenant : métadonnées (paire, `pack_version`, `analyzer_version` compatible, licences), FST formes→lemmes, table de fréquence (rangs), gloses compressées indexées par lemme, et fichier NOTICE. Le cœur SHALL refuser un pack dont la version d'analyseur est incompatible.

#### Scenario: Chargement du pack EN→FR
- **WHEN** l'extension démarre avec le pack (en → fr) embarqué
- **THEN** le cœur expose lemmatisation, rangs de fréquence et gloses françaises pour l'anglais

#### Scenario: Pack incompatible
- **WHEN** un pack déclare une `analyzer_version` incompatible avec le cœur
- **THEN** le chargement échoue avec une erreur explicite, sans analyse partielle

### Requirement: Hygiène de licences
Le pipeline de construction SHALL n'accepter que des sources dont la licence autorise l'usage commercial (AGID, WordNet, wordfreq CC BY-SA, kaikki CC BY-SA) et SHALL rejeter toute source GPL, AGPL ou non-commerciale (liste noire documentée). Chaque pack SHALL embarquer la pile complète des notices, et l'interface utilisateur SHALL exposer une page d'attributions.

#### Scenario: Notices embarquées
- **WHEN** un pack est construit
- **THEN** son NOTICE contient les attributions AGID (avec sa pile amont, dont WordNet), wordfreq et kaikki, et la page « Attributions » de l'extension les affiche

### Requirement: Construction hors-ligne reproductible
Les packs SHALL être construits par un pipeline scripté et reproductible à partir de sources datées ; les données brutes ne SHALL PAS être commitées. Deux exécutions du pipeline sur les mêmes sources SHALL produire des packs identiques.

#### Scenario: Rebuild du pack
- **WHEN** le pipeline est relancé sur les mêmes fichiers sources
- **THEN** le pack produit est identique octet pour octet au pack committé

### Requirement: Budget de taille
Le pack (en → fr) embarqué dans l'extension SHALL rester sous 5 Mo (FST + fréquences + gloses compressées). En cas de dépassement, le build SHALL échouer ; la remédiation SHALL réduire la couverture des gloses, jamais celle du FST ni des fréquences.

#### Scenario: Arbitrage de taille
- **WHEN** le `gloss.zst` fait dépasser les 5 Mo au build
- **THEN** le build échoue avec la consigne de réduire le nombre de lemmes glosés
