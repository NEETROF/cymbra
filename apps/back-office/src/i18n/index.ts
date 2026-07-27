import { createI18n } from "vue-i18n";

// The console ships English + French. Locale = saved choice → browser language →
// English fallback. `t` is available in templates ($t, globalInjection) and in
// plain modules via `i18n.global.t` (e.g. error mapping).
export const SUPPORTED_LOCALES = ["en", "fr"] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];
const STORAGE_KEY = "cymbra.bo.locale";

const en = {
  common: { signOut: "Sign out", back: "← Back", loading: "Loading…" },
  brand: { suite: "Back-office" },
  stats: { total: "Total catalog", approved: "Approved", pending: "Awaiting review" },
  role: { moderator: "moderator", admin: "admin" },
  status: { pending: "pending", accepted: "accepted", rejected: "rejected" },
  level: { any: "any level", beginner: "beginner", intermediate: "intermediate", advanced: "advanced" },
  signin: {
    title: "Cymbra moderation",
    subtitle: "Sign in with a moderator or admin account.",
    email: "email",
    password: "password",
    submit: "Sign in",
    submitting: "Signing in…",
    googleConfigured: "Google sign-in is configured — the Google button targets the same account.",
  },
  denied: {
    title: "Access denied",
    body: "This account is signed in but is not a moderator or admin, so no moderation actions are available. Ask an admin to grant you the moderator role.",
  },
  nav: { queue: "Catalog review", catalog: "Catalog", roles: "Roles", menu: "Menu" },
  catalog: { title: "Catalog", count: "{n} score | {n} scores" },
  queue: {
    title: "Catalog review",
    priorityOrder: "Priority order",
    pending: "{n} pending",
    hint: "most substantial first. Re-review flagging arrives with app ratings (#2).",
  },
  table: {
    title: "Title",
    composer: "Composer",
    level: "Level",
    notes: "Notes",
    bpm: "BPM",
    source: "Source",
    status: "Status",
    empty: "No scores.",
    sortBy: "sort by {field}",
  },
  filters: {
    query: "title or composer",
    composer: "composer",
    pianoOnly: "piano only",
    searchLabel: "search",
    composerLabel: "composer",
    levelLabel: "level",
    statusLabel: "moderation status",
  },
  detail: { accept: "Accept", reject: "Reject", requeue: "Re-queue", score: "Score" },
  preview: {
    title: "Title",
    composer: "Composer",
    arranger: "Arranger",
    level: "Level",
    licence: "Licence",
    source: "Source",
    timeSig: "Time signature",
    notes: "Notes",
    tempo: "Tempo (BPM)",
    loading: "Loading score…",
    loaded:
      "Score loaded ({bytes} bytes). Notation rendering (Rust layout_systems → wasm) is not wired yet in this slice.",
    noBytes: "No score bytes.",
  },
  roles: {
    title: "Roles",
    intro:
      "Browse accounts and grant or revoke the moderator and admin roles (music scope). Filter by handle or email. Every change is recorded in each account's audit history.",
    searchPlaceholder: "filter by handle or email",
    search: "Search",
    colHandle: "Handle",
    colName: "Name",
    colRoles: "Roles",
    colActions: "Actions",
    noAccounts: "No accounts.",
    noHandle: "—",
    grantRole: "Grant {role}",
    revokeRole: "Revoke {role}",
    showing: "{from}–{to} of {total}",
    prev: "Previous",
    next: "Next",
    history: "History",
    when: "When",
    action: "Action",
    scope: "Scope",
    role: "Role",
    byAdmin: "By admin",
  },
  errors: {
    unauthenticated: "Invalid credentials or expired session.",
    permissionDenied: "Access denied.",
    notFound: "Item not found.",
    notAvailable: "Not available yet. Try again later.",
    invalidArgument: "Invalid request.",
    resourceExhausted: "Too many attempts. Try again later.",
    unavailable: "Service unavailable. Try again.",
    generic: "Something went wrong. Try again.",
  },
};

