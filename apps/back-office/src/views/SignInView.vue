<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const auth = useAuthStore();
const router = useRouter();
const email = ref("");
const password = ref("");
const error = ref<string | null>(null);
const busy = ref(false);
const googleClientId = import.meta.env.VITE_GOOGLE_CLIENT_ID;

async function afterSignIn() {
  // A signed-in user without moderator/admin lands on the access-denied state.
  await router.push({ name: auth.isModerator ? "queue" : "denied" });
}

async function submitLocal() {
  error.value = null;
  busy.value = true;
  try {
    await auth.signInLocal(email.value, password.value);
    await afterSignIn();
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Sign-in failed";
  } finally {
    busy.value = false;
  }
}

// OIDC seam: when a Google client id is configured, exchange a Google credential
// (id_token) for a Cymbra token via SignInOidc. Wired for production; the local
// form above is the always-available path.
async function submitGoogleCredential(idToken: string) {
  error.value = null;
  busy.value = true;
  try {
    await auth.signInOidc(idToken);
    await afterSignIn();
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Sign-in failed";
  } finally {
    busy.value = false;
  }
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
      <button type="submit" :disabled="busy">Sign in</button>
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
