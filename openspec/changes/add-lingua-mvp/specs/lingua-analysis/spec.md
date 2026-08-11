# lingua-analysis — pipeline d'analyse de texte

## ADDED Requirements

### Requirement: Tokenisation des textes en langue étudiée
Le cœur SHALL tokeniser un texte en mots candidats selon UAX #29 (`unicode-segmentation`), avec une pré-passe par langue étudiée (pour l'anglais : contractions `don't` → `do` + `not`, apostrophes de bord retirées). Les tokens d'un seul caractère ne SHALL être comptés que s'ils appartiennent au lexique de la langue étudiée (« I », « a » en anglais).

#### Scenario: Texte anglais courant
- **WHEN** le texte « Teams don't ship code. » est analysé en anglais
- **THEN** les tokens produits sont `Teams`, `do`, `not`, `ship`, `code`

### Requirement: Lemmatisation en cascade
Pour chaque token, le cœur SHALL produire un lemme via la cascade : table d'exceptions (irréguliers) → lookup FST formes→lemmes du pack → repli morphologique (règles type morphy) → repli pluriel régulier pour les formes hors lexique → la forme elle-même. Le lemme retourné SHALL être en minuscules normalisées.

#### Scenario: Forme irrégulière
- **WHEN** le token `went` est lemmatisé
- **THEN** le lemme est `go`

#### Scenario: Forme hors lexique au pluriel régulier
- **WHEN** le token `endeavors` est lemmatisé et que ni `endeavors` ni `endeavor` ne sont dans le lexique de fréquence
- **THEN** le lemme est `endeavor` (repli pluriel), de sorte que `endeavor` et `endeavors` comptent comme un seul mot différent

### Requirement: Détection de langue par bloc
Le cœur SHALL déterminer par bloc de texte si le contenu est dans la langue étudiée et SHALL exclure de l'analyse les blocs qui ne le sont pas (dont les blocs en langue maternelle de l'utilisateur). Un document sans contenu suffisant dans la langue étudiée SHALL être signalé comme non analysable.

#### Scenario: Page majoritairement française pour un apprenant d'anglais
- **WHEN** une page dont le texte est en français est analysée (langue étudiée : anglais, langue maternelle : français)
- **THEN** l'analyse retourne « non analysable » et aucun token n'est compté

### Requirement: Calcul du pourcentage de tokens connus
Le cœur SHALL calculer le pourcentage de mots connus d'un texte comme `tokens connus / tokens comptés`, en comptant les **occurrences** (tokens), pas les mots uniques. Les tokens dont le lemme est ignoré SHALL compter comme connus ; les tokens en statut « en cours » SHALL compter comme non connus ; les noms propres hors lexique SHALL être exclus du décompte.

#### Scenario: Mot répété inconnu
- **WHEN** un texte de 10 tokens comptés contient 2 occurrences d'un même lemme inconnu et 8 tokens connus
- **THEN** le pourcentage connu est 80 %

### Requirement: Déterminisme à version d'analyseur donnée
Le cœur SHALL exposer une `analyzer_version` et SHALL produire, à version égale et pack égal, des résultats identiques quelle que soit la cible de compilation (natif ou WASM).

#### Scenario: Parité natif / WASM
- **WHEN** le même texte de fixture est analysé par le binaire natif et par le module WASM à la même `analyzer_version`
- **THEN** les listes (token, lemme, classement) produites sont identiques octet pour octet
