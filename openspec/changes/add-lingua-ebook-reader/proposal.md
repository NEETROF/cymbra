# add-lingua-ebook-reader — Cymbra Lingua : lecture d'ebooks, % par chapitre, gap decks

## Why

L'extension du MVP instrumente le web sur place — mais une part majeure de la lecture d'un apprenant est faite de **livres**, et un epub acheté ou téléchargé n'a aucun lieu de lecture instrumentable : aucun navigateur ne l'ouvre nativement, et les readers du marché (Kindle, Books, Kobo) sont fermés à toute instrumentation. Le principe produit n° 1 (« jamais de silo ») vise la copie du web ; pour un epub il n'y a rien à copier depuis nulle part — héberger la lecture est la seule option (précédent : le Read Hub de Migaku). Et la donnée que le livre débloque est un différenciateur confirmé par l'étude concurrentielle : **% de couverture personnalisé par chapitre + deck d'écart trié par fréquence-dans-le-livre**, que personne ne fait pour EN/FR/ES (jpdb = japonais seulement ; LingQ compte les types de surface non lemmatisés — plainte documentée de ses propres forums).

Ce change dépend d'`add-lingua-mvp` **uniquement** et reste entièrement local : l'epub ne quitte jamais l'appareil, aucune dépendance serveur — il peut avancer en parallèle du change de sync. Décision produit actée : l'expérience arrive **dans l'extension d'abord** ; l'app conteneur Apple la récupérera dans un change ultérieur.

## What Changes

- **`lingua-core` : profil de texte local (TextProfile)** — { empreinte d'édition (hash du texte normalisé), langue, `analyzer_version` épinglée, entrées (lemme, compte, premier chapitre d'apparition), chapitres } — **counts-only par construction** (aucune phrase, aucun n-gramme), déterministe, et calcul de couverture **par tokens** (global, par chapitre, « N mots pour atteindre S % », rampe par chapitre). Le schéma est prêt pour le partage communautaire futur sans le livrer.
- **Extraction epub côté extension (TS)** : zip + OPF (spine → chapitres, métadonnées) + XHTML via `DOMParser` ; `.txt` accepté comme chapitre unique ; **pas de PDF** ; epub verrouillé par DRM **refusé** avec un message clair — jamais de contournement.
- **Nouvelles pages d'extension** (variantes chromium + firefox, tokens.css Cymbra) : **bibliothèque** (import drag&drop/sélecteur, bandes de couverture « ~87 % » avec provenance « ton exemplaire », jamais de décimales), **fiche livre** (bande globale, courbe par chapitre, repères « ≈N mots pour 95 %, M pour 98 % », rampe « X mots débloquent les chapitres 1-3 »), **gap deck** (mots inconnus triés par fréquence-dans-le-livre, plafonné, hapax filtrés, phrases de contexte extraites de l'exemplaire local), et le **reader** : lecture chapitre par chapitre instrumentée par **le même pipeline de surlignage que les pages web** (Highlight API + popup de mot + capture de sélection — c'est notre DOM cette fois), position de lecture persistée, réglages de confort minimaux.
- **Stockage local** : blobs epub + profils en **IndexedDB de l'origine extension** (pas `chrome.storage.local` pour les blobs), schéma versionné, suppression purgante.
- Vocabulaire UI inchangé : le mot « lemme » n'apparaît **jamais** à l'écran (« forme du dictionnaire », « mots différents »).

## Capabilities

### New Capabilities
- `lingua-ebook-reader` : l'expérience ebook — import local (epub/txt, DRM refusé, pas de PDF), bibliothèque à bandes de couverture avec provenance, fiche livre (courbe par chapitre, seuils 95/98, rampe), gap deck, reader surligné par le pipeline existant, rendu assaini du contenu epub, position de lecture et réglages persistés, stockage IndexedDB versionné.

### Modified Capabilities
- `lingua-analysis` (introduite par `add-lingua-mvp`) : étendue par un délta `## ADDED Requirements` uniquement — profil de texte local (TextProfile) déterministe et counts-only, calcul de couverture par tokens avec chemins vers les seuils. Aucune requirement existante n'est modifiée.

## Impact

- **Produits** : Lingua uniquement (extension) ; **Cymbra ID / Music / Live / back-office / site : intacts** (aucun proto, aucun crate backend, aucune app existante modifiés). Toujours aucun réseau : rien n'est consommé du socle au-delà des conventions CI/coverage déjà en place.
- **Arborescence** : `crates/lingua-core` (module `textprofile/` + bindings wasm), `apps/lingua-extension` (pages bibliothèque/fiche/reader, extraction epub, gap deck, fixtures epub synthétiques de test). Aucune nouvelle unité `apps/*` ni `crates/*` : les lanes CI du MVP (llvm-cov Rust, build wasm + vitest extension) couvrent tout, `ci-units` est déjà satisfait.
- **Dépendances nouvelles** : une lib zip TS minuscule (`fflate`) ; côté Rust, un hash pour l'empreinte d'édition (`sha2`). Rien d'autre.
- **Hors périmètre (changes ultérieurs déjà explorés)** : reader/bibliothèque dans l'**app conteneur Apple** (décision : extension d'abord ; les types `lingua-core` rendront le portage mécanique), catalogue de livres précalculé, partage communautaire de TextProfile + `TextProfileService`, profils HathiTrust, niveaux de confiance multi-provenances (le champ provenance existe dès v1, seul « ton exemplaire » est livré), OCR/images, TTS.
