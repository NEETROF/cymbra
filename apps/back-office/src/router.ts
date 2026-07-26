import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const routes: RouteRecordRaw[] = [
  { path: "/signin", name: "signin", component: () => import("@/views/SignInView.vue"), meta: { public: true } },
  { path: "/denied", name: "denied", component: () => import("@/views/AccessDeniedView.vue"), meta: { public: true } },
  { path: "/", name: "catalog", component: () => import("@/views/CatalogView.vue") },
  { path: "/queue", name: "queue", component: () => import("@/views/QueueView.vue") },
  { path: "/score/:id", name: "score", component: () => import("@/views/ScoreDetailView.vue"), props: true },
  { path: "/roles", name: "roles", component: () => import("@/views/RolesView.vue"), meta: { admin: true } },
  { path: "/:pathMatch(.*)*", redirect: "/" },
];

export function createAppRouter() {
  const router = createRouter({ history: createWebHistory(), routes });

  // Gate: an unauthenticated visitor is sent to sign-in; a signed-in non-moderator
  // gets the access-denied state; admin-only routes require the admin role. This is
  // UX only — every RPC is independently role-guarded server-side.
  router.beforeEach((to) => {
    const auth = useAuthStore();
    if (to.meta.public) return true;
    if (!auth.isAuthenticated) return { name: "signin" };
    if (!auth.isModerator) return { name: "denied" };
    if (to.meta.admin && !auth.isAdmin) return { name: "catalog" };
    return true;
  });

  return router;
}
