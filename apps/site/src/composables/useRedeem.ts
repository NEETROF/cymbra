// The `/redeem` island's state + API calls (rule: components never call an API —
// composables do, behind the injectable `session.ts` seam).
import { computed, ref } from "vue";
import { type Async, idle, run } from "@cymbra/web-auth";
import type { Lang } from "../lib/i18n";
import { codeFromQuery, redeemError, redeemOutcome } from "../lib/plan-view";
import { usePlans, useSession } from "../lib/session";
import type { RedeemView } from "../lib/web-plans";

export function useRedeem(lang: Lang) {
  const session = useSession(lang);
  const plans = usePlans();

  const booted = ref(false);
  const code = ref("");
  const result = ref<Async<RedeemView>>(idle);
  const signedIn = computed(() => session.state.value.kind === "signedIn");
  const busy = computed(() => result.value.status === "loading");
  const outcome = computed(() => (result.value.status === "success" ? redeemOutcome(lang, result.value.data) : null));

  /** Prefill from the page query, then re-mint the session from the cookie. */
  async function boot(search: string): Promise<void> {
    code.value = codeFromQuery(search);
    await session.boot();
    booted.value = true;
  }

  async function submit(): Promise<void> {
    const token = session.accessToken.value;
    if (!token || !code.value.trim()) return;
    await run(result, () => plans.redeem(token, code.value), (e) => redeemError(lang, e));
  }

  function another(): void {
    code.value = "";
    result.value = idle;
  }

  async function signOut(): Promise<void> {
    await session.signOut();
    result.value = idle;
  }

  return { booted, code, result, signedIn, busy, outcome, boot, submit, another, signOut };
}
