# Design — read-lingua-french-web-unmarked-spellings

## Context

See proposal.md (Why). What exists, on `main` at `8ef47870` (change 43b's implementation,
`read-lingua-french-ca-as-ca-cedilla`, #884, merged), which this change builds on:

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer (change 43). `UNMARKED_SPELLINGS` (43b's D1), form → (word, reason), four rows — `ca` → *ça*, `age` and `ages` → *âge*, `forcement` → *forcément* —, read as `ELISIONS` is: `Lexicon.read` makes the named word a row's one candidate and returns before the entry is read, `_inflections` skips the form, an entry's form-of link to a row's spelling is dropped, `unmarked_spellings` seeds every row whether or not an entry names it. Its test: every row has a reason, names a ranked lemma reading as itself, and is neither an elided piece nor `du`/`des`; `cote`, `tache`, `pale`, `foret` and `aine` are never rows |
| The ranks (change 43's D6) | wordfreq's words that are French tokens, the commonest 60,000 kept, each the lemma of its own form. A spelling the dictionary names nowhere is ranked as a word of its own: `meme` 1,018, `tres` 1,697, `francais` 3,738, `deja` 3,793 — unglossed, no French dictionary word, levelless (46's rule 1, « unknown ») |
| The readings (change 45) | `grammar.tsv`, keyed by form, from the inflection tables the section's entries list; a spelling no entry names has none |
| Change 41's cascade | looks every token up lowercased and in NFC; its D2 measured and rejected « a capital read without its accent (`Ecole` → `école`) » as a cascade rule. The names rule sets aside a capital in mid-sentence whose lemma is no dictionary word |
| Change 43b's D6 | measured 475 spellings at Zipf ≥ 3 that are another French word with its marks dropped and less than half as frequent; recorded two follow-ups the owner settled on 2026-10-10: the dictionary's own unmarked spellings (Q2, `read-lingua-french-dictionary-unmarked-spellings`, 43d) and a reviewed list by name of the web's (Q4, this change) |
| Change 43d (proposed in parallel) | reads, through change 43's spelling rule, the words the section gives only as an accented word written without its marks, when the accented word is the commoner; hands this list `tete`, `sure`, `sures`, `surs` and `entrainant` — the last four settled by the owner on 2026-10-10 as this list's to review (D2) |
| `tables/fr-en/`, `tables/fr-es/` | fr-en 30,067 glosses, coverage 93.6 / 86.9 / 76.3 %, the pack 2,537,588 B; fr-es 19,023 glosses, coverage 83.0 / 70.7 / 56.7 %, the pack 1,971,114 B (both as 43b re-pinned them) |

## Goals / Non-Goals

**Goals:**
- The web's French spellings without their marks, down to Zipf 2.5, read as their word, by rows a
  person reviews one by one (the owner's decision), never by a rule on marks alone.
- The candidate list built from data and shown whole with its frequencies, the rows and every group
  left out, for the owner to read.
- What the rows move measured — ranks, levels, dictionary words, readings, glosses, coverage, the
  treebanks, the corpora's false readings, the goldens and the pins — and the spellings forms of
  their word, not new lemmas.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es, and any shared or edition module.
- The spellings the dictionary itself gives (`etre`, `etait`, `etat`): change 43d's.
- A core rule: French's analyser version does not move, and change 41's D2 stands (no cascade rule
  reads a capital without its accent; a row reads only the listed spellings).
- Hyphenated words and spellings with a wrong accent (`trés`, `déja`): a reviewed list of their own,
  later (the owner, 2026-10-10; D7).

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of `8ef47870` with D2's rows, its
redirect and D3's readings behind switches, over the pinned release's `kaikki-French.jsonl` (sha256
`2d7bbe5f…`), GSD's training and development sections (`4b9a87b1…`, `9221e508…`) and wordfreq 3.1.1.
Switches off, it writes the committed `tables/fr/` and `tables/fr-en/` byte for byte (`forms.tsv`,
`freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`, `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`,
`manifest.json`); fr-es's reducer over its pinned sources (`2e724e39…`, `bf44ebb4…`, `bd14a5c4…`)
writes the committed `tables/fr-es/` byte for byte. The candidates read the French section's
headwords, the kind of each sense (a meaning, a form of, a spelling of) and the inflections each
entry lists, lowercased in NFC, and the tables' forms. The goldens were re-blessed in a scratch
checkout with D6's probes, on the committed tables and on the prototype's; every lingua-wasm and
lingua-pack test was run on the prototype's; the extension's word-card snapshots were re-recorded on
its goldens; the packs were measured on UD French-PUD and GSD's test section as `measure/fr-ud.sh`
does. The running-text counts read change 43c's corpus file: UD's French test sections and GSD's
training and development sections, Tatoeba's 727,867 French sentences, change 44c's French Wikipedia
sample and Gutenberg novels, and change 39's pages — 6.8 M words. Every figure below is for the
settled list of 468 rows.

## Decisions

### D1 — The candidate list, and the rows

Every form of `tables/fr/forms.tsv` written with an accent, a cedilla or a diaeresis gives a
spelling: the same word with its marks dropped (Unicode's combining marks; `œ` and `æ` are no marks,
change 43's D7 reads their ASCII spellings). A spelling is a candidate when wordfreq rates it at Zipf
≥ 2.5 — about three times per ten million words of its French, read from the web, subtitles and
social media (settled by the owner, Q1). Then, by what the French section and the tables make of it:

| Step | Zipf ≥ 3 | 2.5–3 | Examples |
|---|---|---|---|
| Spellings of an accented form, attested by wordfreq | 2,441 | 1,337 | |
| — change 43b's rows | 4 | 0 | `ca`, `age`, `ages`, `forcement` |
| — the section gives it a meaning of its own: a word, a form of a word, or an inflection a word's entry lists | 1,846 | 660 | `ou`, `a`, `la`, `sur`, `du`, `des`, `cote`, `tache`, `pale`, `foret`, `aine`, `mais`, `marche`; real verb forms (`publie`, `oblige`); `male` (under *mal*) |
| — the same, taken by the owner's exception (D2) | 1 | 2 | `sure`; `surs`, `entrainant` (and `sures`, Zipf 2.45) |
| — the section gives it only as a spelling, or as a form of a spelling: read by change 43d's rule | 8 | 11 | `etre`, `etait`, `etats`, `derniere`, `arreter`, `etaient`, `egyptiens`, `ecosse`; `scenario`, `repond`, `repondu`, `colere`, `matiere` |
| — the same, which 43d's rule leaves and the tests keep | 1 | 1 | `tete`, `tetes`: listed only as inflections of `teter` « alternative form of téter », which 43d's forms guard keeps out because `tete` is how the web writes *tête* |
| — the same, failing the tests | 18 | 8 | `to`, `diesel`, `el`, `oeuvre` |
| **Named nowhere by the section** | **563** | **655** | |
| — hyphenated: wordfreq cannot count them | 95 | 40 | `peut-etre`, `c'est-a-dire`, `lui-meme` (D7) |
| — the accented word's lemma is a name or no dictionary word | 160 | 137 | `jesus`, `israel`, `quebec`, `eric`, `in` (`în`) |
| — as common as the accented word or more | 48 | 36 | `the` (*thé*), `peter` (*péter*), `media`, `aprem` |
| — English writes it more often than French does: all out (Q2) | 103 | 135 | `video`, `hotel`, `these`, `role`, `region`, `cinema`, `cafe`, `mere`, `premiere`, `memes` |
| — two French words could be meant: read as the commoner (Q4) | 6 | 16 | `ecoute` (*écoute*, or *écouté* of *écouter*), `pate` (*pâte* or *pâté*), `peche` (*pêche* or *péché*) |
| — doubtful, left out (Q3) | 2 | 0 | `dela`, `nes` |
| — the rest | 149 | 291 | `meme`, `tres`, `apres`, `francais`, `deja`; `deteste`, `appele`, `leger`, `biere` |
| **Rows** | **156** | **308** | **464**, and the owner's 4 exceptions (D2): **468** |

The tests, each a number a person can check:
- *No meaning of its own, and not the dictionary's spelling*: the section gives the spelling
  (lowercased, in NFC) no sense that is neither a form nor a spelling of another word — as a headword,
  a form of such a word, or an inflection such a word's entry lists —, and does not give it only as a
  spelling of its accented word (43d's). `tete` and `tetes`, named only as forms of a spelling 43d's
  rule leaves, are the two spellings between the two changes that pass this change's tests.
- *A dictionary word*: the accented form's lemma in `forms.tsv` is in `lexical.tsv` (fr-en glosses
  it, and not as a name alone). Names are no words to learn, and a capital in mid-sentence whose lemma
  is no dictionary word is set aside already.
- *Less than half as frequent* as the accented form (43b's measure): a spelling as common as its
  accented word is another word — English's `the`, `real`, `peter`, `media`.
- *English*: wordfreq's English rates the spelling higher than its French does. The owner's examples
  (`video` 4.34 in French, 5.43 in English; `hotel` 3.93 / 4.95; `these` 3.25 / 6.04) all fall here,
  and the owner keeps every one of them out (Q2), `mere`, `premiere` and `memes` too.
- *Two words*: another accented form of the same spelling, of another lemma, at least a tenth as
  frequent; each row reads as the commoner (Q4), as the accented form itself reads under M8.
- *Doubtful*, by reading the corpora (D5) and the words: `dela` (*delà*, or « de la » written
  together, a common slip) and `nes` (*nés*, or the console « NES »), out (Q3). Rows that are also
  names or other languages' words are kept (Q3): `noel`, `peres`, `tres`, `tot`, `serie`, `tresor`.

Each row names the word its accented form reads as in `forms.tsv` today — `tres` → *très*, `etais` →
*être* (`étais`), `recu` → *recevoir* (`reçu`), `epargne` → *épargner* (`épargne`, M8) — so a row
follows change 43's choices and never makes one.

**The threshold, measured.** Over the 2,340 spellings that pass every test down to Zipf 1.5 (the
section named nowhere), the share of their frequency the rows above each threshold carry, and how
often a reader meets them (occurrences per 10,000 words of wordfreq's French):

| Zipf ≥ | Rows | Share | Per 10,000 words |
|---|---|---|---|
| 4.0 | 9 | 22.0 % | 1.92 |
| 3.5 | 35 | 37.3 % | 3.24 |
| 3.0 | 157 | 60.5 % | 5.27 |
| **2.5 (settled)** | **464** | **79.0 %** | **6.88** |
| 2.0 | 1,118 | 92.2 % | 8.02 |

The 468 rows (`dela` and `nes` out; `tete`, `tetes` and the four exceptions in) carry 6.99
occurrences per 10,000 words. Change 43b's figure, 301, counted every unglossed spelling of its 475
at Zipf ≥ 3 — the dictionary's own, English words and names among them; the 156 rows at Zipf ≥ 3 are
all among those 301.

**The rows**, by Zipf band: `spelling` → accented form (and its word when another) — the spelling's
Zipf / the accented form's.

| Zipf | Rows | Spellings |
|---|---|---|
| 4.00– | 9 | `meme` → même 4.77/6.32, `tres` → très 4.52/6.12, `apres` → après 4.33/6.12, `ecole` → école 4.21/5.29, `eglise` → église 4.14/4.98, `etes` → êtes, *être* 4.12/5.37, `francais` → français 4.07/5.78, `economie` → économie 4.07/5.05, `deja` → déjà 4.06/5.84 |
| 3.50–4.00 | 27 | `evidemment` → évidemment 3.93/4.81, `ete` → été, *être* 3.92/6.22, `ecoute` → écoute 3.90/4.88, `equipe` → équipe 3.85/5.48, `plutot` → plutôt 3.84/5.35, `tete` → tête 3.83/5.53, `probleme` → problème 3.82/5.41, `etant` → étant, *être* 3.81/5.29, `noel` → noël 3.79/4.78, `serie` → série 3.77/5.24, `energie` → énergie 3.74/4.99, `etais` → étais, *être* 3.74/5.38, `etudes` → études, *étude* 3.72/5.09, `etude` → étude 3.71/5.09, `pres` → près 3.66/5.45, `facon` → façon 3.65/5.39, `ecoutez` → écoutez, *écouter* 3.61/4.10, `serieux` → sérieux 3.60/4.98, `espere` → espère, *espérer* 3.58/5.13, `arrete` → arrête, *arrêter* 3.56/4.98, `ame` → âme 3.54/4.74, `idee` → idée 3.53/5.38, `tot` → tôt 3.51/4.96, `controle` → contrôle 3.50/5.07, `francaise` → française, *français* 3.50/5.31, `frere` → frère 3.50/5.18, `pret` → prêt 3.50/4.99 |
| 3.25–3.50 | 35 | `bebe` → bébé 3.49/4.81, `bientot` → bientôt 3.48/5.05, `drole` → drôle 3.48/4.84, `medias` → médias, *média* 3.47/4.94, `derriere` → derrière 3.46/5.16, `hopital` → hôpital 3.45/4.85, `chateau` → château 3.44/4.72, `maniere` → manière 3.43/5.28, `numero` → numéro 3.43/5.03, `annee` → année 3.41/5.66, `ecologie` → écologie 3.41/4.16, `economique` → économique 3.41/5.07, `ecrire` → écrire 3.41/4.87, `ecrit` → écrit, *écrire* 3.41/5.22, `epoque` → époque 3.40/5.23, `etoile` → étoile 3.40/4.38, `egalement` → également 3.39/5.59, `completement` → complètement 3.37/4.93, `societe` → société 3.36/5.51, `systeme` → système 3.36/5.40, `enorme` → énorme 3.36/4.74, `reponse` → réponse 3.35/5.08, `sante` → santé 3.35/5.22, `problemes` → problèmes, *problème* 3.33/5.09, `ecoles` → écoles, *école* 3.33/4.81, `deuxieme` → deuxième 3.32/5.24, `eviter` → éviter 3.32/4.99, `gateau` → gâteau 3.30/4.22, `medecin` → médecin 3.30/4.88, `etranger` → étranger 3.30/4.86, `echos` → échos, *écho* 3.29/3.64, `ecran` → écran 3.28/4.72, `epargne` → épargne, *épargner* 3.26/4.21, `eternel` → éternel 3.26/4.02, `garcon` → garçon 3.25/4.64 |
| 3.00–3.25 | 85 | `lache` → lâche, *lâcher* 3.24/4.40, `recu` → reçu, *recevoir* 3.24/5.01, `achete` → acheté, *acheter* 3.23/4.64, `interieur` → intérieur 3.23/5.13, `lumiere` → lumière 3.23/4.94, `prefere` → préfère, *préférer* 3.23/4.76, `republique` → république 3.23/5.07, `interet` → intérêt 3.22/5.13, `velo` → vélo 3.22/4.56, `resultat` → résultat 3.21/5.00, `annees` → années, *année* 3.20/5.63, `espece` → espèce 3.20/4.91, `fete` → fête 3.20/4.95, `interessant` → intéressant 3.20/4.76, `memoire` → mémoire 3.20/4.89, `batiment` → bâtiment 3.19/4.68, `chere` → chère, *cher* 3.19/4.46, `tele` → télé 3.19/4.78, `arret` → arrêt 3.18/4.80, `journee` → journée 3.18/5.30, `echange` → échange 3.18/4.74, `elu` → élu, *élire* 3.18/4.74, `etrange` → étrange 3.18/4.51, `americain` → américain 3.17/5.06, `exterieur` → extérieur 3.17/4.79, `interesse` → intéresse, *intéresser* 3.17/4.63, `presente` → présente, *présenter* 3.17/5.13, `regles` → règles, *règle* 3.17/4.92, `etudiant` → étudiant 3.17/4.54, `chomage` → chômage 3.16/4.62, `reve` → rêve 3.16/4.84, `serieusement` → sérieusement 3.16/4.57, `ecouter` → écouter 3.16/4.71, `electrique` → électrique 3.16/4.65, `serieuse` → sérieuse, *sérieux* 3.15/4.38, `etudiants` → étudiants, *étudiant* 3.15/4.84, `arriere` → arrière 3.13/4.93, `creer` → créer 3.13/4.97, `debat` → débat 3.13/4.84, `empeche` → empêche, *empêcher* 3.13/4.59, `regle` → règle 3.13/4.72, `etape` → étape 3.13/4.71, `evangile` → évangile 3.13/3.71, `cle` → clé 3.12/4.62, `generale` → générale, *général* 3.12/5.03, `modele` → modèle 3.12/4.97, `planete` → planète 3.12/4.75, `echec` → échec 3.12/4.56, `reseau` → réseau 3.11/5.08, `ecris` → écris, *écrire* 3.11/4.30, `acces` → accès 3.10/5.03, `freres` → frères, *frère* 3.10/4.82, `lacher` → lâcher 3.10/4.20, `peres` → pères, *père* 3.10/4.11, `resultats` → résultats, *résultat* 3.10/5.15, `desole` → désolé 3.09/4.78, `periode` → période 3.09/5.16, `etoiles` → étoiles, *étoile* 3.09/4.59, `decembre` → décembre 3.08/5.15, `eglises` → églises, *église* 3.08/4.27, `dernieres` → dernières, *dernier* 3.07/4.95, `developpement` → développement 3.07/5.21, `fevrier` → février 3.07/5.12, `reussi` → réussi, *réussir* 3.07/4.90, `ecriture` → écriture 3.07/4.59, `fenetre` → fenêtre 3.06/4.52, `pate` → pâte 3.05/4.10, `etablissement` → établissement 3.05/4.77, `americains` → américains, *américain* 3.04/4.85, `arretez` → arrêtez, *arrêter* 3.04/4.54, `medecine` → médecine 3.04/4.62, `securite` → sécurité 3.04/5.24, `impots` → impôts, *impôt* 3.03/4.54, `merite` → mérite 3.03/4.78, `troisieme` → troisième 3.03/5.10, `idees` → idées, *idée* 3.02/5.01, `necessaire` → nécessaire 3.02/4.98, `soiree` → soirée 3.02/5.00, `telecharger` → télécharger 3.02/4.02, `echelle` → échelle 3.01/4.64, `armee` → armée 3.00/5.15, `entierement` → entièrement 3.00/4.73, `genial` → génial 3.00/4.51, `economiques` → économiques, *économique* 3.00/4.71, `evite` → évite, *éviter* 3.00/4.16 |
| 2.75–3.00 | 106 | `deteste` → déteste, *détester* 2.99/4.54, `vetements` → vêtements, *vêtement* 2.99/4.64, `appele` → appelé, *appeler* 2.97/4.92, `inquiete` → inquiète, *inquiéter* 2.97/4.59, `leger` → léger 2.97/4.49, `metier` → métier 2.97/4.66, `reflexion` → réflexion 2.97/4.58, `agreable` → agréable 2.96/4.51, `gerer` → gérer 2.96/4.66, `differentes` → différentes, *différent* 2.95/5.03, `ecoutes` → écoutes, *écoute* 2.95/4.00, `equipement` → équipement 2.95/4.36, `etonnant` → étonnant 2.95/4.32, `biere` → bière 2.94/4.46, `batard` → bâtard 2.94/3.75, `carriere` → carrière 2.94/4.91, `differents` → différents, *différent* 2.94/5.05, `enquete` → enquête 2.94/4.86, `pates` → pâtes 2.94/3.99, `editeur` → éditeur 2.94/4.31, `bete` → bête 2.93/4.51, `caractere` → caractère 2.93/4.80, `celibataire` → célibataire 2.93/4.20, `defendre` → défendre 2.93/4.70, `qualite` → qualité 2.93/5.12, `cherie` → chérie, *chéri* 2.92/4.17, `calin` → câlin 2.92/3.59, `decouvrir` → découvrir 2.92/4.80, `menage` → ménage 2.92/4.40, `peche` → pêche 2.92/4.49, `ethique` → éthique 2.92/4.26, `etrangers` → étrangers, *étranger* 2.92/4.70, `americaine` → américaine, *américain* 2.91/4.90, `meteo` → météo 2.91/4.34, `prete` → prête, *prêter* 2.91/4.62, `electronique` → électronique 2.91/4.44, `evitez` → évitez, *éviter* 2.91/3.60, `liberte` → liberté 2.90/5.05, `eclair` → éclair 2.90/3.88, `empecher` → empêcher 2.89/4.72, `succes` → succès 2.89/5.07, `verite` → vérité 2.89/5.02, `epouse` → épouse 2.89/4.58, `enleve` → enlève, *enlever* 2.88/4.21, `entiere` → entière, *entier* 2.88/4.50, `bibliotheque` → bibliothèque 2.87/4.60, `materiel` → matériel 2.87/4.68, `etudier` → étudier 2.87/4.47, `reseaux` → réseaux, *réseau* 2.86/4.82, `superieur` → supérieur 2.86/4.80, `universite` → université 2.86/5.04, `defaut` → défaut 2.85/4.63, `greve` → grève 2.85/4.52, `regler` → régler 2.85/4.50, `vecu` → vécu, *vivre* 2.85/4.67, `academie` → académie 2.84/4.58, `feminin` → féminin 2.84/4.39, `garcons` → garçons, *garçon* 2.84/4.48, `gateaux` → gâteaux, *gâteau* 2.84/3.86, `proteger` → protéger 2.84/4.76, `prenom` → prénom 2.84/4.48, `prets` → prêts, *prêt* 2.84/4.63, `theorie` → théorie 2.84/4.75, `cine` → ciné 2.83/3.90, `controler` → contrôler 2.83/4.42, `diplome` → diplôme 2.83/4.38, `departement` → département 2.82/4.87, `leve` → lève, *lever* 2.82/4.40, `represente` → représente, *représenter* 2.82/4.84, `vetement` → vêtement 2.82/3.98, `etablissements` → établissements, *établissement* 2.82/4.58, `aeroport` → aéroport 2.81/4.63, `entree` → entrée 2.81/5.04, `gue` → gué 2.81/3.18, `impot` → impôt 2.81/4.57, `proces` → procès 2.81/4.82, `ane` → âne 2.81/3.80, `elus` → élus, *élire* 2.81/4.63, `donnees` → données, *donner* 2.80/5.08, `depeche` → dépêche 2.80/3.83, `dejeuner` → déjeuner 2.79/4.47, `extremement` → extrêmement 2.79/4.60, `particulierement` → particulièrement 2.79/4.99, `riviere` → rivière 2.79/4.59, `reponses` → réponses, *réponse* 2.79/4.59, `secretaire` → secrétaire 2.79/4.80, `epreuve` → épreuve 2.79/4.59, `equipes` → équipes, *équipe* 2.79/4.81, `gere` → gère, *gérer* 2.78/4.25, `honnete` → honnête 2.78/4.39, `recoit` → reçoit, *recevoir* 2.78/4.53, `verifier` → vérifier 2.78/4.59, `ble` → blé 2.77/4.12, `democratie` → démocratie 2.77/4.61, `francaises` → françaises, *français* 2.77/4.70, `generique` → générique 2.77/4.07, `pepin` → pépin 2.77/3.52, `petrole` → pétrole 2.77/4.45, `vehicule` → véhicule 2.77/4.61, `beaute` → beauté 2.76/4.75, `eleves` → élèves, *élever* 2.76/4.85, `lycee` → lycée 2.76/4.75, `medecins` → médecins, *médecin* 2.76/4.59, `aupres` → auprès 2.75/5.00, `ministere` → ministère 2.75/4.91, `reglement` → règlement 2.75/4.54 |
| 2.50–2.75 | 202 | `eleve` → élevé 2.74/4.76, `electricien` → électricien 2.74/3.32, `etudiante` → étudiante, *étudiant* 2.74/4.13, `cles` → clés, *clé* 2.73/4.48, `debile` → débile 2.73/4.11, `lachez` → lâchez, *lâcher* 2.73/3.36, `reflechir` → réfléchir 2.73/4.53, `equilibre` → équilibre 2.73/4.53, `cheri` → chéri 2.72/4.06, `considere` → considéré, *considérer* 2.72/4.61, `calins` → câlins, *câlin* 2.72/3.40, `enormement` → énormément 2.72/4.38, `esperer` → espérer 2.72/4.38, `evenement` → événement 2.72/4.66, `interessante` → intéressante, *intéressant* 2.72/4.33, `numerique` → numérique 2.72/4.65, `prepa` → prépa 2.72/3.57, `reveille` → réveille, *réveiller* 2.72/4.24, `recois` → reçois, *recevoir* 2.72/4.05, `etage` → étage 2.72/4.52, `chatelet` → châtelet 2.71/3.61, `derange` → dérange, *déranger* 2.71/4.32, `poesie` → poésie 2.71/4.40, `privee` → privée, *privé* 2.71/4.74, `echanges` → échanges, *échange* 2.71/4.47, `epicerie` → épicerie 2.71/3.83, `batiments` → bâtiments, *bâtiment* 2.70/4.46, `communaute` → communauté 2.70/5.01, `desormais` → désormais 2.70/4.97, `detruire` → détruire 2.70/4.46, `hopitaux` → hôpitaux, *hôpital* 2.70/4.20, `misere` → misère 2.70/4.24, `moitie` → moitié 2.70/4.99, `musee` → musée 2.70/4.80, `notres` → nôtres, *nôtre* 2.70/3.79, `pensee` → pensée 2.70/4.74, `pretexte` → prétexte 2.70/4.25, `realite` → réalité 2.70/5.04, `reves` → rêves, *rêve* 2.70/4.51, `speciale` → spéciale, *spécial* 2.70/4.54, `elevage` → élevage 2.70/4.15, `apprecie` → apprécie, *apprécier* 2.69/4.29, `melange` → mélange 2.69/4.52, `priere` → prière 2.69/4.31, `seance` → séance 2.69/4.67, `eclairage` → éclairage 2.69/4.05, `ecritures` → écritures, *écriture* 2.69/3.59, `chretien` → chrétien 2.68/4.27, `decouvert` → découvert, *découvrir* 2.68/4.72, `mene` → mène, *mener* 2.68/4.46, `presidente` → présidente, *président* 2.68/4.35, `realiser` → réaliser 2.68/4.70, `seche` → sèche, *sec* 2.68/4.14, `senat` → sénat 2.68/4.47, `amene` → amené, *amener* 2.67/4.28, `carrement` → carrément 2.67/4.36, `evenements` → événements, *événement* 2.67/4.74, `expres` → exprès 2.67/4.13, `litterature` → littérature 2.67/4.59, `malgre` → malgré 2.67/5.17, `patisserie` → pâtisserie 2.67/3.58, `symptomes` → symptômes, *symptôme* 2.67/4.25, `electeurs` → électeurs, *électeur* 2.67/4.47, `fidele` → fidèle 2.66/4.40, `recuperer` → récupérer 2.66/4.55, `strategie` → stratégie 2.66/4.63, `ecrivez` → écrivez, *écrire* 2.66/3.56, `electriques` → électriques, *électrique* 2.66/4.32, `differente` → différente, *différent* 2.65/4.50, `esperant` → espérant, *espérer* 2.65/4.12, `prevu` → prévu, *prévoir* 2.65/4.78, `rever` → rêver 2.65/4.22, `etiez` → étiez, *être* 2.65/4.06, `activite` → activité 2.64/5.06, `beton` → béton 2.64/4.17, `desir` → désir 2.64/4.48, `hesite` → hésite, *hésiter* 2.64/4.30, `ingenieur` → ingénieur 2.64/4.42, `tresor` → trésor 2.64/4.44, `edito` → édito 2.64/3.11, `etres` → êtres, *être* 2.64/4.34, `independant` → indépendant 2.63/4.39, `nucleaire` → nucléaire 2.63/4.56, `reussir` → réussir 2.63/4.51, `echecs` → échecs, *échec* 2.63/4.02, `ecrivain` → écrivain 2.63/4.41, `reveil` → réveil 2.62/4.15, `eventuellement` → éventuellement 2.62/4.24, `categorie` → catégorie 2.61/4.73, `decouverte` → découverte 2.61/4.73, `developper` → développer 2.61/4.66, `emmene` → emmène, *emmener* 2.61/3.99, `harcelement` → harcèlement 2.61/4.13, `honnetement` → honnêtement 2.61/4.08, `interesser` → intéresser 2.61/4.18, `mechant` → méchant 2.61/4.35, `ramene` → ramène, *ramener* 2.61/4.17, `rale` → râle, *râler* 2.61/3.40, `reduit` → réduit, *réduire* 2.61/4.55, `ecart` → écart 2.61/4.39, `etang` → étang 2.61/3.71, `actualites` → actualités, *actualité* 2.60/4.11, `aigue` → aiguë, *aigu* 2.60/3.55, `comite` → comité 2.60/4.89, `machoire` → mâchoire 2.60/3.66, `reduire` → réduire 2.60/4.68, `economiste` → économiste 2.60/3.89, `bresil` → brésil 2.59/4.58, `controles` → contrôles, *contrôle* 2.59/4.11, `generales` → générales, *général* 2.59/4.30, `vehicules` → véhicules, *véhicule* 2.59/4.54, `algerien` → algérien 2.58/4.13, `amitie` → amitié 2.58/4.55, `collegues` → collègues, *collègue* 2.58/4.49, `debats` → débats, *débat* 2.58/4.47, `enerve` → énerve, *énerver* 2.58/3.97, `facons` → façons, *façon* 2.58/4.23, `frontiere` → frontière 2.58/4.60, `independance` → indépendance 2.58/4.68, `lumieres` → lumières, *lumière* 2.58/4.27, `prevenir` → prévenir 2.58/4.42, `rateau` → râteau 2.58/2.96, `reveiller` → réveiller 2.58/4.19, `sejour` → séjour 2.58/4.49, `equitation` → équitation 2.58/3.58, `acceder` → accéder 2.57/4.35, `arrivee` → arrivée 2.57/4.99, `delire` → délire 2.57/4.16, `europeen` → européen 2.57/4.78, `europeenne` → européenne, *européen* 2.57/4.92, `integrer` → intégrer 2.57/4.32, `leche` → lèche 2.57/3.50, `pekin` → pékin 2.57/3.99, `recemment` → récemment 2.57/4.70, `secu` → sécu 2.57/3.51, `electricite` → électricité 2.56/4.57, `esperons` → espérons, *espérer* 2.56/4.16, `galere` → galère 2.56/3.97, `lecon` → leçon 2.56/4.35, `negatif` → négatif 2.56/4.24, `progres` → progrès 2.56/4.53, `reflexions` → réflexions, *réflexion* 2.56/4.04, `systemes` → systèmes, *système* 2.56/4.65, `eloigne` → éloigne, *éloigner* 2.56/4.00, `duree` → durée 2.55/4.91, `degueulasse` → dégueulasse 2.55/3.96, `felicitations` → félicitations, *félicitation* 2.55/4.23, `feminisme` → féminisme 2.55/3.78, `proprietaire` → propriétaire 2.55/4.74, `ecologique` → écologique 2.55/4.22, `ecoutons` → écoutons, *écouter* 2.55/3.01, `erotique` → érotique 2.55/3.60, `etablir` → établir 2.55/4.52, `alles` → allés, *aller* 2.54/4.01, `chretiens` → chrétiens, *chrétien* 2.54/4.51, `collegue` → collègue 2.54/4.31, `depenses` → dépenses, *dépense* 2.54/4.64, `esperance` → espérance 2.54/4.10, `mariee` → mariée 2.54/4.26, `meres` → mères, *mère* 2.54/4.25, `reparer` → réparer 2.54/4.17, `repete` → répète, *répéter* 2.54/4.29, `allee` → allée, *aller* 2.53/4.28, `arretes` → arrêtés, *arrêter* 2.53/4.18, `chateaux` → châteaux, *château* 2.53/3.82, `cloture` → clôture 2.53/4.11, `interets` → intérêts, *intérêt* 2.53/4.75, `naif` → naïf 2.53/3.68, `telechargement` → téléchargement 2.53/3.85, `alleluia` → alléluia 2.52/2.86, `celebre` → célèbre 2.52/4.74, `europeens` → européens, *européen* 2.52/4.64, `geant` → géant 2.52/4.28, `immediatement` → immédiatement 2.52/4.72, `legende` → légende 2.52/4.58, `possede` → possède, *posséder* 2.52/4.69, `pref` → préf 2.52/3.01, `sincerement` → sincèrement 2.52/4.13, `tetes` → têtes, *tête* 2.52/4.42, `enormes` → énormes, *énorme* 2.52/4.14, `actualite` → actualité 2.51/4.51, `batards` → bâtards, *bâtard* 2.51/3.34, `depasse` → dépasse, *dépasser* 2.51/4.36, `detruit` → détruit, *détruire* 2.51/4.44, `fetes` → fêtes, *fête* 2.51/4.43, `generalement` → généralement 2.51/4.88, `modeles` → modèles, *modèle* 2.51/4.57, `reviser` → réviser 2.51/3.70, `societes` → sociétés, *société* 2.51/4.78, `eliminer` → éliminer 2.51/4.22, `epreuves` → épreuves, *épreuve* 2.51/4.29, `etudie` → étudié, *étudier* 2.51/4.27, `achetent` → achètent, *acheter* 2.50/3.74, `ameliorer` → améliorer 2.50/4.66, `canape` → canapé 2.50/4.15, `comedie` → comédie 2.50/4.39, `defi` → défi 2.50/4.35, `frontieres` → frontières, *frontière* 2.50/4.46, `helas` → hélas 2.50/4.15, `majorite` → majorité 2.50/4.97, `mystere` → mystère 2.50/4.35, `etrangement` → étrangement 2.50/3.67 |
| exceptions (D2) | 4 | `sure` → sûre, *sûr* 3.89/4.43, `entrainant` → entraînant, *entraîner* 2.93/3.77, `surs` → sûrs, *sûr* 2.71/4.01, `sures` → sûres, *sûr* 2.45/3.44 |

The 22 spellings two words share, each read as the commoner (Q4): `ecoute` (écoute / écouté), `echange` (échange / échangé), `reve` (rêve / rêvé), `regle` (règle / réglé), `pate` (pâte / pâté), `merite` (mérite / mérité), `ecoutes` (écoutes / écoutés), `pates` (pâtes / pâtés), `peche` (pêche / péché), `epouse` (épouse / épousé), `diplome` (diplôme / diplômé), `depeche` (dépêche / dépêché), `equipes` (équipes / équipés), `eleves` (élèves / élevés), `eleve` (élevé / élève), `equilibre` (équilibre / équilibré), `derange` (dérange / dérangé), `melange` (mélange / mélangé), `seche` (sèche / séché), `controles` (contrôles / contrôlés), `leche` (lèche / léché), `galere` (galère / galéré).

**Left out: English writes it more often** (Q2: all out; 239 — 103 at Zipf ≥ 3, 136 from 2.5; French
/ English Zipf):

`video` 4.34/5.43, `grace` 4.21/4.56, `general` 4.05/5.47, `education` 4.04/5.28, `hotel` 3.93/4.95, `premiere` 3.90/4.16, `edition` 3.88/4.80, `editions` 3.83/3.85, `zero` 3.79/4.63, `college` 3.78/5.27, `mere` 3.78/4.13, `metro` 3.76/4.18, `videos` 3.74/4.79, `series` 3.69/5.35, `episode` 3.69/4.97, `pole` 3.64/4.23, `piece` 3.63/5.04, `cable` 3.60/4.54, `evolution` 3.59/4.41, `experience` 3.57/5.27, `special` 3.57/5.34, `crane` 3.55/3.92, `echo` 3.55/4.04, `cinema` 3.54/4.17, `president` 3.54/5.45, `gene` 3.53/4.36, `telephone` 3.53/4.41, `credit` 3.49/5.00, `ocean` 3.49/4.70, `debut` 3.48/4.52, `pre` 3.48/4.95, `present` 3.47/5.18, `hate` 3.45/5.11, `scene` 3.45/4.97, `opera` 3.43/4.27, `wikipedia` 3.43/3.92, `defense` 3.42/4.93, `region` 3.41/5.00, `memes` 3.40/3.75, `elections` 3.40/4.66, `role` 3.39/5.16, `extreme` 3.38/4.60, `generation` 3.38/4.82, `revolution` 3.38/4.56, `medical` 3.34/5.16, `elite` 3.34/4.40, `pieces` 3.33/4.72, `conference` 3.32/5.01, `theatre` 3.32/4.52, `different` 3.31/5.59, `difference` 3.30/5.01, `emission` 3.28/3.70, `complete` 3.26/5.10, `these` 3.25/6.04, `siege` 3.24/3.93, `reference` 3.23/4.74, `theme` 3.23/4.62, `hair` 3.22/5.08, `federal` 3.20/5.13, `election` 3.20/5.08, `reserve` 3.19/4.60, `cafe` 3.18/4.09, `legal` 3.18/5.08, `ames` 3.18/3.42, `regime` 3.17/4.30, `reunion` 3.17/3.93, `cesar` 3.15/3.29, `desert` 3.14/4.40, `detective` 3.14/4.21, `selection` 3.14/4.62, `deco` 3.13/3.41, `retro` 3.13/3.73, `adele` 3.12/3.45, `depend` 3.12/4.20, `hotels` 3.11/4.22, `operation` 3.11/4.92, `reel` 3.10/3.68, `baton` 3.09/3.64, `clement` 3.09/3.43, `resident` 3.09/4.38, `celeste` 3.08/3.19, `decision` 3.08/5.09, `desire` 3.08/4.61, `eclipse` 3.08/3.81, `creation` 3.07/4.56, `definition` 3.07/4.58, `details` 3.07/4.91, `elements` 3.07/4.70, `revision` 3.07/3.82, `depart` 3.06/3.67, `heritage` 3.06/4.36, `irene` 3.05/3.71, `resistance` 3.05/4.55, `sense` 3.05/5.19, `decide` 3.02/4.78, `demon` 3.02/4.06, `presence` 3.02/4.73, `reaction` 3.02/4.73, `reduction` 3.02/4.39, `television` 3.01/4.79, `emissions` 3.01/4.07, `competition` 3.00/4.91, `evaluation` 3.00/4.21, `creme` 2.99/3.12, `imperial` 2.99/4.21, `interim` 2.99/4.00, `penis` 2.99/4.05, `supreme` 2.98/4.69, `emir` 2.98/3.05, `evidence` 2.98/5.14, `demo` 2.97/4.06, `premieres` 2.97/3.25, `dome` 2.96/3.90, `cree` 2.94/3.02, `suede` 2.94/3.25, `cables` 2.93/3.87, `legion` 2.93/3.86, `episodes` 2.93/4.43, `rene` 2.92/3.34, `temperature` 2.92/4.64, `regional` 2.91/4.72, `detail` 2.90/4.57, `zenith` 2.90/3.15, `ere` 2.90/3.14, `energies` 2.90/3.54, `napoleon` 2.89/3.77, `ideal` 2.88/4.44, `elephant` 2.86/4.08, `metres` 2.86/4.17, `operations` 2.86/4.87, `reputation` 2.86/4.47, `coincidence` 2.85/3.96, `federation` 2.85/4.26, `metre` 2.85/3.76, `declaration` 2.84/4.10, `depot` 2.84/3.95, `liberation` 2.84/3.91, `macon` 2.84/3.13, `presenter` 2.84/3.77, `scenes` 2.84/4.58, `veritable` 2.84/3.01, `depression` 2.83/4.56, `benin` 2.82/3.03, `element` 2.81/4.46, `experiences` 2.81/4.55, `presentation` 2.81/4.31, `resume` 2.81/4.12, `resolution` 2.81/4.59, `economies` 2.81/3.89, `evasion` 2.81/3.41, `recent` 2.80/5.04, `regions` 2.80/4.53, `emotions` 2.80/4.37, `decider` 2.78/2.86, `heros` 2.77/2.83, `interpretation` 2.77/4.22, `angelique` 2.76/2.79, `prepare` 2.76/4.55, `residence` 2.76/4.36, `emotion` 2.76/4.19, `integration` 2.75/4.22, `peri` 2.75/2.99, `execution` 2.74/4.29, `elan` 2.74/2.77, `daphne` 2.73/3.26, `privilege` 2.73/4.20, `stereo` 2.73/3.77, `presents` 2.72/4.41, `prevention` 2.72/4.28, `liberal` 2.69/4.66, `leopard` 2.69/3.59, `cocaine` 2.68/3.98, `credits` 2.68/4.27, `liege` 2.68/2.95, `redemption` 2.68/3.82, `editorial` 2.67/4.11, `evident` 2.67/3.99, `consequences` 2.66/4.47, `rea` 2.66/3.31, `electron` 2.66/3.82, `legumes` 2.65/2.86, `realise` 2.65/4.26, `sincere` 2.65/3.84, `conde` 2.64/2.77, `declare` 2.64/4.05, `fee` 2.64/4.51, `gre` 2.64/2.98, `illegal` 2.64/4.68, `memo` 2.64/3.81, `procedure` 2.64/4.44, `protege` 2.64/2.70, `precision` 2.64/4.01, `unite` 2.64/3.90, `references` 2.63/4.42, `reservoir` 2.63/3.90, `eminence` 2.63/3.21, `blame` 2.62/4.63, `defend` 2.62/4.47, `demons` 2.62/4.00, `hetero` 2.62/2.74, `medieval` 2.62/4.05, `precise` 2.62/4.13, `reception` 2.62/4.23, `cheque` 2.61/3.49, `expedition` 2.61/3.99, `genocide` 2.61/3.91, `preparation` 2.61/4.32, `rodeo` 2.61/3.47, `atmosphere` 2.60/4.46, `karaoke` 2.60/3.43, `roles` 2.60/4.44, `separation` 2.60/4.13, `generations` 2.59/4.29, `renee` 2.59/3.35, `representation` 2.59/4.34, `equivalent` 2.59/4.50, `cone` 2.58/3.83, `negative` 2.58/4.79, `evacuation` 2.58/3.74, `theories` 2.57/4.22, `consequence` 2.56/4.07, `eugenie` 2.56/2.74, `feminine` 2.56/3.91, `nee` 2.56/3.21, `sphere` 2.55/3.91, `determination` 2.54/4.16, `multimedia` 2.54/3.56, `pretend` 2.54/4.33, `regent` 2.54/3.45, `deception` 2.53/3.65, `clementine` 2.52/2.89, `differences` 2.52/4.52, `celebration` 2.51/4.29, `genes` 2.51/4.14, `revelation` 2.51/3.96, `strategies` 2.51/4.29, `decisions` 2.50/4.72, `diners` 2.50/3.13, `supplement` 2.50/4.03

**Left out: as common as the accented word or more** (97; spelling / accented form):

`the` 5.35/4.34 (thé), `to` 4.91/2.09 (tô), `oeuvre` 4.59/2.54 (oeuvré), `peter` 4.56/3.96 (péter), `re` 4.49/4.11 (ré), `real` 4.28/3.57 (réal), `be` 4.27/3.00 (bé), `diesel` 4.05/2.16 (diésel), `he` 4.02/4.18 (hé), `media` 4.02/4.10 (média), `ve` 3.94/2.73 (vé), `bo` 3.87/2.45 (bô), `eden` 3.69/2.86 (éden), `museum` 3.62/3.51 (muséum), `mega` 3.61/3.78 (méga), `edit` 3.61/3.86 (édit), `mel` 3.59/2.60 (mél), `gates` 3.58/3.09 (gâtés), `arena` 3.56/2.91 (aréna), `reed` 3.55/2.63 (rééd), `event` 3.53/2.57 (évent), `telecom` 3.51/3.47 (télécom), `oil` 3.49/2.46 (oïl), `beta` 3.48/3.56 (bêta), `beat` 3.44/2.79 (béat), `omega` 3.41/3.21 (oméga), `maj` 3.39/2.71 (màj), `elise` 3.39/3.22 (élise), `tate` 3.37/2.94 (tâte), `beni` 3.36/3.58 (béni), `ref` 3.36/3.52 (réf), `gate` 3.35/3.32 (gâté), `memorial` 3.32/3.58 (mémorial), `pepe` 3.32/3.02 (pépé), `creative` 3.30/3.41 (créative), `din` 3.29/2.47 (dîn), `aprem` 3.28/2.91 (aprèm), `federer` 3.26/3.18 (fédérer), `credo` 3.23/2.48 (crédo), `negro` 3.23/3.27 (négro), `zoe` 3.22/3.52 (zoé), `medium` 3.21/3.49 (médium), `ye` 3.18/3.08 (yé), `matte` 3.17/1.91 (matté), `ecolo` 3.17/3.45 (écolo), `beats` 3.16/1.98 (béats), `geo` 3.15/3.41 (géo), `events` 3.14/2.36 (évents), `aid` 3.13/3.03 (aïd), `este` 3.13/2.51 (esté), `thai` 3.13/3.28 (thaï), `meta` 3.10/3.30 (méta), `bee` 3.08/2.94 (bée), `hehe` 3.08/3.18 (héhé), `fame` 3.07/2.34 (famé), `moe` 3.06/2.32 (moé), `hai` 3.03/3.04 (haï), `emirats` 3.00/3.30 (émirats), `executive` 2.98/3.14 (exécutive), `millenium` 2.98/2.61 (millénium), `cate` 2.95/2.09 (caté), `deo` 2.95/2.71 (déo), `decathlon` 2.94/3.15 (décathlon), `metropolis` 2.91/2.55 (métropolis), `delia` 2.90/2.51 (délia), `aigues` 2.89/2.94 (aiguës), `medina` 2.89/2.91 (médina), `melo` 2.88/2.92 (mélo), `infinite` 2.86/3.15 (infinité), `tse` 2.86/3.09 (tsé), `epi` 2.86/3.14 (épi), `privat` 2.82/1.06 (privât), `eros` 2.82/2.62 (éros), `peta` 2.79/1.84 (péta), `neon` 2.78/2.97 (néon), `ves` 2.77/1.33 (vés), `velin` 2.76/2.54 (vélin), `genet` 2.75/2.44 (genêt), `vegans` 2.74/2.03 (végans), `eureka` 2.71/2.33 (eurêka), `cherif` 2.69/2.97 (chérif), `angelus` 2.68/2.54 (angélus), `petri` 2.68/2.80 (pétri), `toe` 2.68/2.34 (toé), `aero` 2.66/2.91 (aéro), `libera` 2.63/2.54 (libéra), `mediator` 2.63/2.41 (médiator), `eon` 2.62/2.15 (éon), `trimballe` 2.61/1.98 (trimballé), `mee` 2.60/2.46 (mée), `vegeta` 2.60/2.27 (végéta), `feta` 2.57/2.02 (fêta), `memento` 2.56/2.35 (mémento), `synthe` 2.56/2.70 (synthé), `esperanto` 2.53/2.74 (espéranto), `tet` 2.53/2.70 (têt), `benef` 2.50/2.53 (bénef)

**Left out: the accented word is a name or no dictionary word** (308; the first 45):

`in` 5.24 (în), `el` 4.76 (él), `eric` 4.44 (éric), `red` 4.35 (réd), `da` 4.27 (dà), `android` 4.25 (androïd), `xixe` 4.11 (xixè), `po` 4.09 (pô), `ia` 4.06 (ià), `eva` 3.99 (éva), `antonio` 3.97 (antônio), `xxe` 3.96 (xxè), `allah` 3.94 (allâh), `elizabeth` 3.94 (élizabeth), `come` 3.90 (côme), `ep` 3.90 (ép), `lo` 3.89 (lô), `em` 3.86 (ém), `leo` 3.84 (léo), `see` 3.84 (sée), `elisabeth` 3.81 (élisabeth), `america` 3.79 (américa), `jesus` 3.79 (jésus), `ste` 3.75 (sté), `israel` 3.74 (israël), `che` 3.73 (ché), `jeremy` 3.72 (jérémy), `uber` 3.71 (über), `sergio` 3.68 (sérgio), `ge` 3.67 (gé), `leonard` 3.67 (léonard), `emile` 3.66 (émile), `ikea` 3.64 (ikéa), `sebastian` 3.64 (sébastian), `vera` 3.63 (véra), `eco` 3.63 (éco), `rafael` 3.62 (rafaël), `res` 3.62 (rés), `abu` 3.61 (abû), `montreal` 3.61 (montréal), `leonardo` 3.60 (léonardo), `mediapart` 3.60 (médiapart), `quebec` 3.60 (québec), `torres` 3.60 (torrès), `fe` 3.59 (fé), …

**The dictionary's own spellings, change 43d's** (19): `etats` 4.84, `etre` 4.77, `etait` 4.42, `ecosse` 3.67, `derniere` 3.60, `arreter` 3.58, `etaient` 3.48, `egyptiens` 3.15, `scenario` 2.97, `rebellion` 2.93, `matiere` 2.91, `cameraman` 2.90, `repond` 2.84, `colere` 2.83, `repondu` 2.72, `reponds` 2.62, `methodes` 2.60, `cooperation` 2.51, `matieres` 2.50.

### D2 — The rows join change 43b's table, kept apart from the dictionary's spellings

`reduce-fr-en.py` gains one literal, `WEB_SPELLINGS`, spelling → word, the 464 rows the tests keep, in
alphabetical order, merged into `UNMARKED_SPELLINGS` with one shared reason (« written on the web
without its marks; the owner's reviewed list, 2026-10-10 »): one row a line would be 464 reasons
saying the same thing. Each row is then read exactly as 43b's rows are — the named word its one
candidate, its own entries and any entry's inflection never making it another word's form, seeded
whether or not an entry names it. 43b's test holds for every row; its never-list grows by the
owner's: `ou`, `a`, `la`, `sur`, `du`, `des`, beside `cote`, `tache`, `pale`, `foret`, `aine`.

**An entry pointing at a row's spelling follows the row.** 43b drops an entry's form-of link to its
rows' spellings (`CAs` must not make `cas` a form of the acronym). For this list the link reads the
row's word instead: `patisseries`, whose entry says « plural of patisserie » (a spelling no entry
heads), reads as *pâtisserie* — dropped, it would have become a ranked word of its own (rank 51,713) —
and `entrainante`, `entrainantes`, « feminine (plural) of entrainant », read as *entraîner* as
`entraînante` does. Three forms move so, measured.

**The boundary is checked at each reduction.** Once the section is read, the reduction fails, naming
them, on any spelling of `WEB_SPELLINGS` the section gives a meaning of its own (a headword with a
sense that is neither a form nor a spelling of another word, a form of such a word, or an inflection
such a word's entry lists) or gives only as a spelling of its accented word (an entry every sense of
which is an alternative, obsolete or misspelt spelling of that word) — as it fails on a word ending in
a pronoun the forms table does not list. So no row of the web's list ever shadows a word of its own
(`cote`) or a spelling the dictionary gives (change 43d's `etait`), and a dictionary update that names
a row's spelling so stops for a person to decide. A spelling named only as a form of such a spelling
passes (`tete`, under `teter`): its entry says nothing of *tête*.

**The owner's exceptions** (2026-10-10, on change 43d's handover). Four spellings the section gives a
sense of their own are also the web's accent-less spellings of a commoner word; the check would refuse
them, so each is a row of its own in `UNMARKED_SPELLINGS`, beside 43b's, with its reason — not in
`WEB_SPELLINGS`, so the check never reads it, as it never reads 43b's rows. Measured in the corpora
(D5) and in wordfreq:

| Spelling | Its sense in the section | Zipf, French / English | The accented word | In the corpora: the accented word / its own sense | Today | Row |
|---|---|---|---|---|---|---|
| `sure` | feminine of *sur* « sour »; « alternative spelling of sûre » | 3.89 / 5.67 | `sûre` 4.43 | 31 / 2 (« Cette pomme est sure », « agrume moins sure que le citron ») | a form of *sur*, read « on, upon… sour » | → *sûr* |
| `surs` | masculine plural of *sur*; « alternative spelling of sûrs » | 2.71 / 1.31 | `sûrs` 4.01 | 0 / 1 (« Les raisins sont tellement surs ») | its own word « sour » (rank 27,366) | → *sûr* |
| `sures` | feminine plural of *sur*; « alternative spelling of sûres » | 2.45 / 1.51 | `sûres` 3.44 | 3 / 0 | its own word « sour » (rank 37,598) | → *sûr* |
| `entrainant` | present participle of `entrainer` (« post-1990 spelling of entraîner »); « alternative form of entraînant » | 2.93 / — | `entraînant` 3.77 | 4 / 0 (« entrainant une série d'émeutes », « plus entrainant que n'importe quel livre ») | its own word, glossed « post-1990 spelling of entraîner » (rank 20,559) | → *entraîner* |

Together 38 of 41 occurrences are the accented word, 3 the sour adjective. In Tatoeba the accented
`sûre` stands 510 times, `sûres` 96, `sûrs` 127: the unaccented `sure` and `sures` run at 6 % and 3 % of
them, the web's usual share (`francais` 2 %, `tres` 3 %), and `surs`' Zipf is 5 % of `sûrs`'. **All
four are rows (recommended; the owner confirms, Q6)**: `sure` and `sures` by the corpora; `surs` with
them, one adjective's agreement — its one corpus occurrence is the sour sense, but « ils sont surs »
is far likelier `sûrs`; `entrainant` because both its senses are *entraîner*'s family, and it reads
today as a word whose gloss only points elsewhere. `sures` sits under the threshold (2.45) and is a
row by name with its siblings. `sure` also fails the English test (Q2 would keep it out): it is an
exception to it too, because today it reads as the preposition *sur*, wrong in all 33 occurrences,
and English's « sure » means *sûr*'s. The sour adjective keeps `sur`, which is never a row; its
feminine and plurals read as *sûr*, the cost of 3 occurrences in 41.

*Rejected — a rule by marks alone* (any unaccented word retried with its marks): the owner's
decision; it would read `video`, `hotel` and `these` as French words, and `ou`, `a`, `la`, `sur` as
the words they are not. *Rejected — a table of its own with its own reading*: two mechanisms for one
thing; 43b's reads a spelling as its word already. *Rejected — rows naming the accented form*
(`etais` → `étais`): 43b's rows name the word and its test reads them so; the readings come from the
forms anyway (D3). *Rejected — the list as a data file*: the rule digest names the reducer's Python
modules (`pack_sources.py rules`); a `.tsv` would need that shared module edited, and every pair's
digest reread.

### D3 — A spelling carries its word's readings

After `grammar_rows`, each spelling of `UNMARKED_SPELLINGS` takes the readings of every form of its
word it writes without their marks, their marks (`-`, `other`) kept, each its own tag: `recu` those
of `reçu`; `arrete` those of `arrête` and `arrêté` together, both forms of *arrêter*; `lache` those
of `lâche` and `lâché`; `etais` the imperfect of *être* and, as `étais` does, the plural of *étai*
(`other`); `ecoute` those `écoute` carries — the noun, and *écouter*'s present as `other`; `sure` the
feminine singular of *sûr* instead of *sur*'s. A spelling writes both forms alike, so it carries both
readings, as `parle` carries five.

Measured: 867 readings for 442 of the 468 rows; the 26 others are adverbs and prepositions (`tres`,
`deja`, `apres`, `plutot`, `evidemment`, `malgre`, `aupres`…) and `alleluia`, whose words carry none
either.
43b's rows gain theirs: `age` (masculine singular noun), `ages` (masculine plural) and `ca` (`ça`'s
noun reading, as `ça` itself carries) — three; `forcement` none. 43b's D1 wrote no reading for its
rows; this change supersedes that for one rule over the table. The tag pool is byte for byte: every
reading is one the pool already holds.

The card then says what the form is, as for the accented form: « past participle of recevoir » for
`recu`, « past participle, first- and third-person singular present indicative or subjunctive and
second-person singular imperative of arrêter » for `arrete`.

*Rejected — no readings* (43b as written): the card of `etais` would show *être* « to be » and not
which form. *Rejected — the commonest accented form's readings alone*: `arrete` would claim the
present where « il a arrete » is the participle.

### D4 — What moves in the tables, the pins and the measurements

| File | Before → after | What |
|---|---|---|
| `tables/fr/forms.tsv` | 124,101 → 124,669 rows | the 468 spellings read as their words (465 were their own lemma, `sure` a form of *sur*, `tete` and `tetes` no form: `teter` is unranked); `patisseries`, `entrainante`, `entrainantes` follow the rows (D2); `désossé`, a form of *désosser*, its own lemma as it enters the cut; 566 forms of the entering lemmas |
| `tables/fr/freq.tsv` | 60,000 rows | the 465 ranked spellings out (12 in the top 5,000: `meme` 1,018, `tres` 1,697, `apres` 2,343, `ecole` 2,908 …; 42 in the top 10,000; 165 in the top 20,000; the last `sures` 37,598); 465 lemmas in at 59,533–59,997 (`dilapidation`, `désarçonner`, `enceint`, `mocassin`, and web words such as `depui`, `embete`); 58,515 ranks up by 1 to 465. The words keep their ranks (`même` 40, `très` 50, `déjà` 84, `français` 97) |
| `tables/fr/grammar.tsv` | 125,177 → 126,295 rows | in 1,127: the spellings' 867 readings, 43b's rows' 3, the entering lemmas' 257; out 9: `désossé`'s under *désosser*, `sure`'s under *sur*, the `other` readings `sure`, `sures`, `surs` and `entrainant` had toward the accented word (now their own), `entrainante`'s and `entrainantes`' under `entrainant`, `patisseries`' under `patisserie` |
| `tables/fr/level.tsv` | byte for byte | no spelling was a levelled word (46's rule 1, « unknown »); the reduction's report moves alone: 1,395 → 1,346 unknown lemmas passed over, the spans one to 49 ranks earlier (A1 1–1,084, C2 9,437–10,771) |
| `tables/fr/lexical.tsv` | 26,484 → 26,570 words | 89 entering lemmas fr-en glosses; out `surs`, `sures` (« sour ») and `entrainant` |
| `tables/fr/tags.tsv`, `studied.json` | byte for byte | |
| `tables/fr-en/gloss.tsv`, `senses.tsv` | 30,067 → 30,165 rows | 101 in, the entering lemmas (`dilapidation` « waste », `désarçonner` « to unhorse », `enceint` « pregnant », `mocassin`, `pingre` « miserly », names among them); out `surs` and `sures` « sour » and `entrainant` « post-1990 spelling of entraîner »; none changed |
| `tables/fr-en/mwe.tsv`, `NOTICE` | byte for byte | |
| fr-en's coverage | 93.6 / 86.9 / 76.3 → 93.8 / 87.2 / 76.8 % | 4,679 / 8,688 / 15,259 → 4,691 / 8,715 / 15,350 of the 5,000 / 10,000 / 20,000 commonest: unglossed spellings leave the tops, glossed lemmas take their place; floor 91.9 / 85.1 / 74.4 |
| `tables/fr-en/pin.json`, `manifest.json` | `reducer`, `pack`, `pack_version` | `snapshot` and `sources` byte for byte |
| `tables/fr-es/gloss.tsv`, `senses.tsv` | 19,023 → 19,058 rows | out 4, the French Wiktionary's words of their own fr-es glossed for a spelling: `tres` « Tres » (the Cuban guitar, shown for « tres »), `comite` « Cómitre » (the galley's overseer, for « comite »), `expres` « Intencional » and `reviser` « Revisar » — the last two now read through *exprès* « Expreso; Deliberadamente » and *réviser* « Revisar »; in 39, the entering lemmas (`désarçonner` « Desarzonar… », `enceint` « Embarazada… », `gloser` « Glosar »); every other row byte for byte |
| `tables/fr-es/mwe.tsv`, `NOTICE` | byte for byte | |
| fr-es's coverage | 83.0 / 70.7 / 56.7 → 83.1 / 70.8 / 57.0 % | 4,150 / 7,067 / 11,342 → 4,156 / 7,083 / 11,404; floor 81.4 / 68.8 / 54.5 |
| `tables/fr-es/pin.json`, `manifest.json` | `studied` (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `lexical.tsv`), `pack`, `pack_version`'s studied digest | its reducer digest and sources byte for byte; `level.tsv`'s record unchanged |
| The packs | fr-en 2,537,588 → 2,548,334 B; fr-es 1,971,114 → 1,980,699 B | at today's manifests' lengths |
| UD French-PUD (gated) | 99.13 / 96.41 / 99.90 → 99.13 / 96.40 / 100.00 % | « Etant » (AUX) now *être*; « Evidemment », which PUD lemmatises as written (`evidemment`), now *évidemment* — the row is right, the treebank keeps the unaccented lemma; the gates are 98.5 / 93.5 / 97. No row of the lower band stands in PUD or GSD's test section |
| UD French-GSD, test (reported) | 98.89 / 95.86 / 99.72 → 98.91 / 95.94 / 99.72 % | « Eglise » twice and « Equipe », now *église* and *équipe* |
| Readings on the treebanks (reported) | PUD participles 973 → 974 words read, pronouns 16.60 → 16.98 %; GSD nouns 1,772 → 1,776 | |

**en-fr, es-fr, es-en and en-es cannot move.** The rows, the check, the redirect and the readings
live in `reduce-fr-en.py`, which fr-en's rule digest names and no other pair's does; `tables/en/`,
`tables/es/` and their pairs' folders are not written; no core, wasm or extension code changes. Over
the prototype every lingua-wasm test passed — `english_baseline`, `spanish_baseline`,
`es_en_baseline`, `en_es_baseline`, `cross_native`, `parity`, `card_gloss_language`, `deck`
unmoved, the two French ones once re-blessed —, and `lingua-pack`'s committed-tables tests failed only
on fr-en's and fr-es's pins (their pack sha256 and fr-es's studied record), which the re-pin records;
43b's `spec_scenario_a_french_spelling_without_its_marks` passed.

### D5 — The rows in running text: the false readings

Every occurrence of a row's spelling as a word in the corpora (*Measured*), and what it is. A
hyphenated run the pack lists is read whole and untouched (`aigue-marine`, 3), and a page title of
the corpus file is no text (1):

| Rows | Met | The accented word | Names | Other languages' words | English |
|---|---|---|---|---|---|
| The 156 at Zipf ≥ 3 | 159 | 128 (80.5 %) | 12 (« Noel Murray » and four other Noels, « Shimon Peres », « Stephen Pate », « Fabienne Fevrier-Tabarnoust », « Ame V », « Huguet-Chateau », « Georg-Tele », « Je me nomme Meme ») | 18 (Catalan « Tres i Quatre », « tres savis »; Latin « TRES »; Walloon « tot » ×4; Occitan « ÒC tele » ×3, « Lou Medecin »; Italian « Serie A ») | 1 (« The First Noel ») |
| The 308 from Zipf 2.5 | 70 | 51 (73 %) | 4 (« Depeche Mode », « Ponte a Greve », « Cine Doré », « Berliner Allee ») | 15 (Occitan « Lou Tresor dóu Felibrige » ×10, one article on Mistral; Spanish « leche », « Copa Presidente », « presidente de Catalunya »; Italian « Trenta poesie »; Old French « Aigue (eau) ») | 0 |
| The owner's 4 exceptions | 41 | 38 | — | 3, the sour adjective (`sure` ×2, `surs`) | 0 |
| **All 468** | **270** | **217 (80.4 %)** | **16** | **36** | **1** |

By corpus, for the 468: Tatoeba, its sentences written by users, 138 met, 132 the accented word (the
lower band's 37 there, 36 right — « Elle a toujours vecu… », « Appele un taxi », « Il a un diplome
de cuisinier » —, the one false « Berliner Allee »); UD 77 met, 59 right; Wikipedia 44 met, 15
right — nearly all its false readings are other languages' text quoted in articles about them;
Gutenberg 9 of 9; change 39's pages 2 of 2.

**What the 308 lower-band rows add in false readings: 19 of their 70 occurrences** — 4 foreign names
and 15 words of other languages, ten of them one Occitan title repeated in one article. Today each of
these tokens reads as an unknown word with no gloss or, a capital in mid-sentence, as a name set
aside; with the rows they read as a French word they are not (`tresor` « treasure », `presidente`
« president »), the cost of a word a name or a neighbour language writes alike, as `Rose` or `Pierre`
already pay — and the owner keeps such rows (Q3). The edited corpora under-represent the web's lower
case; the capitals (« l'Eglise », « Etant donné », « Ecrivez ») are French's habit of unaccented
capitals — change 41's D2 rejected reading every capital without its accent as a rule; a row reads
only its own spelling, in any case.

### D6 — The goldens and the snapshots

**Re-blessed on the prototype's tables, 4 of the 221 probes on today's goldens move in each**, the
same four:

| Probe | fr-en | fr-es |
|---|---|---|
| `pack` | 2,537,588 → 2,548,334 B (and the new `pack_version`) | 1,971,114 → 1,980,699 B (and the new `pack_version`) |
| `analyse new-reader informel` | « apres » *apres*, no gloss → *après* « after… »; « genial » → *génial* « ingenious…; great, fantastic »; the page's count (37) and every other token unchanged — « la » in « chuis pas la » stays *le*, « etait » stays itself (43d) | « apres » → *après* « Después de… », « genial » → *génial* « Genial… » |
| `vocabulary-estimate new-reader` | universe 26,484 → 26,570 | the same |
| `vocabulary-estimate reader` | universe 26,483 → 26,569, estimated 2,185 unchanged | the same |

The deck the levels seed (`review-current`, `export-*`, `backup`) does not move: no level moves.

**Five phrase probes and two grammar probes are added** to the French scenario (`support/french.rs`:
the phrases after « à mon age », the grammar probes at the end of `REFERENCE_GRAMMAR`), blessed on
today's tables and on the prototype's:

| Probe | fr-en, today → this change | fr-es, today → this change |
|---|---|---|
| « c'etait deja tres bien » (the owner's) | `etait`, `deja`, `tres` unknown, no gloss → *déjà* « already… », *très*, and the expression `très bien` « very good, very well »; `etait` unknown until 43d | → *déjà* « Ya », *très*, *bien* (fr-es holds no `très bien`) |
| « une facon de parler francais » | `facon`, `francais` unknown → *façon*, the expression `façon de parler`, *français* « French » | → *façon* « Manera, forma… », *français* « Francés… » (fr-es holds no `façon de parler`) |
| « j'ai recu ta reponse » | `recu`, `reponse` unknown → *recevoir* « to receive », *réponse* « response, answer » | → *recevoir* « Recibir… », *réponse* « Respuesta » |
| « L'Ecole est fermée. » | `Ecole` the unknown `ecole` → *école* « school » | → *école* « Escuela » |
| « regarde cette video » | `video` unknown, unglossed → the same: left out (the owner's example, Q2) | the same |
| `word-grammar recu recevoir` | no reading → the masculine singular past participle | the same (« participio pasado de recevoir ») |
| `word-grammar arrete arrêter` | no reading → the participle, the present indicative and subjunctive (1, 3 sg) and the imperative (2 sg) | the same |

Each golden grows: `fr-en.golden` 380,843 → 387,318 B, `fr-es.golden` 253,589 → 258,029 B (the probes
added, then the four moves). `french_baseline.rs`'s
`the_probes_are_the_references_then_the_cards_then_forty_lemmas` counts 33 reference probes (31).
`test/word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts` render every grammar and phrase probe:
their counts go 92 → 94 and 42 → 47, and their snapshots gain the seven cards (27 and 26 lines) and
move nothing else — « grammar: past participle of recevoir », « row: très bien — very good, very
well », « row: façon de parler — façon de parler ». The lower band moves no probe: no page of the
corpus writes one of its spellings.

### D7 — Not here, and the changes around this one

- **Hyphenated words — later, a reviewed list of their own** (the owner, 2026-10-10, Q5). wordfreq
  splits a word at its hyphen and answers a mix of the pieces (`c'est-a-dire` 5.86 is `a`'s
  frequency, not the web's), so it cannot say how often the web writes `peut-etre`. 227 hyphenated
  forms of dictionary words have a spelling the dictionary names nowhere, 45 of them forms of the
  5,000 commonest lemmas: `lui-meme`, `c'est-a-dire`, `peut-etre`, `au-dela`, `eux-memes`,
  `vis-a-vis`, `apres-midi`, `grand-pere`, `grand-mere`, `beau-pere`, `celui-la`, `celle-la`,
  `ceux-la`. Today « peut-etre » reads as two words, `peut` (*pouvoir*) and `etre` (a word of its own
  until 43d). The table can name them as it names words, by the owner's judgement alone.
- **Wrong accents — later, a reviewed list of their own** (Q5). 14 spellings at Zipf ≥ 3 carry a
  mark, but not their word's, and the dictionary names them nowhere: `déja` 4.00, `trés` 3.92, `aprés`
  3.82, `á` 3.47, `egalité` 3.26, `étre` 3.23, `oú` 3.22, `electricité` 3.21, `mére` 3.15, `eté` 3.14,
  `théatre` 3.09, `verité` 3.08, `pêle` 3.01, `agé` 3.00; `á`, `oú` and `pêle` need care.
- **The dictionary's own spellings — change 43d.** The spellings the section gives only as their
  accented word (`etat`, `etre`, `etait`, `derniere`, `arreter`, `etaient`, `repondre`, `siecle`) and
  the inflections they list (`repond`, `repondu`) are 43d's, read by change 43's spelling rule, which
  files their readings under the accented word itself; D2's check keeps them out of the web's list.
  43d hands this list `tete` (taken, with `tetes`), and `sure`, `sures`, `surs` and `entrainant`
  (D2's exceptions). The four it says this list could name are no rows: `chouia` (« brochette »),
  `colon` (« colonist ») and `mercedes` (the car) are words of their own, which D2's check refuses,
  and `to` (« variant of tô », the terabyte's symbol) is 660 times commoner than *tô*. Both changes
  edit `reduce-fr-en.py` and re-pin fr-en and fr-es: whichever lands second re-reduces on the other's
  tables, re-blesses the two French goldens (« c'etait » moves with 43d) and holds that no spelling is
  read by both.
- **Found while measuring, not this change's.** `entrainer`, « post-1990 spelling of entraîner », is
  ranked as a word of its own (8,889) glossed by that pointer, while `entraine` and `entrainé` read as
  *entraîner* and `connaitre` as *connaître*: change 43's spelling rule (D7) does not read this one.
  Recorded for 43d and the owner.
- **Change 43c** (« CA » in capitals): no interplay. A word wholly in capitals that its acronym table
  does not hold falls through to the cascade, so « ECOLE » among lowercase words reads as *école*.
  43c also edits `reduce-fr-en.py` and re-pins both French pairs: the second to land re-reduces.
- **Change 43b** (merged): this change extends its table and its readings.

## Risks / Trade-offs

- **[A name or another language's word spelled like a row reads as a French word]** → 53 of 270
  occurrences in the corpora (D5), 19 of them added by the lower band; the owner keeps such rows (Q3),
  and Tatoeba's users' French reads 132 of 138 right.
- **[A spelling two words share]** → the 22 read as the commoner (Q4); `ecoute` carries *écouter*'s
  readings as `other`, as `écoute` does.
- **[The sour adjective's feminine and plurals read as *sûr*]** → 3 occurrences in 41 (D2); `sur`
  itself keeps the sour sense.
- **[The list misses the web's rarer spellings]** → the rows carry 79 % of such spellings' frequency
  down to Zipf 1.5.
- **[The dictionary names a row's spelling after an update]** → the reduction fails, naming it (D2).
- **[A gated treebank figure moves]** → PUD's content lemmas 96.41 → 96.40 %, one word, PUD's own
  unaccented lemma for « Evidemment »; 2.9 points above the gate.
- **[Another French table change lands first]** (43c, 43d, a fr-es refinement) → the second to land
  re-reduces on the committed `tables/fr/` and re-blesses `fr-en.golden` and `fr-es.golden`.
- **[wordfreq's next version]** → the rows do not depend on frequencies once reviewed; an update
  re-reduces with them, 43b's test and D2's check hold each row.

## Migration Plan

Nothing to migrate: no package lists a French pair, and no reader holds a status, card or level for
a spelling of the list. Rollback is a revert of the rows, the check, the redirect and the readings,
the re-reduced tables, the pins and the re-blessed files.

## Effort

1.25–2 ideal days, outside the programme's estimates: the rows, the exceptions, the boundary check,
the redirect and the readings with their tests 0.5–1; the re-reduction of fr-en and fr-es with their
pins, READMEs and `SOURCES.md` 0.25–0.5; the probes, the two re-blesses and the snapshots 0.25; the
committed-tables test and the gates 0.25.

## Settled by the owner (2026-10-10, in session)

- **Q1 — How common must a spelling be?** *Settled*: the longer list — Zipf ≥ 2.5, the 307 rows
  between 2.5 and 3 joining the 156 (`deteste`, `appele`, `leger`, `biere`), with the same tests.
- **Q2 — Spellings English writes more often than French.** *Settled*: all out — `video`, `hotel`,
  `these`, `role`, `region`, `cinema`, `cafe`, and `mere`, `premiere`, `memes` too (239).
- **Q3 — Rows that are also names or other languages' words.** *Settled*: kept (`noel`, `tres`,
  `serie`, `tresor`…); `dela` and `nes` stay out.
- **Q4 — Spellings two French words share.** *Settled*: read as the commoner (22).
- **Q5 — Hyphenated words and wrong accents.** *Settled*: later, a reviewed list of their own each
  (D7).
- **Change 43d's handover** (settled on 43d, 2026-10-10): `sure`, `surs`, `sures` and `entrainant` go
  to this list's review — D2 measures them and makes them exceptions with their reasons.

## Open question for the owner

- **Q6 — The four exceptions.** Recommended: all four rows — `sure`, `surs`, `sures` → *sûr*,
  `entrainant` → *entraîner* (D2: 38 of 41 occurrences the accented word; the sour adjective 3). The
  alternative is to keep `surs` as « sour » (its one corpus occurrence), at the cost of one adjective
  read two ways. Confirmed with the list's reading (task 0.2).
