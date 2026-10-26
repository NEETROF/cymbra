import type { accountErrors as fr } from "../fr/account-errors.ts";

// The account's errors in plain words in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1), reviewed by the Italian reviewer agent (I2). « tu », never « Lei ».

export const accountErrors: typeof fr = {
  unavailable: "Impossibile raggiungere Cymbra. Controlla la connessione e riprova.",
  rateLimited: "Troppi tentativi. Riprova tra qualche minuto.",
  generic: "Si è verificato un errore. Riprova.",
  storageFull:
    "La memoria dell’estensione è piena su questo dispositivo. Reimposta i tuoi dati locali in Impostazioni, poi riprova.",
  badCode: "Codice non valido o scaduto. Richiedi un nuovo codice.",
  sessionExpired: "La sessione è scaduta. Accedi di nuovo.",
  linkAlreadyLinked: (provider) => `Questo account ${provider} è già collegato a un altro account Cymbra.`,
  linkUnauthenticated: (provider) =>
    `Il collegamento con ${provider} non è andato a buon fine. Riprova; se il problema persiste, accedi di nuovo.`,
  linkFailed: (provider) => `Impossibile collegare ${provider}. Riprova.`,
  wrongCredentials: "Email o password errate.",
  emailNotVerified: "Il tuo indirizzo email non è ancora verificato.",
  checkCredentials: "Controlla l'email e la password inserite.",
  googleFailed: "L'accesso con Google non è riuscito. Riprova.",
  appleFailed: "L'accesso con Apple non è riuscito. Riprova.",
  emailTaken: "Esiste già un account con questa email.",
  weakPassword: "Email non valida o password troppo debole: scegline una più lunga.",
  badCodeOrWeakPassword: "Codice non valido o scaduto, oppure password troppo debole.",
  handleJustTaken: "Questo nome utente è appena stato preso: scegline un altro.",
  sessionExpiredErase: "La sessione è scaduta. Accedi di nuovo per cancellare i tuoi dati.",
  eraseFailed: "La cancellazione non è andata a buon fine. I tuoi dati sono intatti: riprova.",
  identitiesFailed: "Impossibile caricare i tuoi metodi di accesso. Riprova.",
  onlyMethod: "Non puoi rimuovere il tuo unico metodo di accesso.",
  unlinkFailed: "Impossibile rimuovere questo metodo. Riprova.",
  addressTakenOrHasPassword:
    "Questo indirizzo è già usato da un account Cymbra, oppure il tuo account ha già una password.",
  addressJustTaken: "Questo indirizzo è appena stato preso da un altro account. Ricomincia con un altro.",
  providerGoogle: "Google",
  providerApple: "Apple",
  handleInvalid: (max) => `Solo lettere o numeri, da 1 a ${max} (senza spazi né simboli).`,
};
