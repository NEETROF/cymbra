# Tasks — add-lingua-knowledge-model

## 1. lingua-core — knowledge model (spec lingua-knowledge-model)

- [ ] 1.1 Types statuts (`learning/known/ignored` + provenance `manual/calibration/srs/import`) et résolution connu-implicite par rang ≤ calibration ; le statut explicite prime
- [ ] 1.2 Résolution multi-candidats (connu si un candidat l'est) avec tests
- [ ] 1.3 Profil L1/L2 : `native_language` + langues étudiées ; toutes les API clées par paire ; tests avec paire factice
- [ ] 1.4 Import LingQ (CSV) → statuts `known` provenance `import`, entrées lemmatisées ; test sur échantillon réel anonymisé
- [ ] 1.5 Compteurs d'exposition par (langue, lemme) : incréments à l'ingestion, source + horodatage, sans effet sur les statuts ; tests
