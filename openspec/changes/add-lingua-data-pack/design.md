# Design — add-lingua-data-pack

## Context

Quatrième étage de la pile Lingua : le cœur (`add-lingua-analysis`) expose la cascade de lemmatisation, les rangs et l'`analyzer_version` ; ses tests tournent sur des mini-fixtures. Ce change apporte les vraies données (EN→FR) et le format qui les transporte. Décisions héritées et non rediscutées : cascade AGID + morphy + repli pluriel (`add-lingua-analysis`), profil L1/L2 `native_language` qui choisit la paire (`add-lingua-knowledge-model`).

## Decisions

### D1 — Format conteneur versionné par paire (L2→L1)
`pack.lingua` = magic + TOC : `meta` (paire, versions, licences), `forms.fst`, `lemmas.bin`, `freq.bin` (rangs quantisés wordfreq), `gloss.zst` (gloses FR extraites de kaikki, offset-indexées, top ~30 k lemmes), `NOTICE`. Le lecteur (dans `lingua-core`, lecture depuis un slice `include_bytes!`-compatible) refuse un pack dont l'`analyzer_version` déclarée est incompatible : l'analyse est déterministe à (version, pack) donnés — contrat posé par `add-lingua-analysis` — et un pack d'une autre génération ne doit jamais produire une analyse partielle. MVP : un seul pack (EN→FR), mais **tout le code est pair-keyed** — ajouter (ES→FR) = données, pas du code.

### D2 — Pipeline hors-ligne reproductible, pack jamais commité
Construit hors-ligne par `scripts/lingua-data` (reproductible, sources datées, données brutes non commitées) ; le pack lui-même n'est **pas commité** : il est reconstruit en CI (déterminisme testé) et mis en cache, le dev local le construit une fois via le script.

### D3 — Hygiène de licences : vendable ou rien
Sources retenues : AGID (licence permissive, pile de notices shippée — pile amont WordNet incluse), fréquences wordfreq (CC BY-SA), gloses kaikki (CC BY-SA). Rien de GPL/AGPL/NC n'entre dans le build (liste noire documentée dans `scripts/lingua-data`). `NOTICE` embarqué dans le pack + les tables dérivées sont publiées (share-alike satisfait) ; la page attributions côté extension arrive avec `add-lingua-extension-reading`.

### D4 — Budget : 5 Mo, arbitré sur les gloses
Le pack embarqué doit rester sous 5 Mo : c'est la contrainte aval des surfaces (code wasm ~1 Mo + pack ≤ 5 Mo instanciés par onglet dans l'extension — `add-lingua-wasm` / `add-lingua-extension-reading`). Dépassement = échec de build ; la variable d'ajustement est la couverture des gloses (top 20 k vs 30 k lemmes), jamais le FST ni les fréquences : un lemme sans glose reste compté juste, un lemme absent du FST fausse le comptage.

## Risks / Trade-offs

- [Licences données (CC BY-SA wordfreq/kaikki, notices AGID)] → `NOTICE` embarqué dans le pack + tables dérivées publiées (share-alike satisfait) + liste noire documentée.
- [Sources vivantes (kaikki est réextrait en continu)] → sources datées et archivées localement ; le test de reproductibilité compare deux builds sur les **mêmes fichiers sources**, pas sur un re-téléchargement.
- [Qualité lemmatiseur v1 (AGID+morphy sans POS)] → suffisant pour le comptage (ambiguïté résolue pro-apprenant) ; les erreurs résiduelles sont le différenciateur du pack v2, pas un bloquant.

## Open Questions

- Le seuil exact de gloses embarquées (top 20 k vs 30 k lemmes) — mesurer la taille réelle du `gloss.zst` et arbitrer sous 5 Mo de pack total.
