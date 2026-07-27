<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useAuthStore } from "@/stores/auth";
import { currentLocale, setLocale, SUPPORTED_LOCALES } from "@/i18n";

const auth = useAuthStore();
const router = useRouter();
const { t } = useI18n();

function signOut() {
  auth.signOut();
  router.push({ name: "signin" });
}
</script>

<template>
  <header class="topbar">
    <nav v-if="auth.isAuthenticated && auth.isModerator">
      <RouterLink to="/queue">{{ t("nav.queue") }}</RouterLink>
      <RouterLink to="/">{{ t("nav.catalog") }}</RouterLink>
      <RouterLink v-if="auth.isAdmin" to="/roles">{{ t("nav.roles") }}</RouterLink>
    </nav>
    <span v-else />
    <div class="who">
      <span v-if="auth.isAuthenticated && auth.isModerator" class="badge">
        {{ auth.isAdmin ? t("role.admin") : t("role.moderator") }}
      </span>
      <!-- Language toggle is always available (incl. the sign-in page). -->
      <div class="lang" role="group" aria-label="language">
        <button
          v-for="l in SUPPORTED_LOCALES"
          :key="l"
          :class="{ active: currentLocale() === l }"
          @click="setLocale(l)"
        >
          {{ l.toUpperCase() }}
        </button>
      </div>
      <button v-if="auth.isAuthenticated && auth.isModerator" @click="signOut">
        {{ t("common.signOut") }}
      </button>
    </div>
  </header>
  <main class="content">
    <RouterView />
  </main>
</template>

<style scoped>
.topbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.6rem 1rem;
  border-bottom: 1px solid var(--border);
  background: var(--panel);
}
nav a {
  margin-right: 1rem;
  text-decoration: none;
}
nav a.router-link-active {
  color: var(--text);
  font-weight: 600;
}
.who {
  display: flex;
  gap: 0.6rem;
  align-items: center;
}
.lang {
  display: inline-flex;
  gap: 0.2rem;
}
.lang button {
  padding: 0.2rem 0.5rem;
  font-size: 0.8rem;
}
.lang button.active {
  border-color: var(--accent);
  color: var(--accent);
}
.content {
  max-width: 1100px;
  margin: 0 auto;
  padding: 1rem;
}
</style>
