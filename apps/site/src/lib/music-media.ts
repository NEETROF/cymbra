// The media of Cymbra Music's page (change: add-site-music-showcase): which store
// captures and which showcase video the page shows, and what they say in each language.
//
// One source for the sync script (`scripts/sync-music-media.mjs`), the page component
// (`MusicShowcase.astro`) and the check (`test/music-media.spec.ts`). Every file is a
// copy of one the Music capture harness generated under `apps/music/store/` — never an
// image edited or taken by hand — and the check holds each copy to its source.

import type { Lang } from "./i18n";

/** Where the copies live, relative to `apps/site/`: `<dir>/<lang>/<file>`. */
export const MEDIA_DIR = "src/assets/music";

/** The store captures the gallery copies: the iPad 13" set (design D2). */
export const CAPTURE_SOURCE = "ios/ipad_13";

/** Where the showcase video and its poster come from: `apps/music/store/video/<lang>/`. */
export const VIDEO_SOURCE = "video";
export const VIDEO_FILE = "showcase.mp4";
export const POSTER_FILE = "poster.jpg";

/** The site's budget for one video file (spec: site-music-page). */
export const VIDEO_MAX_BYTES = 4 * 1024 * 1024;
export const VIDEO_MAX_SECONDS = 30;

interface Shot {
  /** The capture's file name in `apps/music/store/<CAPTURE_SOURCE>/<lang>/`. */
  file: string;
  /** A short caption under the image. */
  caption: Record<Lang, string>;
  /** What the image shows, for a reader who cannot see it. */
  alt: Record<Lang, string>;
}

/** The gallery, in page order: the claims the page's cards make, shown. */
export const GALLERY: Shot[] = [
  {
    file: "03_staff.png",
    caption: {
      fr: "La vraie partition, qui avance avec vous",
      en: "The real score, moving with you",
      es: "La partitura real, que avanza contigo",
    },
    alt: {
      fr: "Cymbra Music sur iPad : une partition de piano sur la portée, les notes à jouer en surbrillance, le clavier MIDI connecté et le score de justesse en haut de l'écran.",
      en: "Cymbra Music on iPad: a piano score on the staff, the notes to play highlighted, the MIDI keyboard connected and the accuracy score at the top of the screen.",
      es: "Cymbra Music en iPad: una partitura de piano en el pentagrama, las notas que tocar resaltadas, el teclado MIDI conectado y la puntuación de precisión arriba de la pantalla.",
    },
  },
  {
    file: "02_synthesia.png",
    caption: {
      fr: "Ou les notes qui tombent, pour apprendre vite",
      en: "Or falling notes, to learn fast",
      es: "O notas que caen, para aprender rápido",
    },
    alt: {
      fr: "La même pièce en notes qui tombent : des barres de couleur descendent vers un clavier de piano, une couleur par main.",
      en: "The same piece as falling notes: coloured bars come down towards a piano keyboard, one colour per hand.",
      es: "La misma pieza en notas que caen: barras de color bajan hacia un teclado de piano, un color por mano.",
    },
  },
  {
    file: "05_measures.png",
    caption: {
      fr: "Travaillez les mesures difficiles",
      en: "Work on the hard bars",
      es: "Trabaja los compases difíciles",
    },
    alt: {
      fr: "La sélection de mesures : la partition entière en miniature, une plage de mesures choisie pour la répéter.",
      en: "Bar selection: the whole score in miniature, with a range of bars picked to practise again and again.",
      es: "La selección de compases: la partitura entera en miniatura, con un rango de compases elegido para repetirlo.",
    },
  },
  {
    file: "04_courses.png",
    caption: {
      fr: "Des cours, leçon par leçon",
      en: "Lessons, step by step",
      es: "Cursos, lección a lección",
    },
    alt: {
      fr: "Le parcours d'apprentissage : les leçons regroupées par niveau, celles déjà terminées cochées.",
      en: "The learning path: lessons grouped by level, the finished ones ticked.",
      es: "El recorrido de aprendizaje: las lecciones agrupadas por nivel, las ya terminadas marcadas.",
    },
  },
  {
    file: "08_drums_stage.png",
    caption: {
      fr: "La batterie aussi",
      en: "Drums too",
      es: "La batería también",
    },
    alt: {
      fr: "Le mode batterie : un kit animé en perspective, chaque élément s'allume au moment de le frapper.",
      en: "Drum mode: an animated kit in perspective, each piece lighting up when it is time to hit it.",
      es: "El modo batería: un kit animado en perspectiva, cada pieza se ilumina cuando toca golpearla.",
    },
  },
];

/** The video block's copy. */
export const VIDEO_COPY: Record<
  Lang,
  { heading: string; text: string; listen: string; mute: string; label: string }
> = {
  fr: {
    heading: "Voyez-la jouer",
    text: "Une pièce jouée au clavier MIDI, en mode Attente : la partition attend la bonne note, puis passe en notes qui tombent. Le son est celui de l'application.",
    listen: "🔊 Écouter",
    mute: "🔇 Couper le son",
    label: "Vidéo : Cymbra Music suit un morceau joué au clavier MIDI",
  },
  en: {
    heading: "See it play",
    text: "A piece played on a MIDI keyboard, in Wait Mode: the score waits for the right note, then switches to falling notes. The sound is the app's own.",
    listen: "🔊 Listen",
    mute: "🔇 Mute",
    label: "Video: Cymbra Music following a piece played on a MIDI keyboard",
  },
  es: {
    heading: "Mírala tocar",
    text: "Una pieza tocada con un teclado MIDI, en modo Espera: la partitura espera la nota correcta y luego pasa a notas que caen. El sonido es el de la aplicación.",
    listen: "🔊 Escuchar",
    mute: "🔇 Silenciar",
    label: "Vídeo: Cymbra Music sigue una pieza tocada con un teclado MIDI",
  },
};

/** The gallery's heading. */
export const GALLERY_HEADING: Record<Lang, string> = {
  fr: "L'application en images",
  en: "The app in pictures",
  es: "La aplicación en imágenes",
};

/** The share preview: its image file in `MEDIA_DIR/<lang>/` and what it says. */
export const SHARE_FILE = "share.png";
export const SHARE_WIDTH = 1200;
export const SHARE_HEIGHT = 630;
export const SHARE_COPY: Record<Lang, { headline: string; sub: string; alt: string }> = {
  fr: {
    headline: "Apprenez la musique, connecté à votre instrument",
    sub: "Clavier MIDI · Partition en temps réel · Mode Attente",
    alt: "Cymbra Music : une partition de piano sur iPad, qui suit le clavier MIDI.",
  },
  en: {
    headline: "Learn music, connected to your instrument",
    sub: "MIDI keyboard · Real-time score · Wait Mode",
    alt: "Cymbra Music: a piano score on iPad, following the MIDI keyboard.",
  },
  es: {
    headline: "Aprende música, con tu instrumento conectado",
    sub: "Teclado MIDI · Partitura en tiempo real · Modo Espera",
    alt: "Cymbra Music: una partitura de piano en iPad, que sigue el teclado MIDI.",
  },
};
