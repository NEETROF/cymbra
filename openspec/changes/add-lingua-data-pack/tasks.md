# Tasks — add-lingua-data-pack

## 1. Packs de données (spec lingua-data-packs)

- [ ] 1.1 `scripts/lingua-data/` : pipeline reproductible (sources datées, données brutes non commitées) ; téléchargement AGID + export wordfreq + extrait kaikki fr-glosses
- [ ] 1.2 Construction du FST (AGID inversé) + table de fréquence (rangs quantisés) + `gloss.zst` offset-indexé (top lemmes, budget)
- [ ] 1.3 Format conteneur `pack.lingua` (magic, TOC, meta avec `pack_version`/`analyzer_version`/licences, NOTICE) + lecteur dans `lingua-core` avec refus des versions incompatibles
- [ ] 1.4 Garde-fous licences : liste noire GPL/AGPL/NC documentée + vérification du NOTICE au build ; échec de build si pack > 5 Mo
- [ ] 1.5 Test de reproductibilité (double build identique) ; le pack (en→fr) est construit en CI et mis en cache (jamais commité), build local documenté
