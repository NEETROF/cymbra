# First-party course catalogue

Supporting material for `add-notation-courses` (not a validated artifact).

The shipped curriculum is **42 solfège courses** — 7 units × 6 lessons, `track: "solfege"`,
`instrument: "piano"` — authored as JSON under **`backend/content/courses/`** (one file per
course; that directory's README is the authoring guide). The corpus is the source of truth:
`backend/scripts/gen_seed_courses.py` compiles it into the idempotent `seed_courses.sql`, and
`apps/music/test/courses/content_corpus_test.dart` gates every file with the real client parser.

| Unit | Level | Thème | Leçons |
|------|-------|-------|--------|
| u1 | beginner | La portée et tes premières notes | clavier + do central, la portée, la clé de sol, do-ré-mi, fa-sol, bilan (Au clair de la lune) |
| u2 | beginner | La pulsation et le rythme | pulsation, noire/blanche, ronde + mesure 4/4, silences, 3/4, bilan |
| u3 | beginner | Toute la clé de sol | la-si, do aigu + octave, lignes/interlignes, lignes supplémentaires, secondes/tierces, bilan (Ode à la joie) |
| u4 | intermediate | La clé de fa et la main gauche | clé de fa, do-si-la-sol, toute la portée de fa, do central pont, deux côtés, bilan |
| u5 | intermediate | Croches et nouvelles mesures | croches, noire pointée, blanche pointée + valse, 6/8, contretemps, bilan |
| u6 | intermediate | Dièses, bémols et tonalités | ton/demi-ton, dièse, bémol/bécarre, gamme majeure, armure (sol/fa M), bilan |
| u7 | advanced | Intervalles, accords et oreille | secondes/tierces à l'oreille, quartes/quintes, triade majeure, majeur/mineur, trois piliers, grand final |

Every lesson: a ≤2-sentence hook, one concept, then ≥4 interactive exercises (≥2 among
`readPlay`/`nameNote`/`placeNote`/`rhythmTap`/`earChoice`/`buildChord`), a payoff, a recap —
inline-i18n `{en, fr, es, it}` throughout. The earlier app-usage tutorials were retired (the
target is solfège, not the app); a future technique/drums track slots in as more rows with a
different `track`, reusing the same blocks.

Adding or revising a course: edit/add a JSON file → run the corpus test → regenerate the seed →
`psql -f backend/scripts/seed_courses.sql`. No app release.