const fr: typeof en = {
  common: { signOut: "Se déconnecter", back: "← Retour", loading: "Chargement…" },
  brand: { suite: "Back-office" },
  stats: { total: "Catalogue total", approved: "Acceptées", pending: "En attente" },
  role: { moderator: "modérateur", admin: "admin" },
  status: { pending: "en attente", accepted: "acceptée", rejected: "rejetée" },
  level: { any: "tous niveaux", beginner: "débutant", intermediate: "intermédiaire", advanced: "avancé" },
  signin: {
    title: "Modération Cymbra",
    subtitle: "Connecte-toi avec un compte modérateur ou admin.",
    email: "e-mail",
    password: "mot de passe",
    submit: "Se connecter",
    submitting: "Connexion…",
    googleConfigured: "La connexion Google est configurée — le bouton Google cible le même compte.",
  },
  denied: {
    title: "Accès refusé",
    body: "Ce compte est connecté mais n'est ni modérateur ni admin : aucune action de modération n'est disponible. Demande à un admin de t'attribuer le rôle modérateur.",
  },
  nav: { queue: "Revue du catalogue", catalog: "Catalogue", roles: "Rôles", menu: "Menu" },
  catalog: { title: "Catalogue", count: "{n} partition | {n} partitions" },
  queue: {
    title: "Revue du catalogue",
    priorityOrder: "Ordre de priorité",
    pending: "{n} en attente",
    hint: "les plus substantielles d'abord. Le signalement de re-revue arrive avec les notes de l'app (#2).",
  },
  table: {
    title: "Titre",
    composer: "Compositeur",
    level: "Niveau",
    notes: "Notes",
    bpm: "BPM",
    source: "Source",
    status: "Statut",
    empty: "Aucune partition.",
    sortBy: "trier par {field}",
  },
  filters: {
    query: "titre ou compositeur",
    composer: "compositeur",
    pianoOnly: "piano uniquement",
    searchLabel: "recherche",
    composerLabel: "compositeur",
    levelLabel: "niveau",
    statusLabel: "statut de modération",
  },
  detail: { accept: "Accepter", reject: "Rejeter", requeue: "Remettre en file", score: "Partition" },
  preview: {
    title: "Titre",
    composer: "Compositeur",
    arranger: "Arrangeur",
    level: "Niveau",
    licence: "Licence",
    source: "Source",
    timeSig: "Signature rythmique",
    notes: "Notes",
    tempo: "Tempo (BPM)",
    loading: "Chargement de la partition…",
    loaded:
      "Partition chargée ({bytes} octets). Le rendu de notation (Rust layout_systems → wasm) n'est pas encore branché dans cette tranche.",
    noBytes: "Aucun octet de partition.",
  },
  roles: {
    title: "Rôles",
    intro:
      "Parcours les comptes et attribue ou retire les rôles modérateur et admin (scope music). Filtre par handle ou e-mail. Chaque changement est enregistré dans l'historique d'audit du compte.",
    searchPlaceholder: "filtrer par handle ou e-mail",
    search: "Rechercher",
    colHandle: "Handle",
    colName: "Nom",
    colRoles: "Rôles",
    colActions: "Actions",
    noAccounts: "Aucun compte.",
    noHandle: "—",
    grantRole: "Attribuer {role}",
    revokeRole: "Retirer {role}",
    showing: "{from}–{to} sur {total}",
    prev: "Précédent",
    next: "Suivant",
    history: "Historique",
    when: "Quand",
    action: "Action",
    scope: "Scope",
    role: "Rôle",
    byAdmin: "Par l'admin",
  },
  errors: {
    unauthenticated: "Identifiants invalides ou session expirée.",
    permissionDenied: "Accès refusé.",
    notFound: "Élément introuvable.",
    notAvailable: "Pas encore disponible. Réessaie plus tard.",
    invalidArgument: "Requête invalide.",
    resourceExhausted: "Trop de tentatives. Réessaie plus tard.",
    unavailable: "Service indisponible. Réessaie.",
    generic: "Une erreur est survenue. Réessaie.",
  },
};

function detectLocale(): Locale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "en" || saved === "fr") return saved;
  } catch {
    /* no storage (SSR/tests) */
  }
  const nav = (globalThis.navigator?.language ?? "en").slice(0, 2).toLowerCase();
  return nav === "fr" ? "fr" : "en";
}

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: detectLocale(),
  fallbackLocale: "en",
  messages: { en, fr },
});

export function setLocale(locale: Locale): void {
  i18n.global.locale.value = locale;
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    /* ignore */
  }
  if (globalThis.document) document.documentElement.lang = locale;
}

export function currentLocale(): Locale {
  return i18n.global.locale.value as Locale;
}

/** Plain-module translate (outside components) — used by the error mapper. */
export const t = i18n.global.t;
