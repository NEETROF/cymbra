import type { app as fr } from "../fr/app.ts";

// The Cymbra Lingua app's copy in Spanish — a draft after the French (src/i18n/README.md).

export const app: typeof fr = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Repaso",
  stats: "Estadísticas",
  loading: "Cargando tus tarjetas…",
  savedAt: (when: string) => `Tarjetas de Safari, ${when}`,
};
