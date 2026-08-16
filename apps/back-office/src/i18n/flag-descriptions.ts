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
  "data.retention.usage_events_days":
    "Nombre de jours de conservation des événements d’usage bruts avant leur purge (les agrégats permanents ne sont pas touchés).",
  "analytics.collection.enabled":
    "Interrupteur général de la collecte d’usage (actif par défaut ; couper stoppe l’émission des événements par tous les clients sans nouvelle version).",
  "notifications.enabled": "Coupe-circuit global des notifications push : désactivé, aucune catégorie n’envoie.",
  "notifications.category.practice_streak.enabled":
    "Rappel du soir aux joueurs dont la série de pratique est sur le point de se rompre.",
  "notifications.category.practice_streak.hour":
    "Heure locale (0-23) à laquelle le rappel de série est envoyé à chaque joueur.",
  "notifications.category.practice_streak.foreground":
    "Afficher le rappel de série dans l’app quand il arrive alors qu’elle est ouverte.",
  "streak.freeze_cost": "Points que coûte le rétablissement confirmé d’une série de pratique.",
  "streak.grace_days":
    "Nombre de jours manqués pendant lesquels une série rompue reste récupérable (0 désactive la récupération).",
  "catalog.daily_access.enabled":
    "Quota quotidien d’ouvertures gratuites du catalogue (désactivé = toute ouverture est servie).",
  "catalog.daily_access.free_quota":
    "Nombre de morceaux distincts du catalogue qu’un utilisateur peut ouvrir gratuitement par jour (jour serveur).",
  "catalog.daily_access.day_slot_cost": "Points que coûte un morceau supplémentaire pour la journée.",
  "catalog.preview.max_ms": "Durée maximale (ms) de l’extrait audio rendu pour un morceau du catalogue.",
  "catalog.preview.soundfont_id":
    "Identifiant de la SoundFont acceptée du catalogue utilisée pour rendre les extraits (vide = extraits inactifs).",
  "catalog.access_limits.enabled":
    "Limites d’accès au catalogue par utilisateur (actives par défaut ; couper désactive le garde-fou anti-aspiration sans nouvelle version).",
  "catalog.access_limits.download.burst_max": "Nombre maximum de téléchargements de partition par fenêtre de rafale.",
  "catalog.access_limits.download.burst_window_s": "Durée en secondes de la fenêtre de rafale.",
  "catalog.access_limits.download.volume_window_s":
    "Durée en secondes de la fenêtre glissante sur laquelle le volume de téléchargement est compté.",
  "catalog.access_limits.download.base_floor":
    "Téléchargements toujours autorisés par fenêtre, quelle que soit l’activité (le plancher).",
  "catalog.access_limits.download.per_engagement":
    "Téléchargements supplémentaires gagnés par événement d’activité dans la fenêtre (une session de jeu ou une notation).",
  "catalog.access_limits.download.hard_ceiling":
    "Plafond absolu de l’allocation de téléchargement, quelle que soit l’activité.",
  "catalog.access_limits.enum_max":
    "Nombre maximum de requêtes de parcours du catalogue (recherche / navigation / deck de notation) par fenêtre.",
  "catalog.access_limits.enum_window_s": "Durée en secondes de la fenêtre de parcours du catalogue.",
  // -- plans & billing (change: add-premium-subscription) --
  "plans.enabled": "Coupe-circuit du système de plans : désactivé, tout le monde est gratuit, sans bêta ni paywall.",
  "plans.grace_days": "Jours pendant lesquels un abonnement en relance de paiement reste actif après sa fin.",
  "plans.premium.products": "Identifiants produit (stores / MoR) proposés par le paywall (les prix viennent du store).",
  "plans.soundfont_library.max_fonts.free": "Plafond de la bibliothèque .sf2 privée sur le plan gratuit.",
  "plans.soundfont_library.max_fonts.premium":
    "Plafond de la bibliothèque .sf2 privée avec le déblocage « bibliothèque étendue ».",
  "plans.scores.upload_quota.free": "Quota glissant d’envois de partitions sur le plan gratuit ({max, window_days}).",
  "plans.scores.upload_quota.premium": "Quota glissant d’envois de partitions avec le déblocage « quotas étendus ».",
  "plans.scores.library_max.free":
    "Plafond de la bibliothèque de partitions privée sur le plan gratuit (partitions acceptées du catalogue exclues).",
  "plans.scores.library_max.premium":
    "Plafond de la bibliothèque de partitions privée avec le déblocage « quotas étendus ».",
  "billing.apple.enabled": "Canal d’achat Apple : bouton du paywall + route des notifications App Store.",
  "billing.google.enabled": "Canal d’achat Google : bouton du paywall + route des notifications Play.",
  "billing.web.enabled": "Canal d’achat web (marchand officiel) : checkout hébergé + webhook.",
};

const BY_LOCALE: Record<string, Record<string, string>> = { fr: FR };

/// The localized description for [key], or [fallback] (the backend `doc`) when no
/// translation exists for the current locale.
export function flagDescription(key: string, fallback: string, locale: string): string {
  return BY_LOCALE[locale]?.[key] ?? fallback;
}
