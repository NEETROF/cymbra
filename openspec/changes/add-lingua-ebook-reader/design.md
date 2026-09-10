# Design — add-lingua-ebook-reader

## Context

`add-lingua-mvp` livre le moteur (analyse WASM déterministe, statuts lemma-first, decks FSRS, surlignage Highlight API) et son extension. Le 3ᵉ workflow d'exploration (2026-08) a validé la feature « % connu par livre + gap decks » sur pièces : science des seuils (Hu & Nation 98 % confortable, Laufer 95 % assisté, Schmitt 2011 relation linéaire sans falaise), tri du gap deck par fréquence-dans-le-livre, légalité de l'analyse 100 % locale, et le pilier communautaire (profils partagés, catalogue, HathiTrust) — exploré mais **différé** : ce change n'en livre que le schéma local, prêt. Décision utilisateur : l'expérience arrive **dans l'extension d'abord**, l'app Apple suivra dans un change ultérieur.

Contraintes héritées : local-only, jamais « lemme » à l'écran, coverage ≥ 80 %, déterminisme à `analyzer_version` donnée, tokens.css Cymbra, cœurs host-testables.

## Goals / Non-Goals

**Goals :**
- Importer un epub (ou txt) local et voir, hors ligne, son % de couverture en bandes honnêtes (tokens, lemma-first), globalement et par chapitre, avec sa provenance.
- Générer un gap deck utile : fréquence-dans-le-livre, plafonné, hapax filtrés, phrases du livre — qui rejoint la révision FSRS existante telle quelle.
- Lire le livre dans l'extension avec exactement la même instrumentation que le web (surlignage, popup, capture).

**Non-Goals :**
- Catalogue serveur, partage de profils, `TextProfileService`, profils HathiTrust (changes ultérieurs — le schéma TextProfile les prépare).
- Reader dans l'app conteneur Apple (change ultérieur ; les types `lingua-core` rendent le portage mécanique).
- PDF, contournement de DRM (jamais), images de l'epub (v1 texte), TTS/audio, annotations libres.

## Decisions

### D1 — Le reader n'est pas un silo : raffinement du principe n° 1
Le principe « le lieu de lecture n'est jamais l'app » vise la **copie du web** : recopier une page dans un reader alors qu'elle se lit instrumentée sur place. Un epub importé n'a aucun autre lieu de lecture instrumentable — aucun navigateur ne l'ouvre, les readers du marché sont fermés. Le reader n'est donc pas un silo : c'est **le seul foyer possible** pour ce contenu (précédent assumé : le Read Hub de Migaku — un onglet parmi d'autres, pas un produit-navigateur). Formulation retenue, à opposer à toute dérive future : « **on instrumente le web sur place, et on héberge ce qui n'a pas d'ailleurs** ». Corollaire concret : le reader n'accepte jamais d'URL web — une page web se lit dans le navigateur, surlignée par le content script.

