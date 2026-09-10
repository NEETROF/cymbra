# lingua-agent-capture — ingestion des sessions d'agents IA

## ADDED Requirements

### Requirement: Ingestion des transcripts Claude Code, locale par construction
Un hook `Stop` SHALL invoquer le binaire `lingua` avec le chemin du transcript ; le binaire SHALL extraire le texte des messages assistant du JSONL, l'analyser (natif, même `analyzer_version` que l'extension) et mettre à jour les compteurs d'exposition par lemme dans le store local (`~/.lingua/`). Le contenu des transcripts (phrases, code) ne SHALL PAS être persisté ni transmis sur le réseau — seuls lemmes, compteurs et horodatages le sont, plus les phrases explicitement capturées par l'utilisateur via `/vocab`.

#### Scenario: Fin de tour Claude Code
- **WHEN** Claude Code termine une réponse et le hook `Stop` s'exécute
- **THEN** les lemmes du texte assistant voient leurs compteurs d'exposition incrémentés et aucune phrase du transcript n'apparaît dans le store

#### Scenario: Aucune sortie réseau
- **WHEN** l'ingestion s'exécute
- **THEN** le binaire n'ouvre aucune connexion réseau

### Requirement: Statusline de couverture
La statusline SHALL afficher, pour la session courante, le pourcentage de mots connus du dernier message assistant et le nombre de mots nouveaux rencontrés dans la session, en se dégradant silencieusement (aucune sortie d'erreur) si le transcript est illisible.

#### Scenario: Affichage après une réponse
- **WHEN** la statusline s'exécute après une réponse contenant 3 lemmes inconnus
- **THEN** elle affiche le % connu et « 3 nouveaux »

### Requirement: Commande /vocab
Une skill `/vocab` SHALL lister les mots inconnus de la session courante avec leur forme du dictionnaire, leur glose et leur rareté, et SHALL permettre d'ajouter une sélection de ces mots au deck (cartes avec phrase d'origine du transcript, ajoutée avec le consentement explicite de l'utilisateur).

#### Scenario: Revue de session
- **WHEN** l'utilisateur invoque `/vocab` après une session contenant 4 mots inconnus
- **THEN** les 4 mots sont listés avec gloses et l'utilisateur peut en ajouter tout ou partie au deck

### Requirement: Serveur MCP pour les opérations de deck
Le plugin SHALL exposer un serveur MCP avec au minimum `list_decks`, `add_words`, `due_cards` et `answer_card` (notation FSRS `again/hard/good/easy`), opérant sur le store local. Les entrées SHALL être validées (lemmes normalisés, tailles bornées). La révision des cartes du store plugin SHALL être possible via l'agent (quiz conversationnel `due_cards` → `answer_card`).

#### Scenario: Ajout par l'agent
- **WHEN** l'utilisateur demande à Claude « ajoute ces trois mots à mon deck » et que Claude appelle `add_words`
- **THEN** trois cartes sont créées dans le store local et `due_cards` les reflète

#### Scenario: Révision conversationnelle
- **WHEN** l'agent fait réviser l'utilisateur et note une carte via `answer_card` avec `good`
- **THEN** l'état FSRS et l'échéance de la carte sont mis à jour dans le store local

### Requirement: Sources de session extensibles
L'ingestion SHALL passer par une abstraction `SessionSource` (localisation des sessions, extraction du texte assistant) dont Claude Code est la première implémentation, de sorte qu'un nouvel agent s'ajoute sans modifier le pipeline d'analyse.

#### Scenario: Ajout d'une source
- **WHEN** une implémentation `SessionSource` de test fournit un transcrit factice
- **THEN** le pipeline d'ingestion produit les mêmes compteurs que pour un transcript Claude Code équivalent
