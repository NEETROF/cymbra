import type { accountErrors as fr } from "../fr/account-errors.ts";

// The account's errors in plain words in Spanish — a draft after the French (src/i18n/README.md).

export const accountErrors: typeof fr = {
  unavailable: "No se puede contactar con Cymbra. Comprueba tu conexión y reintenta.",
  rateLimited: "Demasiados intentos. Reintenta dentro de unos minutos.",
  generic: "Se ha producido un error. Reintenta.",
  storageFull:
    "La memoria de la extensión está llena en este dispositivo. Restablece tus datos locales en Ajustes y reintenta.",
  badCode: "Código no válido o caducado. Pide un código nuevo.",
  sessionExpired: "Tu sesión ha caducado. Vuelve a iniciar sesión.",
  linkAlreadyLinked: (provider) => `Esta cuenta de ${provider} ya está vinculada a otra cuenta de Cymbra.`,
  linkUnauthenticated: (provider) =>
    `La vinculación con ${provider} no se ha completado. Reintenta; si sigue fallando, vuelve a iniciar sesión.`,
  linkFailed: (provider) => `No se ha podido vincular ${provider}. Reintenta.`,
  wrongCredentials: "Correo o contraseña incorrectos.",
  emailNotVerified: "Tu dirección de correo aún no está verificada.",
  checkCredentials: "Comprueba el correo y la contraseña introducidos.",
  googleFailed: "El inicio de sesión con Google ha fallado. Reintenta.",
  appleFailed: "El inicio de sesión con Apple ha fallado. Reintenta.",
  emailTaken: "Ya hay una cuenta que usa este correo.",
  weakPassword: "Correo no válido o contraseña demasiado débil: elige una más larga.",
  badCodeOrWeakPassword: "Código no válido o caducado, o contraseña demasiado débil.",
  handleJustTaken: "Este nombre de usuario acaba de ser ocupado: elige otro.",
  sessionExpiredErase: "Tu sesión ha caducado. Vuelve a iniciar sesión para borrar tus datos.",
  eraseFailed: "El borrado no se ha completado. Tus datos están intactos: reintenta.",
  identitiesFailed: "No se han podido cargar tus métodos de inicio de sesión. Reintenta.",
  onlyMethod: "No puedes quitar tu único método de inicio de sesión.",
  unlinkFailed: "No se ha podido quitar este método. Reintenta.",
  addressTakenOrHasPassword: "Esta dirección ya la usa una cuenta de Cymbra, o tu cuenta ya tiene una contraseña.",
  addressJustTaken: "Esta dirección acaba de ser ocupada por otra cuenta. Vuelve a empezar con otra.",
  providerGoogle: "Google",
  providerApple: "Apple",
  handleInvalid: (max) => `De 1 a ${max} letras o números únicamente (sin espacios ni símbolos).`,
};
