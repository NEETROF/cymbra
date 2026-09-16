import type { RouteLocationNamedRaw, Router } from "vue-router";
import { hasRoleInScope, isAdmin, isModerator, type Scope, type TokenClaims } from "@/lib/jwt";

// Who may open which page, and how the sidebar is organised (change:
// restructure-back-office-navigation). The router guard and the sidebar read the SAME
// rule — `canOpen` over a route's `meta.access` — so the sidebar can no longer offer a
// page the guard refuses. Both are UX only: every RPC is re-gated server-side.

/** A page's access rule. `moderator` admits a moderator OR an admin; a `scope` narrows
 *  the rule to that scope (a `global` role counts in every scope); no scope admits the
 *  role held in any scope. */
export interface Access {
  readonly role: "moderator" | "admin";
  readonly scope?: Scope;
}

declare module "vue-router" {
  interface RouteMeta {
    /** Reachable without a session (sign-in, access denied). */
    public?: boolean;
    /** Who may open the page. A non-public page without one is never openable. */
    access?: Access;
  }
}

type Claims = Pick<TokenClaims, "roles" | "rolesByScope">;

/** Whether the signed-in claims satisfy `access`. A missing rule fails closed. */
export function canOpen(access: Access | undefined, claims: Claims): boolean {
  if (!access) return false;
  if (!access.scope) return access.role === "admin" ? isAdmin(claims.roles) : isModerator(claims.roles);
  const admin = hasRoleInScope(claims.rolesByScope, access.scope, "admin");
  if (access.role === "admin") return admin;
  return admin || hasRoleInScope(claims.rolesByScope, access.scope, "moderator");
}

/** Icon ids understood by the sidebar's icon table. */
export type NavIcon =
  | "queue"
  | "catalog"
  | "privateScores"
  | "soundfonts"
  | "campaigns"
  | "usage"
  | "lingua"
  | "users"
  | "flags"
  | "notifications"
  | "jobs";

export interface NavEntry {
  /** The route NAME the entry opens — its access rule is read from that route. */
  readonly route: string;
  /** i18n key of the label. */
  readonly label: string;
  readonly icon: NavIcon;
}

export interface NavSection {
  readonly id: "music" | "lingua" | "admin";
  /** i18n key of the heading. */
  readonly heading: string;
  readonly entries: readonly NavEntry[];
}

/** The sidebar, in display order. The first entry an operator can open is their
 *  landing page, so the order is also a priority order. */
export const NAV_SECTIONS: readonly NavSection[] = [
  {
    id: "music",
    heading: "nav.sections.music",
    entries: [
      { route: "music-queue", label: "nav.queue", icon: "queue" },
      { route: "music-catalog", label: "nav.catalog", icon: "catalog" },
      { route: "music-private-scores", label: "nav.privateScores", icon: "privateScores" },
      { route: "music-soundfonts", label: "nav.soundfonts", icon: "soundfonts" },
      { route: "music-campaigns", label: "nav.campaigns", icon: "campaigns" },
      { route: "music-usage", label: "nav.usage", icon: "usage" },
    ],
  },
  {
    id: "lingua",
    heading: "nav.sections.lingua",
    entries: [{ route: "lingua-overview", label: "nav.linguaOverview", icon: "lingua" }],
  },
  {
    id: "admin",
    heading: "nav.sections.admin",
    entries: [
      { route: "admin-users", label: "nav.users", icon: "users" },
      { route: "admin-flags", label: "nav.flags", icon: "flags" },
      { route: "admin-notifications", label: "nav.notifications", icon: "notifications" },
      { route: "admin-jobs", label: "nav.jobs", icon: "jobs" },
    ],
  },
];

/** Whether the named route exists and the claims may open it. */
export function canOpenRoute(router: Router, name: string, claims: Claims): boolean {
  return router.hasRoute(name) && canOpen(router.resolve({ name }).meta.access, claims);
}

/** The sidebar as this operator sees it: entries they can open, empty sections dropped. */
export function visibleSections(router: Router, claims: Claims): NavSection[] {
  return NAV_SECTIONS.map((s) => ({
    ...s,
    entries: s.entries.filter((e) => canOpenRoute(router, e.route, claims)),
  })).filter((s) => s.entries.length > 0);
}

/** Where an operator belongs: sign-in without a session, else the first sidebar entry
 *  they can open, else the access-denied page. Only ever a public route or an openable
 *  one, so a guard that redirects here cannot loop. */
export function landing(router: Router, session: { isAuthenticated: boolean; claims: Claims }): RouteLocationNamedRaw {
  if (!session.isAuthenticated) return { name: "signin" };
  const first = visibleSections(router, session.claims)[0]?.entries[0];
  return first ? { name: first.route } : { name: "denied" };
}
