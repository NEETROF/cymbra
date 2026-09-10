# Design — add-lingua-knowledge-model

## Context

Deuxième brique de la pile Lingua, directement sur `add-lingua-analysis` (dont elle hérite le crate `lingua-core`, la cascade de lemmatisation, les rangs de fréquence et le contrat `analyzer_version`). Le knowledge model est le vocabulaire de types que toutes les surfaces ultérieures partagent — c'est ce partage qui rendra mécanique la fusion des stores locaux au change de sync (`add-lingua-backend`).

## Goals / Non-Goals

**Goals :**
- L'état de connaissance par `(langue étudiée, lemme)` : statuts, provenance, connu-implicite par calibration, résolution multi-candidats.
- Le profil L1/L2 pair-keyed dès le jour 1 (zéro retrofit à l'ajout d'une paire).
- Les deux amorces du démarrage à froid : calibration par rang, import LingQ.
- Les compteurs d'exposition, socle de l'inférence future.

**Non-Goals :**
- Cartes, FSRS et l'inférence « connu » depuis la révision (`add-lingua-decks-review` — le champ de provenance `srs` est prévu ici, câblé là-bas).
- Données réelles de fréquence/gloses (`add-lingua-data-pack`) ; toute UI ; toute persistance de surface (`chrome.storage`, SQLite — chaque surface stocke, le modèle définit les types).

## Decisions

### D1 — Clé `(langue étudiée, lemme)`, sans POS, statuts à 4 états
- Clé = lemme texte normalisé par langue étudiée ; les expressions multi-mots sont des lemmes avec espaces. Le **POS ne fait pas partie de la clé** en v1 : l'ambiguïté (`can` nom/verbe) se résout en faveur de l'apprenant — connu si un candidat l'est — et le popup montrera tous les sens. Alternative rejetée : clé (lemme, POS) — double la taille des tables, exige un tagger (qualité/poids), et le gain pédagogique est marginal au niveau visé.
- Statuts : `learning`, `known`, `ignored` explicites ; « nouveau » est l'absence d'entrée ; `known` est aussi **implicite** sous le seuil de calibration (rang de fréquence ≤ N). `ignored` compte comme connu dans le %. Le statut explicite prime toujours sur la calibration.
- Le champ `known_source` (`manual | calibration | srs | import`) prépare l'inférence SRS façon Migaku sans la câbler (la provenance `srs` sera écrite par `add-lingua-decks-review`).

### D2 — Calibration par rang de fréquence : la réponse au démarrage à froid
Sans amorce, le jour 1 surligne ~60 % de la page (le décrochage documenté de la catégorie). Un seul curseur — « je connais les N mots les plus courants » — rend la page lisible immédiatement, sans marquer un seul mot. La calibration est un *seuil*, pas une écriture en masse : aucun statut explicite n'est créé (l'implicite se recalcule à chaque analyse), donc changer le curseur est gratuit et réversible. Les rangs viennent de la table de fréquence du pack (mini-fixtures ici, wordfreq réel avec `add-lingua-data-pack`).

### D3 — Profil L1/L2 : `native_language` distincte, tout est pair-keyed
Le profil porte `native_language` (langue de confort : gloses, futures traductions ; défaut : locale système, plus tard locale Cymbra ID — la locale UI n'est pas la langue des gloses) et les langues étudiées. **Tout le downstream est clé par paire (L2→L1)** dès le jour 1 : gloses, packs, état de connaissance, future direction de MT. MVP = (en→fr) uniquement, mais ajouter (es→fr) = données, pas du code. La règle « ne jamais analyser la L1 » (gate de langue) est déjà portée par la détection par bloc de `add-lingua-analysis` ; le profil lui fournit la L1.

### D4 — Import LingQ : lemmatiser à l'import, provenance `import`
L'export LingQ (CSV) contient des formes de surface (c'est précisément le défaut du produit d'origine) : chaque entrée passe par la cascade de lemmatisation avant marquage `known` provenance `import`. Double rôle : démarrage à froid du segment cible (utilisateurs LingQ existants — dont le fondateur) et arme d'acquisition (« migre depuis LingQ, garde ton historique »). Testé sur un échantillon réel anonymisé.

### D5 — Compteurs d'exposition : enregistrer sans interpréter
Par (langue étudiée, lemme) : compteur d'occurrences rencontrées, source de la dernière rencontre, horodatage. En v1 l'exposition **ne modifie jamais un statut** — piège n°1 de la catégorie : l'auto-known-on-page-turn de LingQ, explicitement rejeté. C'est une donnée d'entrée pour l'inférence future (« connu » déduit du SRS/de l'exposition), alimentée par l'ingestion agent (`add-lingua-agent`) et la lecture (`add-lingua-extension-reading`).

### D6 — Vocabulaire d'interface : jamais « lemme » à l'écran
L'utilisateur cible ne connaît pas le mot « lemme » (leçon utilisateur directe, re-apprise pendant le preshot). L'invariant est déclaré dans cette capability — c'est le modèle qui nomme les concepts — et appliqué/linté par chaque surface de la pile (`add-lingua-extension-reading`, `add-lingua-agent`) : « forme du dictionnaire » pour la forme canonique, « mots différents » pour les comptes de lemmes uniques.

## Risks / Trade-offs

- [Sans-POS : faux « connus » sur homographes (`can` nom vs verbe)] → assumé pro-apprenant ; le coût d'un tagger (poids, qualité, clé doublée) dépasse le gain au niveau visé. Réévaluable au pack v2 sans casser la clé (le POS resterait un attribut d'affichage).
- [Statuts implicites = résultat dépendant de la table de fréquence] → la table est versionnée avec le pack ; le statut *explicite* prime toujours, donc une mise à jour de pack ne réécrit jamais une décision de l'utilisateur.
- [Import LingQ : qualité inégale des exports] → lemmatisation systématique + provenance `import` distincte : un import douteux reste identifiable et corrigeable en masse plus tard.
- [Deux stores locaux (extension / plugin) non réconciliés en v1] → assumé (décision héritée de la pile) : le change de sync les fusionnera côté serveur ; ce change fournit précisément le vocabulaire de types partagé qui rendra la fusion mécanique.

## Migration Plan

Rien à migrer (module nouveau, aucune persistance de surface encore). Les types sont sérialisables versionnés dès le jour 1 — c'est le contrat que les stores des surfaces (storage extension, SQLite plugin) et la sync consommeront.

## Open Questions

- Défaut du curseur de calibration (0 ? 1 000 ?) à la première ouverture — à trancher avec l'UI de calibration (`add-lingua-extension-reading`), sans impact sur le modèle.
- Colonnes exactes reconnues de l'export LingQ (variantes CSV/Anki constatées) — figer à l'implémentation sur échantillons réels.
