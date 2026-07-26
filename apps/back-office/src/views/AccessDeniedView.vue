<script setup lang="ts">
import { useRouter } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const auth = useAuthStore();
const router = useRouter();

function signOut() {
  auth.signOut();
  router.push({ name: "signin" });
}
</script>

<template>
  <section class="denied">
    <h1>Access denied</h1>
    <p class="muted">
      This account is signed in but is not a moderator or admin, so no moderation
      actions are available. Ask an admin to grant you the <code>moderator</code> role.
    </p>
    <button @click="signOut">Sign out</button>
  </section>
</template>

<style scoped>
.denied {
  max-width: 460px;
  margin: 4rem auto;
  text-align: center;
}
.muted {
  color: var(--muted);
}
</style>
