import type { reader as fr } from "../fr/reader.ts";

// The book reader's copy in Spanish — a draft after the French (src/i18n/README.md).

export const reader: typeof fr = {
  pageTitle: "Biblioteca: Cymbra Lingua",
  title: "Biblioteca",
  importButton: "Importar un libro (EPUB)",
  empty:
    "Tu biblioteca está vacía. Importa un libro EPUB sin DRM: se queda en este dispositivo y se abre sin conexión.",
  importing: "Importando…",
  otherLanguages: "Otros idiomas",
  imported: (title) => `«${title}» está en tu biblioteca.`,
  alreadyThere: (title) => `«${title}» ya estaba en tu biblioteca.`,
  importProtected: "Este libro está protegido por DRM: Cymbra Lingua no puede abrirlo.",
  importNotEpub: "Este archivo no es un libro EPUB.",
  importUnreadable: "Este archivo no se puede leer: puede que esté incompleto o dañado.",
  importStorage: "Ya no queda espacio suficiente en este dispositivo para guardar este libro.",
  persistenceRefused:
    "Tu navegador no se ha comprometido a conservar tus libros: si le falta espacio, podría borrarlos. Siempre podrás volver a importarlos.",
  open: (title) => `Abrir «${title}»`,
  remove: "Eliminar",
  removeConfirm: (title) => `¿Eliminar «${title}»? Las tarjetas que sacaste de él se quedan en tu mazo.`,
  removeYes: "Sí, eliminar",
  cancel: "Cancelar",
  missing: "Este libro ya no está en tu biblioteca.",
  openFailed: "No se ha podido abrir este libro.",
  back: "Biblioteca",
  toc: "Índice",
  display: "Aa",
  displayTitle: "Tamaño del texto y página",
  fullscreen: "Pantalla completa",
  leaveFullscreen: "Salir de la pantalla completa",
  noToc: "Este libro no tiene índice.",
  review: "Repasar",
  stats: "Estadísticas",
  settings: "Ajustes",
  prev: "Página anterior",
  next: "Página siguiente",
  percentTitle: "Palabras conocidas en este capítulo",
  untitled: "Libro sin título",
};
