<script setup lang="ts">
// `/account` (spec `site-account`): the plan, its dated line, active betas, and the
// channel-aware manage action — provider portal (fetched at click time) for a web
// row, the store's own page for a store row, the web checkout when purchasable.
// No account management here (the app owns it). State + calls live in `useAccount`.
import { onMounted } from "vue";
import SignInForm from "./SignInForm.vue";
import { useAccount } from "../composables/useAccount";
import { formatDate, t, type Lang } from "../lib/i18n";
import { identityLabel, productLabel } from "../lib/plan-view";

const props = defineProps<{ lang: Lang }>();
const {
  booted,
  plan,
  account,
  action,
  signedIn,
  summary,
  manage,
  betas,
  actionBusy,
  boot,
  load,
  openPortal,
  checkout,
  signOut,
} = useAccount(props.lang, (url) => globalThis.location?.assign(url));

onMounted(boot);
</script>

<template>
  <div class="island">
    <p v-if="!booted" class="muted">{{ t(lang, "checkingSession") }}</p>
    <SignInForm v-else-if="!signedIn" :lang="lang" @signed-in="load" />

    <section v-else class="card" data-testid="account">
      <h2>{{ t(lang, "accountTitle") }}</h2>

      <dl v-if="account.status === 'success'" class="plan identity" data-testid="identity">
        <dt>{{ t(lang, "handleLabel") }}</dt>
        <dd>
          <strong v-if="account.data.handle">@{{ account.data.handle }}</strong>
          <span v-else class="muted">{{ t(lang, "noHandle") }}</span>
        </dd>
        <dt>{{ t(lang, "signInMethods") }}</dt>
        <dd>
          <span v-for="i in account.data.identities" :key="i.provider + (i.email ?? '')" class="line">
            {{ identityLabel(lang, i) }}
          </span>
        </dd>
      </dl>

      <p v-if="plan.status === 'loading' || plan.status === 'idle'" class="muted">…</p>
      <p v-else-if="plan.status === 'error'" class="error" role="alert">{{ plan.error }}</p>

      <template v-else-if="summary && manage">
        <dl class="plan">
          <dt>{{ t(lang, "planLabel") }}</dt>
          <dd>
            <strong data-testid="plan-title">{{ summary.title }}</strong>
            <span v-for="line in summary.lines" :key="line" class="line">{{ line }}</span>
          </dd>
        </dl>

        <div class="manage">
          <template v-if="manage.kind === 'portal'">
            <button type="button" class="btn btn-primary" :disabled="actionBusy" @click="openPortal">
              {{ actionBusy ? t(lang, "openingPortal") : t(lang, "manage") }}
            </button>
          </template>
          <template v-else-if="manage.kind === 'store'">
            <p class="muted">{{ manage.note }}</p>
            <a class="btn btn-ghost" :href="manage.url" target="_blank" rel="noopener">{{ t(lang, "openStore") }}</a>
          </template>
          <template v-else-if="manage.kind === 'purchase'">
            <p class="muted">{{ t(lang, "choosePlan") }}</p>
            <div class="products">
              <button
                v-for="p in manage.products"
                :key="p"
                type="button"
                class="btn btn-primary"
                :disabled="actionBusy"
                @click="checkout(p)"
              >
                {{ actionBusy ? t(lang, "startingCheckout") : `${t(lang, "goPremium")} — ${productLabel(lang, p)}` }}
              </button>
            </div>
          </template>
          <p v-if="action.status === 'error'" class="error" role="alert">{{ action.error }}</p>
        </div>

        <h3>{{ t(lang, "betasTitle") }}</h3>
        <ul v-if="betas.length" class="betas">
          <li v-for="b in betas" :key="b.campaign_key">
            <strong>{{ b.campaign_name }}</strong>
            <span class="muted">
              — {{ t(lang, b.kind === "premium_trial" ? "betaTrial" : "betaFeature") }}
              <template v-if="b.ends_at">· {{ t(lang, "rightsEndOn", { date: formatDate(lang, b.ends_at) }) }}</template>
            </span>
          </li>
        </ul>
        <p v-else class="muted">{{ t(lang, "noBetas") }}</p>
      </template>

      <p class="muted small">{{ t(lang, "accountAppNote") }}</p>
      <p class="signout">
        <a href="#" @click.prevent="signOut">{{ t(lang, "signOut") }}</a>
      </p>
    </section>
  </div>
</template>
