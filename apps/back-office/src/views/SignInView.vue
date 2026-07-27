<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import { useAuthStore } from "@/stores/auth";
import { type Async, idle, run } from "@/lib/async";

const auth = useAuthStore();
const router = useRouter();
const email = ref("");
const password = ref("");
const submit = ref<Async<void>>(idle);
const googleClientId = import.meta.env.VITE_GOOGLE_CLIENT_ID;

const busy = computed(() => submit.value.status === "loading");
const error = computed(() =>
  match(submit.value)
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null),
);

async function afterSignIn() {
  // A signed-in user without moderator/admin lands on the access-denied state.
  await router.push({ name: auth.isModerator ? "queue" : "denied" });
}

async function submitLocal() {
  const outcome = await run(submit, () => auth.signInLocal(email.value, password.value));
  if (outcome.status === "success") await afterSignIn();
}

// OIDC seam: when a Google client id is configured, exchange a Google credential
// (id_token) for a Cymbra token via SignInOidc. Wired for production; the local
// form above is the always-available path.
async function submitGoogleCredential(idToken: string) {
  const outcome = await run(submit, () => auth.signInOidc(idToken));
  if (outcome.status === "success") await afterSignIn();
}
defineExpose({ submitGoogleCredential });
</script>

<template>
  <section class="signin">
    <h1>Cymbra moderation</h1>
    <p class="muted">Sign in with a moderator or admin account.</p>

    <form @submit.prevent="submitLocal">
      <input v-model="email" type="email" placeholder="email" autocomplete="username" required />
      <input
        v-model="password"
        type="password"
        placeholder="password"
        autocomplete="current-password"
        required
      />
      <button type="submit" :disabled="busy">{{ busy ? "Signing in…" : "Sign in" }}</button>
    </form>

    <p v-if="googleClientId" class="muted">
      Google sign-in is configured — the Google button targets the same account.
    </p>

    <p v-if="error" class="error" role="alert">{{ error }}</p>
  </section>
</template>

<style scoped>
.signin {
  max-width: 340px;
  margin: 4rem auto;
  text-align: center;
}
form {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin-top: 1rem;
}
.muted {
  color: var(--muted);
}
.error {
  color: var(--reject);
  margin-top: 1rem;
}
</style>
