import type { AuthErrorKind } from "../state/auth-errors.ts";

// Reader-facing copy for account failures, chosen by flow context × category
// (add-lingua-account-parity, design D7). French, like the rest of the extension. A
// provider failure is never worded as a password error.

export type FlowContext =
  "signInEmail" | "signInGoogle" | "signInApple" | "signUp" | "verify" | "resend" | "forgot" | "reset" | "handle";

const UNAVAILABLE = "Impossible de joindre Cymbra. Vérifie ta connexion et réessaie.";
const RATE_LIMITED = "Trop de tentatives. Réessaie dans quelques minutes.";
const GENERIC = "Une erreur est survenue. Réessaie.";
const BAD_CODE = "Code invalide ou expiré. Demande un nouveau code.";

export function errorCopy(context: FlowContext, kind: AuthErrorKind): string {
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
    case "resend":
    case "forgot":
      return GENERIC;
  }
}
