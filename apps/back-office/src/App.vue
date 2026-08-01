<script setup lang="ts">
import { computed, ref } from "vue";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useAuthStore } from "@/stores/auth";
import { useLocaleStore } from "@/stores/locale";
import { currentLocale, SUPPORTED_LOCALES } from "@/i18n";
import AppTag from "@/components/AppTag.vue";

const auth = useAuthStore();
// Language selection goes through the store so a signed-in choice is also recorded
// on the account (change: sync-account-language-preference).
const locale = useLocaleStore();
const router = useRouter();
const route = useRoute();
const { t } = useI18n();

// Mobile: the sidebar is an off-canvas drawer toggled by the hamburger; it closes
// on navigation and on backdrop tap. On desktop the drawer state is inert (the
// sidebar is always shown).
const menuOpen = ref(false);
function closeMenu() {
  menuOpen.value = false;
}

// The full app shell (sidebar) only shows to a signed-in moderator/admin AND never on
// a public route — so the sign-in / access-denied screens render on a bare, centered
// canvas even when an in-memory session is still active (e.g. navigating back to
// /signin after signing in).
const shell = computed(() => auth.isAuthenticated && auth.isModerator && !route.meta.public);

// Minimal line icons (Lucide-style paths) so the nav reads like the mockup
// without pulling an icon dependency.
const ICONS: Record<string, string> = {
  queue: "M3 5h18M3 12h18M3 19h12",
  catalog: "M9 18V5l12-2v13M9 13l12-2M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0Zm12-2a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
  roles: "M12 2 4 6v6c0 5 3.5 8 8 10 4.5-2 8-5 8-10V6l-8-4Z",
  flags: "M4 22V4m0 0 8-2 8 3v9l-8-2-8 2",
};

const nav = computed(() => {
  const items = [
    { to: "/music/queue", key: "nav.queue", icon: "queue" },
    { to: "/music/catalog", key: "nav.catalog", icon: "catalog" },
  ];
  if (auth.isAdmin) {
    items.push({ to: "/roles", key: "nav.roles", icon: "roles" }, { to: "/flags", key: "nav.flags", icon: "flags" });
  }
  return items;
});

async function signOut() {
  closeMenu();
  await auth.signOut();
  router.push({ name: "signin" });
}
</script>

<template>
  <!-- Signed-in shell: fixed sidebar + scrollable main. On mobile the sidebar is an
       off-canvas drawer opened by the hamburger in the mobile bar. -->
  <div v-if="shell" class="shell" :class="{ open: menuOpen }">
    <div class="mobile-bar">
      <button type="button" class="burger" :aria-label="t('nav.menu')" @click="menuOpen = !menuOpen">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M3 6h18M3 12h18M3 18h18" />
        </svg>
      </button>
      <span class="brand-mini">Cymbra</span>
    </div>
    <div class="backdrop" @click="closeMenu" />
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-mark">C</span>
        <span class="brand-text">
          <span class="brand-name">Cymbra</span>
          <span class="brand-suite">{{ t("brand.suite") }}</span>
        </span>
      </div>

      <nav class="nav">
        <RouterLink v-for="item in nav" :key="item.to" :to="item.to" class="nav-item" @click="closeMenu">
          <svg
            class="ic"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path :d="ICONS[item.icon]" />
          </svg>
          <span>{{ t(item.key) }}</span>
        </RouterLink>
      </nav>

      <div class="foot">
        <div class="user-chip">
          <span class="avatar" aria-hidden="true">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M20 21a8 8 0 1 0-16 0" />
              <circle cx="12" cy="7" r="4" />
            </svg>
          </span>
          <AppTag variant="accent" cap>{{ auth.isAdmin ? t("role.admin") : t("role.moderator") }}</AppTag>
        </div>
        <div class="lang" role="toolbar" aria-label="language">
          <button
            v-for="l in SUPPORTED_LOCALES"
            :key="l"
            type="button"
            :class="{ active: currentLocale() === l }"
            @click="locale.choose(l)"
          >
            {{ l.toUpperCase() }}
          </button>
        </div>
        <button type="button" class="signout" @click="signOut">{{ t("common.signOut") }}</button>
      </div>
    </aside>

    <main class="main">
      <RouterView />
    </main>
  </div>

  <!-- Unauthenticated canvas: sign-in / access-denied, with a corner language toggle. -->
  <div v-else class="plain">
    <div class="plain-top">
      <div class="lang" role="toolbar" aria-label="language">
        <button
          v-for="l in SUPPORTED_LOCALES"
          :key="l"
          type="button"
          :class="{ active: currentLocale() === l }"
          @click="locale.choose(l)"
        >
          {{ l.toUpperCase() }}
        </button>
      </div>
    </div>
    <RouterView />
  </div>
