# lingua-analysis — profil de texte local et couverture (délta)

## ADDED Requirements

### Requirement: Profil de texte local (TextProfile)
Le cœur SHALL construire, à partir d'une liste ordonnée de chapitres (titre, blocs de texte), un profil de texte local contenant : l'empreinte d'édition (hash du texte normalisé — NFC, espaces réduits), la langue, l'`analyzer_version` épinglée, les entrées `(lemme, compte, premier chapitre d'apparition)` et la liste des chapitres avec leur nombre de tokens comptés. Le profil ne SHALL contenir que des comptes — aucune phrase, aucun n-gramme, aucun ordre de mots — de sorte que le schéma soit partageable tel quel par un futur change communautaire, sans re-design.

#### Scenario: Profil d'un livre en deux chapitres
- **WHEN** un texte de deux chapitres où le lemme `conundrum` apparaît 3 fois (première apparition au chapitre 2) est profilé
- **THEN** le profil contient l'entrée (`conundrum`, 3, chapitre 2) et le nombre de tokens comptés de chaque chapitre, et aucune phrase du texte n'apparaît dans le profil sérialisé

#### Scenario: Déterminisme du profil et de l'empreinte
- **WHEN** la même liste de blocs est profilée en natif puis en WASM à la même `analyzer_version`
- **THEN** les deux profils, empreinte d'édition comprise, sont identiques octet pour octet

### Requirement: Couverture d'un profil par tokens
Le cœur SHALL calculer la couverture d'un TextProfile à partir de l'état de connaissance en comptant les tokens (chaque occurrence), jamais les mots uniques, avec les mêmes règles de classification que les pages (lemme ignoré = connu, « en cours » = non connu, noms propres hors lexique exclus du décompte) : pourcentage global, pourcentage par chapitre, nombre de lemmes à apprendre pour atteindre un seuil donné (lemmes pris par compte décroissant dans le livre), et la même projection par préfixe de chapitres (rampe).

#### Scenario: Mot fréquent inconnu pesé par ses occurrences
- **WHEN** un profil de 1 000 tokens comptés contient un lemme inconnu à 50 occurrences et 950 tokens connus
- **THEN** la couverture globale est 95 %, et apprendre ce seul lemme la porte à 100 %

#### Scenario: Chemin vers un seuil
- **WHEN** la couverture d'un profil est sous 95 % et que le chemin vers 95 % est calculé
- **THEN** le cœur retourne le plus petit nombre N de lemmes inconnus, pris par compte décroissant dans le livre, dont l'apprentissage porte la couverture à au moins 95 %
