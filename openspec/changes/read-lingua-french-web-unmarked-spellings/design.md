# Design — read-lingua-french-web-unmarked-spellings

## Context

See proposal.md (Why). What exists, on change 43b's implementation (`read-lingua-french-ca-as-ca-cedilla`,
pull request #884, at `52414fcd`), which this change builds on:

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer (change 43). `UNMARKED_SPELLINGS` (43b's D1), form → (word, reason), four rows — `ca` → *ça*, `age` and `ages` → *âge*, `forcement` → *forcément* —, read as `ELISIONS` is: `Lexicon.read` makes the named word a row's one candidate and returns before the entry is read, `_inflections` skips the form, `unmarked_spellings` seeds every row whether or not an entry names it. Its test: every row has a reason, names a ranked lemma reading as itself, and is neither an elided piece nor `du`/`des`; `cote`, `tache`, `pale`, `foret` and `aine` are never rows |
| The ranks (change 43's D6) | wordfreq's words that are French tokens, the commonest 60,000 kept, each the lemma of its own form. A spelling the dictionary names nowhere is ranked as a word of its own: `meme` 1,018, `tres` 1,697, `francais` 3,738, `deja` 3,793 — unglossed, no French dictionary word, levelless (46's rule 1, « unknown ») |
| The readings (change 45) | `grammar.tsv`, keyed by form, from the inflection tables the section's entries list; a spelling no entry names has none |
| Change 41's cascade | looks every token up lowercased and in NFC; its D2 measured and rejected « a capital read without its accent (`Ecole` → `école`) » as a cascade rule. The names rule sets aside a capital in mid-sentence whose lemma is no dictionary word, so « l'Eglise » reads today as the unknown `eglise` or as a name |
| Change 43b's D6 | measured 475 spellings at Zipf ≥ 3 that are another French word with its marks dropped and less than half as frequent; recorded two follow-ups the owner settled on 2026-10-10: the dictionary's own unmarked spellings (Q2, now `read-lingua-french-dictionary-unmarked-spellings`, 43d) and a reviewed list by name of the web's (Q4, this change) |
| `tables/fr-en/`, `tables/fr-es/` | fr-en 30,067 glosses, coverage 93.6 / 86.9 / 76.3 %, the pack 2,537,588 B; fr-es 19,023 glosses, coverage 83.0 / 70.7 / 56.7 %, the pack 1,971,114 B (both as 43b re-pinned them) |

## Goals / Non-Goals

**Goals:**
- The web's commonest French spellings without their marks read as their word, by rows a person
  reviews one by one (the owner's decision), never by a rule on marks alone.
- The candidate list built from data and shown whole with its frequencies, the proposed rows and
  every group left out — for the owner to read before anything is implemented.
- What the rows move measured — ranks, levels, dictionary words, readings, glosses, coverage, the
  treebanks, the goldens and the pins — and the spellings forms of their word, not new lemmas.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es, and any shared or edition module.
- The spellings the dictionary itself gives (`etre`, `etait`, `etat`): change 43d's.
- A core rule: French's analyser version does not move, and change 41's D2 stands (no cascade rule
  reads a capital without its accent; a row reads only the listed spellings).
- Hyphenated words and spellings with a wrong accent (`trés`, `déja`): measured, not here (D7).

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of change 43b's implementation
with D2's rows and D3's readings behind two switches, over the pinned release's `kaikki-French.jsonl`
(sha256 `2d7bbe5f…`), GSD's training and development sections (`4b9a87b1…`, `9221e508…`) and
wordfreq 3.1.1. Both switches off, it writes 43b's committed `tables/fr/` and `tables/fr-en/` byte for
byte (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`, `gloss.tsv`, `senses.tsv`,
`mwe.tsv`, `NOTICE`, `manifest.json`); fr-es's reducer over its pinned sources (`2e724e39…`,
`bf44ebb4…`, `bd14a5c4…`) writes 43b's `tables/fr-es/` byte for byte. The candidates read the French
section's headwords and inflections (386,734 and 368,719, lowercased in NFC) and the tables' forms.
The goldens were re-blessed in a scratch checkout of `52414fcd` with D6's probes, on 43b's tables and
on the prototype's; every lingua-wasm and lingua-pack test was run on the prototype's; the extension's
word-card snapshots were re-recorded on its goldens; the packs were measured on UD French-PUD and
GSD's test section as `measure/fr-ud.sh` does. The running-text counts read change 43c's corpus file:
UD's French test sections and GSD's training and development sections, Tatoeba's 727,867 French
sentences, change 44c's French Wikipedia sample and Gutenberg novels, and change 39's pages —
6.8 M words.

## Decisions

### D1 — The candidate list, and the rows proposed for review

Every form of `tables/fr/forms.tsv` written with an accent, a cedilla or a diaeresis gives a
spelling: the same word with its marks dropped (Unicode's combining marks; `œ` and `æ` are no marks,
change 43's D7 reads their ASCII spellings). A spelling is a candidate when wordfreq rates it at Zipf
≥ 3 — once per million words of its French, read from the web, subtitles and social media. Then, by
what the French section and the tables make of it:

| Step | At Zipf ≥ 3 | Examples |
|---|---|---|
| Spellings of an accented form, attested by wordfreq | 2,441 | |
| — the section gives it a meaning of its own: a word, a form of a word, or an inflection a word's entry lists | 1,847 | `ou`, `a`, `la`, `sur`, `du`, `des`, `cote`, `tache`, `pale`, `foret`, `aine`, `mais`, `marche`; real verb forms (`publie`, `oblige`); `sure`, `surs` (*sur* « sour »); `male` (under *mal*) |
| — the section gives it only as a spelling, or as a form of a spelling | 27 | 7 pass this change's tests and change 43d's rule reads them (`etre`, `etait`, `etats`, `derniere`, `arreter`, `etaient`, `egyptiens`); 1 passes and 43d's rule leaves it — `tete`, listed only as an inflection of `teter` « alternative form of téter », which 43d's forms guard keeps out because `tete` is how the web writes *tête* —: proposed here; 19 fail the tests (`to`, `diesel`, `ecosse`) |
| — change 43b's rows | 4 | `ca`, `age`, `ages`, `forcement` |
| **Named nowhere by the section** | **563** | |
| — hyphenated: wordfreq cannot count them | 95 | `peut-etre`, `c'est-a-dire`, `lui-meme` (D7) |
| — the accented word's lemma is a name or no dictionary word | 160 | `jesus`, `israel`, `quebec`, `eric`, `in` (`în`) |
| — as common as the accented word or more | 48 | `the` (*thé*), `peter` (*péter*), `media`, `aprem` |
| — English writes it more often than French does | 103 | `video`, `hotel`, `general`, `education`, `these` |
| — two French words could be meant | 6 | `ecoute` (*écoute*, or *écouté* of *écouter*), `pate` (*pâte* or *pâté*) |
| — the rest | 151 | `meme`, `tres`, `apres`, `francais`, `deja` |
| **Proposed rows**: the rest but 2 doubtful (`dela`, `nes`), the 6 read as the commoner word, and `tete` | **156** | |

The tests, each a number a person can check:
- *No meaning of its own, and not the dictionary's spelling*: the section gives the spelling
  (lowercased, in NFC) no sense that is neither a form nor a spelling of another word — as a headword,
  a form of such a word, or an inflection such a word's entry lists —, and does not give it only as a
  spelling of its accented word. This is the boundary with change 43d, which reads what the
  dictionary says; `tete`, named only as a form of a spelling 43d's rule leaves, is the one spelling
  at Zipf ≥ 3 between the two, and 43d hands it to this list.
- *A dictionary word*: the accented form's lemma in `forms.tsv` is in `lexical.tsv` (fr-en glosses
  it, and not as a name alone). Names are no words to learn, and a capital in mid-sentence whose lemma
  is no dictionary word is set aside already.
- *Less than half as frequent* as the accented form (43b's measure): a spelling as common as its
  accented word is another word — English's `the`, `real`, `peter`, `media`.
- *English*: wordfreq's English rates the spelling higher than its French does. The owner's examples
  (`video` 4.34 in French, 5.43 in English; `hotel` 3.93 / 4.95; `these` 3.25 / 6.04) all fall here.
- *Two words*: another accented form of the same spelling, of another lemma, at least a tenth as
  frequent. In the six, the commoner is 4.3 to 10 times the other (`ecoute`: *écoute* 4.88,
  *écouté* 4.15); each row reads as the commoner, as the accented form itself reads under M8.
- *Doubtful*, by reading the corpora (D5) and the words: `dela` (*delà*, or « de la » written
  together, a common slip) and `nes` (*nés*, or the console « NES »), left out.

Each row names the word its accented form reads as in `forms.tsv` today — `tres` → *très*, `etais` →
*être* (`étais`), `recu` → *recevoir* (`reçu`), `epargne` → *épargner* (`épargne`, M8) — so a row
follows change 43's choices and never makes one. 43 of the 156 are an inflected form (`ete`,
`francaise`, `problemes`, `recu`), the rest their word's own form.

**The threshold, measured.** Over the 2,340 spellings that pass every test down to Zipf 1.5, the
share of their frequency the rows above each threshold carry, and how often a reader meets them
(occurrences per 10,000 words of wordfreq's French):

| Zipf ≥ | Rows | Share | Per 10,000 words |
|---|---|---|---|
| 4.0 | 9 | 22.0 % | 1.92 |
| 3.5 | 35 | 37.3 % | 3.24 |
| **3.0** | **157** (155 proposed, `tete` beside them) | **60.5 %** | **5.27** |
| 2.5 | 464 | 79.0 % | 6.88 |
| 2.0 | 1,118 | 92.2 % | 8.02 |

Zipf 3 is proposed: a list one person reads in a sitting, each row met at least once per million
words; the band from 2.5 adds 307 rows to read for 1.6 occurrences per 10,000 words (Q1). Change
43b's figure, 301, counted every unglossed spelling of its 475 — the dictionary's own, English words
and names among them; the 156 proposed rows are all among those 301.

**The proposed rows**, by Zipf band: `spelling` → accented form (and its word when another) — the
spelling's Zipf / the accented form's.

| Zipf | Rows | Spellings |
|---|---|---|
| 4.00– | 9 | `meme` → même 4.77/6.32, `tres` → très 4.52/6.12, `apres` → après 4.33/6.12, `ecole` → école 4.21/5.29, `eglise` → église 4.14/4.98, `etes` → êtes, *être* 4.12/5.37, `francais` → français 4.07/5.78, `economie` → économie 4.07/5.05, `deja` → déjà 4.06/5.84 |
| 3.50–4.00 | 27 | `evidemment` → évidemment 3.93/4.81, `ete` → été, *être* 3.92/6.22, `ecoute` → écoute 3.90/4.88, `equipe` → équipe 3.85/5.48, `plutot` → plutôt 3.84/5.35, `tete` → tête 3.83/5.53, `probleme` → problème 3.82/5.41, `etant` → étant, *être* 3.81/5.29, `noel` → noël 3.79/4.78, `serie` → série 3.77/5.24, `energie` → énergie 3.74/4.99, `etais` → étais, *être* 3.74/5.38, `etudes` → études, *étude* 3.72/5.09, `etude` → étude 3.71/5.09, `pres` → près 3.66/5.45, `facon` → façon 3.65/5.39, `ecoutez` → écoutez, *écouter* 3.61/4.10, `serieux` → sérieux 3.60/4.98, `espere` → espère, *espérer* 3.58/5.13, `arrete` → arrête, *arrêter* 3.56/4.98, `ame` → âme 3.54/4.74, `idee` → idée 3.53/5.38, `tot` → tôt 3.51/4.96, `controle` → contrôle 3.50/5.07, `francaise` → française, *français* 3.50/5.31, `frere` → frère 3.50/5.18, `pret` → prêt 3.50/4.99 |
| 3.25–3.50 | 35 | `bebe` → bébé 3.49/4.81, `bientot` → bientôt 3.48/5.05, `drole` → drôle 3.48/4.84, `medias` → médias, *média* 3.47/4.94, `derriere` → derrière 3.46/5.16, `hopital` → hôpital 3.45/4.85, `chateau` → château 3.44/4.72, `maniere` → manière 3.43/5.28, `numero` → numéro 3.43/5.03, `annee` → année 3.41/5.66, `ecologie` → écologie 3.41/4.16, `economique` → économique 3.41/5.07, `ecrire` → écrire 3.41/4.87, `ecrit` → écrit, *écrire* 3.41/5.22, `epoque` → époque 3.40/5.23, `etoile` → étoile 3.40/4.38, `egalement` → également 3.39/5.59, `completement` → complètement 3.37/4.93, `societe` → société 3.36/5.51, `systeme` → système 3.36/5.40, `enorme` → énorme 3.36/4.74, `reponse` → réponse 3.35/5.08, `sante` → santé 3.35/5.22, `problemes` → problèmes, *problème* 3.33/5.09, `ecoles` → écoles, *école* 3.33/4.81, `deuxieme` → deuxième 3.32/5.24, `eviter` → éviter 3.32/4.99, `gateau` → gâteau 3.30/4.22, `medecin` → médecin 3.30/4.88, `etranger` → étranger 3.30/4.86, `echos` → échos, *écho* 3.29/3.64, `ecran` → écran 3.28/4.72, `epargne` → épargne, *épargner* 3.26/4.21, `eternel` → éternel 3.26/4.02, `garcon` → garçon 3.25/4.64 |
| 3.00–3.25 | 85 | `lache` → lâche, *lâcher* 3.24/4.40, `recu` → reçu, *recevoir* 3.24/5.01, `achete` → acheté, *acheter* 3.23/4.64, `interieur` → intérieur 3.23/5.13, `lumiere` → lumière 3.23/4.94, `prefere` → préfère, *préférer* 3.23/4.76, `republique` → république 3.23/5.07, `interet` → intérêt 3.22/5.13, `velo` → vélo 3.22/4.56, `resultat` → résultat 3.21/5.00, `annees` → années, *année* 3.20/5.63, `espece` → espèce 3.20/4.91, `fete` → fête 3.20/4.95, `interessant` → intéressant 3.20/4.76, `memoire` → mémoire 3.20/4.89, `batiment` → bâtiment 3.19/4.68, `chere` → chère, *cher* 3.19/4.46, `tele` → télé 3.19/4.78, `arret` → arrêt 3.18/4.80, `journee` → journée 3.18/5.30, `echange` → échange 3.18/4.74, `elu` → élu, *élire* 3.18/4.74, `etrange` → étrange 3.18/4.51, `americain` → américain 3.17/5.06, `exterieur` → extérieur 3.17/4.79, `interesse` → intéresse, *intéresser* 3.17/4.63, `presente` → présente, *présenter* 3.17/5.13, `regles` → règles, *règle* 3.17/4.92, `etudiant` → étudiant 3.17/4.54, `chomage` → chômage 3.16/4.62, `reve` → rêve 3.16/4.84, `serieusement` → sérieusement 3.16/4.57, `ecouter` → écouter 3.16/4.71, `electrique` → électrique 3.16/4.65, `serieuse` → sérieuse, *sérieux* 3.15/4.38, `etudiants` → étudiants, *étudiant* 3.15/4.84, `arriere` → arrière 3.13/4.93, `creer` → créer 3.13/4.97, `debat` → débat 3.13/4.84, `empeche` → empêche, *empêcher* 3.13/4.59, `regle` → règle 3.13/4.72, `etape` → étape 3.13/4.71, `evangile` → évangile 3.13/3.71, `cle` → clé 3.12/4.62, `generale` → générale, *général* 3.12/5.03, `modele` → modèle 3.12/4.97, `planete` → planète 3.12/4.75, `echec` → échec 3.12/4.56, `reseau` → réseau 3.11/5.08, `ecris` → écris, *écrire* 3.11/4.30, `acces` → accès 3.10/5.03, `freres` → frères, *frère* 3.10/4.82, `lacher` → lâcher 3.10/4.20, `peres` → pères, *père* 3.10/4.11, `resultats` → résultats, *résultat* 3.10/5.15, `desole` → désolé 3.09/4.78, `periode` → période 3.09/5.16, `etoiles` → étoiles, *étoile* 3.09/4.59, `decembre` → décembre 3.08/5.15, `eglises` → églises, *église* 3.08/4.27, `dernieres` → dernières, *dernier* 3.07/4.95, `developpement` → développement 3.07/5.21, `fevrier` → février 3.07/5.12, `reussi` → réussi, *réussir* 3.07/4.90, `ecriture` → écriture 3.07/4.59, `fenetre` → fenêtre 3.06/4.52, `pate` → pâte 3.05/4.10, `etablissement` → établissement 3.05/4.77, `americains` → américains, *américain* 3.04/4.85, `arretez` → arrêtez, *arrêter* 3.04/4.54, `medecine` → médecine 3.04/4.62, `securite` → sécurité 3.04/5.24, `impots` → impôts, *impôt* 3.03/4.54, `merite` → mérite 3.03/4.78, `troisieme` → troisième 3.03/5.10, `idees` → idées, *idée* 3.02/5.01, `necessaire` → nécessaire 3.02/4.98, `soiree` → soirée 3.02/5.00, `telecharger` → télécharger 3.02/4.02, `echelle` → échelle 3.01/4.64, `armee` → armée 3.00/5.15, `entierement` → entièrement 3.00/4.73, `genial` → génial 3.00/4.51, `economiques` → économiques, *économique* 3.00/4.71, `evite` → évite, *éviter* 3.00/4.16 |

Flagged inside the list: `ecoute`, `echange`, `reve`, `regle`, `pate`, `merite` (two words, Q4);
`noel`, `peres`, `pate`, `fevrier` (also names, D5, Q3); `tres`, `tot`, `tele`, `serie` (also other
languages' words, D5).

**Left out: English writes it more often** (103 at Zipf ≥ 3; French / English Zipf). The likeliest
French on a French page are `mere`, `premiere` and `memes` (Q2):

`video` 4.34/5.43, `grace` 4.21/4.56, `general` 4.05/5.47, `education` 4.04/5.28, `hotel` 3.93/4.95, `premiere` 3.90/4.16, `edition` 3.88/4.80, `editions` 3.83/3.85, `zero` 3.79/4.63, `college` 3.78/5.27, `mere` 3.78/4.13, `metro` 3.76/4.18, `videos` 3.74/4.79, `series` 3.69/5.35, `episode` 3.69/4.97, `pole` 3.64/4.23, `piece` 3.63/5.04, `cable` 3.60/4.54, `evolution` 3.59/4.41, `experience` 3.57/5.27, `special` 3.57/5.34, `crane` 3.55/3.92, `echo` 3.55/4.04, `cinema` 3.54/4.17, `president` 3.54/5.45, `gene` 3.53/4.36, `telephone` 3.53/4.41, `credit` 3.49/5.00, `ocean` 3.49/4.70, `debut` 3.48/4.52, `pre` 3.48/4.95, `present` 3.47/5.18, `hate` 3.45/5.11, `scene` 3.45/4.97, `opera` 3.43/4.27, `wikipedia` 3.43/3.92, `defense` 3.42/4.93, `region` 3.41/5.00, `memes` 3.40/3.75, `elections` 3.40/4.66, `role` 3.39/5.16, `extreme` 3.38/4.60, `generation` 3.38/4.82, `revolution` 3.38/4.56, `medical` 3.34/5.16, `elite` 3.34/4.40, `pieces` 3.33/4.72, `conference` 3.32/5.01, `theatre` 3.32/4.52, `different` 3.31/5.59, `difference` 3.30/5.01, `emission` 3.28/3.70, `complete` 3.26/5.10, `these` 3.25/6.04, `siege` 3.24/3.93, `reference` 3.23/4.74, `theme` 3.23/4.62, `hair` 3.22/5.08, `federal` 3.20/5.13, `election` 3.20/5.08, `reserve` 3.19/4.60, `cafe` 3.18/4.09, `legal` 3.18/5.08, `ames` 3.18/3.42, `regime` 3.17/4.30, `reunion` 3.17/3.93, `cesar` 3.15/3.29, `desert` 3.14/4.40, `detective` 3.14/4.21, `selection` 3.14/4.62, `deco` 3.13/3.41, `retro` 3.13/3.73, `adele` 3.12/3.45, `depend` 3.12/4.20, `hotels` 3.11/4.22, `operation` 3.11/4.92, `reel` 3.10/3.68, `baton` 3.09/3.64, `clement` 3.09/3.43, `resident` 3.09/4.38, `celeste` 3.08/3.19, `decision` 3.08/5.09, `desire` 3.08/4.61, `eclipse` 3.08/3.81, `creation` 3.07/4.56, `definition` 3.07/4.58, `details` 3.07/4.91, `elements` 3.07/4.70, `revision` 3.07/3.82, `depart` 3.06/3.67, `heritage` 3.06/4.36, `irene` 3.05/3.71, `resistance` 3.05/4.55, `sense` 3.05/5.19, `decide` 3.02/4.78, `demon` 3.02/4.06, `presence` 3.02/4.73, `reaction` 3.02/4.73, `reduction` 3.02/4.39, `television` 3.01/4.79, `emissions` 3.01/4.07, `competition` 3.00/4.91, `evaluation` 3.00/4.21

**Left out: as common as the accented word or more** (48; spelling / accented form):

`the` 5.35/4.34 (thé), `peter` 4.56/3.96 (péter), `real` 4.28/3.57 (réal), `be` 4.27/3.00 (bé), `media` 4.02/4.10 (média), `eden` 3.69/2.86 (éden), `museum` 3.62/3.51 (muséum), `mega` 3.61/3.78 (méga), `edit` 3.61/3.86 (édit), `mel` 3.59/2.60 (mél), `gates` 3.58/3.09 (gâtés), `arena` 3.56/2.91 (aréna), `reed` 3.55/2.63 (rééd), `event` 3.53/2.57 (évent), `telecom` 3.51/3.47 (télécom), `oil` 3.49/2.46 (oïl), `beta` 3.48/3.56 (bêta), `beat` 3.44/2.79 (béat), `omega` 3.41/3.21 (oméga), `elise` 3.39/3.22 (élise), `tate` 3.37/2.94 (tâte), `beni` 3.36/3.58 (béni), `ref` 3.36/3.52 (réf), `gate` 3.35/3.32 (gâté), `memorial` 3.32/3.58 (mémorial), `pepe` 3.32/3.02 (pépé), `creative` 3.30/3.41 (créative), `din` 3.29/2.47 (dîn), `aprem` 3.28/2.91 (aprèm), `federer` 3.26/3.18 (fédérer), `credo` 3.23/2.48 (crédo), `negro` 3.23/3.27 (négro), `zoe` 3.22/3.52 (zoé), `medium` 3.21/3.49 (médium), `ye` 3.18/3.08 (yé), `ecolo` 3.17/3.45 (écolo), `beats` 3.16/1.98 (béats), `geo` 3.15/3.41 (géo), `events` 3.14/2.36 (évents), `aid` 3.13/3.03 (aïd), `este` 3.13/2.51 (esté), `thai` 3.13/3.28 (thaï), `meta` 3.10/3.30 (méta), `bee` 3.08/2.94 (bée), `hehe` 3.08/3.18 (héhé), `fame` 3.07/2.34 (famé), `moe` 3.06/2.32 (moé), `emirats` 3.00/3.30 (émirats)

**Left out: the accented word is a name or no dictionary word** (160; the first 45):

`in` 5.24 (în), `eric` 4.44 (éric), `red` 4.35 (réd), `da` 4.27 (dà), `android` 4.25 (androïd), `xixe` 4.11 (xixè), `eva` 3.99 (éva), `antonio` 3.97 (antônio), `xxe` 3.96 (xxè), `allah` 3.94 (allâh), `elizabeth` 3.94 (élizabeth), `come` 3.90 (côme), `lo` 3.89 (lô), `em` 3.86 (ém), `leo` 3.84 (léo), `see` 3.84 (sée), `elisabeth` 3.81 (élisabeth), `america` 3.79 (américa), `jesus` 3.79 (jésus), `israel` 3.74 (israël), `che` 3.73 (ché), `jeremy` 3.72 (jérémy), `uber` 3.71 (über), `sergio` 3.68 (sérgio), `ge` 3.67 (gé), `leonard` 3.67 (léonard), `emile` 3.66 (émile), `ikea` 3.64 (ikéa), `sebastian` 3.64 (sébastian), `vera` 3.63 (véra), `eco` 3.63 (éco), `rafael` 3.62 (rafaël), `res` 3.62 (rés), `abu` 3.61 (abû), `montreal` 3.61 (montréal), `leonardo` 3.60 (léonardo), `mediapart` 3.60 (médiapart), `quebec` 3.60 (québec), `torres` 3.60 (torrès), `erik` 3.58 (érik), `jesse` 3.57 (jessé), `tre` 3.57 (tré), `regina` 3.55 (régina), `joel` 3.54 (joël), `ke` 3.53 (ké), …

**Below the threshold, if the owner lowers it** (Q1): 307 more rows between Zipf 2.5 and 3 — 291
that pass every test and 16 of two words; the first 40:

`deteste` → déteste, *détester* 2.99/4.54, `vetements` → vêtements, *vêtement* 2.99/4.64, `appele` → appelé, *appeler* 2.97/4.92, `inquiete` → inquiète, *inquiéter* 2.97/4.59, `leger` → léger 2.97/4.49, `metier` → métier 2.97/4.66, `reflexion` → réflexion 2.97/4.58, `agreable` → agréable 2.96/4.51, `gerer` → gérer 2.96/4.66, `differentes` → différentes, *différent* 2.95/5.03, `ecoutes` → écoutes, *écoute* 2.95/4.00, `equipement` → équipement 2.95/4.36, `etonnant` → étonnant 2.95/4.32, `biere` → bière 2.94/4.46, `batard` → bâtard 2.94/3.75, `carriere` → carrière 2.94/4.91, `differents` → différents, *différent* 2.94/5.05, `enquete` → enquête 2.94/4.86, `pates` → pâtes 2.94/3.99, `editeur` → éditeur 2.94/4.31, `bete` → bête 2.93/4.51, `caractere` → caractère 2.93/4.80, `celibataire` → célibataire 2.93/4.20, `defendre` → défendre 2.93/4.70, `qualite` → qualité 2.93/5.12, `cherie` → chérie, *chéri* 2.92/4.17, `calin` → câlin 2.92/3.59, `decouvrir` → découvrir 2.92/4.80, `menage` → ménage 2.92/4.40, `peche` → pêche 2.92/4.49, `ethique` → éthique 2.92/4.26, `etrangers` → étrangers, *étranger* 2.92/4.70, `americaine` → américaine, *américain* 2.91/4.90, `meteo` → météo 2.91/4.34, `prete` → prête, *prêter* 2.91/4.62, `electronique` → électronique 2.91/4.44, `evitez` → évitez, *éviter* 2.91/3.60, `liberte` → liberté 2.90/5.05, `eclair` → éclair 2.90/3.88, `empecher` → empêcher 2.89/4.72, …

### D2 — The rows join change 43b's table, kept apart from the dictionary's spellings

`reduce-fr-en.py` gains one literal, `WEB_SPELLINGS`, spelling → word, the owner's rows in
alphabetical order, merged into `UNMARKED_SPELLINGS` with one shared reason (« written on the web
without its marks; the owner's reviewed list, 2026-10-10 »): one row a line would be 156 reasons
saying the same thing. Each row is then read exactly as 43b's rows are — the named word its one
candidate, its own entries (none) and any entry's inflection never making it another word's form,
seeded whether or not an entry names it. 43b's test holds for every row; its never-list grows by the
owner's: `ou`, `a`, `la`, `sur`, `du`, `des`, beside `cote`, `tache`, `pale`, `foret`, `aine`.

**The boundary is checked at each reduction.** Once the section is read, the reduction fails,
naming them, on any spelling of `WEB_SPELLINGS` the section gives a meaning of its own (a headword
with a sense that is neither a form nor a spelling of another word, a form of such a word, or an
inflection such a word's entry lists) or gives only as a spelling of its accented word (an entry
every sense of which is an alternative, obsolete or misspelt spelling of that word) — as it fails on
a word ending in a pronoun the forms table does not list. So no row ever shadows a word of its own
(`cote`, `sure`) or a spelling the dictionary gives (change 43d's `etait`), and a dictionary update
that names a row's spelling so stops for a person to decide: delete the row, or leave the spelling
to 43d's rule. A spelling named only as a form of such a spelling passes (`tete`, under `teter`): its
entry says nothing of *tête*. 43b's four rows are not checked: their entries are what they replace.

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
(`other`); `ecoute` those `écoute` carries — the noun, and *écouter*'s present as `other`. A spelling
writes both forms alike, so it carries both readings, as `parle` carries five.

Measured: 279 readings for 145 of the 156 rows (`tete` the feminine singular noun *tête* carries);
the 11 others are adverbs (`tres`, `deja`, `apres`,
`plutot`, `bientot`), whose words carry none either. 43b's rows gain theirs: `age` (masculine
singular noun), `ages` (masculine plural) and `ca` (`ça`'s noun reading, as `ça` itself carries) —
three; `forcement` none. 43b's D1 wrote no reading for its rows; this change supersedes that for one
rule over the table. The tag pool is byte for byte: every reading is one the pool already holds.

The card then says what the form is, as for the accented form: « past participle of recevoir » for
`recu`, « past participle, first- and third-person singular present indicative or subjunctive and
second-person singular imperative of arrêter » for `arrete`.

*Rejected — no readings* (43b as written): the card of `etais` would show *être* « to be » and not
which form; 43 rows are inflected forms. *Rejected — the commonest accented form's readings alone*:
`arrete` would claim the present where « il a arrete » is the participle.

### D4 — What moves in the tables, the pins and the measurements

| File | Before → after | What |
|---|---|---|
| `tables/fr/forms.tsv` | 124,101 → 124,298 rows | the 155 spellings the section names nowhere read as their words (each was its own lemma); `tete`, no form today (its entry's word, `teter`, is unranked), a form of *tête*; `désossé`, a form of *désosser*, its own lemma as it enters the cut; 196 forms of the entering lemmas |
| `tables/fr/freq.tsv` | 60,000 rows | the 155 out (12 in the top 5,000: `meme` 1,018, `tres` 1,697, `apres` 2,343, `ecole` 2,908 …; 42 in the top 10,000; all in the top 20,000, the last `genial` 18,731); 155 lemmas in at 59,843–59,997 (`dilapidation`, `désarçonner`, `enceint`, and web words such as `depui`, `embete`); 58,825 ranks up by 1 to 155. The words keep their ranks (`même` 40, `très` 50, `déjà` 84, `français` 97) |
| `tables/fr/grammar.tsv` | 125,177 → 125,576 rows | the spellings' 279 readings, 43b's rows' 3, the entering lemmas' 118; out one, `désossé`'s reading under *désosser* |
| `tables/fr/level.tsv` | byte for byte | no spelling was a levelled word (46's rule 1, « unknown »); the reduction's report moves alone: 1,395 → 1,346 unknown lemmas passed over, the spans one to 49 ranks earlier (A1 1–1,084, C2 9,437–10,771) |
| `tables/fr/lexical.tsv` | 26,484 → 26,517 words | 33 entering lemmas fr-en glosses; none out |
| `tables/fr/tags.tsv`, `studied.json` | byte for byte | |
| `tables/fr-en/gloss.tsv`, `senses.tsv` | 30,067 → 30,101 rows | 34 in, the entering lemmas (`dilapidation` « waste », `désarçonner` « to unhorse », `enceint` « pregnant », a surname); none out or changed — no spelling was glossed |
| `tables/fr-en/mwe.tsv`, `NOTICE` | byte for byte | |
| fr-en's coverage | 93.6 / 86.9 / 76.3 → 93.8 / 87.2 / 76.7 % | 4,679 / 8,688 / 15,259 → 4,691 / 8,715 / 15,345 of the 5,000 / 10,000 / 20,000 commonest: unglossed spellings leave the tops, glossed lemmas take their place; floor 91.9 / 85.1 / 74.4 |
| `tables/fr-en/pin.json`, `manifest.json` | `reducer`, `pack`, `pack_version` | `snapshot` and `sources` byte for byte |
| `tables/fr-es/gloss.tsv`, `senses.tsv` | 19,023 → 19,033 rows | out: `tres` « Tres » — the French Wiktionary's Cuban guitar, a word of its own there, which fr-es showed for « tres »; in: 11 entering lemmas (`désarçonner` « Desarzonar, desconcertar, confundir », `enceint` « Embarazada, preñada, en estado », `gloser` « Glosar » …); every other row byte for byte |
| `tables/fr-es/mwe.tsv`, `NOTICE` | byte for byte | |
| fr-es's coverage | 83.0 / 70.7 / 56.7 → 83.1 / 70.8 / 57.0 % | 4,150 / 7,067 / 11,342 → 4,156 / 7,083 / 11,401; floor 81.4 / 68.8 / 54.5 |
| `tables/fr-es/pin.json`, `manifest.json` | `studied` (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `lexical.tsv`), `pack`, `pack_version`'s studied digest | its reducer digest and sources byte for byte; `level.tsv`'s record unchanged |
| The packs | fr-en 2,537,588 → 2,540,636 B; fr-es 1,971,114 → 1,973,142 B | at today's manifests' lengths |
| UD French-PUD (gated) | 99.13 / 96.41 / 99.90 → 99.13 / 96.40 / 100.00 % | « Etant » (AUX) now *être*; « Evidemment », which PUD lemmatises as written (`evidemment`), now *évidemment* — the row is right, the treebank keeps the unaccented lemma; the gates are 98.5 / 93.5 / 97 |
| UD French-GSD, test (reported) | 98.89 / 95.86 / 99.72 → 98.91 / 95.94 / 99.72 % | « Eglise » twice and « Equipe », now *église* and *équipe* |
| Readings on the treebanks (reported) | PUD participles 973 → 974 words read, pronouns 16.60 → 16.98 %; GSD nouns 1,772 → 1,776 | |

**en-fr, es-fr, es-en and en-es cannot move.** The rows, the check and the readings live in
`reduce-fr-en.py`, which fr-en's rule digest names and no other pair's does; `tables/en/`,
`tables/es/` and their pairs' folders are not written; no core, wasm or extension code changes. Over
the prototype every lingua-wasm test passed — `english_baseline`, `spanish_baseline`,
`es_en_baseline`, `en_es_baseline`, `cross_native`, `parity`, `card_gloss_language`, `deck`
unmoved, the two French ones once re-blessed —, and `lingua-pack`'s committed-tables tests failed only
on fr-en's and fr-es's pins (their pack sha256 and fr-es's studied record), which the re-pin records;
43b's `spec_scenario_a_french_spelling_without_its_marks` passed.

### D5 — The rows in running text

Every occurrence of a row's spelling as a word in the corpora (*Measured*), by how it is written,
and what it is:

| Corpus | Words | Rows met | Lower case | Capital | The accented word | Names | Other languages | English |
|---|---|---|---|---|---|---|---|---|
| Tatoeba | 5,767,417 | 63 | 36 | 27 | 60 | 2 (« Noel ne connaît rien… », « Je me nomme Meme. ») | 0 | 1 (« The First Noel ») |
| UD (GSD's three sections, the other treebanks' test sections) | 420,218 | 58 | 12 | 46 | 45 | 10 (« Noel Murray » and three other Noels, « Shimon Peres », « Stephen Pate », « Fabienne Fevrier-Tabarnoust », « Ame V », « Huguet-Chateau », « Georg-Tele ») | 3 (« Serie A », « Serie D ») | 0 |
| Wikipedia (44c's sample) | 376,000 | 28 | 11 | 17 | 13 | 0 | 15 (Catalan « Tres i Quatre », « Les Tres Torres », « tres savis »; Latin « TRES > tres »; Walloon « tot » ×4; Occitan « ÒC tele » ×3, « Lou Medecin », « Joan-l'an-pres ») | 0 |
| Gutenberg (44c's novels) | 226,576 | 8 | 0 | 8 | 8 | 0 | 0 | 0 |
| Change 39's pages | 1,093 | 2 | 2 | 0 | 2 | 0 | 0 | 0 |
| **All** | **6.8 M** | **159** | **61** | **98** | **128 (80.5 %)** | **12** (`noel` 5) | **18** | **1** |

Today each of the 159 reads as an unknown word with no gloss or, a capital in mid-sentence, as a name
set aside; with the rows 128 read as their word, and 31 as a French word they are not — names spelled
like a word (`noel`, `peres`, `pate`, `fevrier`) and other languages' words quoted in French text
(`tres`, `tot`, `tele`, `serie`), which is the cost of a word a name or a neighbour language writes
alike, as `Rose` or `Pierre` already pay. The edited corpora under-represent the web's lower case
(61 of 159), and Tatoeba, its sentences written by users, reads 60 of 63 right. 98 are capitals
French leaves unaccented (« l'Eglise », « Etant donné », « Evidemment ») — change 41's D2 rejected
reading every capital without its accent as a rule; a row reads only its own spelling, in any case.

### D6 — The goldens and the snapshots

**Re-blessed on the prototype's tables, 4 of the 221 probes on 43b's goldens move in each**, the
same four:

| Probe | fr-en | fr-es |
|---|---|---|
| `pack` | 2,537,588 → 2,540,636 B (and the new `pack_version`) | 1,971,114 → 1,973,142 B (and the new `pack_version`) |
| `analyse new-reader informel` | « apres » *apres*, no gloss → *après* « after… »; « genial » → *génial* « ingenious…; great, fantastic »; the page's count (37) and every other token unchanged — « la » in « chuis pas la » stays *le*, « etait » stays itself (43d) | « apres » → *après* « Después de… », « genial » → *génial* « Genial… » |
| `vocabulary-estimate new-reader` | universe 26,484 → 26,517 | the same |
| `vocabulary-estimate reader` | universe 26,483 → 26,516, estimated 2,185 unchanged | the same |

The deck the levels seed (`review-current`, `export-*`, `backup`) does not move: no level moves.

**Five phrase probes and two grammar probes are added** to the French scenario (`support/french.rs`:
the phrases after « à mon age », the grammar probes at the end of `REFERENCE_GRAMMAR`), blessed on
43b's tables and on the prototype's:

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
well », « row: façon de parler — façon de parler ».

### D7 — Not here, and the changes around this one

- **Hyphenated words** (Q5). wordfreq splits a word at its hyphen and answers a mix of the pieces
  (`c'est-a-dire` 5.86 is `a`'s frequency, not the web's), so it cannot say how often the web writes
  `peut-etre`. 227 hyphenated forms of dictionary words have a spelling the dictionary names nowhere,
  45 of them forms of the 5,000 commonest lemmas: `lui-meme`, `c'est-a-dire`, `peut-etre`,
  `au-dela`, `eux-memes`, `vis-a-vis`, `apres-midi`, `grand-pere`, `grand-mere`, `beau-pere`,
  `celui-la`, `celle-la`, `ceux-la`. Today « peut-etre » reads as two words, `peut` (*pouvoir*) and
  `etre` (a word of its own until 43d). The table could name them as it names words, by the owner's
  judgement alone.
- **Wrong accents** (Q5). 14 spellings at Zipf ≥ 3 carry a mark, but not their word's, and the
  dictionary names them nowhere: `déja` 4.00, `trés` 3.92, `aprés` 3.82, `á` 3.47, `egalité` 3.26,
  `étre` 3.23, `oú` 3.22, `electricité` 3.21, `mére` 3.15, `eté` 3.14, `théatre` 3.09, `verité` 3.08,
  `pêle` 3.01, `agé` 3.00. The owner's decision names spellings without accents; these are another
  kind, and `á`, `oú` and `pêle` need care.
- **The dictionary's own spellings — change 43d.** The spellings the section gives only as their
  accented word (`etat`, `etre`, `etait`, `derniere`, `arreter`, `etaient`, `repondre`, `siecle`) and
  the inflections they list (`repond`, `repondu`) are 43d's, read by change 43's spelling rule, which
  files their readings under the accented word itself; D2's check keeps them out of this list. 43d's
  design hands this list five spellings: `tete`, which it leaves (its forms guard), proposed here
  (D1); `sure`, `sures` and `surs`, forms of *sur* « sour » — words of their own, and the forms of the
  owner's never `sur`/`sûr` —, never rows; `entrainant` (Zipf 2.93), the participle of `entrainer`,
  a word of the section and below the threshold. The four it says this list could name are no rows: `chouia`
  (« brochette »), `colon` (« colonist ») and `mercedes` (the car) are words of their own, which D2's
  check refuses, and `to` (« variant of tô », the terabyte's symbol) is 660 times commoner than *tô*. Both changes edit `reduce-fr-en.py` and re-pin fr-en and fr-es: whichever lands second
  re-reduces on the other's tables, re-blesses the two French goldens (« c'etait » moves with 43d) and
  holds that no spelling is read by both.
- **Change 43c** (« CA » in capitals): no interplay. A word wholly in capitals that its acronym table
  does not hold falls through to the cascade, so « ECOLE » among lowercase words reads as *école*.
  43c also edits `reduce-fr-en.py` and re-pins both French pairs: the second to land re-reduces.
- **Change 43b**: this change extends its table and its readings; it lands after it, and ships with
  or after it.

## Risks / Trade-offs

- **[A name or another language's word spelled like a row reads as a French word]** → 31 of 159
  occurrences in the corpora (D5); the rows most exposed are named for the owner (Q3), and Tatoeba's
  users' French reads 60 of 63 right.
- **[A spelling two words share]** → the six read as the commoner, 4.3 to 10 times the other (Q4);
  `ecoute` carries *écouter*'s readings as `other`, as `écoute` does.
- **[The list misses the web's rarer spellings]** → the rows carry 60 % of such spellings' frequency
  down to Zipf 1.5; the next band is Q1's.
- **[The dictionary names a row's spelling after an update]** → the reduction fails, naming it (D2).
- **[A gated treebank figure moves]** → PUD's content lemmas 96.41 → 96.40 %, one word, PUD's own
  unaccented lemma for « Evidemment »; 2.9 points above the gate.
- **[Another French table change lands first]** (43c, 43d, a fr-es refinement) → the second to land
  re-reduces on the committed `tables/fr/` and re-blesses `fr-en.golden` and `fr-es.golden`.
- **[wordfreq's next version]** → the rows do not depend on frequencies once reviewed; an update
  re-reduces with them, 43b's test and D2's check hold each row.

## Migration Plan

Nothing to migrate: no package lists a French pair, and no reader holds a status, card or level for
a spelling of the list. Rollback is a revert of the rows, the check and the readings, the re-reduced
tables, the pins and the re-blessed files.

## Effort

1–1.75 ideal days, outside the programme's estimates: the rows, the boundary check and the readings
with their tests 0.5–0.75; the re-reduction of fr-en and fr-es with their pins, READMEs and
`SOURCES.md` 0.25–0.5; the probes, the two re-blesses and the snapshots 0.25; the committed-tables
test and the gates 0.25. The owner's reading of the list (task 0.1) comes before.

## Open questions for the owner

Each settled before the implementation (task 0.1); the recommendation first.

- **Q1 — How common must a spelling be?** Zipf ≥ 3, once per million words: 156 rows (recommended).
  Lowering to 2.5 adds 307 rows to read (`deteste`, `vetements`, `appele`, `leger`, `metier`,
  `biere`, `peche`…) for 1.6 more occurrences per 10,000 words.
- **Q2 — Spellings English also writes, more often than French** (103: `video`, `hotel`, `these`,
  `general`, `role`, `region`, `cinema`, `theatre`, `cafe`, `president`). Left out, all of them
  (recommended, as the owner said of `video` and `hotel`): an English word on a French page would
  read as a French one. Or name some back: `mere` (« ma mere »), `premiere` (« la premiere fois »)
  and `memes` are the likeliest French.
- **Q3 — Rows that are also names or other languages' words.** `noel` (*noël*; « Noel » the name 5
  times of 14 in the corpora), `peres` (« Shimon Peres »), `pate` (« Stephen Pate »), `fevrier`;
  `tres` (Catalan, Spanish « three »), `tot`, `tele`, `serie` (« Serie A »). Kept (recommended): the
  French word is what a French page means most often. And the two left out: `dela` (*delà*, or « de
  la » written together) and `nes` (*nés*, or « NES ») — out (recommended).
- **Q4 — Spellings two French words share** (6): `ecoute` (*écoute* / *écouté*), `echange`, `reve`,
  `regle`, `pate` (*pâte* / *pâté*), `merite`. Read as the commoner, 4.3 to 10 times the other
  (recommended), or left out.
- **Q5 — Hyphenated words and wrong accents.** `peut-etre`, `c'est-a-dire`, `lui-meme`,
  `apres-midi` (45 among the commonest words; wordfreq cannot count them), and `trés`, `déja`,
  `aprés`, `mére` (14): not here (recommended); a later reviewed list of each if the owner wants it.
