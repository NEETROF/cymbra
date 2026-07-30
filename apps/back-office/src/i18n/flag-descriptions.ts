// Localized feature-flag descriptions, keyed by the backend registry key.
//
// The canonical (English) description lives in the backend registry
// (`backend/feature-flags/src/registry.rs`) and is served as `FlagDefinition.doc`.
// Here we only add translations; a key with no entry (e.g. a newly added flag)
// falls back to the backend `doc`, so nothing ever renders blank.
//
// Kept as a flat map rather than vue-i18n catalog entries because the flag keys
// contain dots (`rating.review.min_votes`), which vue-i18n would treat as a
// message path.

const FR: Record<string, string> = {
  "rating.enabled": "Flux de re-notation / révision des partitions.",
  "rewards.enabled": "Points de récompense pour la curation.",
  "rewards.shop.enabled": "Échanges dans la boutique de récompenses.",
  "profiles.public.enabled": "Profils de joueur publics.",
  "leaderboard.per_piece.enabled": "Classements de performance par morceau.",
  "leaderboard.global.enabled": "Le classement de performance global.",
  "onboarding.enabled": "Parcours d’accueil (onboarding).",
  "platform.maintenance": "Coupe-circuit partagé : bascule toutes les apps en mode maintenance.",
  "rating.review.min_votes": "Nombre de votes requis avant qu’une note de partition soit ré-évaluée.",
  "rating.review.threshold": "Seuil de note moyenne en dessous duquel une partition est signalée pour révision.",
  "rewards.points.daily_cap": "Nombre maximum de points de récompense gagnables par jour.",
  "rewards.points.bands": "Barèmes de points attribués par action de curation.",
  "rewards.levels": "Seuils de points cumulés pour chaque niveau de récompense.",
  "rewards.shop.costs": "Coûts en points des articles de la boutique de récompenses.",
  "leaderboard.global.best_n":
    "Nombre de meilleures partitions prises en compte dans le classement global d’un utilisateur.",
  "leaderboard.difficulty_weights": "Multiplicateurs de score par difficulté pour le classement.",
  "leaderboard.season.length_days": "Durée d’une saison de classement, en jours.",
  "account.min_public_sharing_age":
    "Âge minimum pour rendre un profil public (légal — âge du consentement numérique dans l’UE).",
  "data.retention.play_detail_days": "Nombre de jours de conservation du détail de session avant purge.",
};

const BY_LOCALE: Record<string, Record<string, string>> = { fr: FR };

/// The localized description for [key], or [fallback] (the backend `doc`) when no
/// translation exists for the current locale.
export function flagDescription(key: string, fallback: string, locale: string): string {
  return BY_LOCALE[locale]?.[key] ?? fallback;
}
