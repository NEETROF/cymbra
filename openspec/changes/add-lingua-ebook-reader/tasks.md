# Tasks — add-lingua-ebook-reader

## 1. lingua-core — TextProfile et couverture (délta lingua-analysis)

- [ ] 1.1 Types `TextProfile` { empreinte d'édition, langue, `analyzer_version`, entrées (lemme, compte, premier chapitre), chapitres (titre, tokens comptés) } sérialisables versionnés ; **counts-only par construction** (aucun champ de séquence dans le schéma) ; module `textprofile/`
- [ ] 1.2 Construction depuis une liste ordonnée de chapitres → blocs de texte : normalisation (NFC, réduction des espaces), empreinte (hash du texte normalisé, `sha2`), tokenisation/lemmatisation par la cascade existante, exclusion des noms propres hors lexique ; tests
- [ ] 1.3 Déterminisme : même liste de blocs ⇒ profil identique octet pour octet, empreinte comprise, en natif et en WASM (extension du test de parité 6.2 du MVP)
- [ ] 1.4 Couverture par tokens depuis (profil × état de connaissance) : % global, % par chapitre, « N lemmes pour atteindre S % » (tri par compte décroissant dans le livre), rampe par préfixe de chapitres ; tests aux seuils 95/98
- [ ] 1.5 Génération de gap deck côté cœur : lemmes inconnus triés par compte décroissant, hapax exclus, plafond paramétré ; tests (tri, cap, exclusions)
- [ ] 1.6 Bindings wasm-bindgen (profil depuis blocs, couverture, gap deck) dans la cible WASM existante

## 2. Extension — extraction epub (TS)

- [ ] 2.1 Dépendance `fflate` ; ouverture du zip, localisation de l'OPF via `container.xml`, spine → liste ordonnée de chapitres, métadonnées titre/auteur
- [ ] 2.2 Détection DRM (`encryption.xml` ciblant le contenu) → refus d'import avec message clair sans jargon ; **aucun code de contournement, jamais**
- [ ] 2.3 XHTML → blocs de texte via `DOMParser` (traversée définie, exclusions script/style) ; `.txt` = un chapitre unique
- [ ] 2.4 Fixtures epub **synthétiques libres de droits** construites dans le repo (bien formé multi-chapitres, mal formé, chiffré/DRM, hostile pour 6.5) + tests vitest de l'extraction

## 3. Extension — import et stockage local

- [ ] 3.1 Schéma IndexedDB versionné (`books` : blob, métadonnées, empreinte, profil, position, date d'import) + migration ascendante + suppression purgante (blob + profil + position)
- [ ] 3.2 Page bibliothèque : drag&drop + sélecteur (`.epub`/`.txt` uniquement, PDF refusé), import → extraction → profil (WASM, dans la page — hors content script) → persistance ; erreurs par livre sans crash de la bibliothèque ; `navigator.storage.persist()` au premier import
- [ ] 3.3 Croisement profil × statuts en mémoire, recalcul des bandes sur `storage.onChanged` ; livre sans contenu suffisant en langue étudiée signalé « non analysable », sans % inventé

## 4. Extension — bibliothèque et fiche livre

- [ ] 4.1 Bibliothèque : cartes de livres (titre, auteur, bande « ~87 % », provenance « ton exemplaire », position de lecture) en tokens.css Cymbra
- [ ] 4.2 Fiche livre : bande globale + couverture par chapitre (courbe/barres), repères « ≈N mots pour 95 %, M pour 98 % », rampe « X mots débloquent les chapitres 1-3 »
- [ ] 4.3 Formatage des % : fonction unique testée (bandes, jamais de décimales, jamais « prêt/pas prêt ») ; lint des chaînes UI étendu aux nouvelles pages (jamais « lemme » — « mots différents », « forme du dictionnaire »)

## 5. Extension — gap deck

- [ ] 5.1 Génération depuis la fiche : appel cœur (tri fréquence-dans-le-livre, cap, hapax exclus) + extraction des phrases de contexte depuis l'exemplaire local au moment de la génération, cartes avec provenance (titre, chapitre) dans le champ source existant
- [ ] 5.2 Intégration aux decks/révision du MVP sans schéma parallèle : cartes révisables en side panel, comptées dans les dues, incluses dans l'export Anki
- [ ] 5.3 Tests : phrase extraite du bon chapitre, plafond respecté, hapax absents, tri stable

## 6. Extension — reader

- [ ] 6.1 Page reader : rendu d'un chapitre **assaini** (liste blanche de balises/attributs, aucun handler inline, aucune référence externe, CSP stricte) ; navigation chapitre précédent/suivant + sommaire depuis le spine
- [ ] 6.2 Branchement du pipeline existant sur le DOM du reader : surlignage Highlight API, popup de mot, capture de sélection au raccourci — zéro logique d'analyse dupliquée ; actions de statut → re-peinture + propagation aux autres surfaces
- [ ] 6.3 Position de lecture persistée par livre (chapitre + ancrage) et restaurée à l'ouverture
- [ ] 6.4 Réglages de confort : taille du texte, thème clair/sombre (tokens Cymbra) ; persistés
- [ ] 6.5 Tests : assainisseur sur les fixtures hostiles (script, `onclick`, image externe — aucune exécution, aucune requête réseau), position restaurée

## 7. Gates et parcours manuel

- [ ] 7.1 `cargo fmt --all --check` + `clippy -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` (nouveaux modules cœur couverts) ; vitest vert sur `apps/lingua-extension`
- [ ] 7.2 Parcours manuel avec un **vrai livre du domaine public (Standard Ebooks)** : import → fiche (bandes, seuils, rampe) → gap deck → lecture surlignée → popup → capture d'expression → révision — sur Chrome et Firefox ; documenté
- [ ] 7.3 `openspec validate add-lingua-ebook-reader --strict` final + mise à jour des specs si l'implémentation a fait bouger un contrat
