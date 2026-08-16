<script setup lang="ts">
// `/redeem` (spec `site-code-redemption`): boot the session from the cookie, gate
// on sign-in, prefill the code from `?code=`, call the web plan API, render the
// outcome or a neutral refusal. Never a price, never a discount. State + calls
// live in `useRedeem`.
import { onMounted } from "vue";
import SignInForm from "./SignInForm.vue";
import { useRedeem } from "../composables/useRedeem";
import { t, type Lang } from "../lib/i18n";

const props = defineProps<{ lang: Lang }>();
const { booted, code, result, signedIn, busy, outcome, boot, submit, another, signOut } = useRedeem(props.lang);

onMounted(() => boot(globalThis.location?.search ?? ""));
</script>

<template>
  <div class="island">
    <p v-if="!booted" class="muted">{{ t(lang, "checkingSession") }}</p>
    <SignInForm v-else-if="!signedIn" :lang="lang" />

    <section v-else class="card">
      <h2>{{ t(lang, "redeemTitle") }}</h2>

      <div v-if="outcome" class="outcome" data-testid="redeem-outcome">
        <h3>{{ outcome.title }}</h3>
        <p>{{ outcome.body }}</p>
        <p class="muted">{{ outcome.next }}</p>
        <button type="button" class="btn btn-ghost" @click="another">{{ t(lang, "redeemAnother") }}</button>
      </div>

      <form v-else class="form" @submit.prevent="submit">
        <p class="muted">{{ t(lang, "redeemIntro") }}</p>
        <label>
          <span>{{ t(lang, "code") }}</span>
          <input
            v-model="code"
            type="text"
            name="code"
            autocomplete="off"
            spellcheck="false"
            class="code"
            required
            :disabled="busy"
          />
        </label>
        <p v-if="result.status === 'error'" class="error" role="alert">{{ result.error }}</p>
        <button type="submit" class="btn btn-primary" :disabled="busy || !code.trim()">
          {{ busy ? t(lang, "redeeming") : t(lang, "redeem") }}
        </button>
      </form>

      <p class="signout">
        <a href="#" @click.prevent="signOut">{{ t(lang, "signOut") }}</a>
      </p>
    </section>
  </div>
</template>
