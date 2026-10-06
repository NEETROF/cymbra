import type { AuthErrorKind } from "../state/auth-errors.ts";

// Reader-facing copy for account failures, chosen by flow context × category
// (add-lingua-account-parity, design D7). French, like the rest of the extension. A
// provider failure is never worded as a password error.

export type FlowContext =
  | "signInEmail"
  | "signInGoogle"
  | "signInApple"
  | "signUp"
  | "verify"
  | "resend"
  | "forgot"
  | "reset"
  | "handle"
  | "eraseData"
  // Comptes connectés (add-lingua-connected-accounts D6)
  | "connected"
  | "linkGoogle"
  | "linkApple"
  | "unlink"
  | "setPassword"
  | "verifyPassword";

const UNAVAILABLE = "Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.";
const RATE_LIMITED = "Trop de tentatives. Réessaie dans quelques minutes.";
const GENERIC = "Une erreur est survenue. Réessaie.";
const STORAGE_FULL =
  "La mémoire de l’extension est pleine sur cet appareil. Réinitialise tes données locales dans Réglages, puis réessaie.";
const BAD_CODE = "Code invalide ou expiré. Demande un nouveau code.";
const SESSION_EXPIRED = "Ta session a expiré. Reconnecte-toi.";

/** Linking Google or Apple: the provider's own words, never a password error. */
function linkCopy(provider: string, kind: AuthErrorKind): string {
  if (kind === "alreadyExists") return `Ce compte ${provider} est déjà lié à un autre compte Cymbra.`;
  // A rejected bearer or a rejected id_token: both say so the same way, and retrying settles it.
  if (kind === "unauthenticated")
    return `La liaison avec ${provider} n'a pas abouti. Réessaie — si ça continue, reconnecte-toi.`;
  return `Impossible de lier ${provider}. Réessaie.`;
}

export function errorCopy(context: FlowContext, kind: AuthErrorKind): string {
  if (kind === "storageFull") return STORAGE_FULL;
  if (kind === "unavailable") return UNAVAILABLE;
  if (kind === "rateLimited") return RATE_LIMITED;
  switch (context) {
    case "signInEmail":
      if (kind === "unauthenticated") return "Email ou mot de passe incorrect.";
      if (kind === "failedPrecondition") return "Ton adresse email n'est pas encore vérifiée.";
      if (kind === "invalidArgument") return "Vérifie l'email et le mot de passe saisis.";
      return GENERIC;
    case "signInGoogle":
      return "La connexion avec Google a échoué. Réessaie.";
    case "signInApple":
      return "La connexion avec Apple a échoué. Réessaie.";
    case "signUp":
      if (kind === "alreadyExists") return "Un compte utilise déjà cet email.";
      if (kind === "invalidArgument") return "Email invalide ou mot de passe trop faible : choisis-en un plus long.";
      return GENERIC;
    case "verify":
      if (kind === "invalidArgument" || kind === "notFound" || kind === "unauthenticated") return BAD_CODE;
      return GENERIC;
    case "reset":
      if (kind === "invalidArgument" || kind === "notFound" || kind === "unauthenticated")
        return "Code invalide ou expiré, ou mot de passe trop faible.";
      return GENERIC;
    case "handle":
      if (kind === "alreadyExists" || kind === "conflict") return "Ce pseudo vient d'être pris — choisis-en un autre.";
      if (kind === "invalidArgument") return "1 à 15 lettres ou chiffres uniquement (sans espaces ni symboles).";
      if (kind === "unauthenticated") return "Ta session a expiré. Reconnecte-toi.";
      return GENERIC;
    case "eraseData":
      if (kind === "unauthenticated") return "Ta session a expiré. Reconnecte-toi pour effacer tes données.";
      return "L'effacement n'a pas abouti. Tes données sont intactes : réessaie.";
    case "connected":
      if (kind === "unauthenticated") return SESSION_EXPIRED;
      return "Impossible de charger tes méthodes de connexion. Réessaie.";
    case "linkGoogle":
      return linkCopy("Google", kind);
    case "linkApple":
      return linkCopy("Apple", kind);
    case "unlink":
      if (kind === "failedPrecondition") return "Tu ne peux pas retirer ta seule méthode de connexion.";
      if (kind === "unauthenticated") return SESSION_EXPIRED;
      return "Impossible de retirer cette méthode. Réessaie.";
    case "setPassword":
      if (kind === "invalidArgument") return "Email invalide ou mot de passe trop faible : choisis-en un plus long.";
      if (kind === "alreadyExists")
        return "Cette adresse est déjà utilisée par un compte Cymbra, ou ton compte a déjà un mot de passe.";
      if (kind === "unauthenticated") return SESSION_EXPIRED;
      return GENERIC;
    case "verifyPassword":
      if (kind === "alreadyExists")
        return "Cette adresse vient d'être prise par un autre compte. Recommence avec une autre.";
      if (kind === "invalidArgument" || kind === "notFound" || kind === "unauthenticated") return BAD_CODE;
      return GENERIC;
    case "resend":
    case "forgot":
      return GENERIC;
  }
}
