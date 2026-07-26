<script setup lang="ts">
import { RouterLink, RouterView, useRouter } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const auth = useAuthStore();
const router = useRouter();

function signOut() {
  auth.signOut();
  router.push({ name: "signin" });
}
</script>

<template>
  <header v-if="auth.isAuthenticated && auth.isModerator" class="topbar">
    <nav>
      <RouterLink to="/queue">Queue</RouterLink>
      <RouterLink to="/">Catalog</RouterLink>
      <RouterLink v-if="auth.isAdmin" to="/roles">Roles</RouterLink>
    </nav>
    <div class="who">
      <span class="badge">{{ auth.isAdmin ? "admin" : "moderator" }}</span>
      <button @click="signOut">Sign out</button>
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
.content {
  max-width: 1100px;
  margin: 0 auto;
  padding: 1rem;
}
</style>
