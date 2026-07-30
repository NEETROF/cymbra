<script setup lang="ts">
import { computed, onMounted, ref, useTemplateRef } from "vue";
import { useRouter } from "vue-router";
import { match } from "ts-pattern";
import { useAuthStore } from "@/stores/auth";
import { type Async, idle, run } from "@/lib/async";
import { currentLocale } from "@/i18n";
import { useGoogleSignIn } from "@/composables/useGoogleSignIn";
import { useAppleSignIn } from "@/composables/useAppleSignIn";

const auth = useAuthStore();
const router = useRouter();
const email = ref("");
const password = ref("");
const submit = ref<Async<void>>(idle);
const googleClientId = import.meta.env.VITE_GOOGLE_CLIENT_ID;
const appleClientId = import.meta.env.VITE_APPLE_CLIENT_ID;
const appleRedirectUri = import.meta.env.VITE_APPLE_REDIRECT_URI ?? globalThis.location?.origin ?? "";

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

// OIDC seam: exchange a provider credential (id_token, from Google or Apple) for a
// Cymbra token via SignInOidc. Wired for production; the local form above is the
// always-available path. The backend picks the provider from the token's issuer.
async function submitOidcCredential(idToken: string) {
  const outcome = await run(submit, () => auth.signInOidc(idToken));
  if (outcome.status === "success") await afterSignIn();
}
defineExpose({ submitOidcCredential });

// Provider buttons render only when their client id is configured (unset in dev/e2e
// → the local form is the sole path). Each hands us an id_token that
// `submitOidcCredential` exchanges for a Cymbra token.
const googleButton = useTemplateRef<HTMLDivElement>("googleButton");
const google = googleClientId ? useGoogleSignIn(googleClientId, submitOidcCredential) : null;
const googleLoadFailed = computed(() => google?.status.value.status === "error");

const apple = appleClientId ? useAppleSignIn(appleClientId, appleRedirectUri, submitOidcCredential) : null;
const appleLoadFailed = computed(() => apple?.status.value.status === "error");

// The Apple SDK renders no button — we show our own and start its popup on click.
// A cancelled/closed popup rejects; swallow it (Apple's own UI told the user).
async function signInWithApple() {
  try {
    await apple?.signIn();
  } catch {
    /* user cancelled or Apple popup error — nothing to surface */
  }
}

onMounted(() => {
  if (google && googleButton.value) {
    void google.render(googleButton.value, { locale: currentLocale() });
  }
  if (apple) void apple.load(currentLocale());
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

    <template v-if="googleClientId || appleClientId">
      <div class="divider">
        <span>{{ $t("signin.or") }}</span>
      </div>
      <!-- GSI renders its own button into this slot on mount. -->
      <div v-if="googleClientId" ref="googleButton" class="google-slot"></div>
      <!-- Apple renders no button; we provide an Apple-styled one per its guidelines. -->
      <button v-if="appleClientId" type="button" class="btn-apple" @click="signInWithApple">
        <svg class="apple-logo" viewBox="0 0 14 17" aria-hidden="true">
          <path
            fill="currentColor"
            d="M11.6 9.02c-.02-1.83 1.5-2.71 1.57-2.75-.86-1.25-2.19-1.42-2.66-1.44-1.13-.11-2.21.66-2.78.66-.57 0-1.46-.64-2.4-.63-1.23.02-2.37.72-3 1.82-1.28 2.22-.33 5.5.92 7.3.61.88 1.34 1.87 2.3 1.83.92-.04 1.27-.59 2.38-.59 1.11 0 1.42.59 2.39.57.99-.02 1.61-.9 2.21-1.78.7-1.02.99-2.01 1-2.06-.02-.01-1.92-.74-1.94-2.93zM9.98 3.5c.5-.61.84-1.46.75-2.3-.72.03-1.6.48-2.12 1.09-.46.54-.87 1.4-.76 2.23.8.06 1.62-.41 2.13-1.02z"
          />
        </svg>
        {{ $t("signin.signinWithApple") }}
      </button>
      <p v-if="googleLoadFailed" class="muted small">{{ $t("signin.googleUnavailable") }}</p>
      <p v-if="appleLoadFailed" class="muted small">{{ $t("signin.appleUnavailable") }}</p>
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
/* Apple button — follows Apple's HIG: black pill, Apple mark + localized wording. */
.btn-apple {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  width: 100%;
  margin-top: 0.75rem;
  padding: 0.6rem 1rem;
  border: none;
  border-radius: 999px;
  background: #000;
  color: #fff;
  font-size: 0.95rem;
  font-weight: 600;
  cursor: pointer;
}
.btn-apple:hover {
  background: #1a1a1a;
}
.apple-logo {
  width: 14px;
  height: 17px;
  margin-top: -2px;
}
.error {
  color: var(--reject);
  margin-top: 1rem;
}
</style>
