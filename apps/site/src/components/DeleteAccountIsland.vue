<script setup lang="ts">
// `/suppression-compte` (`/en/delete-account`): boot the session from the cookie,
// gate on sign-in, name the account, then erase it behind a type-to-confirm gate.
// Irreversible — the copy says so before the field, not after. State + calls live
// in `useDeleteAccount`.
import { onMounted } from "vue";
import SignInForm from "./SignInForm.vue";
import { useDeleteAccount } from "../composables/useDeleteAccount";
import { t, type Lang } from "../lib/i18n";
import { identityLabel } from "../lib/plan-view";

const props = defineProps<{ lang: Lang }>();
const {
  booted,
  account,
  deletion,
  typed,
  signedIn,
  deleting,
  deleted,
  confirmWord,
  armed,
  boot,
  load,
  confirmDelete,
  signOut,
} = useDeleteAccount(props.lang);

onMounted(boot);
</script>

<template>
  <div class="island">
    <p v-if="!booted" class="muted">{{ t(lang, "checkingSession") }}</p>

    <section v-else-if="deleted" class="card" data-testid="deleted">
      <h2>{{ t(lang, "deletedTitle") }}</h2>
      <p>{{ t(lang, "deletedBody") }}</p>
    </section>

    <template v-else-if="!signedIn">
      <p class="muted">{{ t(lang, "deleteSignInNote") }}</p>
      <SignInForm :lang="lang" @signed-in="load" />
    </template>

    <section v-else class="card" data-testid="delete-account">
      <h2>{{ t(lang, "deleteTitle") }}</h2>
      <p>{{ t(lang, "deleteIntro") }}</p>

      <dl v-if="account.status === 'success'" class="plan identity" data-testid="identity">
        <dt>{{ t(lang, "deleteAccountLabel") }}</dt>
        <dd>
          <strong v-if="account.data.handle">@{{ account.data.handle }}</strong>
          <span v-else class="muted">{{ t(lang, "noHandle") }}</span>
          <span v-for="i in account.data.identities" :key="i.provider + (i.email ?? '')" class="line">
            {{ identityLabel(lang, i) }}
          </span>
        </dd>
      </dl>
      <p v-else-if="account.status === 'error'" class="error" role="alert">{{ account.error }}</p>

      <h3>{{ t(lang, "deleteWhatTitle") }}</h3>
      <ul class="plain">
        <li>{{ t(lang, "deleteWhat1") }}</li>
        <li>{{ t(lang, "deleteWhat2") }}</li>
        <li>{{ t(lang, "deleteWhat3") }}</li>
      </ul>

      <h3>{{ t(lang, "deleteKeepsTitle") }}</h3>
      <ul class="plain">
        <li>{{ t(lang, "deleteKeepsSubscription") }}</li>
        <li>{{ t(lang, "deleteKeepsPublished") }}</li>
      </ul>

      <form class="form" @submit.prevent="confirmDelete">
        <label>
          <span>{{ t(lang, "deleteConfirmLabel") }}</span>
          <span class="muted small">{{ t(lang, "deleteConfirmPrompt", { word: confirmWord }) }}</span>
          <input
            v-model="typed"
            type="text"
            name="confirm"
            autocomplete="off"
            spellcheck="false"
            :disabled="deleting"
          />
        </label>
        <p v-if="deletion.status === 'error'" class="error" role="alert">{{ deletion.error }}</p>
        <button type="submit" class="btn btn-danger" data-testid="delete-submit" :disabled="!armed">
          {{ deleting ? t(lang, "deleting") : t(lang, "deleteButton") }}
        </button>
      </form>

      <p class="muted small">{{ t(lang, "deleteAppNote") }}</p>
      <p class="muted small">{{ t(lang, "deleteContact") }}</p>
      <p class="signout">
        <a href="#" @click.prevent="signOut">{{ t(lang, "signOut") }}</a>
      </p>
    </section>
  </div>
</template>
