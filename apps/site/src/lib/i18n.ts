// Copy of the interactive islands, fr (default) / en — the site's convention is the
// page's `lang`, no i18n library. Keys are grouped by island.

export type Lang = "fr" | "en";

const fr = {
  // sign-in
  signInTitle: "Connexion",
  signInIntro: "Connectez-vous avec votre compte Cymbra (le même que dans l'application).",
  email: "E-mail",
  password: "Mot de passe",
  signIn: "Se connecter",
  signingIn: "Connexion…",
  signOut: "Se déconnecter",
  continueWithGoogle: "Continuer avec Google",
  continueWithApple: "Continuer avec Apple",
  or: "ou",
  socialUnavailable: "Connexion sociale indisponible pour le moment — utilisez votre e-mail.",
  checkingSession: "Vérification de la session…",
  // redeem
  redeemTitle: "Utiliser un code d'accès",
  redeemIntro: "Un code bêta reçu sur la communauté Cymbra ? Entrez-le ici : il s'appliquera à votre compte.",
  code: "Code",
  redeem: "Valider le code",
  redeeming: "Validation…",
  redeemedTitle: "Code appliqué !",
  redeemedTrial: "Vous êtes inscrit·e à la bêta « {name} » : accès Premium jusqu'au {date}.",
  redeemedFeature: "Vous êtes inscrit·e à la bêta « {name} » : accès anticipé aux fonctionnalités en test.",
  redeemedNext: "Ouvrez (ou rafraîchissez) l'application Cymbra : vos nouveaux droits apparaissent à la prochaine connexion.",
  redeemAnother: "Utiliser un autre code",
  // account
  accountTitle: "Mon compte",
  planLabel: "Formule",
  planFree: "Gratuit",
  planPremium: "Premium",
  planTrial: "Premium (bêta d'essai)",
  managedOn: "Géré via {channel}",
  channelApple: "l'App Store",
  channelGoogle: "Google Play",
  channelWeb: "le web",
  renewsOn: "Renouvellement le {date}",
  rightsEndOn: "Fin des droits le {date}",
  trialEndsOn: "Bêta « {name} » — jusqu'au {date}",
  betasTitle: "Bêtas actives",
  noBetas: "Aucune bêta active.",
  betaFeature: "accès anticipé",
  betaTrial: "essai Premium",
  manage: "Gérer mon abonnement",
  manageStoreApple: "Votre abonnement est géré par l'App Store : gérez-le depuis vos réglages Apple.",
  manageStoreGoogle: "Votre abonnement est géré par Google Play : gérez-le depuis vos abonnements Google Play.",
  openStore: "Ouvrir la page de gestion",
  openingPortal: "Ouverture du portail…",
  goPremium: "Passer Premium",
  choosePlan: "Choisir une offre",
  productMonthly: "Mensuel",
  productYearly: "Annuel",
  startingCheckout: "Ouverture du paiement…",
  accountAppNote: "La gestion du compte (e-mail, mot de passe, suppression) se fait dans l'application.",
  downloadTitle: "Télécharger Cymbra",
  // checkout
  checkoutTitle: "Paiement",
  checkoutLoading: "Ouverture du paiement sécurisé…",
  checkoutMissing: "Aucune transaction à régler. Le paiement démarre depuis l'application ou depuis votre compte.",
  checkoutUnavailable: "Le paiement web n'est pas encore disponible.",
  checkoutDoneTitle: "Merci !",
  checkoutDoneBody:
    "Votre paiement est enregistré. Retournez dans l'application Cymbra et rafraîchissez : votre formule Premium s'active dans quelques instants.",
  goToAccount: "Voir mon compte",
  // errors
  errUnauthenticated: "Identifiants incorrects, ou session expirée. Reconnectez-vous.",
  errForbidden: "Action non autorisée.",
  errNotFound: "Introuvable.",
  errPrecondition: "Cette action n'est pas possible pour votre compte actuellement.",
  errInvalid: "Saisie invalide.",
  errRate: "Trop de tentatives — réessayez dans quelques minutes.",
  errUnavailable: "Service momentanément indisponible. Réessayez plus tard.",
  errGeneric: "Une erreur est survenue. Réessayez.",
  errCodeInvalid: "Code invalide ou déjà utilisé.",
  errCodeRefused: "Ce code ne peut pas être appliqué à votre compte (déjà inscrit·e, autre essai en cours, ou bêta fermée).",
};

