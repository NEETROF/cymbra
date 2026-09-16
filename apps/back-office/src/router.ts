import { createRouter, createWebHistory, type RouteRecordRaw, type RouterHistory } from "vue-router";
import { useAuthStore } from "@/stores/auth";
import { canOpen, landing, type Access } from "@/lib/navigation";

// Every page lives under the sidebar section it belongs to — `/music/`, `/lingua/`,
// `/admin/` — and its name carries the same prefix (change:
// restructure-back-office-navigation). Each page declares WHO may open it
// (`meta.access`); the guard below and the sidebar both read that one rule.

const MUSIC_MODERATOR: Access = { role: "moderator", scope: "music" };
const MUSIC_ADMIN: Access = { role: "admin", scope: "music" };
const LINGUA_ADMIN: Access = { role: "admin", scope: "lingua" };
const ANY_ADMIN: Access = { role: "admin" };
const GLOBAL_ADMIN: Access = { role: "admin", scope: "global" };

const pages: RouteRecordRaw[] = [
  { path: "/signin", name: "signin", component: () => import("@/views/SignInView.vue"), meta: { public: true } },
  { path: "/denied", name: "denied", component: () => import("@/views/AccessDeniedView.vue"), meta: { public: true } },

  // Music
  {
    path: "/music/queue",
    name: "music-queue",
    component: () => import("@/views/QueueView.vue"),
    meta: { access: MUSIC_MODERATOR },
  },
  {
    path: "/music/catalog",
    name: "music-catalog",
    component: () => import("@/views/CatalogView.vue"),
    meta: { access: MUSIC_MODERATOR },
  },
  {
    path: "/music/review",
    name: "music-review",
    component: () => import("@/views/ReviewView.vue"),
    meta: { access: MUSIC_MODERATOR },
  },
  {
    path: "/music/score/:id",
    name: "music-score",
    component: () => import("@/views/ScoreDetailView.vue"),
    props: true,
    meta: { access: MUSIC_MODERATOR },
  },
  {
    // Private-score takedown (change: add-private-score-catalog): scores users imported
    // for their own use, never in the catalog. Its lookup criteria ride in the query.
    path: "/music/private-scores",
    name: "music-private-scores",
    component: () => import("@/views/PrivateScoresView.vue"),
    meta: { access: MUSIC_ADMIN },
  },
  {
    path: "/music/soundfonts",
    name: "music-soundfonts",
    component: () => import("@/views/SoundFontsView.vue"),
    meta: { access: MUSIC_ADMIN },
  },
  {
    // Plan RPCs are music-admin gated (`require_admin_in_scope("music")`).
    path: "/music/campaigns",
    name: "music-campaigns",
    component: () => import("@/views/CampaignsView.vue"),
    meta: { access: MUSIC_ADMIN },
  },
  {
    // Usage analytics come from the Music app and are music-admin gated server-side.
    path: "/music/usage",
    name: "music-usage",
    component: () => import("@/views/UsageView.vue"),
    meta: { access: MUSIC_ADMIN },
  },

  // Lingua — aggregates + the pack registry, never a per-account view
  // (change: add-lingua-back-office).
  {
    path: "/lingua/overview",
    name: "lingua-overview",
    component: () => import("@/views/LinguaView.vue"),
    meta: { access: LINGUA_ADMIN },
  },

  // Administration — cross-product. One account = one address: the directory finds an
  // account, the detail page acts on it (change: restructure-back-office-users-console).
  {
    path: "/admin/users",
    name: "admin-users",
    component: () => import("@/views/UsersView.vue"),
    meta: { access: ANY_ADMIN },
  },
  {
    path: "/admin/users/:userId",
    name: "admin-user-detail",
    component: () => import("@/views/UserDetailView.vue"),
    props: true,
    meta: { access: ANY_ADMIN },
  },
  {
    path: "/admin/flags",
    name: "admin-flags",
    component: () => import("@/views/FlagsView.vue"),
    meta: { access: ANY_ADMIN },
  },
  {
    path: "/admin/notifications",
    name: "admin-notifications",
    component: () => import("@/views/NotificationsView.vue"),
    meta: { access: ANY_ADMIN },
  },
  {
    // The queue spans every product (identity emails, erasure, Music renders), so only a
    // global admin reads it (change: add-admin-jobs-console).
    path: "/admin/jobs",
    name: "admin-jobs",
    component: () => import("@/views/JobsView.vue"),
    meta: { access: GLOBAL_ADMIN },
  },
];

// A section root opens the section's first page; the guard re-routes if that page is
// not the operator's.
const sectionRoots: RouteRecordRaw[] = [
  { path: "/music", redirect: { name: "music-queue" } },
  { path: "/lingua", redirect: { name: "lingua-overview" } },
  { path: "/admin", redirect: { name: "admin-users" } },
];

// Every path a page had before the sections — admins have them in bookmarks and in
// pasted links, and a silent landing elsewhere on an internal tool is paid for in
// tickets. A named redirect carries the path params and the query (`?tab=`, `?owner=`).
export const FORMER_PATHS: Readonly<Record<string, string>> = {
  "/takedowns": "music-private-scores",
  "/soundfonts": "music-soundfonts",
  "/campaigns": "music-campaigns",
  "/plans": "music-campaigns",
  "/usage": "music-usage",
  "/users": "admin-users",
  "/roles": "admin-users",
  "/users/:userId": "admin-user-detail",
  "/flags": "admin-flags",
  "/notifications": "admin-notifications",
  "/jobs": "admin-jobs",
};

const formerPaths: RouteRecordRaw[] = Object.entries(FORMER_PATHS).map(([path, name]) => ({
  path,
  redirect: { name },
}));

export function createAppRouter(history: RouterHistory = createWebHistory()) {
  // `params: {}` — a redirect by name otherwise inherits the unknown path's `pathMatch`.
  const home = () => {
    const auth = useAuthStore();
    return { ...landing(router, { isAuthenticated: auth.isAuthenticated, claims: auth.claims }), params: {} };
  };
  const router = createRouter({
    history,
    routes: [
      ...pages,
      ...sectionRoots,
      ...formerPaths,
      // Root + anything unknown land on the operator's first openable page.
      { path: "/", name: "home", redirect: home },
      { path: "/:pathMatch(.*)*", redirect: home },
    ],
  });

  // Gate: an unauthenticated visitor is sent to sign-in; a signed-in operator who may
  // not open a page is sent to their landing page (access denied when they have none).
  // UX only — every RPC is independently role-guarded server-side.
  router.beforeEach((to) => {
    const auth = useAuthStore();
    const session = { isAuthenticated: auth.isAuthenticated, claims: auth.claims };
    if (to.meta.public) {
      // An already-signed-in operator has no business on the sign-in page (the
      // in-memory session survives navigating back to /signin) — send them to work.
      if (to.name === "signin" && auth.isAuthenticated) return landing(router, session);
      return true;
    }
    if (!auth.isAuthenticated) return { name: "signin" };
    if (canOpen(to.meta.access, auth.claims)) return true;
    return landing(router, session);
  });

  return router;
}
