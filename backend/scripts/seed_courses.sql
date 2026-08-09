-- First-wave interactive courses (change: add-notation-courses, tranche 6).
--
-- The real first-party seed: idempotent upsert, safe to re-run. Run against the
-- music DB, e.g. from the repo root:
--   set -a; source backend/.env; set +a
--   psql "$CYMBRA_MUSIC_DATABASE_URL" -f backend/scripts/seed_courses.sql
--
-- The rest of the catalogue (openspec/changes/add-notation-courses/catalogue.md)
-- is a data-only backlog: adding a course is another row here, no app release.

INSERT INTO music.courses
  (id, status, instrument, track, level, sort_order, schema_version, title, content)
VALUES
-- ─── Track A · solfège · beginner ───────────────────────────────────────────
(
  'sol-reading-staff', 'published', 'piano', 'solfege', 'beginner', 1, 1,
  $${"en":"Reading the staff","fr":"Lire la portée","es":"Leer el pentagrama","it":"Leggere il pentagramma"}$$,
  $${
    "schemaVersion":1,"id":"sol-reading-staff","instrument":"piano","track":"solfege","level":"beginner",
    "title":{"en":"Reading the staff","fr":"Lire la portée","es":"Leer el pentagrama","it":"Leggere il pentagramma"},
    "blocks":[
      {"type":"text","text":{
        "en":"Music is written on 5 lines and 4 spaces — the staff. The higher a note sits, the higher it sounds.",
        "fr":"La musique s'écrit sur 5 lignes et 4 interlignes : la portée. Plus une note est haute, plus le son est aigu.",
        "es":"La música se escribe en 5 líneas y 4 espacios: el pentagrama. Cuanto más alta está una nota, más agudo suena.",
        "it":"La musica si scrive su 5 righe e 4 spazi: il pentagramma. Più una nota è in alto, più il suono è acuto."}},
      {"type":"diagram","id":"staff-lines"},
      {"type":"diagram","id":"treble-clef"},
      {"type":"text","text":{
        "en":"The treble clef curls around the G line and usually carries the right hand.",
        "fr":"La clé de sol s'enroule autour de la ligne de sol et porte en général la main droite.",
        "es":"La clave de sol rodea la línea de sol y suele llevar la mano derecha.",
        "it":"La chiave di violino avvolge la riga del sol e di solito porta la mano destra."}},
      {"type":"question",
        "prompt":{"en":"Where does a higher-sounding note sit?","fr":"Où se place une note plus aiguë ?","es":"¿Dónde se coloca una nota más aguda?","it":"Dove si trova una nota più acuta?"},
        "options":[
          {"en":"Higher on the staff","fr":"Plus haut sur la portée","es":"Más arriba en el pentagrama","it":"Più in alto sul pentagramma"},
          {"en":"Lower on the staff","fr":"Plus bas sur la portée","es":"Más abajo en el pentagrama","it":"Più in basso sul pentagramma"}],
        "answerIndex":0,
        "feedback":{"en":"Higher on the staff = higher pitch.","fr":"Plus haut sur la portée = plus aigu.","es":"Más arriba = más agudo.","it":"Più in alto = più acuto."}},
      {"type":"text","text":{"en":"Nice — that's the staff!","fr":"Bravo — voilà la portée !","es":"¡Bien, eso es el pentagrama!","it":"Bravo — ecco il pentagramma!"}}
    ]
  }$$
),
(
  'sol-note-names', 'published', 'piano', 'solfege', 'beginner', 2, 1,
  $${"en":"Note names","fr":"Le nom des notes","es":"El nombre de las notas","it":"Il nome delle note"}$$,
  $${
    "schemaVersion":1,"id":"sol-note-names","instrument":"piano","track":"solfege","level":"beginner",
    "title":{"en":"Note names","fr":"Le nom des notes","es":"El nombre de las notas","it":"Il nome delle note"},
    "blocks":[
      {"type":"text","text":{
        "en":"Seven note names repeat over and over: C D E F G A B (do ré mi fa sol la si), then start again.",
        "fr":"Sept noms de notes reviennent en boucle : do ré mi fa sol la si, puis ça recommence.",
        "es":"Siete nombres de notas se repiten: do re mi fa sol la si, y vuelven a empezar.",
        "it":"Sette nomi di note si ripetono: do re mi fa sol la si, poi ricominciano."}},
      {"type":"diagram","id":"treble-clef"},
      {"type":"question",
        "prompt":{"en":"How many note names before they repeat?","fr":"Combien de noms de notes avant que ça recommence ?","es":"¿Cuántos nombres antes de repetirse?","it":"Quanti nomi prima che si ripetano?"},
        "options":[{"en":"7","fr":"7","es":"7","it":"7"},{"en":"8","fr":"8","es":"8","it":"8"},{"en":"12","fr":"12","es":"12","it":"12"}],
        "answerIndex":0,
        "feedback":{"en":"Seven, then they cycle.","fr":"Sept, puis ça tourne.","es":"Siete, y giran.","it":"Sette, poi ciclano."}},
      {"type":"playKey","notes":[60],
        "prompt":{"en":"Play a C (do) on the keyboard.","fr":"Joue un do sur le clavier.","es":"Toca un do en el teclado.","it":"Suona un do sulla tastiera."}},
      {"type":"text","text":{"en":"Great — you played a C!","fr":"Super — tu as joué un do !","es":"¡Genial, tocaste un do!","it":"Ottimo — hai suonato un do!"}}
    ]
  }$$
),
(
  'sol-note-values', 'published', 'piano', 'solfege', 'beginner', 3, 1,
  $${"en":"Note values","fr":"Les valeurs de notes","es":"Los valores de las notas","it":"I valori delle note"}$$,
  $${
    "schemaVersion":1,"id":"sol-note-values","instrument":"piano","track":"solfege","level":"beginner",
    "title":{"en":"Note values","fr":"Les valeurs de notes","es":"Los valores de las notas","it":"I valori delle note"},
    "blocks":[
      {"type":"text","text":{
        "en":"A note's shape tells how long to hold it: a quarter note lasts one beat, an eighth note half a beat.",
        "fr":"La forme d'une note dit combien de temps la tenir : une noire vaut un temps, une croche un demi-temps.",
        "es":"La forma de una nota indica cuánto dura: una negra vale un tiempo, una corchea medio tiempo.",
        "it":"La forma di una nota dice quanto tenerla: una semiminima vale un movimento, una croma mezzo movimento."}},
      {"type":"diagram","id":"note-quarter"},
      {"type":"diagram","id":"note-eighth"},
      {"type":"question",
        "prompt":{"en":"Which lasts longer?","fr":"Laquelle dure le plus longtemps ?","es":"¿Cuál dura más?","it":"Quale dura di più?"},
        "options":[
          {"en":"A quarter note","fr":"Une noire","es":"Una negra","it":"Una semiminima"},
          {"en":"An eighth note","fr":"Une croche","es":"Una corchea","it":"Una croma"}],
        "answerIndex":0,
        "feedback":{"en":"The quarter note — one whole beat.","fr":"La noire — un temps entier.","es":"La negra — un tiempo entero.","it":"La semiminima — un movimento intero."}},
      {"type":"text","text":{"en":"That's note durations!","fr":"Voilà les durées !","es":"¡Esas son las duraciones!","it":"Ecco le durate!"}}
    ]
  }$$
),
-- ─── Track B · app-usage · beginner ─────────────────────────────────────────
(
  'app-synthesia', 'published', 'piano', 'app-usage', 'beginner', 1, 1,
  $${"en":"Synthesia mode","fr":"Le mode Synthesia","es":"Modo Synthesia","it":"Modalità Synthesia"}$$,
  $${
    "schemaVersion":1,"id":"app-synthesia","instrument":"piano","track":"app-usage","level":"beginner",
    "title":{"en":"Synthesia mode","fr":"Le mode Synthesia","es":"Modo Synthesia","it":"Modalità Synthesia"},
    "blocks":[
      {"type":"text","text":{
        "en":"In Synthesia mode, falling tiles show which key to press and when.",
        "fr":"En mode Synthesia, des tuiles qui tombent montrent quelle touche presser et quand.",
        "es":"En el modo Synthesia, fichas que caen muestran qué tecla pulsar y cuándo.",
        "it":"In modalità Synthesia, tessere che cadono mostrano quale tasto premere e quando."}},
      {"type":"question",
        "prompt":{"en":"When do you press a key?","fr":"Quand presse-t-on une touche ?","es":"¿Cuándo pulsas una tecla?","it":"Quando premi un tasto?"},
        "options":[
          {"en":"When its tile reaches the keyboard","fr":"Quand sa tuile atteint le clavier","es":"Cuando su ficha llega al teclado","it":"Quando la tessera arriva alla tastiera"},
          {"en":"As soon as the tile appears","fr":"Dès que la tuile apparaît","es":"En cuanto aparece la ficha","it":"Appena appare la tessera"}],
        "answerIndex":0,
        "feedback":{"en":"Play it as the tile lands.","fr":"Joue au moment où la tuile arrive.","es":"Toca cuando la ficha aterriza.","it":"Suona quando la tessera atterra."}},
      {"type":"text","text":{"en":"Try a piece in Synthesia!","fr":"Essaie un morceau en Synthesia !","es":"¡Prueba una pieza en Synthesia!","it":"Prova un brano in Synthesia!"}}
    ]
  }$$
),
(
  'app-partition', 'published', 'piano', 'app-usage', 'beginner', 2, 1,
  $${"en":"Partition mode","fr":"Le mode Partition","es":"Modo Partitura","it":"Modalità Spartito"}$$,
  $${
    "schemaVersion":1,"id":"app-partition","instrument":"piano","track":"app-usage","level":"beginner",
    "title":{"en":"Partition mode","fr":"Le mode Partition","es":"Modo Partitura","it":"Modalità Spartito"},
    "blocks":[
      {"type":"text","text":{
        "en":"Partition mode shows real engraved sheet music, scrolling as you play.",
        "fr":"Le mode Partition affiche une vraie partition gravée, qui défile pendant que tu joues.",
        "es":"El modo Partitura muestra una partitura real grabada, que se desplaza mientras tocas.",
        "it":"La modalità Spartito mostra un vero spartito inciso, che scorre mentre suoni."}},
      {"type":"question",
        "prompt":{"en":"What does Partition mode show?","fr":"Qu'affiche le mode Partition ?","es":"¿Qué muestra el modo Partitura?","it":"Cosa mostra la modalità Spartito?"},
        "options":[
          {"en":"Engraved sheet music","fr":"Une partition gravée","es":"Una partitura grabada","it":"Uno spartito inciso"},
          {"en":"Falling tiles","fr":"Des tuiles qui tombent","es":"Fichas que caen","it":"Tessere che cadono"}],
        "answerIndex":0,
        "feedback":{"en":"Real notation.","fr":"De la vraie notation.","es":"Notación real.","it":"Notazione vera."}},
      {"type":"text","text":{"en":"Great — that's Partition mode!","fr":"Bravo — voilà le mode Partition !","es":"¡Bien, ese es el modo Partitura!","it":"Bravo — ecco la modalità Spartito!"}}
    ]
  }$$
)
ON CONFLICT (id) DO UPDATE SET
  status = EXCLUDED.status, instrument = EXCLUDED.instrument,
  track = EXCLUDED.track, level = EXCLUDED.level, sort_order = EXCLUDED.sort_order,
  schema_version = EXCLUDED.schema_version, title = EXCLUDED.title,
  content = EXCLUDED.content, updated_at = now();
