<script setup lang="ts">
// `/checkout` (spec `site-checkout`): the merchant-of-record hosted checkout. Loads
// Paddle.js, opens the overlay for the `_ptxn` transaction (created server-side,
// already bound to the account) and returns to `/checkout/done`. No sign-in, no
// session, no personal data of its own.
import { onMounted, ref } from "vue";
import { config } from "../lib/config";
import { t, type Lang } from "../lib/i18n";
import { transactionFromQuery } from "../lib/plan-view";

const props = defineProps<{ lang: Lang; doneUrl: string }>();

type State = "loading" | "missing" | "unavailable" | "open" | "failed";
const state = ref<State>("loading");

const PADDLE_SRC = "https://cdn.paddle.com/paddle/v2/paddle.js";

interface PaddleJs {
  Environment: { set(env: "sandbox" | "production"): void };
  Initialize(opts: { token: string }): void;
  Checkout: {
    open(opts: { transactionId: string; settings?: { displayMode?: "overlay" | "inline"; successUrl?: string } }): void;
  };
}
declare global {
  interface Window {
    Paddle?: PaddleJs;
  }
}

function loadPaddle(): Promise<PaddleJs> {
  if (window.Paddle) return Promise.resolve(window.Paddle);
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = PADDLE_SRC;
    script.async = true;
    script.addEventListener("load", () => (window.Paddle ? resolve(window.Paddle) : reject(new Error("Paddle.js"))), {
      once: true,
    });
    script.addEventListener("error", () => reject(new Error("Paddle.js load")), { once: true });
    document.head.appendChild(script);
  });
}

onMounted(async () => {
  const txn = transactionFromQuery(globalThis.location?.search ?? "");
  if (!txn) {
    state.value = "missing";
    return;
  }
  if (!config.paddleClientToken) {
    state.value = "unavailable";
    return;
  }
  try {
    const paddle = await loadPaddle();
    if (config.paddleEnv === "sandbox") paddle.Environment.set("sandbox");
    paddle.Initialize({ token: config.paddleClientToken });
    paddle.Checkout.open({
      transactionId: txn,
      settings: { displayMode: "overlay", successUrl: new URL(props.doneUrl, globalThis.location.origin).toString() },
    });
    state.value = "open";
  } catch {
    state.value = "failed";
  }
});
</script>

<template>
  <div class="island">
    <p v-if="state === 'loading' || state === 'open'" class="muted">{{ t(lang, "checkoutLoading") }}</p>
    <p v-else-if="state === 'missing'" class="muted" data-testid="checkout-missing">{{ t(lang, "checkoutMissing") }}</p>
    <p v-else-if="state === 'unavailable'" class="muted">{{ t(lang, "checkoutUnavailable") }}</p>
    <p v-else class="error" role="alert">{{ t(lang, "errUnavailable") }}</p>
  </div>
</template>