const en: typeof fr = {
  signInTitle: "Sign in",
  signInIntro: "Sign in with your Cymbra account (the same one as in the app).",
  email: "Email",
  password: "Password",
  signIn: "Sign in",
  signingIn: "Signing in…",
  signOut: "Sign out",
  continueWithGoogle: "Continue with Google",
  continueWithApple: "Continue with Apple",
  or: "or",
  socialUnavailable: "Social sign-in is unavailable right now — use your email.",
  checkingSession: "Checking your session…",
  redeemTitle: "Redeem an access code",
  redeemIntro: "Got a beta code from the Cymbra community? Enter it here: it applies to your account.",
  code: "Code",
  redeem: "Redeem",
  redeeming: "Redeeming…",
  redeemedTitle: "Code applied!",
  redeemedTrial: "You joined the “{name}” beta: Premium access until {date}.",
  redeemedFeature: "You joined the “{name}” beta: early access to the features under test.",
  redeemedNext: "Open (or refresh) the Cymbra app: your new access shows up at the next connection.",
  redeemAnother: "Redeem another code",
  accountTitle: "My account",
  planLabel: "Plan",
  planFree: "Free",
  planPremium: "Premium",
  planTrial: "Premium (trial beta)",
  managedOn: "Managed on {channel}",
  channelApple: "the App Store",
  channelGoogle: "Google Play",
  channelWeb: "the web",
  renewsOn: "Renews on {date}",
  rightsEndOn: "Rights end on {date}",
  trialEndsOn: "“{name}” beta — until {date}",
  betasTitle: "Active betas",
  noBetas: "No active beta.",
  betaFeature: "early access",
  betaTrial: "Premium trial",
  manage: "Manage my subscription",
  manageStoreApple: "Your subscription is managed by the App Store: manage it from your Apple settings.",
  manageStoreGoogle: "Your subscription is managed by Google Play: manage it from your Google Play subscriptions.",
  openStore: "Open the management page",
  openingPortal: "Opening the portal…",
  goPremium: "Go Premium",
  choosePlan: "Choose a plan",
  productMonthly: "Monthly",
  productYearly: "Yearly",
  startingCheckout: "Opening checkout…",
  accountAppNote: "Account management (email, password, deletion) happens in the app.",
  downloadTitle: "Get Cymbra",
  checkoutTitle: "Checkout",
  checkoutLoading: "Opening the secure checkout…",
  checkoutMissing: "Nothing to pay here. A purchase starts from the app or from your account page.",
  checkoutUnavailable: "Web checkout is not available yet.",
  checkoutDoneTitle: "Thank you!",
  checkoutDoneBody:
    "Your payment is recorded. Go back to the Cymbra app and refresh: your Premium plan activates in a moment.",
  goToAccount: "See my account",
  errUnauthenticated: "Wrong credentials, or your session ended. Sign in again.",
  errForbidden: "Not allowed.",
  errNotFound: "Not found.",
  errPrecondition: "This action is not possible for your account right now.",
  errInvalid: "Invalid input.",
  errRate: "Too many attempts — try again in a few minutes.",
  errUnavailable: "Service temporarily unavailable. Try again later.",
  errGeneric: "Something went wrong. Try again.",
  errCodeInvalid: "Invalid or already used code.",
  errCodeRefused: "This code cannot be applied to your account (already enrolled, another trial running, or beta closed).",
};

export type MessageKey = keyof typeof fr;

const dict: Record<Lang, typeof fr> = { fr, en };

/** Localized message with `{name}`-style interpolation. */
export function t(lang: Lang, key: MessageKey, params: Record<string, string> = {}): string {
  const raw = dict[lang][key];
  return raw.replace(/\{(\w+)\}/g, (_, k: string) => params[k] ?? `{${k}}`);
}

/** A localized long date (e.g. "16 août 2026" / "August 16, 2026") from an RFC 3339 string. */
export function formatDate(lang: Lang, iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return new Intl.DateTimeFormat(lang === "fr" ? "fr-FR" : "en-US", { dateStyle: "long" }).format(d);
}
