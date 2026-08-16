<script setup lang="ts">
// The site's sign-in form (spec `site-web-signin`): email + password, "Continue
// with Google" (GSI button rendered into a slot), "Continue with Apple" (popup) —
// each social button only when its client id is configured. Signs in with the
// `web` audience through the shared session; the access token stays in memory.
import { computed, onMounted, ref } from "vue";
import { useAppleSignIn, useGoogleSignIn } from "@cymbra/web-auth";
import { config } from "../lib/config";
import { t, type Lang } from "../lib/i18n";
import { humanError } from "../lib/plan-view";
import { useSession } from "../lib/session";

const props = defineProps<{ lang: Lang }>();
const emit = defineEmits<{ (e: "signedIn"): void }>();

const session = useSession(props.lang);
const email = ref("");
const password = ref("");
const busy = computed(() => session.attempt.value.status === "loading");
const error = computed(() => (session.attempt.value.status === "error" ? session.attempt.value.error : null));

async function submitLocal(): Promise<void> {
  const out = await session.signInLocal(email.value.trim(), password.value);
  if (out.status === "success") {
    password.value = "";
    emit("signedIn");
  }
}

async function submitOidc(idToken: string): Promise<void> {
  const out = await session.signInOidc(idToken);
  if (out.status === "success") emit("signedIn");
}

const mapError = (e: unknown) => humanError(props.lang, e);
const googleSlot = ref<HTMLElement | null>(null);
const google = config.googleClientId ? useGoogleSignIn(config.googleClientId, submitOidc, { mapError }) : null;
const googleFailed = computed(() => google?.status.value.status === "error");
const appleRedirect = globalThis.location ? `${globalThis.location.origin}${globalThis.location.pathname}` : "";
const apple = config.appleClientId
  ? useAppleSignIn(config.appleClientId, appleRedirect, submitOidc, { mapError })
  : null;
const appleReady = computed(() => apple?.status.value.status === "success");
const appleFailed = computed(() => apple?.status.value.status === "error");
const socialFailed = computed(() => (google ? googleFailed.value : false) || (apple ? appleFailed.value : false));

async function appleSignIn(): Promise<void> {
  try {
    await apple?.signIn();
  } catch {
    // A cancelled popup rejects — Apple's own UI already told the user.
  }
}

onMounted(() => {
  if (google && googleSlot.value) void google.render(googleSlot.value, { locale: props.lang, text: "continue_with" });
  if (apple) void apple.load(props.lang);
});
</script>

<template>
  <section class="signin card">
    <h2>{{ t(lang, "signInTitle") }}</h2>
    <p class="muted">{{ t(lang, "signInIntro") }}</p>

    <div v-if="google || apple" class="social">
      <div v-if="google" ref="googleSlot" class="google-slot" data-testid="google-slot"></div>
      <button
        v-if="apple"
        type="button"
        class="btn btn-apple"
        :disabled="!appleReady || busy"
        data-testid="apple-button"
        @click="appleSignIn"
      >
         {{ t(lang, "continueWithApple") }}
      </button>
      <p v-if="socialFailed" class="muted small">{{ t(lang, "socialUnavailable") }}</p>
      <div class="or"><span>{{ t(lang, "or") }}</span></div>
    </div>

    <form class="form" @submit.prevent="submitLocal">
      <label>
        <span>{{ t(lang, "email") }}</span>
        <input v-model="email" type="email" name="email" autocomplete="username" required :disabled="busy" />
      </label>
      <label>
        <span>{{ t(lang, "password") }}</span>
        <input
          v-model="password"
          type="password"
          name="password"
          autocomplete="current-password"
          required
          :disabled="busy"
        />
      </label>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <button type="submit" class="btn btn-primary" :disabled="busy">
        {{ busy ? t(lang, "signingIn") : t(lang, "signIn") }}
      </button>
    </form>
  </section>
</template>
