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
  signedInAs: (account: string) => `Sesión iniciada como ${account} para la traducción en línea`,
  signOut: "Cerrar sesión",
  signIn: "Iniciar sesión para la traducción en línea",
  emailCancel: "Cancelar",
  syncOffer: "Inicia sesión para sincronizar tus repasos y usar la traducción en línea.",
  syncSignIn: "Iniciar sesión",
  syncLater: "Ahora no",
  syncOn: "Sesión iniciada en Cymbra: tus repasos se sincronizan",
  syncOnAs: (account: string) => `Sesión iniciada como ${account}: tus repasos se sincronizan`,
  syncOtherAccount: "Safari tiene otra cuenta conectada: tus repasos pasarán por Safari.",
  syncSafariSignedOut: "Conecta también Safari a esta cuenta para sincronizar desde la app.",
  syncOlderExtension: "Actualiza la extensión de Safari para sincronizar desde la app.",
  syncFailed: "No se puede sincronizar por ahora: tus repasos se enviarán más tarde.",
};
