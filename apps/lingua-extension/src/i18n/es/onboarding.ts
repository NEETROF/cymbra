import type { onboarding as fr } from "../fr/onboarding.ts";

// The onboarding page's copy in Spanish — a draft after the French (src/i18n/README.md).

export const onboarding: typeof fr = {
  pageTitle: "Bienvenida — Cymbra Lingua",
  heading: "Te damos la bienvenida a Cymbra Lingua",
  lead: "Resalta las palabras que aún no conoces mientras lees y repásalas en el momento oportuno, sin salir de tu página.",
  whichLanguages: "¿Qué idiomas aprendes?",
  levelNote:
    "Las palabras por debajo de tu nivel no se resaltarán. No se presupone nada mientras no elijas, y podrás cambiarlo en cualquier momento en los ajustes.",
  toStart: "Para empezar",
  stepPin: "Fija el icono de Cymbra Lingua en la barra del navegador.",
  stepOpen: "Abre una página en el idioma que aprendes y haz clic en el icono para analizarla.",
  stepClick: "Haz clic en una palabra resaltada para traducirla o añadirla a tu mazo.",
  accountTitle: "Recupera tus palabras en todas partes (opcional)",
  accountLead:
    "Con una cuenta de Cymbra, la misma que en Cymbra Music, tus palabras, tu mazo y tus estadísticas se sincronizan entre tus dispositivos. Sin cuenta, tus palabras, tu mazo y tus estados se quedan en este dispositivo.",
  createAccount: "Crear una cuenta",
  later: "Más tarde",
  beginnerChip: "Principiante: empiezo de cero",
  levelSaved: (level) => `Nivel guardado: ${level}. Puedes cerrar esta pestaña y empezar a leer.`,
  beginnerSaved: "Anotado: empezamos de cero. Puedes cerrar esta pestaña y empezar a leer.",
  // — The anonymous counts' first-run sentence (add-lingua-usage-counts D7) —
  usageInform:
    "Lingua cuenta de forma anónima algunos pasos —instalación, primeros pasos, regresos—, sin ningún identificador; nada de lo que lees sale de tu dispositivo. Puedes desactivar estos recuentos en Ajustes → Datos.",
  usageAsk:
    "¿Permites que Lingua cuente de forma anónima algunos pasos —instalación, primeros pasos, regresos—, sin ningún identificador? Nada de lo que lees sale de tu dispositivo, y podrás cambiar de opinión en Ajustes → Datos.",
  usageYes: "Sí, contar",
  usageNo: "No, gracias",
  usageYesDone: "¡Gracias! Puedes desactivar estos recuentos en cualquier momento en Ajustes → Datos.",
  usageNoDone: "Anotado: no se contará nada.",
  usagePrivacy: "Política de privacidad",
};
