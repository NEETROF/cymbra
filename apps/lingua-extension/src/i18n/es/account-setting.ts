import type { accountSetting as fr } from "../fr/account-setting.ts";

// The Cuenta block's copy in Spanish — a draft after the French (src/i18n/README.md).

export const accountSetting: typeof fr = {
  invite: "Inicia sesión para sincronizar tu mazo y tus palabras entre dispositivos.",
  google: "Continuar con Google",
  apple: "Continuar con Apple",
  local: "Correo y contraseña",
  signIn: "Iniciar sesión",
  forgot: "¿Has olvidado la contraseña?",
  signUp: "Crear una cuenta",
  signedIn: "Sesión iniciada",
  handleMissing: "Nombre de usuario por elegir",
  handleCta: "Elige tu nombre de usuario para conservar esta cuenta.",
  handleOpen: "Elegir mi nombre de usuario",
  connected: "Cuentas vinculadas",
  providerHint:
    "¿Cuenta creada con Google o Apple? Aquí, inicia sesión por correo una vez definida una contraseña en «Cuentas vinculadas», desde un navegador que ofrezca Google o Apple, o en Cymbra Music. Crear una cuenta con la misma dirección haría una segunda.",
  signOut: "Cerrar sesión",
  email: "Correo electrónico",
  password: "Contraseña",
  handle: (handle) => `@${handle}`,
};
