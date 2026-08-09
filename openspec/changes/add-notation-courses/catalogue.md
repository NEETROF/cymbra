# First-party course catalogue (proposal)

Supporting material for `add-notation-courses` (not a validated artifact). A proposed first-party
catalogue of **50 courses** across three tracks, each `instrument: "piano"`, tagged `track` + `level`
so the home screen groups them. Ordering follows common beginner→advanced piano/theory progressions
(method-book style: Faber/Alfred for playing, ABRSM-grade-style for theory). Titles are French here
(chat language); shipped manifests carry inline i18n `{en, fr, es, it}`.

Legend — typical blocks each leans on: **T**ext, **D**iagram, **I**mage, **V**ideo, **Q**uestion,
**K** = playKey, **S** = score.

## Track A — Solfège / lecture (music reading & theory)

### Débutant
| id | Titre | But | Blocs |
|----|-------|-----|-------|
| `sol-portee-notes` | La portée : lignes et interlignes | Situer une note en hauteur | T D Q |
| `sol-cles` | Les clés de sol et de fa | Comprendre ce que fixe une clé | T D Q |
| `sol-nom-notes` | Nommer les notes (do→si) | Lire le nom dans les 2 clés | T D Q K |
| `sol-valeurs` | Ronde, blanche, noire, croche | Durées relatives | T D Q |
| `sol-silences` | Les silences | Reconnaître/compter un silence | T D Q |
| `sol-mesure` | Mesures, barres, pulsation | Découper le temps | T D Q |
| `sol-chiffrage` | Le chiffrage (4/4, 3/4, 2/4) | Lire une mesure | T D Q |
| `sol-alterations` | Dièse, bémol, bécarre | Altérer une note | T D Q K |
| `sol-armure-intro` | L'armure (intro) | À quoi sert l'armure | T D Q |
| `sol-point-liaison` | Le point et les liaisons | Prolonger une durée | T D Q |

### Intermédiaire
| id | Titre | But | Blocs |
|----|-------|-----|-------|
| `sol-intervalles` | Les intervalles | Mesurer un écart | T D Q K |
| `sol-gammes-majeures` | Les gammes majeures | Construire une gamme | T D Q K S |
| `sol-cycle-quintes` | Tonalités & cycle des quintes | Relier armures et tons | T D Q |
| `sol-triades` | Les triades (M/m) | Former un accord | T D Q K |
| `sol-rythmes-pointes` | Pointés & doubles croches | Rythmes plus fins | T D Q |
| `sol-articulations` | Legato, staccato, accents | Articuler | T D Q S |
| `sol-nuances` | Nuances & phrasé | Jouer l'intensité | T D Q |
| `sol-mesures-composees` | Mesures composées (6/8) | Sentir le ternaire | T D Q |
| `sol-anacrouse` | Anacrouse / levée | Départ avant le temps | T D Q |
| `sol-lecture-rythmique` | Lecture rythmique | Frapper un rythme | T D Q |

### Avancé
| id | Titre | But | Blocs |
|----|-------|-----|-------|
| `sol-gammes-mineures` | Mineures (nat./harm./mél.) | 3 formes mineures | T D Q K S |
| `sol-modulations` | Modulations | Changer de ton | T D Q S |
| `sol-septiemes` | Septièmes & renversements | Enrichir l'accord | T D Q K |
| `sol-cadences` | Cadences & ii–V–I | Enchaîner les accords | T D Q S |
| `sol-syncope-triolets` | Syncopes & triolets | Rythmes décalés | T D Q |
| `sol-ornements` | Ornements (trilles…) | Décorer une note | T D Q S |
| `sol-modes` | Les modes | Couleurs modales | T D Q |
| `sol-lecture-a-vue` | Lecture à vue (méthode) | Déchiffrer vite | T Q S |
| `sol-analyse` | Analyse d'une pièce | Comprendre une structure | T D Q S |
| `sol-transposition` | Transposition | Changer de hauteur | T D Q K |

## Track B — Utilisation de l'app (par mode & fonctions)

| id | Titre | But | Blocs |
|----|-------|-----|-------|
| `app-prise-en-main` | Prise en main : choisir & jouer | Le cœur de boucle | T V Q |
| `app-mode-synthesia` | Le mode Synthesia (waterfall) | Lire les tuiles | T I Q S |
| `app-mode-horizontal` | La portée horizontale | Suivre la portée défilante | T I Q S |
| `app-mode-partition` | Le mode Partition (gravé) | Lire une vraie partition | T I Q S |
| `app-midi` | Connecter un clavier MIDI | Brancher son piano | T I K |
| `app-sons-piano` | Sons de piano & soundfonts | Changer le son | T I Q |
| `app-mains` | Choisir la/les main(s) | Main droite/gauche/deux | T I Q |
| `app-metronome` | Métronome & tempo | Jouer en rythme | T Q |
| `app-practice` | Wait-mode & entraînement | Répéter par mesures | T I Q |
| `app-score-resume` | Score, jauge & résumé | Comprendre sa note | T I Q |
| `app-catalogue-favoris` | Catalogue, hub & favoris | Trouver des morceaux | T I Q |
| `app-communaute` | Leaderboards & profil | La dimension sociale | T I Q |

## Track C — Technique piano (unités C1, C2…)

| id | Titre | But | Blocs |
|----|-------|-----|-------|
| `tech-posture` | C1 — Posture & mains | Bien se placer | T I V Q |
| `tech-doigtes` | C2 — Doigtés de base | Numéroter les doigts | T D Q K |
| `tech-gamme-do` | C3 — La gamme de Do majeur | Jouer une gamme | T K S |
| `tech-accords` | C4 — Premiers accords | Poser une triade | T K S |
| `tech-arpeges` | C5 — Arpèges | Égrener un accord | T K S |
| `tech-independance` | C6 — Indépendance des mains | Deux rythmes | T K S |
| `tech-coordination` | C7 — Mains ensemble | Coordonner | T K S |
| `tech-premier-morceau` | C8 — Ton premier morceau | Jouer en entier | T S |

## Delivery approach

- **First wave (authored fully, seeded first)** — the beginner path + app basics, enough to prove the
  engine end-to-end: `sol-portee-notes`, `sol-cles`, `sol-nom-notes`, `sol-valeurs`, `sol-silences`,
  `sol-mesure`, `sol-alterations` (Track A débutant) + `app-prise-en-main`, `app-mode-synthesia`,
  `app-mode-horizontal`, `app-mode-partition` (Track B). ~11 courses.
- **The rest is a content backlog** — same format, seeded incrementally; no engine change needed. A
  course is data, so new ones ship by inserting rows, not releasing the app.
- **Instrument dimension** — every manifest is `instrument: "piano"`. A later drums track reuses the
  passive blocks (text/diagram/image/video/question) and swaps the interactive ones (`playKey` →
  a drum-pad block), so the format extends without a rewrite.
