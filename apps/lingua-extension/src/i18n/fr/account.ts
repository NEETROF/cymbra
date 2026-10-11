// The account page's copy (account/copy.ts, account/flow.ts, account/view.ts and account.html),
// in French — the source module (add-lingua-interface-language). Its errors in plain words, and the
// providers' names they take, are `account-errors.ts` (localise-lingua-account-onboarding D3).

export const account = {
  pageTitle: "Compte — Cymbra Lingua",
  heading: "Cymbra Lingua",
  pageFootnote:
    "Ton compte Cymbra est le même que dans Cymbra Music. Sans compte, l'extension fonctionne entièrement sur cet appareil.",

  // — The flow's notices (account/flow.ts) —
  erased: "Tes données Lingua sont effacées. Tes autres appareils les effaceront à leur prochaine synchronisation.",
  providerLocal: "Email et mot de passe",
  verifyFirst: "Ton adresse email n'est pas encore vérifiée. Saisis le code reçu par email.",
  codeSent: (email: string) => `Un code de vérification a été envoyé à ${email}.`,
  verifiedSignIn: "Adresse vérifiée. Connecte-toi.",
  verifiedSignedIn: "Adresse vérifiée, tu es connecté.",
  verified: "Adresse vérifiée.",
  newCodeSent: "Un nouveau code a été envoyé.",
  resetCodeSent: (email: string) => `Si un compte existe pour ${email}, un code vient d'y être envoyé.`,
  passwordChanged: "Mot de passe modifié. Connecte-toi avec le nouveau.",
  finishInApp: "Termine la connexion dans l'app Cymbra Lingua, puis reviens ici.",
  linked: (provider: string) => `${provider} est lié à ton compte.`,
  methodRemoved: (provider: string) => `Méthode retirée : ${provider}.`,
  passwordSet: (email: string) =>
    `Mot de passe défini : tu peux aussi te connecter avec ${email}, sur tous tes navigateurs.`,
  handleSaved: (handle: string) => `C'est noté : ton pseudo est @${handle}.`,
  signedOut: "Tu es déconnecté. Connecte-toi avec un autre compte ou crée-en un.",

  // — The views (account/view.ts) —
  /** The handle field's help, by availability (`handleInvalid` is `account-errors.ts`'s); `max` is the handle's longest length. */
  handleEmpty: (max: string) => `1 à ${max} lettres ou chiffres.`,
  handleChecking: "Vérification…",
  handleAvailable: "Disponible !",
  handleTaken: "Ce pseudo est pris — essaies-en un autre.",
  handleUnchecked: "Impossible de vérifier pour l'instant — tu peux quand même valider.",
  continueWithGoogle: "Continuer avec Google",
  continueWithApple: "Continuer avec Apple",
  orWithEmail: "ou avec ton email",
  yourData: "Tes données",
  eraseWarning:
    "Tes mots, ton niveau, ton deck et tes statistiques Lingua seront effacés sur le serveur et sur tous " +
    "tes appareils. C'est définitif. Ton compte Cymbra et Cymbra Music ne sont pas touchés.",
  eraseYes: "Oui, effacer mes données Lingua",
  cancel: "Annuler",
  eraseLead: "Efface ce que Lingua a enregistré pour ce compte, sans supprimer le compte.",
  erase: "Effacer mes données Lingua…",
  deleteFootnote:
    "Supprimer ton compte Cymbra le supprime pour toutes les apps Cymbra, Music compris. " +
    "Pour ne retirer que Lingua, utilise « Effacer mes données Lingua ».",
  deleteAccount: "Supprimer mon compte Cymbra",
  /** « Lié le 4 octobre 2026 » — the date as `formatDate` writes it, day, month and year. */
  linkedOn: (date: string) => `Lié le ${date}`,
  removeAsk: (provider: string) => `Retirer ${provider} de ton compte ?`,
  remove: "Retirer",
  setPassword: "Définir un mot de passe",
  setPasswordLead:
    "Ajoute un email et un mot de passe pour aussi te connecter par email — sur un navigateur sans " +
    "Google ni Apple, par exemple. Nous t'enverrons un code pour vérifier l'adresse.",
  email: "Email",
  password: "Mot de passe",
  setPasswordSubmit: "Définir le mot de passe",
  verifyEmail: "Vérifie ton adresse email",
  /** « Saisis le code envoyé à … », the address in bold. */
  codeSentTo: (email: string) => `Saisis le code envoyé à ${email}`,
  verificationCode: "Code de vérification",
  validate: "Valider",
  restart: "Recommencer",
  connectedLead: "Les méthodes de connexion liées à ton compte Cymbra — les mêmes que dans Cymbra Music.",
  retry: "Réessayer",
  back: "Retour",
  linkGoogle: "Lier Google",
  linkApple: "Lier Apple",
  signIn: "Se connecter",
  forgotPassword: "Mot de passe oublié ?",
  createAccount: "Créer un compte",
  createAccountTitle: "Créer un compte Cymbra",
  signUpLead: "Retrouve tes mots, ton deck et tes statistiques sur tous tes appareils.",
  createMyAccount: "Créer mon compte",
  haveAccount: "J'ai déjà un compte",
  verify: "Vérifier",
  resendCode: "Renvoyer le code",
  useAnotherEmail: "Utiliser un autre email",
  forgotTitle: "Mot de passe oublié",
  forgotLead: "Indique ton email : on t'envoie un code pour choisir un nouveau mot de passe.",
  sendCode: "Envoyer un code",
  backToSignIn: "Retour à la connexion",
  newPasswordTitle: "Nouveau mot de passe",
  codeByEmail: "Code reçu par email",
  newPassword: "Nouveau mot de passe",
  changePassword: "Modifier le mot de passe",
  resendACode: "Renvoyer un code",
  chooseHandle: "Choisis ton pseudo",
  handleLead:
    "Ton pseudo identifie ton compte Cymbra — le même que dans Cymbra Music. Tu pourras le retrouver partout.",
  handle: "Pseudo",
  continue: "Continuer",
  handleFootnote: "Sans pseudo, ce compte n'est pas conservé.",
  useAnotherAccount: "Utiliser un autre compte",
  signedInTitle: "Tu es connecté",
  handleAt: (handle: string) => `@${handle}`,
  signedInLead:
    "Tes mots, ton deck et tes statistiques se synchronisent entre tes appareils. Tu peux fermer cet onglet.",
  signOut: "Se déconnecter",
  connectedAccounts: "Comptes connectés",
};
