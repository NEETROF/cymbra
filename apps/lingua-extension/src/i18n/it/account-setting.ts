import type { accountSetting as fr } from "../fr/account-setting.ts";

// The Account block's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). The signed-in state is worded without a gender (D4).

export const accountSetting: typeof fr = {
  invite: "Accedi per sincronizzare il tuo mazzo e le tue parole tra i dispositivi.",
  google: "Continua con Google",
  apple: "Continua con Apple",
  local: "Email e password",
  signIn: "Accedi",
  forgot: "Password dimenticata?",
  signUp: "Crea un account",
  signedIn: "Accesso effettuato",
  handleMissing: "Nome utente da scegliere",
  handleCta: "Scegli il tuo nome utente per conservare questo account.",
  handleOpen: "Scegli il nome utente",
  connected: "Account collegati",
  providerHint:
    "Hai creato l'account con Google o Apple? Per accedere qui con l'email, imposta prima una password in «Account collegati», da un browser che offre Google o Apple oppure in Cymbra Music. Creare un account con lo stesso indirizzo ne creerebbe un secondo.",
  signOut: "Esci",
  email: "Email",
  password: "Password",
  handle: (handle) => `@${handle}`,
};