### D2 — Import local uniquement : epub + txt, DRM refusé, pas de PDF
Fichier `.epub`/`.txt` par drag&drop ou sélecteur sur la page bibliothèque ; le fichier ne quitte **jamais** l'appareil (aucune requête réseau à l'import, à l'analyse, à la lecture). Un epub chiffré (DRM détecté via `encryption.xml` ciblant le contenu) est **refusé avec un message clair et sans jargon** — jamais de contournement, ni maintenant ni plus tard. Pas de PDF : l'extraction de texte y est trop peu fiable pour tenir la promesse d'un % honnête. `.txt` = un chapitre unique. Note marché FR : beaucoup d'ebooks français sont vendus sans DRM dure (tatouage — Vivlio, éditeurs de l'imaginaire), donc analysables légalement ; le mur reste Kindle/Adobe, et le message de refus l'explique.

### D3 — Extraction epub en TS dans l'extension ; tout le déterminisme dans `lingua-core` (WASM)
Le TS ouvre le zip (`fflate`), lit `container.xml` → OPF (spine = ordre des chapitres, métadonnées titre/auteur) et parse le XHTML via `DOMParser` (natif, gratuit, tolérant aux epubs mal formés) pour produire une liste ordonnée de chapitres { titre, blocs de texte }. Tout ce qui engage le **contrat de déterminisme** reste dans `lingua-core` : normalisation (NFC, réduction des espaces), empreinte d'édition (hash du texte normalisé), tokenisation, lemmatisation, comptes. Alternative rejetée : parsing epub en Rust (crates rbook/epubparse) — gonflerait le module WASM (budget D5 du MVP), dupliquerait un parseur XML que le navigateur offre nativement, et n'apporterait rien au déterminisme puisque l'extraction de texte n'en fait pas partie : c'est la **normalisation** qui y entre, et elle est dans le cœur. La frontière est tracée pour que l'empreinte soit stable : la normalisation agressive absorbe les écarts d'extraction mineurs entre navigateurs, et le test de parité rejoue la même liste de blocs en natif et en WASM.

### D4 — Le TextProfile vit dans `lingua-analysis` (délta), pas dans le reader
Le TextProfile est un **artefact d'analyse pur** : déterministe à `analyzer_version` donnée, counts-only, indépendant de toute UI. Le futur change communautaire (`TextProfileService`, catalogue, dedup par empreinte d'édition) consommera exactement ce schéma — il ne doit pas dépendre d'une capability d'expérience. Donc : le schéma et la couverture sont des requirements ajoutées à `lingua-analysis` (délta `## ADDED Requirements` sur une capability introduite par `add-lingua-mvp` — permis : aucun requirement existant n'est touché), et `lingua-ebook-reader` décrit l'expérience (import, bibliothèque, fiche, gap deck, reader) qui produit et consomme le profil. Alternative rejetée : tout dans `lingua-ebook-reader` — condamnerait le change catalogue à dépendre du reader ou à dupliquer le schéma.

### D5 — Stockage : IndexedDB (origine extension) pour les blobs, `chrome.storage.local` inchangé pour l'état
L'epub (Blob) et le TextProfile vont dans l'**IndexedDB de l'origine extension** — pas dans `chrome.storage.local` (sérialisation JSON, quotas serrés, pas fait pour les blobs). Table `books` { id, métadonnées OPF, empreinte, profil, position de lecture, date d'import }, schéma versionné avec migration ascendante (même discipline que D9 du MVP). Les statuts/cartes restent dans `chrome.storage.local` : le croisement profil × connaissance se fait **en local, en mémoire**, à l'ouverture et sur `storage.onChanged` — ni le livre ni le lexique ne transitent nulle part. `navigator.storage.persist()` est demandé au premier import ; en cas d'éviction, l'epub est ré-importable et le profil se recalcule — rien d'irremplaçable. Supprimer un livre purge blob + profil + position.

### D6 — % en bandes, comptés par tokens, provenance affichée
La couverture est calculée par **tokens** (occurrences), jamais par mots uniques : c'est ce que mesure la recherche (les seuils de Hu & Nation et Laufer sont des couvertures de tokens), et c'est le correctif direct de la plainte LingQ — compter les types de surface gonfle les compteurs et promet une lisibilité fausse (un mot inconnu répété 50 fois pèse 50 occurrences de friction, pas une). Affichage en **bandes** (« ~87 % »), **jamais de décimales** ni de verdict « prêt/pas prêt » : Schmitt 2011 montre une relation compréhension/couverture linéaire, sans falaise — une décimale serait une précision mensongère. Les seuils 95 % (lecture assistée, Laufer) et 98 % (confortable, Hu & Nation) apparaissent comme repères : « ≈N mots pour 95 %, M pour 98 % ». Chaque % **montre sa provenance** : « ton exemplaire » en v1 — champ enum extensible, prêt pour les futurs niveaux de confiance (profil partagé, HathiTrust, extrait) sans les livrer.

