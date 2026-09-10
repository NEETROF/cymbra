# Design — add-lingua-decks-review

## Context

Troisième étage de la pile Lingua : le cœur sait analyser (`add-lingua-analysis`) et connaît l'état de l'apprenant (`add-lingua-knowledge-model`) ; ce change y ajoute decks, cartes et planification de révision. Les décisions du knowledge model — clé `(langue étudiée, lemme)`, expressions multi-mots = lemmes avec espaces, statuts et champ `known_source` (`manual | calibration | srs | import`) — sont héritées de `add-lingua-knowledge-model` et ne sont pas rediscutées ici. Comme le reste du cœur : logique pure host-testée (`*_core.rs`), coverage ≥ 80 %, aucune UI dans ce change.

## Decisions

### D1 — Schéma de carte : provenance obligatoire, média au schéma dès le jour 1
Une carte porte le lemme, la forme rencontrée, la phrase de contexte d'origine, la source de la rencontre (URL ou identifiant de session d'agent, horodatage) et la glose. La carte gagnante du sentence mining est mot + forme + phrase ; la phrase d'origine ne peut être capturée qu'au moment de la rencontre — la provenance est donc non négociable dès la création, jamais reconstituable après coup. L'emplacement de média (`media`, avec `source: capture|banque|génération` et `sync_policy`) est présent dans le schéma **sans être peuplé** : la capture d'image est différée, mais le schéma sérialisé versionné est un contrat que l'export (D4) et la future sync figeront — l'ajouter plus tard serait une migration, l'ajouter maintenant est une colonne vide. Les expressions multi-mots sont des cartes de plein droit.

### D2 — Révision : FSRS via le crate `fsrs`, le même planning partout
L'état FSRS (stabilité, difficulté, échéance) vit sur la carte, calculé dans `lingua-core` — le même planning partout (extension WASM, app Apple, plugin natif, dans les changes suivants). Notation `again/hard/good/easy` ; le compteur de cartes dues est calculable à tout instant par le cœur. Version du crate épinglée ; les états FSRS stockent leurs paramètres (le crate évolue, ses défauts aussi). Les surfaces d'UI de la révision (side panel, drawer, popup d'icône) relèvent d'`add-lingua-extension-review` — le badge de l'icône restera dédié au % de la page, le compteur de dues vivra dans le popup et le side panel (pas de badge « dues » ni d'alarme en v1).

### D3 — « Je connais » ferme la boucle avec le knowledge model
Marquer « je connais » en révision passe le lemme en `known` provenance `srs` — le champ `known_source` d'`add-lingua-knowledge-model` préparait exactement cette inférence façon Migaku — et retire la carte de la file **sans la supprimer ni effacer son historique** : l'état FSRS reste, et le mot peut revenir en apprentissage plus tard sans perte.

### D4 — Sauvegarde/restauration d'abord, export Anki différé
Pendant toute la phase locale de la pile (avant `add-lingua-backend`/`add-lingua-connected-clients`), l'état vit dans le storage d'un profil de navigateur — fragile (reset de profil = tout perdu). Le filet de sécurité du MVP est donc une **sauvegarde/restauration complète** (fichier versionné : cartes, statuts, calibration, paramètres FSRS ; aller-retour sans perte testé), qui sert aussi de migration d'appareil pré-sync. **L'export au format Anki est différé** (décision utilisateur) : ce qui coûte cher à rattraper n'est pas le sérialiseur mais le schéma (leçon Lute) — le schéma reste exportable par construction (champs propres, provenance, glose séparée), et le CSV Anki viendra comme change dédié quand des utilisateurs le demanderont.

## Risks / Trade-offs

- [Le crate `fsrs` évolue (paramètres par défaut)] → épingler la version ; les états FSRS stockent leurs paramètres.
- [Le requirement « au ras de la lecture » est spécifié avant ses surfaces] → assumé : c'est le contrat de la capability ; `add-lingua-extension-review` le réalise, ce change livre tout ce dont il a besoin (dues, transitions d'état, session de révision côté cœur).
- [Champ `media` inerte dans ce change] → coût quasi nul (colonne vide dans l'export) ; gain : ni migration de schéma ni rupture d'export quand la capture d'image arrivera.
