import type { app as fr } from "../fr/app.ts";

// The Cymbra Lingua app's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). The signed-in lines are worded without a gender (D4).

export const app: typeof fr = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Ripasso",
  stats: "Statistiche",
  loading: "Caricamento delle tue carte…",
  savedAt: (when: string) => `Carte di Safari, ${when}`,
  modelOffer: (size: string) => `Tradurre anche nell'app? Download di ${size}`,
  modelDownload: "Scarica",
  modelLater: "Più tardi",
  modelDownloading: (percent: string) => `Download dei modelli… ${percent}%`,
  modelFailed: "Il download non è andato a buon fine.",
  modelRetry: "Riprova",
  megabytes: (amount: string) => `${amount} MB`,
  signedIn: "Accesso a Cymbra effettuato per la traduzione online",
  signedInAs: (account: string) => `Accesso effettuato come ${account} per la traduzione online`,
  signOut: "Esci",
  signIn: "Accedi per la traduzione online",
  emailCancel: "Annulla",
  syncOffer: "Accedi per sincronizzare i tuoi ripassi e usare la traduzione online.",
  syncSignIn: "Accedi",
  syncLater: "Più tardi",
  syncOn: "Accesso a Cymbra effettuato: i tuoi ripassi si sincronizzano",
  syncOnAs: (account: string) => `Accesso effettuato come ${account}: i tuoi ripassi si sincronizzano`,
  syncOtherAccount: "Safari è collegato a un altro account: i tuoi ripassi passeranno da Safari.",
  syncSafariSignedOut: "Collega anche Safari a questo account per sincronizzare dall'app.",
  syncOlderExtension: "Aggiorna l'estensione di Safari per sincronizzare dall'app.",
  syncFailed: "Sincronizzazione al momento impossibile: i tuoi ripassi verranno inviati più tardi.",
};
