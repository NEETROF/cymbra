// The `/account` island's state + API calls (rule: components never call an API —
// composables do, behind the injectable `session.ts` seam).
import { computed, ref } from "vue";
import { type Async, idle, run } from "@cymbra/web-auth";
import type { Lang } from "../lib/i18n";
import { humanError, manageAction, planSummary } from "../lib/plan-view";
import { usePlans, useSession } from "../lib/session";
import type { AccountView, PlanView } from "../lib/web-plans";

export function useAccount(lang: Lang, navigate: (url: string) => void) {
  const session = useSession(lang);
  const plans = usePlans();

  const booted = ref(false);
  const plan = ref<Async<PlanView>>(idle);
  /** Handle + sign-in methods (Cymbra ID); loaded with the plan, shown when it lands. */
  const account = ref<Async<AccountView>>(idle);
  /** The portal / checkout URL fetch (at click time). */
  const action = ref<Async<string>>(idle);
  const signedIn = computed(() => session.state.value.kind === "signedIn");
  const summary = computed(() => (plan.value.status === "success" ? planSummary(lang, plan.value.data) : null));
  const manage = computed(() => (plan.value.status === "success" ? manageAction(lang, plan.value.data) : null));
  const betas = computed(() => (plan.value.status === "success" ? plan.value.data.betas : []));
  const actionBusy = computed(() => action.value.status === "loading");

  async function load(): Promise<void> {
    const token = session.accessToken.value;
    if (!token) return;
    await Promise.all([
      run(plan, () => plans.me(token), (e) => humanError(lang, e)),
      run(account, () => plans.account(token), (e) => humanError(lang, e)),
    ]);
  }

  async function boot(): Promise<void> {
    await session.boot();
    booted.value = true;
    await load();
  }

  async function openPortal(): Promise<void> {
    const token = session.accessToken.value;
    if (!token) return;
    const out = await run(action, () => plans.portal(token).then((r) => r.portal_url), (e) => humanError(lang, e));
    if (out.status === "success") navigate(out.data);
  }

  async function checkout(productId: string): Promise<void> {
    const token = session.accessToken.value;
    if (!token) return;
    const out = await run(
      action,
      () => plans.checkout(token, productId).then((r) => r.checkout_url),
      (e) => humanError(lang, e),
    );
    if (out.status === "success") navigate(out.data);
  }

  async function signOut(): Promise<void> {
    await session.signOut();
    plan.value = idle;
    account.value = idle;
    action.value = idle;
  }

  return { booted, plan, account, action, signedIn, summary, manage, betas, actionBusy, boot, load, openPortal, checkout, signOut };
}
