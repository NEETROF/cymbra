import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { useAuthStore } from "@/stores/auth";

// The auth shell and cross-product admin live at the root; music-specific pages are
// namespaced under `/music/` (path AND `music-` name) so the console can grow other
// product domains later. Role/account administration is transverse, so it stays at
// `/roles` with a generic name.
const routes: RouteRecordRaw[] = [
  { path: "/signin", name: "signin", component: () => import("@/views/SignInView.vue"), meta: { public: true } },
  { path: "/denied", name: "denied", component: () => import("@/views/AccessDeniedView.vue"), meta: { public: true } },
  { path: "/music/catalog", name: "music-catalog", component: () => import("@/views/CatalogView.vue") },
  { path: "/music/queue", name: "music-queue", component: () => import("@/views/QueueView.vue") },
  { path: "/music/review", name: "music-review", component: () => import("@/views/ReviewView.vue") },
  {
    path: "/music/score/:id",
    name: "music-score",
    component: () => import("@/views/ScoreDetailView.vue"),
    props: true,
  },
  { path: "/roles", name: "roles", component: () => import("@/views/RolesView.vue"), meta: { admin: true } },
  { path: "/flags", name: "flags", component: () => import("@/views/FlagsView.vue"), meta: { admin: true } },
  {
    path: "/soundfonts",
    name: "soundfonts",
    component: () => import("@/views/SoundFontsView.vue"),
    meta: { admin: true },
  },
  // Root + anything unknown land on the review queue (the primary work surface).
  { path: "/", redirect: { name: "music-queue" } },
  { path: "/:pathMatch(.*)*", redirect: { name: "music-queue" } },
];

export function createAppRouter() {
  const router = createRouter({ history: createWebHistory(), routes });

  // Gate: an unauthenticated visitor is sent to sign-in; a signed-in non-moderator
  // gets the access-denied state; admin-only routes require the admin role. This is
  // UX only — every RPC is independently role-guarded server-side.
  router.beforeEach((to) => {
    const auth = useAuthStore();
    if (to.meta.public) {
      // An already-signed-in moderator has no business on the sign-in page (the
      // in-memory session survives navigating back to /signin) — send them to work.
      if (to.name === "signin" && auth.isAuthenticated && auth.isModerator) {
        return { name: "music-queue" };
      }
      return true;
    }
    if (!auth.isAuthenticated) return { name: "signin" };
    if (!auth.isModerator) return { name: "denied" };
    if (to.meta.admin && !auth.isAdmin) return { name: "music-catalog" };
    return true;
  });

  return router;
}
