// The account page's copy (account/copy.ts, account/flow.ts, account/view.ts and account.html),
// in French — the source module (add-lingua-interface-language). A provider failure is never
// worded as a password error (add-lingua-account-parity D7).

export const account = {
  pageTitle: "Compte — Cymbra Lingua",
  heading: "Cymbra Lingua",
  pageFootnote:
    "Ton compte Cymbra est le même que dans Cymbra Music. Sans compte, l'extension fonctionne entièrement sur cet appareil.",

  // — Failures, by flow context × category (account/copy.ts) —
  unavailable: "Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.",
  rateLimited: "Trop de tentatives. Réessaie dans quelques minutes.",
  generic: "Une erreur est survenue. Réessaie.",
  storageFull:
    "La mémoire de l’extension est pleine sur cet appareil. Réinitialise tes données locales dans Réglages, puis réessaie.",
  badCode: "Code invalide ou expiré. Demande un nouveau code.",
  sessionExpired: "Ta session a expiré. Reconnecte-toi.",
  /** `provider` is `providerGoogle` or `providerApple`. */
  linkAlreadyLinked: (provider: string) => `Ce compte ${provider} est déjà lié à un autre compte Cymbra.`,
  linkUnauthenticated: (provider: string) =>
    `La liaison avec ${provider} n'a pas abouti. Réessaie — si ça continue, reconnecte-toi.`,
  linkFailed: (provider: string) => `Impossible de lier ${provider}. Réessaie.`,
  wrongCredentials: "Email ou mot de passe incorrect.",
  emailNotVerified: "Ton adresse email n'est pas encore vérifiée.",
  checkCredentials: "Vérifie l'email et le mot de passe saisis.",
  googleFailed: "La connexion avec Google a échoué. Réessaie.",
  appleFailed: "La connexion avec Apple a échoué. Réessaie.",
  emailTaken: "Un compte utilise déjà cet email.",
  weakPassword: "Email invalide ou mot de passe trop faible : choisis-en un plus long.",
  badCodeOrWeakPassword: "Code invalide ou expiré, ou mot de passe trop faible.",
  handleJustTaken: "Ce pseudo vient d'être pris — choisis-en un autre.",
  sessionExpiredErase: "Ta session a expiré. Reconnecte-toi pour effacer tes données.",
  eraseFailed: "L'effacement n'a pas abouti. Tes données sont intactes : réessaie.",
  identitiesFailed: "Impossible de charger tes méthodes de connexion. Réessaie.",
  onlyMethod: "Tu ne peux pas retirer ta seule méthode de connexion.",
  unlinkFailed: "Impossible de retirer cette méthode. Réessaie.",
  addressTakenOrHasPassword:
    "Cette adresse est déjà utilisée par un compte Cymbra, ou ton compte a déjà un mot de passe.",
  addressJustTaken: "Cette adresse vient d'être prise par un autre compte. Recommence avec une autre.",

  // — The flow's notices (account/flow.ts) —
  erased: "Tes données Lingua sont effacées. Tes autres appareils les effaceront à leur prochaine synchronisation.",
  providerGoogle: "Google",
  providerApple: "Apple",
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
  /** The handle field's help, by availability; `max` is the handle's longest length. */
  handleEmpty: (max: string) => `1 à ${max} lettres ou chiffres.`,
  handleInvalid: (max: string) => `1 à ${max} lettres ou chiffres uniquement (sans espaces ni symboles).`,
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
