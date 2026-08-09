-- Demo courses for MANUAL TESTING of add-notation-courses (tranches 2–3).
-- NOT the real first-party catalogue (that is tranche 6). Safe to re-run
-- (ON CONFLICT upsert). Removes with:
--   DELETE FROM music.courses WHERE id LIKE 'demo-%';
--
-- Run against your local music DB, e.g. from the repo root:
--   set -a; source backend/.env; set +a
--   psql "$CYMBRA_MUSIC_DATABASE_URL" -f backend/scripts/seed_courses_demo.sql
--
-- Then sign in in the app: the "Courses" section appears above the favorites.

INSERT INTO music.courses
  (id, status, instrument, track, level, sort_order, schema_version, title, content)
VALUES
(
  'demo-reading-staff', 'published', 'piano', 'solfege', 'beginner', 1, 1,
  $${"en":"Reading the staff","fr":"Lire la portée","es":"Leer el pentagrama","it":"Leggere il pentagramma"}$$,
  $${
    "schemaVersion": 1,
    "id": "demo-reading-staff",
    "instrument": "piano", "track": "solfege", "level": "beginner",
    "title": {"en":"Reading the staff","fr":"Lire la portée"},
    "blocks": [
      {"type":"text","text":{
        "en":"Music sits on 5 lines and 4 spaces — the staff. Higher on the staff sounds higher.",
        "fr":"La musique se pose sur 5 lignes et 4 interlignes : la portée. Plus c'est haut, plus c'est aigu."}},
      {"type":"diagram","id":"staff-lines"},
      {"type":"diagram","id":"treble-clef"},
      {"type":"text","text":{
        "en":"The treble clef curls around the G line and usually carries the right hand.",
        "fr":"La clé de sol s'enroule autour de la ligne de sol et porte en général la main droite."}},
      {"type":"question",
        "prompt":{"en":"What does a sharp (♯) do?","fr":"Que fait un dièse (♯) ?"},
        "options":[
          {"en":"Raises the note a semitone","fr":"Monte la note d'un demi-ton"},
          {"en":"Lowers the note a semitone","fr":"Descend la note d'un demi-ton"},
          {"en":"Cancels an accidental","fr":"Annule une altération"}],
        "answerIndex":0,
        "feedback":{"en":"Yes — the very next key to the right.","fr":"Oui — la touche juste à droite."}},
      {"type":"diagram","id":"note-quarter"},
      {"type":"text","text":{"en":"Nice — you finished the course!","fr":"Bravo — tu as terminé le cours !"}}
    ]
  }$$
),
(
  'demo-synthesia', 'published', 'piano', 'app-usage', 'beginner', 1, 1,
  $${"en":"Synthesia mode","fr":"Le mode Synthesia","es":"Modo Synthesia","it":"Modalità Synthesia"}$$,
  $${
    "schemaVersion": 1,
    "id": "demo-synthesia",
    "instrument": "piano", "track": "app-usage", "level": "beginner",
    "title": {"en":"Synthesia mode","fr":"Le mode Synthesia"},
    "blocks": [
      {"type":"text","text":{
        "en":"In Synthesia mode, falling tiles show which key to press and when.",
        "fr":"En mode Synthesia, des tuiles qui tombent montrent quelle touche presser et quand."}},
      {"type":"question",
        "prompt":{"en":"When do you press a key?","fr":"Quand presse-t-on une touche ?"},
        "options":[
          {"en":"When its tile reaches the keyboard","fr":"Quand sa tuile atteint le clavier"},
          {"en":"As soon as the tile appears","fr":"Dès que la tuile apparaît"}],
        "answerIndex":0,
        "feedback":{"en":"Right — play it as the tile lands.","fr":"Exact — joue au moment où la tuile arrive."}},
      {"type":"text","text":{"en":"That's it — try a piece in Synthesia!","fr":"Voilà — essaie un morceau en Synthesia !"}}
    ]
  }$$
)
ON CONFLICT (id) DO UPDATE SET
  status = EXCLUDED.status, instrument = EXCLUDED.instrument,
  track = EXCLUDED.track, level = EXCLUDED.level, sort_order = EXCLUDED.sort_order,
  schema_version = EXCLUDED.schema_version, title = EXCLUDED.title,
  content = EXCLUDED.content, updated_at = now();
