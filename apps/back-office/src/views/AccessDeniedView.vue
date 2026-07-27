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
    <h1>{{ $t("denied.title") }}</h1>
    <p class="muted">{{ $t("denied.body") }}</p>
    <button @click="signOut">{{ $t("common.signOut") }}</button>
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
