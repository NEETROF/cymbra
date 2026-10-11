// The account's errors in plain words (account/copy.ts's `errorCopy` and `linkCopy`), in French —
// the source module (add-lingua-interface-language), by flow context × category; a provider failure
// is never worded as a password error (add-lingua-account-parity D7). A module of its own
// (localise-lingua-account-onboarding D3): the reading surfaces' account setting reads these errors
// too, and must not carry the account page's whole copy into the content script, the popup, the side
// panel and the reader.

export const accountErrors = {
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
  providerGoogle: "Google",
  providerApple: "Apple",
  /** The handle refused by the server; `max` is the handle's longest length. */
  handleInvalid: (max: string) => `1 à ${max} lettres ou chiffres uniquement (sans espaces ni symboles).`,
};
