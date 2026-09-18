// The routes cymbra.app must keep serving, and who pins each one.
//
// This file is the SINGLE source (change: pin-music-site-url-contract). The build
// gate (`yarn check:routes`) reads it, and `README.md` points at it instead of
// restating the list — two copies would drift, and a route list nobody trusts is
// the failure this contract exists to prevent.
//
// The obligation comes from the consumers, not from the site. A Music build already
// installed on a device requests the path it was compiled with for as long as it
// exists: a user on 1.30 will ask for `/cgu/` long after the site is redesigned. A
// store listing field is read by reviewers and users without the site being
// consulted. Neither can be updated retroactively.
//
// A pinned route MAY change what it serves; it may NOT move or disappear. If one
// ever has to move, the redirect from the old path ships in the same change
// (`public/_redirects` — host-side, because a native app opening an external
// browser never runs client-side script).
//
// Adding a page does NOT pin it. A route joins this list on the day a shipped
// client or a store listing field starts pointing at it — which is the only moment
// anyone knows the pin exists.

export interface PinnedRoute {
  /** The path as the consumer requests it. */
  path: string;
  /** What breaks if it stops resolving. */
  pinnedBy: string;
  /** Where that consumer is declared, so the claim is checkable. */
  source: string;
}

export const PINNED_ROUTES: PinnedRoute[] = [
  // Compiled into shipped Music binaries — the ones that can never be fixed.
  { path: "/cgu/", pinnedBy: "Music — Terms of Service, fr locale", source: "apps/music/lib/services/legal_links.dart" },
  { path: "/confidentialite/", pinnedBy: "Music — Privacy Policy, fr locale; and the fr privacy URL of both listings", source: "apps/music/lib/services/legal_links.dart" },
  { path: "/en/terms/", pinnedBy: "Music — Terms of Service, every other locale", source: "apps/music/lib/services/legal_links.dart" },
  { path: "/en/privacy/", pinnedBy: "Music — Privacy Policy, every other locale; and the en/it/es privacy URL of both listings", source: "apps/music/lib/services/legal_links.dart" },
  { path: "/account", pinnedBy: "Music — managing a web subscription", source: "apps/music/lib/screens/plan_screen.dart" },

  // Billing. A broken return path strands a payment that already went through.
  { path: "/checkout", pinnedBy: "Paddle checkout page", source: "CYMBRA_PADDLE_CHECKOUT_PAGE (apps/music/store/SUBSCRIPTIONS.md)" },
  { path: "/checkout/done", pinnedBy: "Paddle return URL after payment", source: "apps/music/store/SUBSCRIPTIONS.md" },

  // Handed out as links; an access code that 404s is a support ticket.
  { path: "/redeem", pinnedBy: "Access-code links given to users", source: "apps/music/store/SUBSCRIPTIONS.md" },

  // Store console fields. Editing these is a listing change, not a build.
  { path: "/support/", pinnedBy: "App Store Connect support URL, fr", source: "App Store Connect — per platform × locale" },
  { path: "/en/support/", pinnedBy: "App Store Connect support URL, en/it/es", source: "App Store Connect — per platform × locale" },
  { path: "/en/delete-account/", pinnedBy: "Play data-deletion URL, and Apple's account-deletion requirement", source: "Play Console — app content" },
  { path: "/music/", pinnedBy: "Marketing URL (ASC) / Website (Play), fr", source: "App Store Connect + Play Console" },
  { path: "/en/music/", pinnedBy: "Marketing URL (ASC) / Website (Play), en/it/es", source: "App Store Connect + Play Console" },

  // The locale roots. Entry point for everything else, and the previous value of
  // the listings' website field.
  { path: "/", pinnedBy: "Locale root — entry point for every other surface", source: "cymbra.app" },
  { path: "/en/", pinnedBy: "Locale root — entry point for every other surface", source: "cymbra.app" },
];

/**
 * The file an Astro static build is expected to produce for [path].
 *
 * Astro writes one directory per page with an `index.html` inside, so `/cgu/` and
 * `/account` (which consumers request without the trailing slash) both land on
 * `<dir>/index.html`; the site root is `index.html`.
 */
export function outputFileFor(path: string): string {
  const trimmed = path.replace(/^\/+/, "").replace(/\/+$/, "");
  return trimmed.length ? `${trimmed}/index.html` : "index.html";
}