### D7 — Gap deck : fréquence-dans-le-livre, plafonné, hapax filtrés, phrases de l'exemplaire
Tri = lemmes inconnus par compte décroissant **dans le livre** — exactement optimal pour maximiser la couverture de *ce* livre (chaque mot appris ajoute ses occurrences indépendamment : pas un problème de set cover). Hapax (compte = 1) exclus par défaut : gain de couverture négligeable et concentration du bruit (noms propres ratés, archaïsmes). Génération **plafonnée** (défaut de l'ordre de 200 cartes, re-générable après progression). Les phrases de contexte sont extraites de l'**exemplaire local** au moment de la génération — jamais stockées dans le profil, qui reste counts-only. Les cartes portent la provenance livre (titre, chapitre) dans le champ source existant et rejoignent les decks/révision FSRS du MVP **sans schéma parallèle**.

### D8 — Reader : notre DOM, même pipeline que le web
Le reader affiche **un chapitre à la fois** : XHTML assaini (D9), puis le **même moteur** que les pages web — surlignage Highlight API, popup de mot, capture de sélection au raccourci. Zéro logique dupliquée : pour le pipeline de contenu, le reader est « une page de plus », à ceci près que le DOM nous appartient (pas de MutationObserver de guerre, pas d'hydration adverse). Position de lecture (chapitre + ancrage de défilement) persistée par livre. Réglages de confort **minimaux** : taille du texte, thème clair/sombre — en tokens.css Cymbra (D11 du MVP) ; pas de justification, d'hyphénation ni de polices en v1 : la valeur est le %, le gap deck et le surlignage, pas la parure du reader.

### D9 — Contenu epub = contenu tiers non fiable : assainissement structurel
Un epub est une archive tierce rendue dans l'origine de l'extension — surface XSS directe. Le rendu passe par un assainisseur à **liste blanche** (balises et attributs textuels seulement) : scripts, styles embarqués, gestionnaires d'événements inline et toute référence à une ressource externe sont retirés ; la page reader porte une CSP stricte. Conséquence assumée : les images de l'epub ne sont pas affichées en v1 (roman = texte ; les fixtures Standard Ebooks le confirment). Aucune ressource de l'epub ne peut déclencher de requête réseau — la promesse « rien ne quitte l'appareil » tient par construction, pas par revue.

### D10 — Côté Apple : plus tard, avec l'app conteneur
Décision utilisateur : extension d'abord. Sur iOS/macOS, le foyer naturel du reader est l'app conteneur (analyse nativeMessaging, packs dans le bundle, quotas de storage d'extension iOS) — ce sera un change ultérieur ; les types TextProfile/position de `lingua-core` rendent le portage mécanique. Ce change livre les variantes **chromium + firefox** de l'extension ; la variante safari n'expose pas les pages bibliothèque/reader.

## Risks / Trade-offs

- [XSS via epub hostile rendu dans l'origine extension] → assainisseur liste blanche (D9) + CSP stricte de la page reader ; fixtures d'epubs malveillants (script, handler inline, ressource externe) dans les tests unitaires.
- [Epubs mal formés (zip corrompu, OPF exotique, XHTML cassé)] → `DOMParser` tolérant, import qui échoue **par livre** avec un message clair, jamais de crash de la bibliothèque ; fixtures dédiées.
- [Gros livres : coût du profil à l'import, chapitre long à surligner] → profil calculé une fois à l'import (dans la page bibliothèque, hors content script), chapitre = unité de rendu, mémoïsation par forme existante ; cible < 10 s d'import pour un roman ; mesurer avant d'optimiser.
- [Déterminisme de l'empreinte cross-navigateur (extraction TS)] → la normalisation agressive du cœur (NFC, espaces) absorbe les écarts d'extraction ; parité natif/WASM testée sur blocs identiques ; si un écart réel apparaît, l'empreinte est liée à l'`analyzer_version` — une correction = nouvelle version, pas une ambiguïté silencieuse.
- [Éviction IndexedDB] → `navigator.storage.persist()` demandé, et rien d'irremplaçable : l'epub se ré-importe, le profil se recalcule, les cartes créées vivent dans le store du MVP.
- [Le « problème Harry Potter » (bibliothèque limitée aux epubs DRM-free)] → assumé et annoncé dans l'UI de refus DRM ; les réponses (profils partagés, HathiTrust) sont les changes communautaires ultérieurs, préparés par le schéma.
- [Dérive de périmètre du reader (polices, annotations, TTS)] → v1 volontairement spartiate (D8) ; toute parure attend un signal d'usage.

## Migration Plan

Rien à migrer : nouvelles pages, nouvelles tables IndexedDB — l'état du MVP (statuts, cartes, calibration) est intact et le schéma bibliothèque naît versionné avec migration ascendante. Rollback = retirer les pages ; les cartes déjà générées par des gap decks restent des cartes ordinaires du store MVP, révisables et exportables. La suppression d'un livre purge blob + profil + position.

## Open Questions

- Plafond exact du gap deck (défaut 200) et politique de re-génération (remplacer le deck vs le compléter) — à trancher à l'usage fondateur.
- Extraction de la couverture (image) pour la carte de bibliothèque : seulement si triviale via l'OPF **et** compatible avec D9 (image inline assainie) ; sinon monogramme sur tokens Cymbra.
- Découpe heuristique d'un `.txt` en chapitres (lignes « Chapter N ») — v1 : non, chapitre unique ; à revoir si l'usage txt existe.
