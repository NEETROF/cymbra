import type { display as fr } from "../fr/display.ts";

// The display settings' copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). The text size is a percentage the surface formats
// (`formatPercent`, « 110% » in Italian).

export const display: typeof fr = {
  themePaper: "Carta",
  themeDark: "Scuro",
  turnInstant: "Istantaneo",
  turnSlide: "A scorrimento",
  smallerIcon: "A−",
  smaller: "Riduci il testo",
  largerIcon: "A+",
  larger: "Ingrandisci il testo",
  textSize: "Dimensione del testo",
  theme: "Tema",
  pageTurn: "Cambio pagina",
};
