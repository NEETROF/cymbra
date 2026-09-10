# add-lingua-knowledge-model — Cymbra Lingua : état de connaissance lexicale

## Why

Le pipeline d'analyse (`add-lingua-analysis`) sait produire des lemmes ; il faut maintenant dire **ce que l'utilisateur en sait**. Le knowledge model est le contrat central du produit : c'est lui que consomment les decks, l'extension, le plugin agent et, plus tard, la sync — sa clé `(langue étudiée, lemme)` et ses statuts sont ce qui rend le comptage honnête (le différenciateur face à LingQ/Readlang, qui comptent les formes de surface). Ce change livre aussi les deux réponses au démarrage à froid — sans elles, le jour 1 surligne 60 % de la page : la **calibration par rang de fréquence** et l'**import LingQ** (qui est en même temps une arme d'acquisition : « migre depuis LingQ, garde ton historique »).

**Position dans la pile** (12 changes, ordre d'implémentation) : **2/12.** Prérequis explicite : **add-lingua-analysis** (crate `lingua-core`, lemmatisation, rangs de fréquence, `analyzer_version`). Suite : add-lingua-decks-review → add-lingua-data-pack → add-lingua-wasm → add-lingua-extension-reading → add-lingua-extension-review → add-lingua-firefox → add-lingua-apple → add-lingua-agent → add-lingua-backend → add-lingua-connected-clients.

## What Changes

- **Module `knowledge/` de `crates/lingua-core`** : statuts explicites (`learning`, `known`, `ignored` ; « nouveau » = absence d'entrée) avec provenance (`manual`/`calibration`/`srs`/`import`), « connu » implicite sous le seuil de calibration (rang de fréquence ≤ N), résolution multi-candidats pro-apprenant (connu si un candidat l'est).
- **Profil L1/L2** : `native_language` (langue de confort) distincte des langues étudiées ; toutes les API clées par paire (L2→L1). Le MVP ne livre que (anglais → français), mais ajouter une paire = données, pas du code.
- **Import LingQ (CSV)** : les entrées importées sont lemmatisées puis marquées `known` provenance `import` — le démarrage à froid du segment cible.
- **Compteurs d'exposition** par (langue, lemme) : occurrences rencontrées, source, horodatage — sans effet sur les statuts en v1 (donnée d'entrée de l'inférence future façon Migaku).
- **Invariant de vocabulaire UI** posé dès ce change : le mot « lemme » n'apparaît jamais à l'écran (« forme du dictionnaire », « mots différents ») — chaque surface ultérieure de la pile l'applique et le linte.

## Capabilities

### New Capabilities
- `lingua-knowledge-model` : l'état de connaissance par lemme et par langue étudiée — statuts (nouveau/en cours/connu/ignoré), « connu » inférable du SRS, calibration par rang de fréquence au démarrage, import LingQ (CSV), profil L1/L2 (langue maternelle ≠ langue étudiée, tout est clé par paire), compteurs d'exposition.

### Modified Capabilities
_Aucune. Ce change reste local au crate `lingua-core` : il ne consomme ni ne modifie `id-*`/`platform-*`._

## Impact

- **Produits** : Lingua (nouveau) ; **Cymbra ID / Music / Live / back-office / site : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés).
- **Arborescence** : `crates/lingua-core` uniquement (module `knowledge/` prévu par `add-lingua-analysis`) ; aucune nouvelle unité — la lane CI Rust existante couvre déjà le crate.
- **Dépendances** : aucune nouvelle (parsing CSV minimal ; `serde` déjà présent).
- **Hors périmètre** : cartes/FSRS et l'inférence « connu » depuis le SRS (`add-lingua-decks-review`), pack réel de fréquences/gloses (`add-lingua-data-pack`), les surfaces qui affichent ces données (extension, app Apple, plugin — changes ultérieurs), la réconciliation multi-stores (`add-lingua-backend`).
