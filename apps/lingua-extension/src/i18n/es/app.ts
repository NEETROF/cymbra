import type { app as fr } from "../fr/app.ts";

// The Cymbra Lingua app's copy in Spanish — a draft after the French (src/i18n/README.md).

export const app: typeof fr = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Repaso",
  stats: "Estadísticas",
  loading: "Cargando tus tarjetas…",
  savedAt: (when: string) => `Tarjetas de Safari, ${when}`,
  modelOffer: (size: string) => `¿Traducir también en la app? Descarga de ${size}`,
  modelDownload: "Descargar",
  modelLater: "Ahora no",
  modelDownloading: (percent: string) => `Descargando los modelos… ${percent} %`,
  modelFailed: "La descarga no se completó.",
  modelRetry: "Reintentar",
  megabytes: (amount: string) => `${amount} MB`,
  signedIn: "Sesión iniciada en Cymbra para la traducción en línea",
  signOut: "Cerrar sesión",
  signIn: "Iniciar sesión para la traducción en línea",
  emailCancel: "Cancelar",
};
