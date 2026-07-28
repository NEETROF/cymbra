<script setup lang="ts">
import { computed, onMounted, ref, useTemplateRef } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import { useAuthStore } from "@/stores/auth";
import { type Async, idle, run } from "@/lib/async";
import { currentLocale } from "@/i18n";
import { useGoogleSignIn } from "@/composables/useGoogleSignIn";

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
  await router.push({ name: auth.isModerator ? "music-queue" : "denied" });
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

// Render the Google Identity Services button only when a client id is configured
// (unset in dev/e2e → the local form is the sole path). GSI hands us an id_token,
// which `submitGoogleCredential` exchanges for a Cymbra token.
const googleButton = useTemplateRef<HTMLDivElement>("googleButton");
const google = googleClientId ? useGoogleSignIn(googleClientId, submitGoogleCredential) : null;
const googleLoadFailed = computed(() => google?.status.value.status === "error");

onMounted(() => {
  if (google && googleButton.value) {
    void google.render(googleButton.value, { locale: currentLocale() });
  }
});
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
      <label for="signin-email" class="sr-only">{{ $t("signin.email") }}</label>
      <input
        id="signin-email"
        v-model="email"
        type="email"
        :placeholder="$t('signin.email')"
        autocomplete="username"
        required
      />
      <label for="signin-password" class="sr-only">{{ $t("signin.password") }}</label>
      <input
        id="signin-password"
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

    <template v-if="googleClientId">
      <div class="divider">
        <span>{{ $t("signin.or") }}</span>
      </div>
      <!-- GSI renders its own button into this slot on mount. -->
      <div ref="googleButton" class="google-slot"></div>
      <p v-if="googleLoadFailed" class="muted small">{{ $t("signin.googleUnavailable") }}</p>
    </template>

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
.brand-name {
  font-weight: 800;
  font-size: 1.3rem;
}
h1 {
  margin: 0 0 0.35rem;
  font-size: 1.25rem;
}
form {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin-top: 1.25rem;
}
.muted {
  color: var(--muted);
}
.small {
  font-size: 0.82rem;
  margin-top: 0.9rem;
}
.divider {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin: 1.25rem 0 1rem;
  color: var(--muted);
  font-size: 0.8rem;
}
.divider::before,
.divider::after {
  content: "";
  flex: 1;
  height: 1px;
  background: var(--border);
}
.google-slot {
  display: flex;
  justify-content: center;
  min-height: 40px;
}
.error {
  color: var(--reject);
  margin-top: 1rem;
}
</style>
