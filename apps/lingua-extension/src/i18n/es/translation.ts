import type { translation as fr } from "../fr/translation.ts";

// The translation setting's copy in Spanish — a draft after the French (src/i18n/README.md).
// A size comes formatted («25,8») and takes its unit here.

export const translation: typeof fr = {
  toggle: "Traducción ampliada",
  attribution: "Modelo de traducción: Firefox Translations (Mozilla), licencia MPL 2.0.",
  ready: "Lista: tus selecciones se traducen en este dispositivo.",
  removed: "El navegador ha eliminado el modelo de este dispositivo. Hay que descargarlo de nuevo.",
  missing: "Falta un modelo para uno de tus idiomas.",
  cancel: "Cancelar",
  download: "Descargar",
  retry: "Reintentar",
  resume: "Reanudar",
  again: "Descargar de nuevo",
  megabytes: (amount) => `${amount} MB`,
  theModel: "el modelo",
  memoryPivot: "unos 340 MB",
  memorySingle: "unos 200 MB",
  cost: (download, memory) =>
    `Traduce tus frases en este dispositivo, sin enviar nada. Descarga ${download} una vez y luego usa ` +
    `${memory} de memoria durante la traducción. Ajuste propio de este dispositivo.`,
  failedNetwork: "La descarga ha fallado: no hay conexión. Reintenta cuando estés en línea.",
  failedUnavailable: "La descarga ha fallado: el servidor no responde. Reintenta más tarde.",
  failedNotTheModel: "La descarga ha fallado: el archivo recibido no es el modelo correcto. Reintenta más tarde.",
  failedStorageSized: (size) => `No hay espacio suficiente en este dispositivo para el modelo (${size}).`,
  failedStorage: "No hay espacio suficiente en este dispositivo para el modelo.",
  failedUnknown: "La descarga ha fallado. Reintenta más tarde.",
  progress: (received, total) => ` ${received} de ${total}`,
  downloading: (progress) => `Descargando el modelo…${progress}`,
  interruptedAt: (progress) => `Descarga interrumpida.${progress}`,
  missingSized: (size) => `Falta un modelo para uno de tus idiomas. ${size} por descargar.`,
};