</template>

<style scoped>
.shell {
  display: grid;
  grid-template-columns: 248px 1fr;
  min-height: 100vh;
}

.sidebar {
  position: sticky;
  top: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 1.5rem 1rem;
  background: var(--bg-deep);
  border-right: 1px solid var(--border);
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  padding: 0.2rem 0.4rem;
}
.brand-mark {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: linear-gradient(145deg, var(--accent-strong), #b58bff);
  color: #fff;
  font-weight: 800;
  font-size: 1.15rem;
}
.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.1;
}
.brand-name {
  font-weight: 800;
  font-size: 1.15rem;
  color: var(--text);
}
.brand-suite {
  font-family: var(--mono);
  font-size: 0.62rem;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--muted);
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.65rem 0.75rem;
  border-radius: 11px;
  color: var(--muted);
  font-weight: 600;
  font-size: 0.92rem;
}
.nav-item:hover {
  background: var(--panel-2);
  color: var(--text);
}
.nav-item.router-link-active {
  background: color-mix(in srgb, var(--accent-strong) 22%, transparent);
  color: var(--accent);
}
.nav-item .ic {
  width: 19px;
  height: 19px;
  flex: none;
}

.foot {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
}
.user-chip {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
.avatar {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 999px;
  background: var(--panel-3);
  color: var(--accent);
}
.avatar svg {
  width: 17px;
  height: 17px;
}
.lang {
  display: inline-flex;
  gap: 0.25rem;
}
.lang button {
  padding: 0.25rem 0.55rem;
  font-size: 0.75rem;
  border-radius: 8px;
}
.lang button.active {
  border-color: var(--accent);
  color: var(--accent);
}
.signout {
  width: 100%;
}

.main {
  min-width: 0;
  padding: 2.25rem 2.5rem;
  max-width: 1240px;
}

.plain {
  min-height: 100vh;
}
.plain-top {
  display: flex;
  justify-content: flex-end;
  padding: 1rem 1.25rem;
}
.plain-top .lang button {
  padding: 0.25rem 0.55rem;
  font-size: 0.75rem;
}
.plain-top .lang button.active {
  border-color: var(--accent);
  color: var(--accent);
}

/* Mobile bar + drawer backdrop are desktop-hidden; the media query turns them on. */
.mobile-bar {
  display: none;
}
.backdrop {
  display: none;
}
.brand-mini {
  font-weight: 800;
  font-size: 1.1rem;
}

@media (max-width: 720px) {
  .shell {
    grid-template-columns: 1fr;
    /* Bar = its own height, main takes the rest — otherwise the grid stretches the
       bar to share free vertical space on short pages. */
    grid-template-rows: auto 1fr;
  }
  .mobile-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 1rem;
    background: var(--bg-deep);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 10;
  }
  .burger {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    padding: 0;
  }
  .burger svg {
    width: 20px;
    height: 20px;
  }
  /* The sidebar becomes an off-canvas drawer sliding in from the left. */
  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    height: 100vh;
    width: 260px;
    transform: translateX(-100%);
    transition: transform 0.22s ease;
    z-index: 30;
  }
  .shell.open .sidebar {
    transform: translateX(0);
  }
  .backdrop {
    display: block;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.22s ease;
    z-index: 20;
  }
  .shell.open .backdrop {
    opacity: 1;
    pointer-events: auto;
  }
  .main {
    padding: 1.25rem;
  }
}
</style>
