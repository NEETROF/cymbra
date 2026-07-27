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
    <div class="brand">
      <span class="brand-mark">C</span>
      <span class="brand-name">Cymbra</span>
    </div>
    <h1>{{ $t("signin.title") }}</h1>
    <p class="muted">{{ $t("signin.subtitle") }}</p>

    <form @submit.prevent="submitLocal">
      <input
        v-model="email"
        type="email"
        :placeholder="$t('signin.email')"
        autocomplete="username"
        required
      />
      <input
        v-model="password"
        type="password"
        :placeholder="$t('signin.password')"
        autocomplete="current-password"
        required
      />
      <button class="btn-primary" type="submit" :disabled="busy">
        {{ busy ? $t("signin.submitting") : $t("signin.submit") }}
      </button>
    </form>

    <p v-if="googleClientId" class="muted small">{{ $t("signin.googleConfigured") }}</p>

    <p v-if="error" class="error" role="alert">{{ error }}</p>
  </section>
</template>

<style scoped>
.signin {
  max-width: 360px;
  margin: 6vh auto 0;
  padding: 2rem 2rem 2.25rem;
  text-align: center;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
}
.brand {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 1.25rem;
}
.brand-mark {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: linear-gradient(145deg, var(--accent-strong), #b58bff);
  color: #fff;
  font-weight: 800;
  font-size: 1.2rem;
}
.brand-name { font-weight: 800; font-size: 1.3rem; }
h1 { margin: 0 0 0.35rem; font-size: 1.25rem; }
form {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin-top: 1.25rem;
}
.muted { color: var(--muted); }
.small { font-size: 0.82rem; margin-top: 0.9rem; }
.error {
  color: var(--reject);
  margin-top: 1rem;
}
</style>
